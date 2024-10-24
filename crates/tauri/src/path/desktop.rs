// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{Error, Result};
use crate::{AppHandle, Manager, Runtime};
use std::path::PathBuf;

/// The path resolver is a helper class for general and application-specific path APIs.
pub struct PathResolver<R: Runtime>(pub(crate) AppHandle<R>);

impl<R: Runtime> PathResolver<R> {
  /// Returns the path to the user's audio directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_MUSIC_DIR`.
  /// - **macOS:** Resolves to `$HOME/Music`.
  /// - **Windows:** Resolves to `{FOLDERID_Music}`.
  pub fn audio_dir(&self) -> Result<PathBuf> {
    dirs::audio_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's cache directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$XDG_CACHE_HOME` or `$HOME/.cache`.
  /// - **macOS:** Resolves to `$HOME/Library/Caches`.
  /// - **Windows:** Resolves to `{FOLDERID_LocalAppData}`.
  pub fn cache_dir(&self) -> Result<PathBuf> {
    dirs::cache_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's config directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$XDG_CONFIG_HOME` or `$HOME/.config`.
  /// - **macOS:** Resolves to `$HOME/Library/Application Support`.
  /// - **Windows:** Resolves to `{FOLDERID_RoamingAppData}`.
  pub fn config_dir(&self) -> Result<PathBuf> {
    dirs::config_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's data directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$XDG_DATA_HOME` or `$HOME/.local/share`.
  /// - **macOS:** Resolves to `$HOME/Library/Application Support`.
  /// - **Windows:** Resolves to `{FOLDERID_RoamingAppData}`.
  pub fn data_dir(&self) -> Result<PathBuf> {
    dirs::data_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's local data directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$XDG_DATA_HOME` or `$HOME/.local/share`.
  /// - **macOS:** Resolves to `$HOME/Library/Application Support`.
  /// - **Windows:** Resolves to `{FOLDERID_LocalAppData}`.
  pub fn local_data_dir(&self) -> Result<PathBuf> {
    dirs::data_local_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's desktop directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_DESKTOP_DIR`.
  /// - **macOS:** Resolves to `$HOME/Desktop`.
  /// - **Windows:** Resolves to `{FOLDERID_Desktop}`.
  pub fn desktop_dir(&self) -> Result<PathBuf> {
    dirs::desktop_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's document directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_DOCUMENTS_DIR`.
  /// - **macOS:** Resolves to `$HOME/Documents`.
  /// - **Windows:** Resolves to `{FOLDERID_Documents}`.
  pub fn document_dir(&self) -> Result<PathBuf> {
    dirs::document_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's download directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_DOWNLOAD_DIR`.
  /// - **macOS:** Resolves to `$HOME/Downloads`.
  /// - **Windows:** Resolves to `{FOLDERID_Downloads}`.
  pub fn download_dir(&self) -> Result<PathBuf> {
    dirs::download_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's executable directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$XDG_BIN_HOME/../bin` or `$XDG_DATA_HOME/../bin` or `$HOME/.local/bin`.
  /// - **macOS:** Not supported.
  /// - **Windows:** Not supported.
  pub fn executable_dir(&self) -> Result<PathBuf> {
    dirs::executable_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's font directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$XDG_DATA_HOME/fonts` or `$HOME/.local/share/fonts`.
  /// - **macOS:** Resolves to `$HOME/Library/Fonts`.
  /// - **Windows:** Not supported.
  pub fn font_dir(&self) -> Result<PathBuf> {
    dirs::font_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's home directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$HOME`.
  /// - **macOS:** Resolves to `$HOME`.
  /// - **Windows:** Resolves to `{FOLDERID_Profile}`.
  pub fn home_dir(&self) -> Result<PathBuf> {
    dirs::home_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's picture directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_PICTURES_DIR`.
  /// - **macOS:** Resolves to `$HOME/Pictures`.
  /// - **Windows:** Resolves to `{FOLDERID_Pictures}`.
  pub fn picture_dir(&self) -> Result<PathBuf> {
    dirs::picture_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's public directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_PUBLICSHARE_DIR`.
  /// - **macOS:** Resolves to `$HOME/Public`.
  /// - **Windows:** Resolves to `{FOLDERID_Public}`.
  pub fn public_dir(&self) -> Result<PathBuf> {
    dirs::public_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's runtime directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to `$XDG_RUNTIME_DIR`.
  /// - **macOS:** Not supported.
  /// - **Windows:** Not supported.
  pub fn runtime_dir(&self) -> Result<PathBuf> {
    dirs::runtime_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's template directory.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_TEMPLATES_DIR`.
  /// - **macOS:** Not supported.
  /// - **Windows:** Resolves to `{FOLDERID_Templates}`.
  pub fn template_dir(&self) -> Result<PathBuf> {
    dirs::template_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the user's video dir
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_VIDEOS_DIR`.
  /// - **macOS:** Resolves to `$HOME/Movies`.
  /// - **Windows:** Resolves to `{FOLDERID_Videos}`.
  pub fn video_dir(&self) -> Result<PathBuf> {
    dirs::video_dir().ok_or(Error::UnknownPath)
  }

  /// Returns the path to the resource directory of this app.
  pub fn resource_dir(&self) -> Result<PathBuf> {
    crate::utils::platform::resource_dir(self.0.package_info(), &self.0.env())
      .map_err(|_| Error::UnknownPath)
  }

  /// Returns the path to the suggested directory for your app's config files.
  ///
  /// Resolves to [`config_dir`](Self::config_dir)`/${bundle_identifier}`.
  pub fn app_config_dir(&self) -> Result<PathBuf> {
    dirs::config_dir()
      .ok_or(Error::UnknownPath)
      .map(|dir| dir.join(&self.0.config().identifier))
  }

  /// Returns the path to the suggested directory for your app's data files.
  ///
  /// Resolves to [`data_dir`](Self::data_dir)`/${bundle_identifier}`.
  pub fn app_data_dir(&self) -> Result<PathBuf> {
    dirs::data_dir()
      .ok_or(Error::UnknownPath)
      .map(|dir| dir.join(&self.0.config().identifier))
  }

  /// Returns the path to the suggested directory for your app's local data files.
  ///
  /// Resolves to [`local_data_dir`](Self::local_data_dir)`/${bundle_identifier}`.
  pub fn app_local_data_dir(&self) -> Result<PathBuf> {
    dirs::data_local_dir()
      .ok_or(Error::UnknownPath)
      .map(|dir| dir.join(&self.0.config().identifier))
  }

  /// Returns the path to the suggested directory for your app's cache files.
  ///
  /// Resolves to [`cache_dir`](Self::cache_dir)`/${bundle_identifier}`.
  pub fn app_cache_dir(&self) -> Result<PathBuf> {
    dirs::cache_dir()
      .ok_or(Error::UnknownPath)
      .map(|dir| dir.join(&self.0.config().identifier))
  }

  /// Returns the path to the suggested directory for your app's log files.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`local_data_dir`](Self::local_data_dir)`/${bundle_identifier}/logs`.
  /// - **macOS:** Resolves to [`home_dir`](Self::home_dir)`/Library/Logs/${bundle_identifier}`
  /// - **Windows:** Resolves to [`local_data_dir`](Self::local_data_dir)`/${bundle_identifier}/logs`.
  pub fn app_log_dir(&self) -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    let path = dirs::home_dir()
      .ok_or(Error::UnknownPath)
      .map(|dir| dir.join("Library/Logs").join(&self.0.config().identifier));

    #[cfg(not(target_os = "macos"))]
    let path = dirs::data_local_dir()
      .ok_or(Error::UnknownPath)
      .map(|dir| dir.join(&self.0.config().identifier).join("logs"));

    path
  }

  /// A temporary directory. Resolves to [`std::env::temp_dir`].
  pub fn temp_dir(&self) -> Result<PathBuf> {
    Ok(std::env::temp_dir())
  }
}
