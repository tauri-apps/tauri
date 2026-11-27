// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Resolves the GStreamer plugin for having `asset://` work for audio/video.

use super::tools_directory;
use crate::{
  Settings,
  error::{Context, ErrorExt},
  utils::http_utils::{HashAlgorithm, download_and_verify, verify_file_hash},
};
use std::{
  fs,
  path::{Path, PathBuf},
};

/// File name of the GStreamer plugin
pub const PLUGIN_FILE_NAME: &str = "libgsttauriasset.so";

/// Release the prebuilt plugins are downloaded from.
const PLUGIN_RELEASE_URL: &str =
  "https://github.com/tauri-apps/tauri-gstreamer-plugin/releases/download/gst-plugin-tauri-v0.0.2";
const PLUGIN_SHA256_X86_64: &str =
  "7E94585BD146E20022DF4FEE574F10967840BAE9DED3B5099A13B182CDB0D2F8";
const PLUGIN_SHA256_AARCH64: &str =
  "7B80B57E5BCB8B806FD620CA879D52CCDD9A4D65A196918AD9B0EA8FEFFB8E5A";

/// Directory the plugin is installed to, relative to the bundle root.
///
/// This is a private per-app directory rather than the system-wide
/// `gstreamer-1.0` one, so that bundles cannot collide with distribution
/// packages or with each other. GStreamer only scans directories listed in
/// `GST_PLUGIN_PATH`, which the Tauri runtime points here on startup.
pub fn plugin_dir(product_name: &str) -> PathBuf {
  Path::new("usr/lib")
    .join(product_name)
    .join("gstreamer-1.0")
}

/// Directory the plugin is installed to inside the AppImage's AppDir.
///
/// The media framework's AppRun hook points `GST_PLUGIN_PATH_1_0` here, which
/// GStreamer reads before `GST_PLUGIN_PATH`, so the per-app directory used by
/// deb/rpm would never be scanned inside an AppImage.
pub const APPIMAGE_PLUGIN_DIR: &str = "usr/lib/gstreamer-1.0";

/// Directory the plugin for `arch` is cached in, inside the tools directory.
///
/// The plugin is stored under [`PLUGIN_FILE_NAME`] in a directory of its own,
/// so GStreamer can load it straight from here. The Tauri runtime does that
/// for development binaries, see the lookup in `crates/tauri/src/app.rs`.
pub fn cache_dir(tools_path: &Path, arch: &str) -> PathBuf {
  tools_path.join("gstreamer-1.0").join(arch)
}

/// Downloads the prebuilt plugin for the target architecture, caching it in the
/// tools directory alongside the AppImage tooling.
///
/// Returns `None` when no prebuilt plugin exists for the target architecture.
/// The download and the cached copy are both checked against the pinned hash,
/// a mismatching cached copy is downloaded again.
pub fn download(settings: &Settings) -> crate::Result<Option<PathBuf>> {
  download_for_target(
    settings.target(),
    &tools_directory(
      settings.local_tools_directory(),
      settings.project_out_directory(),
    ),
  )
}

/// Downloads the prebuilt plugin for the `target` triple, caching it in `tools_path`.
///
/// Same as [`download`] for callers that don't have bundler [`Settings`].
pub fn download_for_target(target: &str, tools_path: &Path) -> crate::Result<Option<PathBuf>> {
  let (arch, hash) = if target.starts_with("x86_64") {
    ("x86_64", PLUGIN_SHA256_X86_64)
  } else if target.starts_with("aarch64") {
    ("aarch64", PLUGIN_SHA256_AARCH64)
  } else {
    log::warn!(
      "no prebuilt asset GStreamer plugin for {target}, audio and video over `asset://` will not work"
    );
    return Ok(None);
  };

  let cache_dir = cache_dir(tools_path, arch);
  fs::create_dir_all(&cache_dir)?;

  let file_name = format!("libgsttauriasset-{arch}.so");
  let cached = cache_dir.join(PLUGIN_FILE_NAME);
  if cached.exists() {
    if verify_file_hash(&cached, hash, HashAlgorithm::Sha256).is_ok() {
      return Ok(Some(cached));
    }
    log::warn!("{file_name} in the tools directory is mis-hashed. Redownloading it.");
  }

  let data = download_and_verify(
    &format!("{PLUGIN_RELEASE_URL}/{file_name}"),
    hash,
    HashAlgorithm::Sha256,
  )
  .with_context(|| format!("failed to download {file_name}"))?;
  fs::write(&cached, data)
    .fs_context("failed to save the asset GStreamer plugin", cached.clone())?;

  Ok(Some(cached))
}
