// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{Error, Result};
use crate::{plugin::mobile::ohos_plugin_directory, AppHandle, Runtime};
use std::path::{Path, PathBuf};

/// OHOS resolves app paths from the native Ability's sandbox, not Linux home directories.
pub struct PathResolver<R: Runtime>(pub(crate) AppHandle<R>);
impl<R: Runtime> Clone for PathResolver<R> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}
impl<R: Runtime> PathResolver<R> {
  /// Returns the final component of the supplied path.
  pub fn file_name(&self, path: &str) -> Option<String> {
    Path::new(path)
      .file_name()
      .map(|name| name.to_string_lossy().into_owned())
  }
  /// Returns the app-scoped path supplied by the OHOS Ability.
  pub fn data_dir(&self) -> Result<PathBuf> {
    ohos_plugin_directory("files")
  }
  /// Returns the app-scoped path supplied by the OHOS Ability.
  pub fn local_data_dir(&self) -> Result<PathBuf> {
    ohos_plugin_directory("files")
  }
  /// Returns the app-scoped path supplied by the OHOS Ability.
  pub fn home_dir(&self) -> Result<PathBuf> {
    ohos_plugin_directory("files")
  }
  /// Returns the app-scoped path supplied by the OHOS Ability.
  pub fn app_data_dir(&self) -> Result<PathBuf> {
    ohos_plugin_directory("files")
  }
  /// Returns the app-scoped path supplied by the OHOS Ability.
  pub fn app_local_data_dir(&self) -> Result<PathBuf> {
    ohos_plugin_directory("files")
  }
  /// Returns the app-scoped path supplied by the OHOS Ability.
  pub fn cache_dir(&self) -> Result<PathBuf> {
    ohos_plugin_directory("cache")
  }
  /// Returns the app-scoped path supplied by the OHOS Ability.
  pub fn app_cache_dir(&self) -> Result<PathBuf> {
    ohos_plugin_directory("cache")
  }
  /// Returns the app-scoped path supplied by the OHOS Ability.
  pub fn config_dir(&self) -> Result<PathBuf> {
    Ok(ohos_plugin_directory("files")?.join("config"))
  }
  /// Returns the app-scoped path supplied by the OHOS Ability.
  pub fn app_config_dir(&self) -> Result<PathBuf> {
    Ok(ohos_plugin_directory("files")?.join("config"))
  }
  /// Returns the app-scoped path supplied by the OHOS Ability.
  pub fn app_log_dir(&self) -> Result<PathBuf> {
    Ok(ohos_plugin_directory("files")?.join("logs"))
  }
  /// Returns the app-scoped path supplied by the OHOS Ability.
  pub fn temp_dir(&self) -> Result<PathBuf> {
    ohos_plugin_directory("temp")
  }
  /// Use a native picker or ResourceManager instead of assuming a globally accessible directory.
  pub fn audio_dir(&self) -> Result<PathBuf> {
    Err(Error::UnknownPath)
  }
  /// Use a native picker or ResourceManager instead of assuming a globally accessible directory.
  pub fn desktop_dir(&self) -> Result<PathBuf> {
    Err(Error::UnknownPath)
  }
  /// Use a native picker or ResourceManager instead of assuming a globally accessible directory.
  pub fn document_dir(&self) -> Result<PathBuf> {
    Err(Error::UnknownPath)
  }
  /// Use a native picker or ResourceManager instead of assuming a globally accessible directory.
  pub fn download_dir(&self) -> Result<PathBuf> {
    Err(Error::UnknownPath)
  }
  /// Use a native picker or ResourceManager instead of assuming a globally accessible directory.
  pub fn executable_dir(&self) -> Result<PathBuf> {
    Err(Error::UnknownPath)
  }
  /// Use a native picker or ResourceManager instead of assuming a globally accessible directory.
  pub fn font_dir(&self) -> Result<PathBuf> {
    Err(Error::UnknownPath)
  }
  /// Use a native picker or ResourceManager instead of assuming a globally accessible directory.
  pub fn picture_dir(&self) -> Result<PathBuf> {
    Err(Error::UnknownPath)
  }
  /// Use a native picker or ResourceManager instead of assuming a globally accessible directory.
  pub fn public_dir(&self) -> Result<PathBuf> {
    Err(Error::UnknownPath)
  }
  /// Use a native picker or ResourceManager instead of assuming a globally accessible directory.
  pub fn runtime_dir(&self) -> Result<PathBuf> {
    Err(Error::UnknownPath)
  }
  /// Use a native picker or ResourceManager instead of assuming a globally accessible directory.
  pub fn template_dir(&self) -> Result<PathBuf> {
    Err(Error::UnknownPath)
  }
  /// Use a native picker or ResourceManager instead of assuming a globally accessible directory.
  pub fn video_dir(&self) -> Result<PathBuf> {
    Err(Error::UnknownPath)
  }
  /// Use a native picker or ResourceManager instead of assuming a globally accessible directory.
  pub fn resource_dir(&self) -> Result<PathBuf> {
    Err(Error::UnknownPath)
  }
}
