use std::env;
use std::path::{Path, PathBuf};

use reqwest::{self};
use tauri_api::version;

use crate::{errors::*, get_target, updater::ReleaseUpdate, CheckStatus, ProgressStatus, Release};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

thread_local!(static LISTENERS: Arc<Mutex<HashMap<String, EventHandler>>> = Arc::new(Mutex::new(HashMap::new())));

struct EventHandler {
  on_event: Box<dyn FnMut(ProgressStatus)>,
}

impl Release {
  // Read JSON and confirm this is a valid Schema
  fn from_release(release: &serde_json::Value) -> Result<Release> {
    let name = match &release["version"].is_null() {
      false => release["version"].as_str().unwrap().to_string(),
      true => release["name"]
        .as_str()
        .ok_or_else(|| format_err!(Error::Release, "Release missing `name` or `version`"))?
        .to_string(),
    };

    let date = match &release["pub_date"].is_null() {
      false => release["pub_date"].as_str().unwrap().to_string(),
      true => "N/A".to_string(),
    };

    let url = release["url"]
      .as_str()
      .ok_or_else(|| format_err!(Error::Release, "Release missing `url`"))?;

    let body = release["notes"].as_str().map(String::from);

    // Return our formatted release
    Ok(Release {
      version: name.trim_start_matches('v').to_owned(),
      date,
      download_url: url.to_owned(),
      body,
      should_update: false,
    })
  }
}

/// Main Update Builder Object
#[derive(Debug)]
pub struct UpdateBuilder {
  // api url
  url: Option<String>,
  // target (os)
  target: Option<String>,
  // where the executable is located
  executable_path: Option<PathBuf>,
  // current app version
  current_version: Option<String>,
}

impl Default for UpdateBuilder {
  fn default() -> Self {
    Self {
      target: None,
      url: None,
      executable_path: None,
      current_version: None,
    }
  }
}

impl UpdateBuilder {
  /// Initialize a new builder
  pub fn new() -> Self {
    Default::default()
  }

  /// Set the repo name, used to build a github api url
  pub fn url(&mut self, name: &str) -> &mut Self {
    self.url = Some(name.to_owned());
    self
  }

  /// Set the current app version, used to compare against the latest available version.
  /// The `cargo_crate_version!` macro can be used to pull the version from your `Cargo.toml`
  pub fn current_version(&mut self, ver: &str) -> &mut Self {
    self.current_version = Some(ver.to_owned());
    self
  }

  /// Set the target (os)
  /// win32, win64, darwin and linux are currently supported
  pub fn target(&mut self, target: &str) -> &mut Self {
    self.target = Some(target.to_owned());
    self
  }

  /// Set the executable path
  pub fn executable_path<A: AsRef<Path>>(&mut self, executable_path: A) -> &mut Self {
    self.executable_path = Some(PathBuf::from(executable_path.as_ref()));
    self
  }

