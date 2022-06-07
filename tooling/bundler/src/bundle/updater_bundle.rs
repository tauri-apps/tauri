// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::common;

#[cfg(target_os = "macos")]
use super::macos::app;

#[cfg(target_os = "linux")]
use super::linux::appimage;

#[cfg(target_os = "windows")]
use super::windows::msi;
use log::error;
#[cfg(target_os = "windows")]
use std::{fs::File, io::prelude::*};
#[cfg(target_os = "windows")]
use zip::write::FileOptions;

use crate::{bundle::Bundle, Settings};
use anyhow::Context;
use log::info;
use std::io::Write;
use std::path::{Path, PathBuf};

// Build update
pub fn bundle_project(settings: &Settings, bundles: &[Bundle]) -> crate::Result<Vec<PathBuf>> {
  if cfg!(unix) || cfg!(windows) || cfg!(macos) {
    // Create our archive bundle
    let bundle_result = bundle_update(settings, bundles)?;
    Ok(bundle_result)
  } else {
    error!("Current platform do not support updates");
    Ok(vec![])
  }
}

// Create simple update-macos.tar.gz
// This is the Mac OS App packaged
#[cfg(target_os = "macos")]
fn bundle_update(settings: &Settings, bundles: &[Bundle]) -> crate::Result<Vec<PathBuf>> {
  use std::ffi::OsStr;

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
    None => app::bundle_project(settings)?,
  };

  // we expect our .app to be on bundle_path[0]
  if bundle_path.is_empty() {
    return Err(crate::Error::UnableToFindProject);
  }

  let source_path = &bundle_path[0];

  // add .tar.gz to our path
  let osx_archived = format!("{}.tar.gz", source_path.display());
  let osx_archived_path = PathBuf::from(&osx_archived);

  // Create our gzip file (need to send parent)
  // as we walk the source directory (source isnt added)
  create_tar(source_path, &osx_archived_path)
    .with_context(|| "Failed to tar.gz update directory")?;

  info!(action = "Bundling"; "{} ({})", osx_archived, osx_archived_path.display());

  Ok(vec![osx_archived_path])
}

// Create simple update-linux_<arch>.tar.gz
// Including the AppImage
// Right now in linux we hot replace the bin and request a restart
// No assets are replaced
#[cfg(target_os = "linux")]
fn bundle_update(settings: &Settings, bundles: &[Bundle]) -> crate::Result<Vec<PathBuf>> {
  use std::ffi::OsStr;

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
    None => appimage::bundle_project(settings)?,
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
  create_tar(source_path, &appimage_archived_path)
    .with_context(|| "Failed to tar.gz update directory")?;

  info!(action = "Bundling"; "{} ({})", appimage_archived, appimage_archived_path.display());

  Ok(vec![appimage_archived_path])
}

// Create simple update-win_<arch>.zip
// Including the binary as root
// Right now in windows we hot replace the bin and request a restart
// No assets are replaced
#[cfg(target_os = "windows")]
fn bundle_update(settings: &Settings, bundles: &[Bundle]) -> crate::Result<Vec<PathBuf>> {
  // find our .msi or rebuild
  let mut bundle_paths = bundles
    .iter()
    .find(|bundle| bundle.package_type == crate::PackageType::WindowsMsi)
    .map(|bundle| bundle.bundle_paths.clone())
    .unwrap_or_default();

  // we expect our .msi files to be on `bundle_paths`
  if bundle_paths.is_empty() {
    bundle_paths.extend(msi::bundle_project(settings)?);
  }

  let mut msi_archived_paths = Vec::new();

  for source_path in bundle_paths {
    // add .zip to our path
    let msi_archived = format!("{}.zip", source_path.display());
    let msi_archived_path = PathBuf::from(&msi_archived);

    info!(action = "Bundling"; "{} ({})", msi_archived, msi_archived_path.display());

    // Create our gzip file
    create_zip(&source_path, &msi_archived_path).with_context(|| "Failed to zip update MSI")?;

    msi_archived_paths.push(msi_archived_path);
  }

  Ok(msi_archived_paths)
}

#[cfg(target_os = "windows")]
pub fn create_zip(src_file: &Path, dst_file: &Path) -> crate::Result<PathBuf> {
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
  let dest_file = common::create_file(dest_path)?;
  let gzip_encoder = libflate::gzip::Encoder::new(dest_file)?;

  let mut builder = tar::Builder::new(gzip_encoder);
  builder.follow_symlinks(false);
  builder.append_dir_all(src_dir.file_name().expect("Path has no file_name"), src_dir)?;
  let gzip_encoder = builder.into_inner()?;

  let mut dest_file = gzip_encoder.finish().into_result()?;
  dest_file.flush()?;
  Ok(dest_path.to_owned())
}
