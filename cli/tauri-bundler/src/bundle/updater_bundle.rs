use super::archive_utils;
use super::common;
#[cfg(target_os = "macos")]
use super::osx_bundle;

#[cfg(target_os = "linux")]
use super::appimage_bundle;

#[cfg(target_os = "windows")]
use super::msi_bundle;

use crate::Settings;

use crate::sign::{read_key_from_file, sign_file};
use anyhow::Context;
use std::env;
use std::fs::{self};
use std::path::{Path, PathBuf};

// Build update
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  if cfg!(unix) || cfg!(windows) || cfg!(macos) {
    // Create our archive bundle
    let bundle_result = bundle_update(settings)?;
    // Clone it we need it later to push into
    let mut bundle_result_return = bundle_result.clone();

    // Sign updater archive
    if settings.is_updater_pubkey() {
      let secret_key_password: Option<String>;
      let private_key: Option<String>;

      // Private key password
      match env::var_os("TAURI_KEY_PASSWORD") {
        Some(value) => {
          secret_key_password = Some(String::from(value.to_str().unwrap()));
        }
        None => secret_key_password = Some("".to_string()),
      }

      // make sure we have a private key available
      // Private key can be a path or a String
      match env::var_os("TAURI_PRIVATE_KEY") {
        Some(value) => {
          // check if this file exist..
          let pk_string = String::from(value.to_str().unwrap());
          let pk_dir = Path::new(&pk_string);

          if pk_dir.exists() {
            // read file content
            let pk_dir_content = read_key_from_file(pk_dir)?;
            private_key = Some(pk_dir_content);
          } else {
            private_key = Some(pk_string);
          }
        }
        None => private_key = None,
      }

      // Loop only if we have a private key
      if private_key.is_some() {
        for path_to_sign in &bundle_result {
          let (signature, _) = sign_file(
            private_key.clone().unwrap(),
            secret_key_password.clone().unwrap(),
            path_to_sign,
            false,
          )?;

          let mut added_buffer = PathBuf::new();
          added_buffer.push(signature);
          bundle_result_return.push(added_buffer);
        }
      } else {
        // Print output so they are aware of...
        common::print_warning("A public key has been found, but no private key. Make sure to set `TAURI_PRIVATE_KEY` environment variable.")?;
      }
    }
    Ok(bundle_result_return)
  } else {
    common::print_info("Current platform do not support updates")?;
    Ok(vec![])
  }
}

// Create simple update-macos.tar.gz
// This is the Mac OS App packaged without the .app
// The root folder should be Contents as we can't extract
// in /Applications directly, we NEED to extract in /Applications/<AppName>/
// this way the whole app manifest is replaced
#[cfg(target_os = "macos")]
fn bundle_update(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // build our app
  let osx_bundled = osx_bundle::bundle_project(settings)?;
  // we expect our .app to be on osx_bundled[0]
  if osx_bundled.len() < 1 {
    return Err(crate::Error::UpdateBundler);
  }

  let source_path = &osx_bundled[0];

  // add .tar.gz to our path
  let osx_archived = format!("{}.tar.gz", source_path.display());
  let osx_archived_path = PathBuf::from(&osx_archived);

  // Create our gzip file
  create_tar(&source_path, &osx_archived_path)
    .with_context(|| "Failed to tar.gz update directory")?;

  common::print_bundling(format!("{:?}", &osx_archived_path.clone()).as_str())?;
  Ok(vec![osx_archived_path])
}

// Create simple update-linux_<arch>.tar.gz
// Including the binary as root
// Right now in linux we hot replace the bin and request a restart
// No assets are replaced
#[cfg(target_os = "linux")]
fn bundle_update(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // build our app actually we support only appimage on linux
  let appimage_bundle = appimage_bundle::bundle_project(settings)?;
  // we expect our .app to be on osx_bundled[0]
  if appimage_bundle.len() < 1 {
    return Err(crate::Error::UpdateBundler);
  }

  let source_path = &appimage_bundle[0];

  // add .tar.gz to our path
  let appimage_archived = format!("{}.tar.gz", source_path.display());
  let appimage_archived_path = PathBuf::from(&appimage_archived);

  // Create our gzip file
  create_tar(&source_path, &appimage_archived_path)
    .with_context(|| "Failed to tar.gz update directory")?;

  common::print_bundling(format!("{:?}", &appimage_archived_path.clone()).as_str())?;
  Ok(vec![appimage_archived_path])
}

