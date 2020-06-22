#[macro_use]
pub mod macros;
pub mod error;
pub use error::{Error, Result};

use base64::decode;
use minisign_verify::{PublicKey, Signature};
use reqwest::{self, header, StatusCode};
use std::{
  cmp::min,
  env,
  fs::{read_dir, remove_file, File, OpenOptions},
  io::{self, BufReader, Read},
  path::{Path, PathBuf},
  str::from_utf8,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri_api::{file::Extract, file::Move, version};

#[cfg(not(target_os = "macos"))]
use std::process::Command;

#[derive(Debug)]
pub struct RemoteRelease {
  pub version: String,
  pub date: String,
  pub download_url: String,
  pub body: Option<String>,
  pub signature: Option<String>,
  pub should_update: bool,
}

impl RemoteRelease {
  // Read JSON and confirm this is a valid Schema
  fn from_release(release: &serde_json::Value) -> Result<RemoteRelease> {
    let name = match &release["version"].is_null() {
      false => release["version"]
        .as_str()
        .expect("Can't extract remote version")
        .to_string(),
      true => release["name"]
        .as_str()
        .ok_or_else(|| crate::Error::Release("Release missing `name` or `version`".into()))?
        .to_string(),
    };

    let date = match &release["pub_date"].is_null() {
      false => release["pub_date"]
        .as_str()
        .expect("Can't extract pub_date version")
        .to_string(),
      true => "N/A".to_string(),
    };

    let url = release["url"]
      .as_str()
      .ok_or_else(|| crate::Error::Release("Release missing `name` or `url`".into()))?;

    let body = release["notes"].as_str().map(String::from);

    let signature = match &release["signature"].is_null() {
      false => Some(
        release["signature"]
          .as_str()
          .expect("Can't extract remote version")
          .to_string(),
      ),
      true => None,
    };

    // Return our formatted release
    Ok(RemoteRelease {
      signature,
      version: name.trim_start_matches('v').to_owned(),
      date,
      download_url: url.to_owned(),
      body,
      should_update: false,
    })
  }
}

#[derive(Debug)]
pub struct UpdateBuilder<'a> {
  pub current_version: &'a str,
  pub urls: Vec<String>,
  pub target: Option<String>,
  pub executable_path: Option<PathBuf>,
}

impl<'a> Default for UpdateBuilder<'a> {
  fn default() -> Self {
    UpdateBuilder {
      urls: Vec::new(),
      target: None,
      executable_path: None,
      // set version to current Cargo version
      current_version: env!("CARGO_PKG_VERSION"),
    }
  }
}

// Create new updater instance and return an Update
impl<'a> UpdateBuilder<'a> {
  pub fn new() -> Self {
    UpdateBuilder::default()
  }

  pub fn url(mut self, url: String) -> Self {
    self.urls.push(url);
    self
  }

  /// Add multiple URLS at once inside a Vec for future reference
  pub fn urls(mut self, urls: &[String]) -> Self {
    let mut formatted_vec: Vec<String> = Vec::new();
    for url in urls {
      formatted_vec.push(url.to_owned());
    }
    self.urls = formatted_vec;
    self
  }

  /// Set the current app version, used to compare against the latest available version.
  /// The `cargo_crate_version!` macro can be used to pull the version from your `Cargo.toml`
  pub fn current_version(mut self, ver: &'a str) -> Self {
    self.current_version = ver;
    self
  }

  /// Set the target (os)
  /// win32, win64, darwin and linux are currently supported
  pub fn target(mut self, target: &str) -> Self {
    self.target = Some(target.to_owned());
    self
  }

  /// Set the executable path
  pub fn executable_path<A: AsRef<Path>>(mut self, executable_path: A) -> Self {
    self.executable_path = Some(PathBuf::from(executable_path.as_ref()));
    self
  }

