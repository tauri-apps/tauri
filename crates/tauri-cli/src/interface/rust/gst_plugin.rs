// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Puts the `asset://` GStreamer plugin where the Tauri runtime looks for it
//! next to an unbundled binary: `<exe dir>/../lib/<exe name>/gstreamer-1.0`.
//! Must match the lookup in `crates/tauri/src/app.rs`.

use super::{get_cargo_metadata, Options, RustAppSettings};
use crate::{error::ErrorExt, interface::AppSettings};
use std::{fs, path::Path};
use tauri_bundler::{gst_plugin, tools_directory};
use tauri_utils::platform::Target as TargetPlatform;

/// Downloads the plugin and installs it for the app binary built with `options`.
pub fn install(
  app_settings: &RustAppSettings,
  options: &Options,
  use_local_tools_dir: bool,
  tauri_dir: &Path,
) {
  if !matches!(app_settings.target_platform, TargetPlatform::Linux) {
    return;
  }

  if let Err(error) = try_install(app_settings, options, use_local_tools_dir, tauri_dir) {
    log::warn!(
      "failed to set up the asset GStreamer plugin, audio and video over `asset://` will not work: {error:#}"
    );
  }
}

fn try_install(
  app_settings: &RustAppSettings,
  options: &Options,
  use_local_tools_dir: bool,
  tauri_dir: &Path,
) -> crate::Result<()> {
  let bin_path = app_settings.app_binary_path(options, tauri_dir)?;
  let (Some(out_dir), Some(exe_name)) = (bin_path.parent(), bin_path.file_name()) else {
    return Ok(());
  };

  let local_tools_dir = use_local_tools_dir
    .then(|| get_cargo_metadata(tauri_dir).map(|metadata| metadata.target_directory))
    .transpose()?;
  let tools_path = tools_directory(local_tools_dir.as_deref(), out_dir);

  let Some(plugin) = gst_plugin::download_for_target(&app_settings.target_triple, &tools_path)?
  else {
    return Ok(());
  };

  let dest_dir = out_dir.join("../lib").join(exe_name).join("gstreamer-1.0");
  let dest = dest_dir.join(gst_plugin::PLUGIN_FILE_NAME);

  // Overwriting a shared library in place can crash an app that still has it
  // loaded, so leave an identical copy alone.
  let data = fs::read(&plugin).fs_context("failed to read the asset GStreamer plugin", plugin)?;
  if fs::read(&dest).is_ok_and(|existing| existing == data) {
    return Ok(());
  }

  fs::create_dir_all(&dest_dir).fs_context(
    "failed to create the asset GStreamer plugin directory",
    dest_dir.clone(),
  )?;
  fs::write(&dest, data).fs_context("failed to install the asset GStreamer plugin", dest)?;

  Ok(())
}
