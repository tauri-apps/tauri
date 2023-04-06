// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::{Component, Path, PathBuf};

use crate::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

use serde_repr::{Deserialize_repr, Serialize_repr};

mod commands;
mod error;
pub use error::*;

#[cfg(target_os = "android")]
mod android;
#[cfg(not(target_os = "android"))]
mod desktop;

#[cfg(target_os = "android")]
use android::PathResolver;
#[cfg(not(target_os = "android"))]
use desktop::PathResolver;

/// A base directory to be used in [`resolve_directory`].
///
/// The base directory is the optional root of a file system operation.
/// If informed by the API call, all paths will be relative to the path of the given directory.
///
/// For more information, check the [`dirs_next` documentation](https://docs.rs/dirs_next/).
#[derive(Serialize_repr, Deserialize_repr, Clone, Copy, Debug)]
#[repr(u16)]
#[non_exhaustive]
pub enum BaseDirectory {
  /// The Audio directory.
  Audio = 1,
  /// The Cache directory.
  Cache,
  /// The Config directory.
  Config,
  /// The Data directory.
  Data,
  /// The LocalData directory.
  LocalData,
  /// The Document directory.
  Document,
  /// The Download directory.
  Download,
  /// The Picture directory.
  Picture,
  /// The Public directory.
  Public,
  /// The Video directory.
  Video,
  /// The Resource directory.
  Resource,
  /// A temporary directory.
  /// Resolves to [`temp_dir`].
  Temp,
  /// The default app config directory.
  /// Resolves to [`BaseDirectory::Config`]`/{bundle_identifier}`.
  AppConfig,
  /// The default app data directory.
  /// Resolves to [`BaseDirectory::Data`]`/{bundle_identifier}`.
  AppData,
  /// The default app local data directory.
  /// Resolves to [`BaseDirectory::LocalData`]`/{bundle_identifier}`.
  AppLocalData,
  /// The default app cache directory.
  /// Resolves to [`BaseDirectory::Cache`]`/{bundle_identifier}`.
  AppCache,
  /// The default app log directory.
  /// Resolves to [`BaseDirectory::Home`]`/Library/Logs/{bundle_identifier}` on macOS
  /// and [`BaseDirectory::Config`]`/{bundle_identifier}/logs` on linux and Windows.
  AppLog,

  /// The Desktop directory.
  #[cfg(not(target_os = "android"))]
  Desktop,
  /// The Executable directory.
  #[cfg(not(target_os = "android"))]
  Executable,
  /// The Font directory.
  #[cfg(not(target_os = "android"))]
  Font,
  /// The Home directory.
  #[cfg(not(target_os = "android"))]
  Home,
  /// The Runtime directory.
  #[cfg(not(target_os = "android"))]
  Runtime,
  /// The Template directory.
  #[cfg(not(target_os = "android"))]
  Template,
}