  pub fn build(self) -> Result<Update> {
    let mut remote_release: Option<RemoteRelease> = None;

    // make sure we have at least one url
    if self.urls.is_empty() {
      bail!(crate::Error::Config, "`url` required")
    };

    // set current version if not set
    let current_version = self.current_version;

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
      get_target().ok_or_else(|| crate::Error::Config("Unsuported operating system.".into()))?
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
        &str::replace(url, "{{current_version}}", &current_version),
        "{{target}}",
        &target,
      );

      // we want JSON only
      let mut headers = header::HeaderMap::new();
      headers.insert(header::ACCEPT, "application/json".parse().unwrap());

      let resp = reqwest::blocking::Client::new()
        .get(&fixed_link)
        .headers(headers)
        .timeout(Duration::from_secs(5))
        .send();

      // If we got a success, we stop the loop
      // and we set our remote_release variable
      if let Ok(ref res) = resp {
        // got status code 2XX
        if res.status().is_success() {
          match res.status() {
            StatusCode::NO_CONTENT => {
              // bail out...
              // already up to date
              // todo(lemarier): we should have error handler
              // on the client side who ignore these errors
              bail!(
                crate::Error::UpToDate,
                "Remote server announced StatusCode 204"
              )
            }
            _ => (),
          };

          let json = resp?.json::<serde_json::Value>()?;
          // Convert the remote result to our local struct
          let built_release = RemoteRelease::from_release(&json);
          // make sure all went well and the remote data is compatible
          // with what we need locally
          match built_release {
            Ok(release) => {
              last_error = None;
              remote_release = Some(release);
              break;
            }
            Err(err) => last_error = Some(err),
          }
        } // if status code is not 2XX we keep loopin' our urls
      }
    }

    // Last error is cleaned on success -- shouldn't be triggered if
    // we have a successful call
    if last_error.is_some() {
      bail!(crate::Error::Network, "Api Error: {:?}", last_error)
    }

    // Make sure we have remote release data (metadata)
    if remote_release.is_none() {
      bail!(crate::Error::Network, "Unable to extract remote metadata")
    }

    // Extracted remote metadata
    let final_release =
      remote_release.ok_or_else(|| crate::Error::Network("No remote release available".into()))?;

    // did the announced version is greated than our current one?
    let should_update = match version::is_greater(&current_version, &final_release.version) {
      Ok(v) => v,
      Err(_) => false,
    };

    // create our new updater
    Update::new(
      target,
      extract_path,
      should_update,
      final_release.version,
      final_release.date,
      final_release.download_url,
      final_release.body,
      final_release.signature,
    )
  }
}

pub fn builder<'a>() -> UpdateBuilder<'a> {
  UpdateBuilder::new()
}

// Once an update is available we return an Update instance
#[derive(Debug)]
pub struct Update {
  pub body: Option<String>,
  pub should_update: bool,
  pub version: String,
  pub date: String,
  target: String,
  extract_path: PathBuf,
  download_url: String,
  signature: Option<String>,
}

impl Update {
  fn new(
    target: String,
    extract_path: PathBuf,
    should_update: bool,
    version: String,
    date: String,
    download_url: String,
    body: Option<String>,
    signature: Option<String>,
  ) -> Result<Update> {
    Ok(Update {
      date,
      body,
      download_url,
      extract_path,
      should_update,
      signature,
      target,
      version,
    })
  }