  pub fn on_progress<F: FnMut(ProgressStatus) + 'static>(&self, handler: F) -> &Self {
    // register our callback
    LISTENERS.with(|listeners| {
      let mut l = listeners
        .lock()
        .expect("Failed to lock listeners: listen()");
      l.insert(
        "httpupdater".to_string(),
        EventHandler {
          on_event: Box::new(handler),
        },
      );
    });
    self
  }

  /// Check remotely for latest update
  pub fn check(&self) -> Result<Box<dyn ReleaseUpdate>> {
    // If no executable path provided, we use current_exe from rust
    let executable_path = if let Some(v) = &self.executable_path {
      v.clone()
    } else {
      env::current_exe()?
    };

    // Did the target is provided by the config?
    let target = if let Some(t) = &self.target {
      t.clone()
    } else {
      get_target().to_string()
    };

    // Get the extract_path from the provided executable_path

    // Linux & Windows should need to be extracted in the same directory as the executable
    // C:\Program Files\MyApp\MyApp.exe
    // We need C:\Program Files\MyApp
    let mut extract_path = executable_path.parent().map(PathBuf::from).unwrap();

    // MacOS example binary is in /Applications/TestApp.app/Contents/MacOS/myApp
    // We need to get /Applications/TestApp.app
    // todo(lemarier): Need a better way here
    // Maybe we could search for <*.app> to get the right path
    if target == "darwin" {
      extract_path = extract_path
        .parent()
        .map(PathBuf::from)
        .unwrap()
        .parent()
        .map(PathBuf::from)
        .unwrap();
    };

    // replace {{current_version}} and {{target}} in the provided URL
    // this is usefull if we need to query example
    // https://releases.myapp.com/update/{{target}}/{{current_version}}
    // will be transleted into ->
    // https://releases.myapp.com/update/darwin/1.0.0
    // The main objective is if the update URL is defined via the Cargo.toml
    // the URL will be generated dynamicly
    let url = if let Some(ref url) = self.url {
      str::replace(
        &str::replace(
          url,
          "{{current_version}}",
          &self.current_version.as_ref().unwrap(),
        ),
        "{{target}}",
        &target,
      )
    } else {
      bail!(Error::Config, "`url` required")
    };

    // make sure SSL is correctly set for linux
    set_ssl_vars!();

    // make our remote query
    let resp = reqwest::blocking::Client::new().get(&url).send()?;
    if !resp.status().is_success() {
      bail!(
        Error::Network,
        "api request failed with status: {:?} - for: {:?}",
        resp.status(),
        self.url
      )
    }

    // unmarshall the JSON into a Release
    let json = resp.json::<serde_json::Value>()?;
    let mut remote_release = Release::from_release(&json)?;

    // did the announced version is greated than our current one?
    let should_update = match version::is_greater(
      &self.current_version.as_ref().unwrap(),
      &remote_release.version,
    ) {
      Ok(v) => v,
      Err(_) => false,
    };

    // save it for future reference
    remote_release.should_update = should_update;

    // return our Update structure
    Ok(Box::new(Update {
      url,
      target,
      executable_path,
      extract_path,
      should_update,
      current_version: if let Some(ref ver) = self.current_version {
        ver.to_owned()
      } else {
        bail!(Error::Config, "`current_version` required")
      },
      remote_release: Release::from_release(&json)?,
    }))
  }
}

/// Specific update data
#[derive(Debug)]
pub struct Update {
  url: String,
  target: String,
  current_version: String,
  executable_path: PathBuf,
  extract_path: PathBuf,
  remote_release: Release,
  should_update: bool,
}

impl Update {
  /// Initialize a new `UpdateBuilder`
  pub fn configure() -> UpdateBuilder {
    UpdateBuilder::new()
  }
}

impl ReleaseUpdate for Update {
  // current version
  fn current_version(&self) -> String {
    self.current_version.to_owned()
  }

  // download URL
  fn url(&self) -> String {
    self.url.to_owned()
  }

  // OS (platform)
  fn target(&self) -> String {
    self.target.clone()
  }

  // executable path
  fn executable_path(&self) -> PathBuf {
    self.executable_path.clone()
  }

  // get the extract path from the executable path
  // it'll differ depending of the OS
  fn extract_path(&self) -> PathBuf {
    self.extract_path.clone()
  }

  // Return the status if an update is available or not
  // CheckStatus::UpdateAvailable
  // CheckStatus::UpToDate
  fn status(&self) -> CheckStatus {
    if self.should_update {
      return CheckStatus::UpdateAvailable(self.release_details());
    }

    CheckStatus::UpToDate
  }

  // Cached data extracted in the check() step
  fn release_details(&self) -> Release {
    self.remote_release.clone()
  }

  // Cached data extracted in the check() step
  fn send_progress(&self, status: ProgressStatus) {
    LISTENERS.with(|listeners| {
      let mut l = listeners
        .lock()
        .expect("Failed to lock listeners: on_event()");

      let key = "httpupdater".to_string();

      if l.contains_key(&key) {
        let handler = l.get_mut(&key).expect("Failed to get mutable handler");
        (handler.on_event)(status);
      }
    });
  }
}
