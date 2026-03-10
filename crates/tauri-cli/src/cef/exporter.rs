// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::VersionMetadata;
use crate::error::ErrorExt;
use crate::helpers::cargo_manifest::{cargo_manifest_and_lock, crate_version};
use download_cef::{CefFile, CefIndex, DEFAULT_TARGET, OsAndArch};
use std::{
  fs,
  path::{Path, PathBuf},
  sync::OnceLock,
  time::Duration,
};

fn default_version(workspace_dir: &Path) -> String {
  // Try to get version from Cargo.lock first
  let (_, lock) = cargo_manifest_and_lock(workspace_dir);
  let crate_version = crate_version(workspace_dir, None, lock.as_ref(), "cef");
  if let Some(version) = crate_version.version {
    return download_cef::default_version(&version);
  }

  // Fallback to previous implementation: read from metadata-v2.json
  let metadata: VersionMetadata = serde_json::from_str(include_str!("../../metadata-v2.json"))
    .expect("failed to parse metadata-v2.json");
  download_cef::default_version(&metadata.cef)
}

/// Patched CEF builds hosted on CrabNebula CDN
const DEFAULT_CDN_URL: &str = "https://cdn.crabnebula.app/download/crabnebula/cef-patched/latest";

fn default_download_url() -> &'static str {
  static DEFAULT_DOWNLOAD_URL: OnceLock<String> = OnceLock::new();
  DEFAULT_DOWNLOAD_URL
    .get_or_init(|| {
      if cfg!(any(windows, target_os = "macos")) {
        std::env::var("CEF_DOWNLOAD_URL").unwrap_or(DEFAULT_CDN_URL.to_owned())
      } else {
        download_cef::default_download_url()
      }
    })
    .as_str()
}

pub struct ExporterOptions {
  pub output: PathBuf,
  pub target: String,
  pub version: Option<String>,
  pub mirror_url: Option<String>,
  pub force: bool,
  pub overwrite: bool,
  pub archive: Option<PathBuf>,
}

impl Default for ExporterOptions {
  fn default() -> Self {
    Self {
      output: PathBuf::new(),
      target: DEFAULT_TARGET.to_string(),
      version: None,
      mirror_url: None,
      force: false,
      overwrite: false,
      archive: None,
    }
  }
}

pub fn export_cef_directory(options: ExporterOptions, workspace_dir: &Path) -> crate::Result<()> {
  let output = &options.output;
  let url = options
    .mirror_url
    .unwrap_or_else(|| default_download_url().to_string());
  let version = options
    .version
    .unwrap_or_else(|| default_version(workspace_dir));

  let parent = output.parent().ok_or_else(|| {
    crate::Error::GenericError(format!("invalid target directory: {}", output.display()))
  })?;

  // Check if directory exists and has valid archive.json
  let archive_json_path = output.join("archive.json");
  let needs_update = if output.exists() && archive_json_path.exists() {
    if options.force {
      true
    } else {
      // Check if archive.json indicates we need an update
      match check_archive_outdated(&archive_json_path, &version) {
        Ok(true) => {
          log::info!(action = "CEF"; "CEF directory is outdated, will update");
          true
        }
        Ok(false) => false,
        Err(e) => {
          log::warn!(action = "CEF"; "Failed to check archive.json: {e}. Will update.");
          true
        }
      }
    }
  } else {
    true
  };

  if !needs_update {
    return Ok(());
  }

  if output.exists() {
    if !options.overwrite {
      return Err(crate::Error::GenericError(format!(
        "target directory already exists: {}. Use overwrite=true to overwrite it.",
        output.display()
      )));
    }

    let dir = output
      .file_name()
      .and_then(|dir| dir.to_str())
      .ok_or_else(|| {
        crate::Error::GenericError(format!("invalid target directory: {}", output.display()))
      })?;
    let old_output = parent.join(format!("old_{dir}"));
    if old_output.exists() {
      fs::remove_dir_all(&old_output)
        .fs_context("failed to remove old output directory", old_output.clone())?;
    }
    fs::rename(output, &old_output)
      .fs_context("failed to rename output directory", old_output.clone())?;
    log::info!(action = "CEF"; "Cleaning up: {}", old_output.display());
    fs::remove_dir_all(&old_output)
      .fs_context("failed to remove old output directory", old_output)?;
  }

  let target_str = options.target.as_str();
  let os_arch = OsAndArch::try_from(target_str)
    .map_err(|e| crate::Error::GenericError(format!("invalid target: {e}")))?;
  let cef_dir = os_arch.to_string();
  let cef_dir = parent.join(&cef_dir);

  if cef_dir.exists() {
    let dir = cef_dir
      .file_name()
      .and_then(|dir| dir.to_str())
      .ok_or_else(|| {
        crate::Error::GenericError(format!("invalid target directory: {}", output.display()))
      })?;
    let old_cef_dir = parent.join(format!("old_{dir}"));
    if old_cef_dir.exists() {
      fs::remove_dir_all(&old_cef_dir)
        .fs_context("failed to remove old cef directory", old_cef_dir.clone())?;
    }
    fs::rename(&cef_dir, &old_cef_dir)
      .fs_context("failed to rename cef directory", old_cef_dir.clone())?;
    log::info!(action = "CEF"; "Cleaning up: {}", old_cef_dir.display());
    fs::remove_dir_all(&old_cef_dir)
      .fs_context("failed to remove old cef directory", old_cef_dir)?;
  }

  let (archive, extracted_dir) = match options.archive {
    Some(archive) => {
      let extracted_dir = download_cef::extract_target_archive(target_str, &archive, parent, true)
        .map_err(|e| crate::Error::GenericError(format!("failed to extract archive: {e}")))?;
      let archive = CefFile::try_from(archive.as_path())
        .map_err(|e| crate::Error::GenericError(format!("invalid archive file: {e}")))?;
      (archive, extracted_dir)
    }
    None => {
      log::info!(action = "CEF"; "Downloading CEF version {version} for target {target_str}");
      let index = CefIndex::download_from(&url)
        .map_err(|e| crate::Error::GenericError(format!("failed to download CEF index: {e}")))?;
      let platform = index.platform(target_str).map_err(|e| {
        crate::Error::GenericError(format!("platform not found for target {target_str}: {e}"))
      })?;
      let version = platform
        .version(&version)
        .map_err(|e| crate::Error::GenericError(format!("version {version} not found: {e}")))?;

      let archive = version
        .download_archive_with_retry_from(&url, parent, true, Duration::from_secs(15), 3)
        .map_err(|e| crate::Error::GenericError(format!("failed to download CEF archive: {e}")))?;
      let extracted_dir = download_cef::extract_target_archive(target_str, &archive, parent, true)
        .map_err(|e| crate::Error::GenericError(format!("failed to extract archive: {e}")))?;

      fs::remove_file(&archive).fs_context("failed to remove archive", archive.clone())?;

      let archive = version
        .minimal()
        .map_err(|e| {
          crate::Error::GenericError(format!("failed to get minimal archive info: {e}"))
        })?
        .clone();
      (archive, extracted_dir)
    }
  };

  if extracted_dir != cef_dir {
    return Err(crate::Error::GenericError(format!(
      "extracted dir {extracted_dir:?} does not match cef_dir {cef_dir:?}",
    )));
  }

  archive
    .write_archive_json(extracted_dir.as_path())
    .map_err(|e| crate::Error::GenericError(format!("failed to write archive.json: {e}")))?;

  if output != &cef_dir {
    log::info!(action = "CEF"; "Renaming: {}", output.display());
    fs::rename(&cef_dir, output)
      .fs_context("failed to rename cef directory to output", output.clone())?;
  }

  Ok(())
}