  // Download and install our update
  // @todo(lemarier): Split into download and install (two step) but need to be thread safe
  pub fn download_and_install(&self, pub_key: Option<String>) -> Result {
    // get OS
    let target = self.target.clone();
    // download url for selected release
    let url = self.download_url.clone();
    // extract path
    let extract_path = self.extract_path.clone();

    // make sure we NEED to install it ...
    //current_version

    // make sure we can install the update on linux
    // We fail here because later we can add more linux support
    // actually if we use APPIMAGE, our extract path should already
    // be set with our APPIMAGE env variable, we don't need to do
    // anythin with it yet
    if target == "linux" {
      if let None = env::var_os("APPIMAGE") {
        bail!(
          Error::Config,
          "APPIMAGE env is not defined -- updates are not supported."
        )
      }
    }

    // tmp dir
    let tmp_dir_parent = if cfg!(windows) {
      env::var_os("TEMP").map(PathBuf::from)
    } else {
      extract_path.parent().map(PathBuf::from)
    }
    .ok_or_else(|| crate::Error::Updater("Failed to determine parent dir".into()))?;

    // used for temp file name
    // if we cant extract app name, we use unix epoch duration
    let current_time = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("Unable to get Unix Epoch")
      .subsec_nanos()
      .to_string();

    // get the current app name
    let bin_name = std::env::current_exe()
      .ok()
      .and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
      .and_then(|s| s.into_string().ok())
      .unwrap_or(current_time);

    // tmp dir for extraction
    let tmp_dir = tempfile::Builder::new()
      .prefix(&format!("{}_download", bin_name))
      .tempdir_in(tmp_dir_parent)?;

    // tmp directories are used to create backup of current application
    // if something goes wrong, we can restore to previous state
    let tmp_archive_path = tmp_dir.path().join(detect_archive_in_url(&url, &target));
    let tmp_archive = File::create(&tmp_archive_path)?;

    // prepare our download
    use io::BufRead;
    use std::io::Write;

    // set our headers
    let mut headers = header::HeaderMap::new();
    headers.insert(header::ACCEPT, "application/octet-stream".parse().unwrap());

    // make sure we have a valid agent
    if !headers.contains_key(header::USER_AGENT) {
      headers.insert(
        header::USER_AGENT,
        "tauri/updater".parse().expect("invalid user-agent"),
      );
    }

    // Set SSL for linux
    set_ssl_vars!();

    // Create our request
    let resp = reqwest::blocking::Client::new()
      .get(&url)
      .headers(headers)
      .send()?;

    // Calculate size (percentage done)
    let size = resp
      .headers()
      .get(reqwest::header::CONTENT_LENGTH)
      .map(|val| {
        val
          .to_str()
          .map(|s| s.parse::<u64>().unwrap_or(0))
          .unwrap_or(0)
      })
      .unwrap_or(0);

    // make sure it's success
    if !resp.status().is_success() {
      bail!(
        crate::Error::Updater,
        "Download request failed with status: {:?}",
        resp.status()
      )
    }

    // Init the buffer
    let mut src = io::BufReader::new(resp);
    let mut downloaded = 0;
    let mut dest = &tmp_archive;

    // Download file
    loop {
      let n = {
        let buf = src.fill_buf()?;
        dest.write_all(&buf)?;
        buf.len()
      };
      if n == 0 {
        break;
      }
      src.consume(n);
      // calc the progress
      downloaded = min(downloaded + n as u64, size);

      // TODO: FIX LOOP TO SEND PERCENTAGE
      //let percent = (downloaded * 100) / size;
      //println!("{}", percent);
    }

    // Validate signature ONLY if pubkey is available in tauri.conf.json
    if let Some(pub_key) = pub_key {
      // we need an announced signature by the server
      // if there is no signature, bail out.
      if self.signature.is_none() {
        bail!(
          crate::Error::Updater,
          "Signature not available but pubkey provided, skipping update"
        )
      }

      // we make sure the archive is valid and signed with the private key linked with the publickey
      verify_signature(
        &tmp_archive_path,
        self.signature.clone().expect("Can't validate signature"),
        &pub_key,
      )?;
    }

    // extract using tauri api inside a tmp path
    Extract::from_source(&tmp_archive_path).extract_into(&tmp_dir.path())?;
    // Remove archive (not needed anymore)
    remove_file(&tmp_archive_path)?;
    // we copy the files depending of the operating system
    // we run the setup, appimage re-install or overwrite the
    // macos .app
    copy_files_and_run(tmp_dir, extract_path)?;

    Ok(())
  }
}

