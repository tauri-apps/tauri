use super::common;
use libflate::gzip;
use walkdir::WalkDir;

#[cfg(target_os = "macos")]
use super::osx_bundle;

#[cfg(target_os = "linux")]
use super::appimage_bundle;

#[cfg(target_os = "windows")]
use super::msi_bundle;
#[cfg(target_os = "windows")]
use zip::write::FileOptions;
#[cfg(target_os = "windows")]
use std::fs::File;
#[cfg(target_os = "windows")]
use std::io::prelude::*;

use std::fs::{self};
use std::io::Write;
use crate::Settings;

use crate::sign::{read_key_from_file, sign_file};
use anyhow::Context;
use std::env;
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
  if osx_bundled.is_empty() {
    return Err(crate::Error::UpdateBundler);
  }

  let source_path = &osx_bundled[0];

  // add .tar.gz to our path
  let osx_archived = format!("{}.tar.gz", source_path.display());
  let osx_archived_path = PathBuf::from(&osx_archived);

  // safe unwrap
  //let tar_source = &source_path.parent().unwrap().to_path_buf();

  // Create our gzip file (need to send parent)
  // as we walk the source directory (source isnt added)
  create_tar(&source_path, &osx_archived_path)
    .with_context(|| "Failed to tar.gz update directory")?;

  common::print_bundling(format!("{:?}", &osx_archived_path).as_str())?;
  Ok(vec![osx_archived_path])
}

// Create simple update-linux_<arch>.tar.gz
// Including the AppImage
// Right now in linux we hot replace the bin and request a restart
// No assets are replaced
#[cfg(target_os = "linux")]
fn bundle_update(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  // build our app actually we support only appimage on linux
  let appimage_bundle = appimage_bundle::bundle_project(settings)?;
  // we expect our .app to be on osx_bundled[0]
  if appimage_bundle.is_empty() {
    return Err(crate::Error::UpdateBundler);
  }

  let source_path = &appimage_bundle[0];

  // add .tar.gz to our path
  let appimage_archived = format!("{}.tar.gz", source_path.display());
  let appimage_archived_path = PathBuf::from(&appimage_archived);

  // Create our gzip file
  create_tar(&source_path, &appimage_archived_path)
    .with_context(|| "Failed to tar.gz update directory")?;

  common::print_bundling(format!("{:?}", &appimage_archived_path).as_str())?;
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

  common::print_bundling(format!("{:?}", &msi_archived_path).as_str())?;
  Ok(vec![msi_archived_path])
}

#[cfg(target_os = "windows")]
pub fn create_zip(src_file: &PathBuf, dst_file: &PathBuf) -> crate::Result<PathBuf> {
  let parent_dir = dst_file.parent().expect("No data in parent");
  fs::create_dir_all(parent_dir)?;
  let writer = common::create_file(dst_file)?;

  let file_name = src_file
    .file_name()
    .expect("Can't extract file name from path");

  let mut zip = zip::ZipWriter::new(writer);
  let options = FileOptions::default()
    .compression_method(zip::CompressionMethod::Stored)
    .unix_permissions(0o755);

  zip.start_file(file_name.to_string_lossy(), options)?;
  let mut f = File::open(src_file)?;
  let mut buffer = Vec::new();
  f.read_to_end(&mut buffer)?;
  zip.write_all(&*buffer)?;
  buffer.clear();

  Ok(dst_file.to_owned())
}

fn create_tar(src_dir: &PathBuf, dest_path: &PathBuf) -> crate::Result<PathBuf> {
  let dest_file = common::create_file(&dest_path)?;
  let gzip_encoder = gzip::Encoder::new(dest_file)?;

  let gzip_encoder = create_tar_from_src(src_dir, gzip_encoder)?;
  let mut dest_file = gzip_encoder.finish().into_result()?;
  dest_file.flush()?;
  Ok(dest_path.to_owned())
}

fn create_tar_from_src<P: AsRef<Path>, W: Write>(src_dir: P, dest_file: W) -> crate::Result<W> {
  let src_dir = src_dir.as_ref();
  let mut tar_builder = tar::Builder::new(dest_file);

  // validate source type
  let file_type = fs::metadata(src_dir).expect("Can't read source directory");
  // if it's a file don't need to walkdir
  if file_type.is_file() {
    let mut src_file = fs::File::open(src_dir)?;
    let file_name = src_dir
      .file_name()
      .expect("Can't extract file name from path");

    tar_builder.append_file(file_name, &mut src_file)?;
  } else {
    for entry in WalkDir::new(&src_dir) {
      let entry = entry?;
      let src_path = entry.path();
      if src_path == src_dir {
        continue;
      }

      // todo(lemarier): better error catching
      // We add the .parent() because example if we send a path
      // /dev/src-tauri/target/debug/bundle/osx/app.app
      // We need a tar with app.app/<...> (source root folder should be included)
      let dest_path = src_path.strip_prefix(&src_dir.parent().expect(""))?;
      if entry.file_type().is_dir() {
        tar_builder.append_dir(dest_path, src_path)?;
      } else {
        let mut src_file = fs::File::open(src_path)?;
        tar_builder.append_file(dest_path, &mut src_file)?;
      }
    }
  }
  let dest_file = tar_builder.into_inner()?;
  Ok(dest_file)
}
