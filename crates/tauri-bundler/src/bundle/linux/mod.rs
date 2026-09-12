// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod appimage;
pub mod debian;
pub mod freedesktop;
pub mod gst_plugin;
pub mod rpm;

use crate::Settings;
use std::path::{Path, PathBuf};

/// Directory downloaded bundling tools are cached in.
///
/// `fallback` is only used when the user has not configured a local tools
/// directory and the platform has no cache directory.
pub fn tools_directory(settings: &Settings, fallback: &Path) -> PathBuf {
  settings
    .local_tools_directory()
    .map(|d| d.join(".tauri"))
    .unwrap_or_else(|| {
      dirs::cache_dir().map_or_else(|| fallback.to_path_buf(), |p| p.join("tauri"))
    })
}
