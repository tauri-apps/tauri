// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Makes the `asset://` GStreamer plugin available to the app run by `tauri dev`.
//!
//! The plugin is loaded from the tools directory it is downloaded to. The
//! Tauri runtime finds the default one on its own (see `crates/tauri/src/app.rs`),
//! `GST_PLUGIN_PATH` covers a local tools directory.

use super::{Options, RustAppSettings, get_cargo_metadata};
use crate::{error::Context, interface::AppSettings};
use std::path::Path;
use tauri_bundler::{gst_plugin, tools_directory};
use tauri_utils::platform::Target as TargetPlatform;

/// Downloads the plugin and lists its directory in `GST_PLUGIN_PATH` for the app binary built with `options`.
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
  let Some(out_dir) = bin_path.parent() else {
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

  let Some(plugin_dir) = plugin.parent() else {
    return Ok(());
  };

  let plugin_path = std::env::join_paths(
    std::iter::once(plugin_dir.to_path_buf()).chain(
      std::env::var_os("GST_PLUGIN_PATH")
        .iter()
        .flat_map(std::env::split_paths),
    ),
  )
  .context("failed to add the asset GStreamer plugin to GST_PLUGIN_PATH")?;
  // SAFETY: called from the main thread before the app is built and run.
  unsafe { std::env::set_var("GST_PLUGIN_PATH", plugin_path) };

  Ok(())
}