// We should have an AppImage already installed to be able to copy and install
// the extract_path is the current AppImage path
// tmp_dir is where our new AppImage is found
#[cfg(target_os = "linux")]
fn copy_files_and_run(tmp_dir: tempfile::TempDir, extract_path: PathBuf) -> Result {
  // we delete our current AppImage (we'll create a new one later)
  remove_file(&extract_path)?;

  // In our tempdir we expect 1 directory (should be the <app>.app)
  let paths = read_dir(&tmp_dir).unwrap();

  for path in paths {
    let found_path = path.expect("Unable to extract").path();
    // make sure it's our .AppImage
    if found_path.display().to_string().contains(".AppImage") {
      // Simply overwrite our AppImage (we use the command)
      // because it prevent failing of bytes stream
      Command::new("mv")
        .arg("-f")
        .arg(&found_path)
        .arg(&extract_path)
        .output()?;

      // We now run the AppImage install process
      Command::new(&extract_path)
        .env("APPIMAGE_SILENT_INSTALL", "true")
        .env("APPIMAGE_EXIT_AFTER_INSTALL", "true")
        .output()?;

      // early finish we have everything we need here
      return Ok(());
    }
  }

  Ok(())
}

// We should have a setup (msi or exe)
// we run it -- thats it.
#[cfg(target_os = "windows")]
fn copy_files_and_run(tmp_dir: tempfile::TempDir, _extract_path: PathBuf) -> Result {
  for path in paths {
    let found_path = path.expect("Unable to extract").path();
    // make sure it's our .msi our .exe
    if found_path.display().to_string().contains(".exe")
      || found_path.display().to_string().contains(".msi")
    {
      // Simply overwrite our AppImage (we use the command)
      // because it prevent failing of bytes stream
      Command::new(found_path)
        // To be confirmed with WIX if we can build custom args
        // by example we can pass the _extract_path to tell exactly
        // where to install the app...
        .arg("/S")
        .arg("--updated")
        .arg("--force")
        .output()?;

      // early finish we have everything we need here
      return Ok(());
    }
  }

  Ok(())
}

// We should have a .app example, Tauri.app
// we need to skip Tauri.app and extract eveything in
// extract_path (who should equal to) example
// /Applications/Tauri/ (as we can't extract directly)
// in /Applications  (directory are automatically extracted)
#[cfg(target_os = "macos")]
fn copy_files_and_run(tmp_dir: tempfile::TempDir, extract_path: PathBuf) -> Result {
  // Create our temp file -- we'll copy a backup of our destination before copying'
  //let backup_dir = tmp_dir.path().join("__backup");

  // In our tempdir we expect 1 directory (should be the <app>.app)
  let paths = read_dir(&tmp_dir).unwrap();

  for path in paths {
    let found_path = path.expect("Unable to extract").path();
    // make sure it's our .app
    if found_path.display().to_string().contains(".app") {
      // Walk the temp dir and copy all files by replacing existing files only
      // and creating directories if needed
      Move::from_source(&found_path)
        // BACKUPING FILES MAY CAUSE ISSUE..
        // DISABLED FOR NOW
        //.replace_using_temp(&tmp_file)
        .walk_to_dest(&extract_path)?;

      // early finish we have everything we need here
      return Ok(());
    }
  }

  Ok(())
}

/// Returns a target os -- If none,
/// that mean the updater didnt support
/// the platform
pub fn get_target() -> Option<String> {
  if cfg!(target_os = "linux") {
    Some("linux".into())
  } else if cfg!(target_os = "macos") {
    Some("darwin".into())
  } else if cfg!(target_os = "windows") {
    if cfg!(target_pointer_width = "32") {
      Some("win32".into())
    } else {
      Some("win64".into())
    }
  } else if cfg!(target_os = "freebsd") {
    Some("freebsd".into())
  } else {
    None
  }
}

