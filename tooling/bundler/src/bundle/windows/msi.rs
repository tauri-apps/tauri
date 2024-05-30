// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod wix;

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
  let tauri_tools_path = std::env::current_dir().unwrap().join("target/tools");
  let wix_path = tauri_tools_path.join("Wix");
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