impl BaseDirectory {
  /// Gets the variable that represents this [`BaseDirectory`] for string paths.
  pub fn variable(self) -> &'static str {
    match self {
      Self::Audio => "$AUDIO",
      Self::Cache => "$CACHE",
      Self::Config => "$CONFIG",
      Self::Data => "$DATA",
      Self::LocalData => "$LOCALDATA",
      Self::Document => "$DOCUMENT",
      Self::Download => "$DOWNLOAD",
      Self::Picture => "$PICTURE",
      Self::Public => "$PUBLIC",
      Self::Video => "$VIDEO",
      Self::Resource => "$RESOURCE",
      Self::Temp => "$TEMP",
      Self::AppConfig => "$APPCONFIG",
      Self::AppData => "$APPDATA",
      Self::AppLocalData => "$APPLOCALDATA",
      Self::AppCache => "$APPCACHE",
      Self::AppLog => "$APPLOG",

      #[cfg(not(target_os = "android"))]
      Self::Desktop => "$DESKTOP",
      #[cfg(not(target_os = "android"))]
      Self::Executable => "$EXE",
      #[cfg(not(target_os = "android"))]
      Self::Font => "$FONT",
      #[cfg(not(target_os = "android"))]
      Self::Home => "$HOME",
      #[cfg(not(target_os = "android"))]
      Self::Runtime => "$RUNTIME",
      #[cfg(not(target_os = "android"))]
      Self::Template => "$TEMPLATE",
    }
  }

  /// Gets the [`BaseDirectory`] associated with the given variable, or [`None`] if the variable doesn't match any.
  pub fn from_variable(variable: &str) -> Option<Self> {
    let res = match variable {
      "$AUDIO" => Self::Audio,
      "$CACHE" => Self::Cache,
      "$CONFIG" => Self::Config,
      "$DATA" => Self::Data,
      "$LOCALDATA" => Self::LocalData,
      "$DOCUMENT" => Self::Document,
      "$DOWNLOAD" => Self::Download,

      "$PICTURE" => Self::Picture,
      "$PUBLIC" => Self::Public,
      "$VIDEO" => Self::Video,
      "$RESOURCE" => Self::Resource,
      "$TEMP" => Self::Temp,
      "$APPCONFIG" => Self::AppConfig,
      "$APPDATA" => Self::AppData,
      "$APPLOCALDATA" => Self::AppLocalData,
      "$APPCACHE" => Self::AppCache,
      "$APPLOG" => Self::AppLog,

      #[cfg(not(target_os = "android"))]
      "$DESKTOP" => Self::Desktop,
      #[cfg(not(target_os = "android"))]
      "$EXE" => Self::Executable,
      #[cfg(not(target_os = "android"))]
      "$FONT" => Self::Font,
      #[cfg(not(target_os = "android"))]
      "$HOME" => Self::Home,
      #[cfg(not(target_os = "android"))]
      "$RUNTIME" => Self::Runtime,
      #[cfg(not(target_os = "android"))]
      "$TEMPLATE" => Self::Template,

      _ => return None,
    };
    Some(res)
  }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the path APIs.
pub trait PathExt<R: Runtime> {
  /// The path resolver.
  fn path(&self) -> &PathResolver<R>;
}

impl<R: Runtime, T: Manager<R>> PathExt<R> for T {
  fn path(&self) -> &PathResolver<R> {
    self.state::<PathResolver<R>>().inner()
  }
}

impl<R: Runtime> PathResolver<R> {
  /// Resolves the path with the base directory.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::{path::{BaseDirectory, PathExt}, Manager};
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let path = app.path().resolve("path/to/something", BaseDirectory::Config)?;
  ///     assert_eq!(path.to_str().unwrap(), "/home/${whoami}/.config/path/to/something");
  ///     Ok(())
  ///   });
  /// ```
  pub fn resolve<P: AsRef<Path>>(&self, path: P, base_directory: BaseDirectory) -> Result<PathBuf> {
    commands::resolve_path::<R>(self, base_directory, Some(path.as_ref().to_path_buf()))
  }

  /// Parse the given path, resolving a [`BaseDirectory`] variable if the path starts with one.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::{Manager, path::PathExt};
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let path = app.path().parse("$HOME/.bashrc")?;
  ///     assert_eq!(path.to_str().unwrap(), "/home/${whoami}/.bashrc");
  ///     Ok(())
  ///   });
  /// ```
  pub fn parse<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
    let mut p = PathBuf::new();
    let mut components = path.as_ref().components();
    match components.next() {
      Some(Component::Normal(str)) => {
        if let Some(base_directory) = BaseDirectory::from_variable(&str.to_string_lossy()) {
          p.push(commands::resolve_path::<R>(self, base_directory, None)?);
        } else {
          p.push(str);
        }
      }
      Some(component) => p.push(component),
      None => (),
    }

    for component in components {
      if let Component::ParentDir = component {
        continue;
      }
      p.push(component);
    }

    Ok(p)
  }
}

/// Initializes the plugin.
pub(crate) fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("path")
    .invoke_handler(crate::generate_handler![
      commands::resolve_directory,
      commands::resolve,
      commands::normalize,
      commands::join,
      commands::dirname,
      commands::extname,
      commands::basename,
      commands::is_absolute
    ])
    .setup(|app, _api| {
      #[cfg(target_os = "android")]
      {
        let handle = _api.register_android_plugin("app.tauri", "PathPlugin")?;
        app.manage(PathResolver(handle));
      }

      #[cfg(not(target_os = "android"))]
      {
        app.manage(PathResolver(app.clone()));
      }

      Ok(())
    })
    .build()
}
