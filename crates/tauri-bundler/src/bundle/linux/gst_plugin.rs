// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Resolves the GStreamer plugin for having `asset://` work for audio/video.

use super::tools_directory;
use crate::{
  bundle::settings::Arch,
  error::{Context, ErrorExt},
  utils::http_utils::{download_and_verify, verify_file_hash, HashAlgorithm},
  Settings,
};
use std::{
  fs,
  path::{Path, PathBuf},
};

/// File name of the GStreamer plugin
pub const PLUGIN_FILE_NAME: &str = "libgsttauriasset.so";

/// Release the prebuilt plugins are downloaded from.
const PLUGIN_RELEASE_URL: &str =
  "https://github.com/tauri-apps/tauri-gstreamer-plugin/releases/download/gst-plugin-tauri-v0.0.1";
const PLUGIN_SHA256_X86_64: &str =
  "ABB55C037AC07545C0E19E8D11FBC6BC642C23A24F35F550B5F830D387DB0914";
const PLUGIN_SHA256_AARCH64: &str =
  "A50347365B0949A3BD251689B7B5E1CBB01B4AE9A2E3446D6D28A77D55B35F6F";

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

/// Resolves the plugin to bundle, or `None` when it is not enabled.
pub fn resolve(settings: &Settings) -> crate::Result<Option<PathBuf>> {
  let config = settings.asset_gst_plugin();
  if !config.active {
    return Ok(None);
  }

  match &config.path {
    Some(path) => {
      if !path.is_file() {
        crate::error::bail!(
          "`bundle > linux > assetGstPlugin > path` is not a file: {}",
          path.display()
        );
      }
      Ok(Some(path.clone()))
    }
    None => download_plugin(settings).map(Some),
  }
}

/// Downloads the prebuilt plugin for the target architecture, caching it in the
/// tools directory alongside the AppImage tooling.
///
/// The download and the cached copy are both checked against the pinned hash,
/// a mismatching cached copy is downloaded again.
fn download_plugin(settings: &Settings) -> crate::Result<PathBuf> {
  let (arch, hash) = match settings.binary_arch() {
    Arch::X86_64 => ("x86_64", PLUGIN_SHA256_X86_64),
    Arch::AArch64 => ("aarch64", PLUGIN_SHA256_AARCH64),
    target => {
      return Err(crate::Error::ArchError(format!(
        "no prebuilt asset GStreamer plugin is available for {target:?}; build it yourself and point `bundle > linux > assetGstPlugin > path` at the resulting {PLUGIN_FILE_NAME}"
      )))
    }
  };

  let tools_path = tools_directory(settings, settings.project_out_directory());
  fs::create_dir_all(&tools_path)?;

  let file_name = format!("libgsttauriasset-{arch}.so");
  let cached = tools_path.join(&file_name);
  if cached.exists() {
    if verify_file_hash(&cached, hash, HashAlgorithm::Sha256).is_ok() {
      return Ok(cached);
    }
    log::warn!("{file_name} in the tools directory is mis-hashed. Redownloading it.");
  }

  let data = download_and_verify(
    &format!("{PLUGIN_RELEASE_URL}/{file_name}"),
    hash,
    HashAlgorithm::Sha256,
  )
  .with_context(|| {
    format!("failed to download {file_name}; build the plugin yourself and point `bundle > linux > assetGstPlugin > path` at the resulting {PLUGIN_FILE_NAME}")
  })?;
  fs::write(&cached, data)
    .fs_context("failed to save the asset GStreamer plugin", cached.clone())?;

  Ok(cached)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::bundle::settings::{
    AssetGstPluginSettings, BundleSettings, PackageSettings, SettingsBuilder,
  };

  fn settings(active: bool, path: Option<PathBuf>, out: &Path) -> Settings {
    SettingsBuilder::new()
      .project_out_directory(out)
      .package_settings(PackageSettings {
        product_name: "My App".into(),
        version: "0.1.0".into(),
        description: String::new(),
        homepage: None,
        authors: None,
        default_run: None,
      })
      .bundle_settings(BundleSettings {
        asset_gst_plugin: AssetGstPluginSettings { active, path },
        ..Default::default()
      })
      .build()
      .unwrap()
  }

  #[test]
  fn plugin_dir_is_private_to_the_app() {
    assert_eq!(
      plugin_dir("My App"),
      PathBuf::from("usr/lib/My App/gstreamer-1.0")
    );
  }

  #[test]
  fn inactive_resolves_to_none() {
    let tmp = tempfile::tempdir().unwrap();
    // a path is set but must be ignored while inactive
    let so = tmp.path().join(PLUGIN_FILE_NAME);
    fs::write(&so, b"x").unwrap();
    let s = settings(false, Some(so), tmp.path());
    assert!(resolve(&s).unwrap().is_none());
  }

  #[test]
  fn explicit_path_is_used_verbatim() {
    let tmp = tempfile::tempdir().unwrap();
    let so = tmp.path().join(PLUGIN_FILE_NAME);
    fs::write(&so, b"x").unwrap();
    let s = settings(true, Some(so.clone()), tmp.path());
    assert_eq!(resolve(&s).unwrap(), Some(so));
  }

  #[test]
  fn missing_explicit_path_errors_without_downloading() {
    let tmp = tempfile::tempdir().unwrap();
    let s = settings(true, Some(tmp.path().join("nope.so")), tmp.path());
    let err = resolve(&s).unwrap_err().to_string();
    assert!(err.contains("assetGstPlugin"), "unhelpful error: {err}");
  }

  #[test]
  fn a_directory_is_not_accepted_as_the_plugin() {
    let tmp = tempfile::tempdir().unwrap();
    let s = settings(true, Some(tmp.path().to_path_buf()), tmp.path());
    assert!(resolve(&s).is_err());
  }
}
