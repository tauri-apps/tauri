use std::env;
use std::path::PathBuf;

#[macro_use]
pub mod macros;
pub mod error;
pub mod http;
pub mod updater;
pub use error::{Error, Result};

/// Release information
#[derive(Clone, Debug, Default)]
pub struct Release {
  pub version: String,
  pub date: String,
  pub download_url: String,
  pub body: Option<String>,
  pub signature: Option<String>,
  pub should_update: bool,
}

impl Release {
  pub fn get_download_url(&self) -> String {
    self.download_url.clone()
  }
}

#[derive(Debug)]
pub enum CheckStatus {
  UpToDate,
  UpdateAvailable(Release),
}

#[derive(Debug)]
pub enum InstallStatus {
  Installed,
  Failed,
}

#[derive(Debug)]
pub enum ProgressStatus {
  Download(u64),
  Extract,
  CopyFiles,
}

#[derive(Debug)]
pub enum DownloadStatus {
  Downloaded(DownloadedArchive),
  Failed,
}

#[derive(Debug)]
pub struct DownloadedArchive {
  bin_name: String,
  archive_path: PathBuf,
  tmp_dir: tempfile::TempDir,
}

/// Returns a target os
pub fn get_target() -> &'static str {
  if cfg!(target_os = "linux") {
    "linux"
  } else if cfg!(target_os = "macos") {
    "darwin"
  } else if cfg!(target_os = "windows") {
    if env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap() == "64" {
      return "win64";
    }
    "win32"
  } else if cfg!(target_os = "freebsd") {
    "freebsd"
  } else {
    ""
  }
}


#[cfg(test)]
mod test {
  use super::*;
  use env::current_exe;
  use std::path::Path;
  use totems::{assert_err, assert_ok};

  #[test]
  fn simple_http_updater() {
    let check_update = http::Update::configure()
    .url("https://gist.githubusercontent.com/lemarier/72a2a488f1c87601d11ec44d6a7aff05/raw/f86018772318629b3f15dbb3d15679e7651e36f6/with_sign.json")
    .current_version("0.0.0")
    .check();

    assert_ok!(check_update);

    let updater = check_update.unwrap();
    let mut is_processed = false;

    match updater.status() {
      CheckStatus::UpdateAvailable(_) => is_processed = true,
      CheckStatus::UpToDate => (),
    }

    assert_eq!(is_processed, true);
  }

  #[test]
  fn http_updater_uptodate() {
    let check_update = http::Update::configure()
    .url("https://gist.githubusercontent.com/lemarier/72a2a488f1c87601d11ec44d6a7aff05/raw/f86018772318629b3f15dbb3d15679e7651e36f6/with_sign.json")
    .current_version("10.0.0")
    .check();

    assert_ok!(check_update);

    let updater = check_update.unwrap();
    let mut is_processed = false;

    match updater.status() {
      CheckStatus::UpdateAvailable(_) => (),
      CheckStatus::UpToDate => is_processed = true,
    }

    assert_eq!(is_processed, true);
  }

  #[test]
  fn http_updater_fallback_urls() {
    let check_update = http::Update::configure()
    .url("http://badurl.www.tld/1")
    .url("https://gist.githubusercontent.com/lemarier/72a2a488f1c87601d11ec44d6a7aff05/raw/f86018772318629b3f15dbb3d15679e7651e36f6/with_sign.json")
    .current_version("0.0.1")
    .check();

    assert_ok!(check_update);

    let updater = check_update.unwrap();
    let mut is_processed = false;

    match updater.status() {
      CheckStatus::UpdateAvailable(_) => is_processed = true,
      CheckStatus::UpToDate => (),
    }

    assert_eq!(is_processed, true);
  }

  #[test]
  fn http_updater_fallback_urls_withs_array() {
    let check_update = http::Update::configure()
    .urls(&["http://badurl.www.tld/1", "https://gist.githubusercontent.com/lemarier/72a2a488f1c87601d11ec44d6a7aff05/raw/f86018772318629b3f15dbb3d15679e7651e36f6/with_sign.json"])
    .current_version("0.0.1")
    .check();

    assert_ok!(check_update);

    let updater = check_update.unwrap();
    let mut is_processed = false;

    match updater.status() {
      CheckStatus::UpdateAvailable(_) => is_processed = true,
      CheckStatus::UpToDate => (),
    }

    assert_eq!(is_processed, true);
  }

  #[test]
  fn http_updater_missing_remote_data() {
    let check_update = http::Update::configure()
    .url("https://gist.githubusercontent.com/lemarier/106011e4a5610ef5671af15ce2f78862/raw/d4dd4fa30b9112836e0a201fd3a867446e7bac98/test.json")
    .current_version("0.0.1")
    .check();

    assert_err!(check_update);
  }

  #[test]
  fn http_updater_complete_process() {
    // Test pubkey generated with tauri-bundler
    let pubkey_test = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEY1OTgxQzc0MjVGNjM0Q0IKUldUTE5QWWxkQnlZOWFBK21kekU4OGgzdStleEtkeStHaFR5NjEyRHovRnlUdzAwWGJxWEU2aGYK";

    let mut is_processed = false;

    // Build a tmpdir so we can test our extraction inside
    // We dont want to overwrite our current executable or the directory
    // Otherwise tests are failing...

    let executable_path = current_exe().unwrap();
    let parent_path = executable_path.parent().unwrap();

    let tmp_dir = tempfile::Builder::new()
      .prefix("tauri_updater_test")
      .tempdir_in(parent_path);

    assert_ok!(&tmp_dir);
    let tmp_dir_unwrap = tmp_dir.unwrap();
    let tmp_dir_path = tmp_dir_unwrap.path();

    // Configure our updater
    let check_update = http::Update::configure()
    .url("https://gist.githubusercontent.com/lemarier/72a2a488f1c87601d11ec44d6a7aff05/raw/f86018772318629b3f15dbb3d15679e7651e36f6/with_sign.json")
    .executable_path(&tmp_dir_path.join("my_app.exe"))
    .current_version("0.0.1")
    .check();

    // Make sure we got OK
    assert_ok!(check_update);
    let updater = check_update.unwrap();

    match updater.status() {
      CheckStatus::UpdateAvailable(my_release) => {
        // should have a new version
        assert_eq!(my_release.version, "0.0.4");

        // Download our app
        let download_process = updater.download();
        assert_ok!(download_process);

        // Get our download status
        match download_process.unwrap() {
          // When download is completed
          DownloadStatus::Downloaded(extracted_archive) => {
            // Start the install

            let install_status = updater.install(extracted_archive, Some(pubkey_test));
            assert_ok!(install_status);

            match install_status.unwrap() {
              // if installation went successfully...
              InstallStatus::Installed => {
                // make sure the extraction went well
                let bin_file = tmp_dir_path.join("Contents").join("MacOS").join("app");
                let bin_file_exist = Path::new(&bin_file).exists();
                assert_eq!(bin_file_exist, true);

                is_processed = true
              }
              // if something went wrong inside the installation
              InstallStatus::Failed => (),
            }
          }
          // An error while downloading the announced archive
          DownloadStatus::Failed => (),
        }
      }
      // App is already up to date
      CheckStatus::UpToDate => (),
    }

    // Make sure we got a valid bundle and everything went well
    assert_eq!(is_processed, true);
  }
}
