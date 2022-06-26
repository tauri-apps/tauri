// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod wix;

pub use wix::{MSI_FOLDER_NAME, MSI_UPDATER_FOLDER_NAME};

use crate::Settings;
use log::warn;

use std::{self, path::PathBuf};

const WIX_REQUIRED_FILES: &[&str] = &[
  "candle.exe",
  "candle.exe.config",
  "darice.cub",
  "light.exe",
  "light.exe.config",
  "wconsole.dll",
  "winterop.dll",
  "wix.dll",
  "WixUIExtension.dll",
  "WixUtilExtension.dll",
];

/// Runs all of the commands to build the MSI installer.
/// Returns a vector of PathBuf that shows where the MSI was created.
pub fn bundle_project(settings: &Settings, updater: bool) -> crate::Result<Vec<PathBuf>> {
  let mut wix_path = dirs_next::cache_dir().unwrap();
  wix_path.push("tauri/WixTools");

  if !wix_path.exists() {
    wix::get_and_extract_wix(&wix_path)?;
  } else if WIX_REQUIRED_FILES
    .iter()
    .any(|p| !wix_path.join(p).exists())
  {
    warn!("WixTools directory is missing some files. Recreating it.");
    std::fs::remove_dir_all(&wix_path)?;
    wix::get_and_extract_wix(&wix_path)?;
  }

  wix::build_wix_app_installer(settings, &wix_path, updater)
}
