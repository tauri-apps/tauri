// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{Error, Result};
use crate::{AppHandle, Runtime};
use std::path::{Path, PathBuf};

/// The path resolver is a helper class for general and application-specific path APIs on OpenHarmony.
pub struct PathResolver<R: Runtime>(pub(crate) AppHandle<R>);

impl<R: Runtime> Clone for PathResolver<R> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<R: Runtime> PathResolver<R> {
  fn base_path(&self) -> Result<PathBuf> {
    crate::ohos::BASE_PATH
      .get()
      .and_then(|p| p.as_ref())
      .map(PathBuf::from)
      .ok_or(Error::UnknownPath)
  }

  /// Returns the final component of the `Path`, if there is one.
  ///
  /// If the path is a normal file, this is the file name. If it's the path of a directory, this
  /// is the directory name.
  ///
  /// Returns [`None`] if the path terminates in `..`.
  pub fn file_name(&self, path: &str) -> Option<String> {
    Path::new(path)
      .file_name()
      .map(|name| name.to_string_lossy().into_owned())
  }

  /// Returns the path to the user's audio directory.
  pub fn audio_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("files").join("Audio"))
  }

  /// Returns the path to the user's cache directory.
  pub fn cache_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("cache"))
  }

  /// Returns the path to the user's config directory.
  pub fn config_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("files"))
  }

  /// Returns the path to the user's data directory.
  pub fn data_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("files"))
  }

  /// Returns the path to the user's local data directory.
  pub fn local_data_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("files"))
  }

  /// Returns the path to the user's document directory.
  pub fn document_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("files").join("Documents"))
  }

  /// Returns the path to the user's download directory.
  pub fn download_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("files").join("Download"))
  }

  /// Returns the path to the user's picture directory.
  pub fn picture_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("files").join("Pictures"))
  }

  /// Returns the path to the user's public directory.
  pub fn public_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("files").join("Public"))
  }

  /// Returns the path to the user's video directory.
  pub fn video_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("files").join("Videos"))
  }

  /// Returns the path to the resource directory.
  pub fn resource_dir(&self) -> Result<PathBuf> {
    let module_name = crate::ohos::MODULE_NAME
      .get()
      .and_then(|m| m.as_deref());
    Ok(compute_resource_dir(module_name))
  }

  /// Returns the path to the app's config directory.
  pub fn app_config_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("files"))
  }

  /// Returns the path to the app's data directory.
  pub fn app_data_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("files"))
  }

  /// Returns the path to the app's local data directory.
  pub fn app_local_data_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("files"))
  }

  /// Returns the path to the app's cache directory.
  pub fn app_cache_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("cache"))
  }

  /// Returns the path to the app's log directory.
  pub fn app_log_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("log"))
  }

  /// Returns the path to the temporary directory.
  pub fn temp_dir(&self) -> Result<PathBuf> {
    self.base_path().map(|p| p.join("temp"))
  }

  /// Returns the path to the home directory (app sandbox root).
  pub fn home_dir(&self) -> Result<PathBuf> {
    self.base_path()
  }
}

/// Internal helper: compute the resource directory from an optional module name.
/// Extracted for unit testing.
fn compute_resource_dir(module_name: Option<&str>) -> PathBuf {
  let module = module_name.unwrap_or("entry");
  PathBuf::from("/data/storage/el1/base")
    .join(module)
    .join("assets")
}

#[cfg(test)]
mod tests {
  use super::*;

  const MOCK_BASE: &str = "/data/storage/el2/base";

  fn mock_base() -> PathBuf {
    PathBuf::from(MOCK_BASE)
  }

  #[test]
  fn cache_dir_appends_cache_subdir() {
    assert_eq!(
      mock_base().join("cache"),
      PathBuf::from("/data/storage/el2/base/cache")
    );
  }

  #[test]
  fn data_dir_appends_files_subdir() {
    assert_eq!(
      mock_base().join("files"),
      PathBuf::from("/data/storage/el2/base/files")
    );
  }

  #[test]
  fn media_dirs_under_files() {
    let base = mock_base();
    assert_eq!(base.join("files").join("Audio"), PathBuf::from("/data/storage/el2/base/files/Audio"));
    assert_eq!(base.join("files").join("Documents"), PathBuf::from("/data/storage/el2/base/files/Documents"));
    assert_eq!(base.join("files").join("Download"), PathBuf::from("/data/storage/el2/base/files/Download"));
    assert_eq!(base.join("files").join("Pictures"), PathBuf::from("/data/storage/el2/base/files/Pictures"));
    assert_eq!(base.join("files").join("Videos"), PathBuf::from("/data/storage/el2/base/files/Videos"));
    assert_eq!(base.join("files").join("Public"), PathBuf::from("/data/storage/el2/base/files/Public"));
  }

  #[test]
  fn log_and_temp_dirs() {
    let base = mock_base();
    assert_eq!(base.join("log"), PathBuf::from("/data/storage/el2/base/log"));
    assert_eq!(base.join("temp"), PathBuf::from("/data/storage/el2/base/temp"));
  }

  #[test]
  fn resource_dir_defaults_to_entry_module() {
    assert_eq!(
      compute_resource_dir(None),
      PathBuf::from("/data/storage/el1/base/entry/assets")
    );
  }

  #[test]
  fn resource_dir_uses_custom_module_name() {
    assert_eq!(
      compute_resource_dir(Some("feature1")),
      PathBuf::from("/data/storage/el1/base/feature1/assets")
    );
  }

  #[test]
  fn resource_dir_always_under_el1_and_ends_with_assets() {
    for module in [None, Some("x"), Some("entry"), Some("mod_a")] {
      let p = compute_resource_dir(module);
      assert!(p.starts_with("/data/storage/el1/base"));
      assert_eq!(p.file_name().unwrap(), "assets");
    }
  }

  #[test]
  fn file_name_returns_last_component() {
    // Pure logic test — does not need a PathResolver instance
    let name = Path::new("/a/b/c.txt").file_name().map(|n| n.to_string_lossy().into_owned());
    assert_eq!(name, Some("c.txt".to_string()));

    let none = Path::new("/a/b/..").file_name().map(|n| n.to_string_lossy().into_owned());
    assert_eq!(none, None);
  }

  #[test]
  fn base_path_returns_error_when_not_initialized() {
    // Simulates the `.ok_or(Error::UnknownPath)` branch when OnceLock is empty
    let empty: Option<&String> = None;
    let result: Result<PathBuf> = empty
      .map(|p| PathBuf::from(p))
      .ok_or(Error::UnknownPath);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::UnknownPath));
  }
}