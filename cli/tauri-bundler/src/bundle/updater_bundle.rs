use super::archive_utils;
use super::common;
#[cfg(target_os = "macos")]
use super::osx_bundle;
use crate::Settings;

use anyhow::Context;
use std::fs::{self};
use std::path::PathBuf;

// Build update
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  if cfg!(unix) || cfg!(windows) || cfg!(macos) {
    bundle_update(settings)
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
  osx_bundle::bundle_project(settings)?;
  let app_name = settings.bundle_name();

  // dest
  let output_path = settings.project_out_directory().join("bundle/updater");
  let update_name = "update-macos.tar.gz";
  let update_path = output_path.join(&update_name.clone());

  // source
  let bundle_name = &format!("{}.app", app_name);
  let bundle_dir = settings.project_out_directory().join("bundle/osx");
  let bundle_path = bundle_dir.join(&bundle_name.clone());

  if output_path.exists() {
    fs::remove_dir_all(&output_path)
      .with_context(|| format!("Failed to remove old {}", update_name))?;
  }

  archive_utils::tar_and_gzip_to(&bundle_path, &update_path)
    .with_context(|| "Failed to tar/gzip update directory")?;

  common::print_bundling(format!("{:?}", update_path.clone()).as_str())?;
  Ok(vec![update_path])
}

// Create simple update-linux_<arch>.tar.gz
// Including the binary as root
// Right now in linux we hot replace the bin and request a restart
// No assets are replaced
#[cfg(target_os = "linux")]
fn bundle_update(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  let arch = match settings.binary_arch() {
    "x86" => "i386",
    "x86_64" => "amd64",
    other => other,
  };

  let update_name = format!("update-linux_{}.tar.gz", arch);

  // copy bin in a tmp folder then tar.gz this folder
  let package_dir = settings.project_out_directory().join("bundle/linux");
  let binary_dir = package_dir.join(settings.binary_name());

  if package_dir.exists() {
    fs::remove_dir_all(&package_dir)
      .with_context(|| format!("Failed to remove old `bundle/linux`"))?;
  }

  common::copy_file(settings.binary_path(), &binary_dir)
    .with_context(|| "Failed to copy binary file")?;

  // get the target path
  let output_path = settings.project_out_directory().join("bundle/updater");
  let update_path = output_path.join(&update_name.clone());

  if output_path.exists() {
    fs::remove_dir_all(&output_path)
      .with_context(|| format!("Failed to remove old {}", update_name))?;
  }

  archive_utils::tar_and_gzip_to(&package_dir, &update_path)
    .with_context(|| "Failed to tar/gzip update directory")?;

  if package_dir.exists() {
    fs::remove_dir_all(&package_dir).with_context(|| format!("Failed to remove tmp dir"))?;
  }

  common::print_bundling(format!("{:?}", update_path.clone()).as_str())?;
  Ok(vec![update_path])
}

// Create simple update-win_<arch>.zip
// Including the binary as root
// Right now in windows we hot replace the bin and request a restart
// No assets are replaced
#[cfg(target_os = "windows")]
fn bundle_update(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  let arch = match settings.binary_arch() {
    "x86" => "x86",
    "x86_64" => "x64",
    other => other,
  };

  let update_name = format!("update-win_{}", arch);

  // copy bin in a tmp folder then zip this folder
  let package_dir = settings.project_out_directory().join("bundle/win");
  let binary_dir = package_dir.join(settings.binary_name());

  if package_dir.exists() {
    fs::remove_dir_all(&package_dir)
      .with_context(|| format!("Failed to remove old `bundle/win`"))?;
  }

  common::copy_file(settings.binary_path(), &binary_dir)
    .with_context(|| "Failed to copy binary file")?;

  // get the target path
  let output_path = settings.project_out_directory().join("bundle/updater");
  let update_path = output_path.join(&update_name.clone());

  if output_path.exists() {
    fs::remove_dir_all(&output_path)
      .with_context(|| format!("Failed to remove old {}", update_name))?;
  }

  archive_utils::zip_dir(&package_dir, &update_path)
    .with_context(|| "Failed to zip update directory")?;

  if package_dir.exists() {
    fs::remove_dir_all(&package_dir).with_context(|| format!("Failed to remove tmp dir"))?;
  }

  common::print_bundling(format!("{:?}", update_path.clone()).as_str())?;
  Ok(vec![update_path])
}
