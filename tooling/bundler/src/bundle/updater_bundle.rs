// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::common;
use libflate::gzip;
use walkdir::WalkDir;

#[cfg(target_os = "macos")]
use super::macos_bundle;

#[cfg(target_os = "linux")]
use super::appimage_bundle;

#[cfg(target_os = "windows")]
use super::msi_bundle;
#[cfg(target_os = "windows")]
use std::fs::File;
#[cfg(target_os = "windows")]
use std::io::prelude::*;
#[cfg(target_os = "windows")]
use zip::write::FileOptions;

use crate::{bundle::Bundle, Settings};
use std::{
  ffi::OsStr,
  fs::{self},
  io::Write,
};

use anyhow::Context;
use std::path::{Path, PathBuf};

// Build update
pub fn bundle_project(settings: &Settings, bundles: &[Bundle]) -> crate::Result<Vec<PathBuf>> {
  if cfg!(unix) || cfg!(windows) || cfg!(macos) {
    // Create our archive bundle
    let bundle_result = bundle_update(settings, bundles)?;
    Ok(bundle_result)
  } else {
    common::print_info("Current platform do not support updates")?;
    Ok(vec![])
  }
}

// Create simple update-macos.tar.gz
// This is the Mac OS App packaged
#[cfg(target_os = "macos")]
fn bundle_update(settings: &Settings, bundles: &[Bundle]) -> crate::Result<Vec<PathBuf>> {
  // find our .app or rebuild our bundle
  let bundle_path = match bundles
    .iter()
    .filter(|bundle| bundle.package_type == crate::PackageType::MacOsBundle)
    .find_map(|bundle| {
      bundle
        .bundle_paths
        .iter()
        .find(|path| path.extension() == Some(OsStr::new("app")))
    }) {
    Some(path) => vec![path.clone()],
    None => macos_bundle::bundle_project(settings)?,
  };

  // we expect our .app to be on bundle_path[0]
  if bundle_path.is_empty() {
    return Err(crate::Error::UnableToFindProject);
  }

  let source_path = &bundle_path[0];

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
fn bundle_update(settings: &Settings, bundles: &[Bundle]) -> crate::Result<Vec<PathBuf>> {
  // build our app actually we support only appimage on linux
  let bundle_path = match bundles
    .iter()
    .filter(|bundle| bundle.package_type == crate::PackageType::AppImage)
    .find_map(|bundle| {
      bundle
        .bundle_paths
        .iter()
        .find(|path| path.extension() == Some(OsStr::new("AppImage")))
    }) {
    Some(path) => vec![path.clone()],
    None => appimage_bundle::bundle_project(settings)?,
  };

  // we expect our .app to be on bundle[0]
  if bundle_path.is_empty() {
    return Err(crate::Error::UnableToFindProject);
  }

  let source_path = &bundle_path[0];

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
fn bundle_update(settings: &Settings, bundles: &[Bundle]) -> crate::Result<Vec<PathBuf>> {
  // find our .msi or rebuild
  let bundle_path = match bundles
    .iter()
    .filter(|bundle| bundle.package_type == crate::PackageType::WindowsMsi)
    .find_map(|bundle| {
      bundle
        .bundle_paths
        .iter()
        .find(|path| path.extension() == Some(OsStr::new("msi")))
    }) {
    Some(path) => vec![path.clone()],
    None => msi_bundle::bundle_project(settings)?,
  };

  // we expect our .msi to be on bundle_path[0]
  if bundle_path.is_empty() {
    return Err(crate::Error::UnableToFindProject);
  }

  let source_path = &bundle_path[0];

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

#[cfg(not(target_os = "windows"))]
fn create_tar(src_dir: &Path, dest_path: &Path) -> crate::Result<PathBuf> {
  let dest_file = common::create_file(&dest_path)?;
  let gzip_encoder = gzip::Encoder::new(dest_file)?;

  let gzip_encoder = create_tar_from_src(src_dir, gzip_encoder)?;
  let mut dest_file = gzip_encoder.finish().into_result()?;
  dest_file.flush()?;
  Ok(dest_path.to_owned())
}

#[cfg(not(target_os = "windows"))]
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
