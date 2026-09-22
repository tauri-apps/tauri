// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{Error, Result};
use crate::{AppHandle, Manager, Runtime, path::BaseDirectory};
use std::path::{Component, Path, PathBuf};
use tauri_utils::config::AppDirectoriesOverride;

/// The path resolver is a helper class for general and application-specific path APIs.
pub struct PathResolver<R: Runtime>(pub(crate) AppHandle<R>);

/// An app-specific directory that can be overridden with the `app > appDirectoriesOverride` config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppDirectory {
  Config,
  Data,
  LocalData,
  Cache,
  Log,
}

impl AppDirectory {
  /// The subdirectory this directory resolves to when a single root overrides all app directories.
  fn root_override_subdirectory(self) -> Option<&'static str> {
    match self {
      Self::Cache => Some("caches"),
      Self::Log => Some("logs"),
      Self::Config | Self::Data | Self::LocalData => None,
    }
  }
}

impl<R: Runtime> Clone for PathResolver<R> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
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
    Path::new(path)
      .file_name()
      .map(|name| name.to_string_lossy().into_owned())
  }

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
  /// - **iOS**: Cannot be written to directly, use one of the app paths instead.
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
  ///
  /// ## Platform-specific
  ///
  /// Although we provide the exact path where this function resolves to,
  /// this is not a contract and things might change in the future
  ///
  /// - **Windows:** Resolves to the directory that contains the main executable.
  /// - **Linux:** When running in an AppImage, the `APPDIR` variable will be set to
  ///   the mounted location of the app, and the resource dir will be `${APPDIR}/usr/lib/${exe_name}`.
  ///   If not running in an AppImage, the path is `/usr/lib/${exe_name}`.
  ///   When running the app from `src-tauri/target/(debug|release)/`, the path is `${exe_dir}/../lib/${exe_name}`.
  /// - **macOS:** Resolves to `${exe_dir}/../Resources` (inside .app).
  /// - **iOS:** Resolves to `${exe_dir}/assets`.
  /// - **Android:** Currently the resources are stored in the APK as assets so it's not a normal file system path,
  ///   we return a special URI prefix `asset://localhost/` here that can be used with the [file system plugin](https://tauri.app/plugin/file-system/),
  ///   with that, you can read the files through [`FsExt::fs`](https://docs.rs/tauri-plugin-fs/latest/tauri_plugin_fs/trait.FsExt.html#tymethod.fs)
  ///   like this: `app.fs().read_to_string(app.path().resource_dir().unwrap().join("resource"));`
  pub fn resource_dir(&self) -> Result<PathBuf> {
    crate::utils::platform::resource_dir(self.0.package_info(), &self.0.env())
      .map_err(|_| Error::UnknownPath)
  }

  /// Returns the path to the suggested directory for your app's config files.
  ///
  /// Resolves to [`config_dir`](Self::config_dir)`/${bundle_identifier}`,
  /// unless overridden with the [`app > appDirectoriesOverride`](crate::utils::config::AppConfig::app_directories_override) config.
  pub fn app_config_dir(&self) -> Result<PathBuf> {
    self.app_dir(AppDirectory::Config, || {
      dirs::config_dir()
        .ok_or(Error::UnknownPath)
        .map(|dir| dir.join(&self.0.config().identifier))
    })
  }

  /// Returns the path to the suggested directory for your app's data files.
  ///
  /// Resolves to [`data_dir`](Self::data_dir)`/${bundle_identifier}`,
  /// unless overridden with the [`app > appDirectoriesOverride`](crate::utils::config::AppConfig::app_directories_override) config.
  pub fn app_data_dir(&self) -> Result<PathBuf> {
    self.app_dir(AppDirectory::Data, || {
      dirs::data_dir()
        .ok_or(Error::UnknownPath)
        .map(|dir| dir.join(&self.0.config().identifier))
    })
  }

  /// Returns the path to the suggested directory for your app's local data files.
  ///
  /// Resolves to [`local_data_dir`](Self::local_data_dir)`/${bundle_identifier}`,
  /// unless overridden with the [`app > appDirectoriesOverride`](crate::utils::config::AppConfig::app_directories_override) config.
  ///
  /// On Windows and Linux this is also the default data directory of the webviews.
  pub fn app_local_data_dir(&self) -> Result<PathBuf> {
    self.app_dir(AppDirectory::LocalData, || {
      dirs::data_local_dir()
        .ok_or(Error::UnknownPath)
        .map(|dir| dir.join(&self.0.config().identifier))
    })
  }

  /// Returns the path to the suggested directory for your app's cache files.
  ///
  /// Resolves to [`cache_dir`](Self::cache_dir)`/${bundle_identifier}`,
  /// unless overridden with the [`app > appDirectoriesOverride`](crate::utils::config::AppConfig::app_directories_override) config
  /// (a single root override resolves to `<root>/caches`).
  pub fn app_cache_dir(&self) -> Result<PathBuf> {
    self.app_dir(AppDirectory::Cache, || {
      dirs::cache_dir()
        .ok_or(Error::UnknownPath)
        .map(|dir| dir.join(&self.0.config().identifier))
    })
  }

  /// Returns the path to the suggested directory for your app's log files.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Resolves to [`local_data_dir`](Self::local_data_dir)`/${bundle_identifier}/logs`.
  /// - **macOS:** Resolves to [`home_dir`](Self::home_dir)`/Library/Logs/${bundle_identifier}`
  /// - **Windows:** Resolves to [`local_data_dir`](Self::local_data_dir)`/${bundle_identifier}/logs`.
  ///
  /// All of them can be overridden with the [`app > appDirectoriesOverride`](crate::utils::config::AppConfig::app_directories_override) config
  /// (a single root override resolves to `<root>/logs`).
  pub fn app_log_dir(&self) -> Result<PathBuf> {
    self.app_dir(AppDirectory::Log, || {
      #[cfg(target_os = "macos")]
      let path = dirs::home_dir()
        .ok_or(Error::UnknownPath)
        .map(|dir| dir.join("Library/Logs").join(&self.0.config().identifier));

      #[cfg(not(target_os = "macos"))]
      let path = dirs::data_local_dir()
        .ok_or(Error::UnknownPath)
        .map(|dir| dir.join(&self.0.config().identifier).join("logs"));

      path
    })
  }

  /// A temporary directory. Resolves to [`std::env::temp_dir`].
  pub fn temp_dir(&self) -> Result<PathBuf> {
    Ok(std::env::temp_dir())
  }

  /// Resolves an app directory, honoring the `app > appDirectoriesOverride` config.
  fn app_dir(
    &self,
    dir: AppDirectory,
    default: impl FnOnce() -> Result<PathBuf>,
  ) -> Result<PathBuf> {
    match self.app_directory_override(dir)? {
      Some(path) => Ok(path),
      None => default(),
    }
  }

  /// Resolves the override configured for the given app directory, if any.
  ///
  /// Mobile apps are sandboxed, so the override is ignored on iOS.
  fn app_directory_override(&self, dir: AppDirectory) -> Result<Option<PathBuf>> {
    if cfg!(target_os = "ios") {
      return Ok(None);
    }

    let Some(config) = &self.0.config().app.app_directories_override else {
      return Ok(None);
    };

    let (path, subdirectory) = match config {
      AppDirectoriesOverride::Root(root) => (root, dir.root_override_subdirectory()),
      AppDirectoriesOverride::Directories(directories) => {
        let path = match dir {
          AppDirectory::Config => &directories.config,
          AppDirectory::Data => &directories.data,
          AppDirectory::LocalData => &directories.local_data,
          AppDirectory::Cache => &directories.cache,
          AppDirectory::Log => &directories.log,
        };
        match path {
          Some(path) => (path, None),
          None => return Ok(None),
        }
      }
    };

    let mut path = self.resolve_override_path(path)?;
    if let Some(subdirectory) = subdirectory {
      path.push(subdirectory);
    }

    Ok(Some(path))
  }

  /// Resolves a path from the `app > appDirectoriesOverride` config:
  ///
  /// - a path starting with a base directory variable (e.g. `$DATA/my-app`) is resolved against that directory,
  /// - an absolute path is used as is,
  /// - any other path is resolved relative to the [app binary directory](Self::app_binary_dir).
  fn resolve_override_path(&self, path: &Path) -> Result<PathBuf> {
    let mut components = path.components();
    let first = components.next();

    if let Some(Component::Normal(first)) = first {
      if let Some(variable) = first.to_str().filter(|s| s.starts_with('$')) {
        let base_directory = BaseDirectory::from_variable(variable).ok_or_else(|| {
          Error::InvalidAppDirectoriesOverride(
            path.to_path_buf(),
            format!("unknown base directory variable `{variable}`"),
          )
        })?;

        if matches!(
          base_directory,
          BaseDirectory::AppConfig
            | BaseDirectory::AppData
            | BaseDirectory::AppLocalData
            | BaseDirectory::AppCache
            | BaseDirectory::AppLog
        ) {
          return Err(Error::InvalidAppDirectoriesOverride(
            path.to_path_buf(),
            format!("`{variable}` refers to an app directory, which is what is being overridden"),
          ));
        }

        // unlike `parse`, `resolve` keeps `..` components
        return self
          .resolve(components.as_path(), base_directory)
          .map(normalize);
      }
    }

    if path.is_absolute() {
      return Ok(normalize(path));
    }

    // Windows root-relative (`\foo`) and drive-relative (`C:foo`) paths would replace the base directory on join
    if path.has_root() || matches!(first, Some(Component::Prefix(_))) {
      return Err(Error::InvalidAppDirectoriesOverride(
        path.to_path_buf(),
        "root-relative and drive-relative paths are not supported".into(),
      ));
    }

    Ok(normalize(self.app_binary_dir()?.join(path)))
  }

  /// The directory relative app directory overrides are resolved against: the directory containing the app binary.
  ///
  /// When running from an AppImage on Linux, this is the directory containing the AppImage file,
  /// and when running from a `.app` bundle on macOS, the directory containing the bundle.
  fn app_binary_dir(&self) -> Result<PathBuf> {
    let binary = crate::process::current_binary(&self.0.env())?;
    let dir = binary.parent().ok_or(Error::NoParent)?;

    #[cfg(target_os = "macos")]
    if let Some(bundle_parent) = macos_bundle_parent(dir) {
      return Ok(bundle_parent);
    }

    Ok(dir.to_path_buf())
  }
}

/// Removes `.` components and trailing separators from a path, keeping `..` components.
fn normalize(path: impl AsRef<Path>) -> PathBuf {
  path.as_ref().components().collect()
}

/// For a `<dir>/<name>.app/Contents/MacOS` directory, returns `<dir>`.
#[cfg(any(target_os = "macos", test))]
fn macos_bundle_parent(macos_dir: &Path) -> Option<PathBuf> {
  use std::ffi::OsStr;

  if macos_dir.file_name() != Some(OsStr::new("MacOS")) {
    return None;
  }

  let contents_dir = macos_dir.parent()?;
  if contents_dir.file_name() != Some(OsStr::new("Contents")) {
    return None;
  }

  let bundle = contents_dir.parent()?;
  if bundle.extension() != Some(OsStr::new("app")) {
    return None;
  }

  bundle.parent().map(Path::to_path_buf)
}
