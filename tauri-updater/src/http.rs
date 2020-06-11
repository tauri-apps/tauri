use crate::{
  get_target, updater::extract_path_from_executable, updater::ReleaseUpdate, CheckStatus,
  ProgressStatus, Release,
};
use reqwest::{self};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri_api::version;

thread_local!(static LISTENERS: Arc<Mutex<HashMap<String, EventHandler>>> = Arc::new(Mutex::new(HashMap::new())));

struct EventHandler {
  on_event: Box<dyn FnMut(ProgressStatus)>,
}

impl Release {
  // Read JSON and confirm this is a valid Schema
  fn from_release(release: &serde_json::Value) -> crate::Result<Release> {
    let name = match &release["version"].is_null() {
      false => release["version"].as_str().unwrap().to_string(),
      true => release["name"]
        .as_str()
        .ok_or_else(|| crate::Error::Release("Release missing `name` or `version`".into()))?
        .to_string(),
    };

    let date = match &release["pub_date"].is_null() {
      false => release["pub_date"].as_str().unwrap().to_string(),
      true => "N/A".to_string(),
    };

    let url = release["url"]
      .as_str()
      .ok_or_else(|| crate::Error::Release("Release missing `name` or `url`".into()))?;

    let body = release["notes"].as_str().map(String::from);

    let signature = match &release["signature"].is_null() {
      false => Some(release["signature"].as_str().unwrap().to_string()),
      true => None,
    };

    // Return our formatted release
    Ok(Release {
      signature,
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
  urls: Vec<String>,
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
      urls: Vec::new(),
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

  /// Add remote URL to extract metadata
  pub fn url(&mut self, url: &str) -> &mut Self {
    self.urls.push(url.to_string());
    self
  }

  /// Add multiple URLS at once inside a Vec for future reference
  pub fn urls(&mut self, urls: &[&str]) -> &mut Self {
    let mut formatted_vec: Vec<String> = Vec::new();
    for url in urls {
      formatted_vec.push((*url).to_string());
    }
    self.urls = formatted_vec;
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
  pub fn check(&self) -> crate::Result<Box<dyn ReleaseUpdate>> {
    let mut remote_release: Option<Release> = None;

    // make sure we have at least one url
    if self.urls.is_empty() {
      bail!(crate::Error::Config, "`url` required")
    };

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
    let extract_path = extract_path_from_executable(&executable_path, &target);

    // make sure SSL is correctly set for linux
    set_ssl_vars!();

    // Allow fallback if more than 1 urls is provided
    let mut last_error: Option<crate::Error> = None;
    for url in &self.urls {
      // replace {{current_version}} and {{target}} in the provided URL
      // this is usefull if we need to query example
      // https://releases.myapp.com/update/{{target}}/{{current_version}}
      // will be transleted into ->
      // https://releases.myapp.com/update/darwin/1.0.0
      // The main objective is if the update URL is defined via the Cargo.toml
      // the URL will be generated dynamicly

      let fixed_link = str::replace(
        &str::replace(
          url,
          "{{current_version}}",
          &self.current_version.as_ref().unwrap(),
        ),
        "{{target}}",
        &target,
      );

      let resp = reqwest::blocking::Client::new()
        .get(&fixed_link)
        .timeout(Duration::from_secs(5))
        .send();

      // If we got a success, we stop the loop
      // and we set our remote_release variable
      if let Ok(ref res) = resp {
        if res.status().is_success() {
          let json = resp?.json::<serde_json::Value>()?;
          let built_release = Release::from_release(&json);
          match built_release {
            Ok(release) => {
              last_error = None;
              remote_release = Some(release);
              break;
            }
            Err(err) => last_error = Some(err),
          }
        }
      }
    }

    if last_error.is_some() {
      bail!(crate::Error::Network, "Api Error: {:?}", last_error)
    }

    // Make sure we have remote release data (metadata)
    if remote_release.is_none() {
      bail!(crate::Error::Network, "Unable to extract remote metadata")
    }

    // Need to be mutable
    let mut final_release = remote_release
      .ok_or_else(|| crate::Error::Network("Unable to unwrap remote metadata".into()))?;

    // did the announced version is greated than our current one?
    let should_update = match version::is_greater(
      &self.current_version.as_ref().unwrap(),
      &final_release.version,
    ) {
      Ok(v) => v,
      Err(_) => false,
    };

    // save it for future reference
    final_release.should_update = should_update;

    // return our Update structure
    Ok(Box::new(Update {
      target,
      executable_path,
      extract_path,
      should_update,
      current_version: if let Some(ref ver) = self.current_version {
        ver.to_owned()
      } else {
        bail!(crate::Error::Config, "`current_version` required")
      },
      remote_release: final_release,
    }))
  }
}

/// Specific update data
#[derive(Debug)]
pub struct Update {
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
