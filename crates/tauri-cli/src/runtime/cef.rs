// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Locating the CEF (Chromium Embedded Framework) binary distribution.

use std::path::{Path, PathBuf};

use download_cef::OsAndArch;

use crate::{
  error::{Error, bail},
  helpers::cargo_manifest::{cargo_manifest_and_lock, crate_version},
};

/// The `tauri-runtime-cef` crate.
pub const CRATE_NAME: &str = "tauri-runtime-cef";

/// The directory the `cef` crate build script downloads the CEF binary distribution to
/// when `CEF_PATH` is not set.
pub fn default_path() -> PathBuf {
  dirs::cache_dir()
    .unwrap_or_else(|| PathBuf::from(".cache"))
    .join("tauri-cef")
}

/// The CEF version the app's `cef` crate dependency downloads.
fn default_version(workspace_dir: &Path) -> Option<String> {
  let (_, lock) = cargo_manifest_and_lock(workspace_dir);
  let crate_version = crate_version(workspace_dir, None, lock.as_ref(), "cef");
  crate_version
    .version
    .as_deref()
    .map(download_cef::default_version)
}

/// A file that only exists in a CEF binary distribution for the target.
fn marker_file(target: &str) -> crate::Result<&'static str> {
  if target.contains("darwin") {
    Ok("Chromium Embedded Framework.framework")
  } else if target.contains("windows") {
    Ok("libcef.dll")
  } else if target.contains("linux") {
    Ok("libcef.so")
  } else {
    Err(Error::GenericError(format!(
      "CEF bundling is not supported for target `{target}`"
    )))
  }
}

/// Resolves the CEF binary distribution to ship with the bundle for the target.
///
/// `cef_path` is either a distribution itself or the download cache of the `cef` crate build script,
/// which stores the distributions as `<version>/<os-arch>`.
pub fn resolve_path_for_bundle(
  cef_path: PathBuf,
  target: &str,
  workspace_dir: &Path,
) -> crate::Result<PathBuf> {
  let resolved = if let Some(cef_version) = default_version(workspace_dir) {
    let os_arch = OsAndArch::try_from(target)
      .map_err(|e| Error::GenericError(format!("invalid CEF target {target}: {e}")))?;

    let versioned = cef_path.join(&cef_version).join(os_arch.to_string());
    if versioned.exists() {
      versioned
    } else {
      cef_path
    }
  } else {
    cef_path
  };

  let marker = marker_file(target)?;
  if !resolved.join(marker).exists() {
    bail!(
      "CEF binary distribution not found at {} (missing `{marker}`). \
       Run `cargo tauri build` (or `cargo build`) so the build script downloads CEF, \
       or point CEF_PATH to an extracted CEF binary distribution.",
      resolved.display(),
    );
  }

  Ok(resolved)
}
