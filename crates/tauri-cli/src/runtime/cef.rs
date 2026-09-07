// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Locating the CEF (Chromium Embedded Framework) binary distribution.

use std::{
  path::{Path, PathBuf},
  process::Command,
};

use serde::Deserialize;

use download_cef::OsAndArch;

use crate::{
  error::{Context, Error, bail},
  helpers::cargo_manifest::{cargo_manifest_and_lock, crate_version},
};

/// The `tauri-runtime-cef` crate.
pub const CRATE_NAME: &str = "tauri-runtime-cef";

/// The directory the `cef` crate build script downloads the CEF binary distribution to
/// when `CEF_PATH` is not set.
fn default_path() -> PathBuf {
  dirs::cache_dir()
    .unwrap_or_else(|| PathBuf::from(".cache"))
    .join("tauri-cef")
}

/// `CEF_PATH` for every cargo build that links CEF: where `cef-dll-sys`
/// resolves the CEF binary distribution from, downloading into it when
/// missing. The environment's value, or a cache directory shared by all
/// projects on the machine.
pub(crate) fn cef_path_env() -> PathBuf {
  std::env::var_os("CEF_PATH")
    .map(PathBuf::from)
    .unwrap_or_else(default_path)
}

/// The CEF version the app's `cef` crate dependency downloads.
pub(crate) fn default_version(workspace_dir: &Path) -> Option<String> {
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

// Cargo metadata is the authority for patches and exact package sources. The
// framework version in the crate's build metadata is not its registry version.
#[derive(Deserialize)]
struct ResolvedMetadata {
  packages: Vec<ResolvedPackage>,
}

#[derive(Deserialize)]
struct ResolvedPackage {
  name: String,
  manifest_path: PathBuf,
}

pub(crate) fn resolved_crate_paths(
  workspace_dir: &Path,
  target: &str,
) -> crate::Result<(PathBuf, PathBuf)> {
  let output = Command::new("cargo")
    .args([
      "metadata",
      "--locked",
      "--format-version",
      "1",
      "--all-features",
      "--filter-platform",
      target,
    ])
    .current_dir(workspace_dir)
    .output()
    .map_err(|error| Error::CommandFailed {
      command: "cargo metadata".into(),
      error,
    })?;
  if !output.status.success() {
    return Err(Error::CommandFailed {
      command: "cargo metadata".into(),
      error: std::io::Error::other(String::from_utf8_lossy(&output.stderr)),
    });
  }
  let metadata: ResolvedMetadata = serde_json::from_slice(&output.stdout)
    .context("failed to parse resolved CEF package sources from Cargo metadata")?;
  Ok((
    resolved_crate_path(&metadata, "cef")?,
    resolved_crate_path(&metadata, "cef-dll-sys")?,
  ))
}

fn resolved_crate_path(metadata: &ResolvedMetadata, name: &str) -> crate::Result<PathBuf> {
  let mut packages = metadata
    .packages
    .iter()
    .filter(|package| package.name == name);
  let package = packages
    .next()
    .ok_or_else(|| Error::GenericError(format!("{name} is missing from Cargo metadata")))?;
  if packages.next().is_some() {
    bail!("multiple {name} packages are resolved; the CEF helper source is ambiguous");
  }
  let path = package
    .manifest_path
    .parent()
    .filter(|path| path.is_absolute())
    .ok_or_else(|| Error::GenericError(format!("{name} has no absolute crate directory")))?;
  Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn resolved_sources_preserve_git_and_local_patches() {
    let root = std::env::current_dir().unwrap();
    let cef_path = root.join("cache/git/cef");
    let sys_path = root.join("project/custom/sys");
    let metadata: ResolvedMetadata = serde_json::from_value(serde_json::json!({
      "packages": [
        { "name": "cef", "manifest_path": cef_path.join("Cargo.toml") },
        { "name": "cef-dll-sys", "manifest_path": sys_path.join("Cargo.toml") }
      ]
    }))
    .unwrap();
    assert_eq!(resolved_crate_path(&metadata, "cef").unwrap(), cef_path);
    assert_eq!(
      resolved_crate_path(&metadata, "cef-dll-sys").unwrap(),
      sys_path
    );
  }

  #[test]
  fn missing_or_ambiguous_sources_fail_before_building_a_helper() {
    let mut metadata = ResolvedMetadata { packages: vec![] };
    assert!(resolved_crate_path(&metadata, "cef").is_err());
    for root in ["/one", "/two"] {
      metadata.packages.push(ResolvedPackage {
        name: "cef".into(),
        manifest_path: PathBuf::from(root).join("Cargo.toml"),
      });
    }
    assert!(resolved_crate_path(&metadata, "cef").is_err());
  }
}