#[cfg(target_os = "linux")]
fn export_cef_library_path(cef_path: &Path) {
  let mut ld_library_path = std::env::var_os("LD_LIBRARY_PATH").unwrap_or_default();
  ld_library_path.push(":");
  ld_library_path.push(cef_path);
  unsafe { std::env::set_var("LD_LIBRARY_PATH", ld_library_path) };
}

#[cfg(windows)]
fn export_cef_library_path(cef_path: &Path) {
  let mut path = std::env::var_os("PATH").unwrap_or_default();
  path.push(";");
  path.push(cef_path);
  unsafe { std::env::set_var("PATH", path) };
}

#[cfg(target_os = "macos")]
fn export_cef_library_path(cef_path: &Path) {
  let mut ld_library_path = std::env::var_os("DYLD_FALLBACK_LIBRARY_PATH").unwrap_or_default();
  ld_library_path.push(":");
  ld_library_path.push(cef_path);
  ld_library_path.push(":");
  ld_library_path.push(
    cef_path
      .join("Chromium Embedded Framework.framework")
      .join("Libraries"),
  );
  unsafe { std::env::set_var("DYLD_FALLBACK_LIBRARY_PATH", ld_library_path) };
}

fn check_archive_outdated(archive_json_path: &Path, required_version: &str) -> crate::Result<bool> {
  let content = fs::read_to_string(archive_json_path).fs_context(
    "failed to read archive.json",
    archive_json_path.to_path_buf(),
  )?;
  let archive_info: serde_json::Value = serde_json::from_str(&content)
    .map_err(|e| crate::Error::GenericError(format!("failed to parse archive.json: {e}")))?;

  if let Some(name) = archive_info.get("name").and_then(|v| v.as_str()) {
    Ok(!name.contains(required_version))
  } else {
    Ok(true)
  }
}

/// Ensures the CEF directory is available. Returns `None` if CEF is not in use for this build.
pub fn ensure_cef_directory(target: Option<&str>, workspace_dir: &Path) -> crate::Result<PathBuf> {
  let target = target.unwrap_or(DEFAULT_TARGET);
  let os_arch = OsAndArch::try_from(target)
    .map_err(|e| crate::Error::GenericError(format!("invalid target: {e}")))?;

  let version = default_version(workspace_dir);
  let cef_dir = if let Ok(cef_path) = std::env::var("CEF_PATH") {
    let path = PathBuf::from(cef_path);
    // If CEF_PATH is set but directory doesn't exist, we'll create it
    if !path.exists() {
      log::info!(action = "CEF"; "CEF_PATH set but directory doesn't exist, will export to {}", path.display());
    }
    path.join(&version)
  } else {
    // Default to a directory in the user's cache or home directory
    let base_dir = dirs::cache_dir()
      .or_else(dirs::home_dir)
      .ok_or_else(|| crate::Error::GenericError("failed to find cache or home directory".into()))?;
    base_dir
      .join("tauri-cef")
      .join(os_arch.to_string())
      .join(&version)
  };

  export_cef_directory(
    ExporterOptions {
      output: cef_dir.clone(),
      target: target.to_string(),
      version: Some(version),
      mirror_url: None,
      force: false,
      overwrite: true,
      archive: None,
    },
    workspace_dir,
  )?;

  unsafe { std::env::set_var("CEF_PATH", cef_dir.to_string_lossy().as_ref()) };
  export_cef_library_path(&cef_dir);
  log::info!(action = "CEF"; "CEF directory exported to: {}", cef_dir.display());

  Ok(cef_dir)
}
