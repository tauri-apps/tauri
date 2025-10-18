// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::Result;
use crate::{plugin::PluginHandle, Runtime};
use std::path::{Path, PathBuf};

/// A helper class to access the mobile path APIs.
pub struct PathResolver<R: Runtime>(pub(crate) PluginHandle<R>);

impl<R: Runtime> Clone for PathResolver<R> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

#[derive(serde::Deserialize)]
struct PathResponse {
  path: PathBuf,
}

#[derive(serde::Serialize)]
struct GetFileNameFromUriRequest<'a> {
  uri: &'a str,
}

#[derive(serde::Deserialize)]
struct GetFileNameFromUriResponse {
  name: Option<String>,
}

impl<R: Runtime> PathResolver<R> {
  /// Returns the final component of the `Path`, if there is one.
  ///
  /// If the path is a normal file, this is the file name. If it's the path of a directory, this
  /// is the directory name.
  ///
  /// Returns [`None`] if the path terminates in `..`.
  ///
  /// On Android this also supports checking the file name of content URIs, such as the values returned by the dialog plugin.
  ///
  /// If you are dealing with plain file system paths or not worried about Android content URIs, prefer [`Path::file_name`].
  pub fn file_name(&self, path: &str) -> Option<String> {
    if path.starts_with("content://") || path.starts_with("file://") {
      self
        .0
        .run_mobile_plugin::<GetFileNameFromUriResponse>(
          "getFileNameFromUri",
          GetFileNameFromUriRequest { uri: path },
        )
        .map(|r| r.name)
        .unwrap_or_else(|e| {
          log::error!("failed to get file name from URI: {e}");
          None
        })
    } else {
      Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
    }
  }

  fn call_resolve(&self, dir: &str) -> Result<PathBuf> {
    self
      .0
      .run_mobile_plugin::<PathResponse>(dir, ())
      .map(|r| r.path)
      .map_err(Into::into)
  }

  /// Returns the path to the user's audio directory.
  pub fn audio_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getAudioDir")
  }

  /// Returns the path to the user's cache directory.
  pub fn cache_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getExternalCacheDir")
  }

  /// Returns the path to the user's config directory.
  pub fn config_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getConfigDir")
  }

  /// Returns the path to the user's data directory.
  pub fn data_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getDataDir")
  }

  /// Returns the path to the user's local data directory.
  pub fn local_data_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getDataDir")
  }

  /// Returns the path to the user's document directory.
  pub fn document_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getDocumentDir")
  }

  /// Returns the path to the user's download directory.
  pub fn download_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getDownloadDir")
  }

  /// Returns the path to the user's picture directory.
  pub fn picture_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getPictureDir")
  }

  /// Returns the path to the user's public directory.
  pub fn public_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getPublicDir")
  }

  /// Returns the path to the user's video dir
  pub fn video_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getVideoDir")
  }

  /// Returns the path to the resource directory of this app.
  pub fn resource_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getResourcesDir")
  }

  /// Returns the path to the suggested directory for your app's config files.
  ///
  /// Resolves to [`config_dir`]`/${bundle_identifier}`.
  pub fn app_config_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getConfigDir")
  }

  /// Returns the path to the suggested directory for your app's data files.
  ///
  /// Resolves to [`data_dir`]`/${bundle_identifier}`.
  pub fn app_data_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getDataDir")
  }

  /// Returns the path to the suggested directory for your app's local data files.
  ///
  /// Resolves to [`local_data_dir`]`/${bundle_identifier}`.
  pub fn app_local_data_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getDataDir")
  }

  /// Returns the path to the suggested directory for your app's cache files.
  ///
  /// Resolves to [`cache_dir`]`/${bundle_identifier}`.
  pub fn app_cache_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getCacheDir")
  }

  /// Returns the path to the suggested directory for your app's log files.
  pub fn app_log_dir(&self) -> Result<PathBuf> {
    self
      .call_resolve("getConfigDir")
      .map(|dir| dir.join("logs"))
  }

  /// A temporary directory. Resolves to [`std::env::temp_dir`].
  pub fn temp_dir(&self) -> Result<PathBuf> {
    Ok(std::env::temp_dir())
  }

  /// Returns the path to the user's home directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$HOME`.
  /// - **macOS:** Resolves to `$HOME`.
  /// - **Windows:** Resolves to `{FOLDERID_Profile}`.
  /// - **iOS**: Cannot be written to directly, use one of the app paths instead.
  pub fn home_dir(&self) -> Result<PathBuf> {
    self.call_resolve("getHomeDir")
  }
}