// Create simple update-win_<arch>.zip
// Including the binary as root
// Right now in windows we hot replace the bin and request a restart
// No assets are replaced
#[cfg(target_os = "windows")]
fn bundle_update(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // build our app actually we support only appimage on linux
  let msi_path = msi_bundle::bundle_project(settings)?;
  // we expect our .msi to be on msi_path[0]
  if msi_path.len() < 1 {
    return Err(crate::Error::UpdateBundler);
  }

  let source_path = &msi_path[0];

  // add .tar.gz to our path
  let msi_archived = format!("{}.zip", source_path.display());
  let msi_archived_path = PathBuf::from(&msi_archived);

  // Create our gzip file
  create_zip(&source_path, &msi_archived_path).with_context(|| "Failed to zip update MSI")?;

  common::print_bundling(format!("{:?}", &msi_archived_path.clone()).as_str())?;
  Ok(vec![msi_archived_path])
}

#[cfg(target_os = "windows")]
fn create_zip(source: &PathBuf, archive_path: &PathBuf) -> crate::Result<()> {
  archive_utils::zip_dir(source, archive_path).with_context(|| "Failed to zip update directory")?;

  if source.exists() {
    fs::remove_dir_all(&source).with_context(|| format!("Failed to remove tmp dir"))?;
  }
  Ok(())
}

fn create_tar(source: &PathBuf, archive_path: &PathBuf) -> crate::Result<()> {
  archive_utils::tar_and_gzip_to(source, archive_path)
    .with_context(|| "Failed to zip update directory")?;

  if source.exists() {
    fs::remove_dir_all(&source).with_context(|| format!("Failed to remove tmp dir"))?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std;
  use tauri_updater::verify_signature;
  use totems::assert_ok;

  #[test]
  fn updater_with_signature_bundling() {
    // load our main example
    let mut example_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    example_path.push("test");
    example_path.push("fixture");
    example_path.push("config");

    // Run cargo build in our test project
    std::process::Command::new("cargo")
      .arg("build")
      .current_dir(&example_path)
      .output()
      .expect("Failed to execute cargo build");

    // set our tauri dir to the example path
    std::env::set_var("TAURI_DIR", &example_path);

    // set our private key -- this can also be a file path
    std::env::set_var("TAURI_PRIVATE_KEY", "dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5dGlHbTEvRFhRRis2STdlTzF3eWhOVk9LNjdGRENJMnFSREE3R2V3b3Rwb0FBQkFBQUFBQUFBQUFBQUlBQUFBQWFNZEJTNXFuVjk0bmdJMENRRXVYNG5QVzBDd1NMOWN4Q2RKRXZxRDZNakw3Y241Vkt3aTg2WGtoajJGS1owV0ZuSmo4ZXJ0ZCtyaWF0RWJObFpnd1EveDB4NzBTU2RweG9ZaUpuc3hnQ3BYVG9HNnBXUW5SZ2Q3b3dvZ3Y2UnhQZ1BQZDU3bXl6d3M9Cg==");

    // create fake args
    let temp_args = clap::ArgMatches::new();

    // build our settings
    let settings =
      Settings::new(example_path, &temp_args).expect("Something went wrong when building settings");

    let project_bundle = bundle_project(&settings);

    assert_ok!(&project_bundle);

    let files = project_bundle.expect("Something went wrong when building and signing update");

    // we expect 2 files (archive + archive.sig)
    assert_eq!(files.len(), 2);

    // lets validate our files really exists
    for file in &files {
      assert_eq!(file.exists(), true);
    }

    // now we expect the the archive first and the sign second (archive is always created first..)
    // lets make sure our decryption works as well
    let signature = std::fs::read_to_string(&files[1]).expect("Something wrong with signature");

    // we load the function from our updater directly to make sure
    // it's compatible as we use a light version on the client side
    let signature_valid = verify_signature(
      &files[0],
      signature,
      &settings
        .updater_pubkey()
        .expect("Something wrong with pubkey"),
    );

    assert_ok!(signature_valid);
  }
}