/// Get the extract_path from the provided executable_path
pub fn extract_path_from_executable(executable_path: &PathBuf, target: &str) -> PathBuf {
  // Linux & Windows should need to be extracted in the same directory as the executable
  // C:\Program Files\MyApp\MyApp.exe
  // We need C:\Program Files\MyApp

  let mut extract_path = executable_path
    .parent()
    .map(PathBuf::from)
    .expect("Can't determine extract path");

  let extract_path_as_string = extract_path.display().to_string();

  // MacOS example binary is in /Applications/TestApp.app/Contents/MacOS/myApp
  // We need to get /Applications/TestApp.app
  // todo(lemarier): Need a better way here
  // Maybe we could search for <*.app> to get the right path
  if target == "darwin" && extract_path_as_string.contains("Contents/MacOS") {
    extract_path = extract_path
      .parent()
      .map(PathBuf::from)
      .expect("Unable to find the extract path")
      .parent()
      .map(PathBuf::from)
      .expect("Unable to find the extract path")
  };

  // We should use APPIMAGE exposed env variable
  // This is where our APPIMAGE should sit and should be replaced
  if target == "linux" && env::var_os("APPIMAGE").is_some() {
    extract_path = PathBuf::from(env::var_os("APPIMAGE").expect("Unable to extract APPIMAGE path"))
  }

  extract_path
}

// Return the archive type to save on disk
fn detect_archive_in_url(path: &str, target: &str) -> String {
  path
    .split('/')
    .next_back()
    .unwrap_or(&archive_name_by_os(target))
    .to_string()
}

// Fallback archive name by os
// The main objective is to provide the right extension based on the target
// if we cant extract the archive type in the url we'll fallback to this value
fn archive_name_by_os(target: &str) -> String {
  let archive_name = match target {
    "darwin" | "linux" => "update.tar.gz",
    _ => "update.zip",
  };
  archive_name.to_string()
}

// Convert base64 to string and prevent failing
fn base64_to_string(base64_string: &str) -> crate::Result<String> {
  let decoded_string = &decode(base64_string.to_owned())?;
  let result = from_utf8(&decoded_string)?.to_string();
  Ok(result)
}

// Validate signature
// need to be public because its been used
// by our tests in the bundler
pub fn verify_signature(
  archive_path: &PathBuf,
  release_signature: String,
  pub_key: &str,
) -> crate::Result<bool> {
  // we need to convert the pub key
  let pub_key_decoded = &base64_to_string(pub_key)?;
  let public_key = PublicKey::decode(pub_key_decoded);
  if public_key.is_err() {
    bail!(
      crate::Error::Updater,
      "Something went wrong with pubkey decode"
    )
  }

  let public_key_ready = public_key.expect("Something wrong with the public key");

  let signature_decoded = base64_to_string(&release_signature)?;

  let signature =
    Signature::decode(&signature_decoded).expect("Something wrong with the signature");

  // We need to open the file and extract the datas to make sure its not corrupted
  let file_open = OpenOptions::new().read(true).open(&archive_path)?;
  let mut file_buff: BufReader<File> = BufReader::new(file_open);

  // read all bytes since EOF in the buffer
  let mut data = vec![];
  file_buff.read_to_end(&mut data)?;

  // Validate signature or bail out
  public_key_ready.verify(&data, &signature)?;
  Ok(true)
}

#[cfg(test)]
mod test {
  use super::*;
  use std::env::current_exe;
  use std::path::Path;
  use totems::{assert_err, assert_ok};

  #[test]
  fn simple_http_updater() {
    let check_update = builder()
    .current_version("0.0.0")
    .url("https://gist.githubusercontent.com/lemarier/72a2a488f1c87601d11ec44d6a7aff05/raw/f86018772318629b3f15dbb3d15679e7651e36f6/with_sign.json".into())
    .build();

    assert_ok!(check_update);
    let updater = check_update.expect("Can't check update");

    assert_eq!(updater.should_update, true);
  }

  #[test]
  fn simple_http_updater_without_version() {
    let check_update = builder()
    .url("https://gist.githubusercontent.com/lemarier/72a2a488f1c87601d11ec44d6a7aff05/raw/f86018772318629b3f15dbb3d15679e7651e36f6/with_sign.json".into())
    .build();

    assert_ok!(check_update);
    let updater = check_update.expect("Can't check update");

    assert_eq!(updater.should_update, false);
  }

  #[test]
  fn http_updater_uptodate() {
    let check_update = builder()
    .current_version("10.0.0")
    .url("https://gist.githubusercontent.com/lemarier/72a2a488f1c87601d11ec44d6a7aff05/raw/f86018772318629b3f15dbb3d15679e7651e36f6/with_sign.json".into())
    .build();

    assert_ok!(check_update);
    let updater = check_update.expect("Can't check update");

    assert_eq!(updater.should_update, false);
  }

  #[test]
  fn http_updater_fallback_urls() {
    let check_update = builder()
    .url("http://badurl.www.tld/1".into())
    .url("https://gist.githubusercontent.com/lemarier/72a2a488f1c87601d11ec44d6a7aff05/raw/f86018772318629b3f15dbb3d15679e7651e36f6/with_sign.json".into())
    .current_version("0.0.1")
    .build();

    assert_ok!(check_update);
    let updater = check_update.expect("Can't check remote update");

    assert_eq!(updater.should_update, true);
  }

  #[test]
  fn http_updater_fallback_urls_withs_array() {
    let check_update = builder()
    .urls(&["http://badurl.www.tld/1".into(), "https://gist.githubusercontent.com/lemarier/72a2a488f1c87601d11ec44d6a7aff05/raw/f86018772318629b3f15dbb3d15679e7651e36f6/with_sign.json".into()])
    .current_version("0.0.1")
    .build();

    assert_ok!(check_update);
    let updater = check_update.expect("Can't check remote update");

    assert_eq!(updater.should_update, true);
  }

  #[test]
  fn http_updater_missing_remote_data() {
    let check_update = builder()
    .url("https://gist.githubusercontent.com/lemarier/106011e4a5610ef5671af15ce2f78862/raw/d4dd4fa30b9112836e0a201fd3a867446e7bac98/test.json".into())
    .current_version("0.0.1")
    .build();

    assert_err!(check_update);
  }

  #[test]
  fn http_updater_complete_process() {
    // Test pubkey generated with tauri-bundler
    let pubkey_test = Some("dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEY1OTgxQzc0MjVGNjM0Q0IKUldUTE5QWWxkQnlZOWFBK21kekU4OGgzdStleEtkeStHaFR5NjEyRHovRnlUdzAwWGJxWEU2aGYK".into());

    // Build a tmpdir so we can test our extraction inside
    // We dont want to overwrite our current executable or the directory
    // Otherwise tests are failing...
    let executable_path = current_exe().expect("Can't extract executable path");
    let parent_path = executable_path
      .parent()
      .expect("Can't find the parent path");

    let tmp_dir = tempfile::Builder::new()
      .prefix("tauri_updater_test")
      .tempdir_in(parent_path);

    assert_ok!(&tmp_dir);
    let tmp_dir_unwrap = tmp_dir.expect("Can't find tmp_dir");
    let tmp_dir_path = tmp_dir_unwrap.path();

    // configure the updater
    let check_update = builder()
    .url("https://gist.githubusercontent.com/lemarier/72a2a488f1c87601d11ec44d6a7aff05/raw/ce9764167cf8bd8f3bab9ca8e7846a0b99c4890d/with_sign.json".into())
    // It should represent the executable path, that's why we add my_app.exe in our
    // test path -- in production you shouldn't have to provide it
    .executable_path(&tmp_dir_path.join("my_app.exe"))
    // make sure we force an update
    .current_version("0.0.1")
    .build();

    // make sure the process worked
    assert_ok!(check_update);

    // unwrap our results
    let updater = check_update.expect("Can't check remote update");

    // make sure we need to update
    assert_eq!(updater.should_update, true);
    // make sure we can read announced version
    assert_eq!(updater.version, "0.0.4");

    // download, install and validate signature
    let install_process = updater.download_and_install(pubkey_test);
    assert_ok!(&install_process);

    // make sure the extraction went well (it should have skipped the main app.app folder)
    // as we can't extract in /Applications directly
    let bin_file = tmp_dir_path.join("Contents").join("MacOS").join("app");
    let bin_file_exist = Path::new(&bin_file).exists();
    assert_eq!(bin_file_exist, true);
  }
}
