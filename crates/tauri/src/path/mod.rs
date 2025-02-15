// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  path::{Component, Display, Path, PathBuf},
  str::FromStr,
};

use crate::Runtime;

use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub(crate) mod plugin;

use crate::error::*;

#[cfg(target_os = "android")]
mod android;
#[cfg(not(target_os = "android"))]
mod desktop;

#[cfg(target_os = "android")]
pub use android::PathResolver;
#[cfg(not(target_os = "android"))]
pub use desktop::PathResolver;

/// A wrapper for [`PathBuf`] that prevents path traversal.
#[derive(Clone, Debug, Serialize)]
pub struct SafePathBuf(PathBuf);

impl SafePathBuf {
  /// Validates the path for directory traversal vulnerabilities and returns a new [`SafePathBuf`] instance if it is safe.
  pub fn new(path: PathBuf) -> std::result::Result<Self, &'static str> {
    if path.components().any(|x| matches!(x, Component::ParentDir)) {
      Err("cannot traverse directory, rewrite the path without the use of `../`")
    } else {
      Ok(Self(path))
    }
  }

  /// Returns an object that implements [`std::fmt::Display`] for safely printing paths.
  ///
  /// See [`PathBuf#method.display`] for more information.
  pub fn display(&self) -> Display<'_> {
    self.0.display()
  }
}

impl AsRef<Path> for SafePathBuf {
  fn as_ref(&self) -> &Path {
    self.0.as_ref()
  }
}

impl FromStr for SafePathBuf {
  type Err = &'static str;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    Self::new(s.into())
  }
}

impl<'de> Deserialize<'de> for SafePathBuf {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let path = PathBuf::deserialize(deserializer)?;
    SafePathBuf::new(path).map_err(DeError::custom)
  }
}

/// A base directory for a path.
///
/// The base directory is the optional root of a file system operation.
/// If informed by the API call, all paths will be relative to the path of the given directory.
///
/// For more information, check the [`dirs` documentation](https://docs.rs/dirs/).
#[derive(Serialize_repr, Deserialize_repr, Clone, Copy, Debug)]
#[repr(u16)]
#[non_exhaustive]
pub enum BaseDirectory {
  /// The Audio directory.
  Audio = 1,
  /// The Cache directory.
  Cache = 2,
  /// The Config directory.
  Config = 3,
  /// The Data directory.
  Data = 4,
  /// The LocalData directory.
  LocalData = 5,
  /// The Document directory.
  Document = 6,
  /// The Download directory.
  Download = 7,
  /// The Picture directory.
  Picture = 8,
  /// The Public directory.
  Public = 9,
  /// The Video directory.
  Video = 10,
  /// The Resource directory.
  Resource = 11,
  /// A temporary directory. Resolves to [`std::env::temp_dir`].
  Temp = 12,
  /// The default app config directory.
  /// Resolves to [`BaseDirectory::Config`]`/{bundle_identifier}`.
  AppConfig = 13,
  /// The default app data directory.
  /// Resolves to [`BaseDirectory::Data`]`/{bundle_identifier}`.
  AppData = 14,
  /// The default app local data directory.
  /// Resolves to [`BaseDirectory::LocalData`]`/{bundle_identifier}`.
  AppLocalData = 15,
  /// The default app cache directory.
  /// Resolves to [`BaseDirectory::Cache`]`/{bundle_identifier}`.
  AppCache = 16,
  /// The default app log directory.
  /// Resolves to [`BaseDirectory::Home`]`/Library/Logs/{bundle_identifier}` on macOS
  /// and [`BaseDirectory::Config`]`/{bundle_identifier}/logs` on linux and Windows.
  AppLog = 17,
  /// The Desktop directory.
  #[cfg(not(target_os = "android"))]
  Desktop = 18,
  /// The Executable directory.
  #[cfg(not(target_os = "android"))]
  Executable = 19,
  /// The Font directory.
  #[cfg(not(target_os = "android"))]
  Font = 20,
  /// The Home directory.
  Home = 21,
  /// The Runtime directory.
  #[cfg(not(target_os = "android"))]
  Runtime = 22,
  /// The Template directory.
  #[cfg(not(target_os = "android"))]
  Template = 23,
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
      Self::Home => "$HOME",

      #[cfg(not(target_os = "android"))]
      Self::Desktop => "$DESKTOP",
      #[cfg(not(target_os = "android"))]
      Self::Executable => "$EXE",
      #[cfg(not(target_os = "android"))]
      Self::Font => "$FONT",
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
      "$HOME" => Self::Home,

      #[cfg(not(target_os = "android"))]
      "$DESKTOP" => Self::Desktop,
      #[cfg(not(target_os = "android"))]
      "$EXE" => Self::Executable,
      #[cfg(not(target_os = "android"))]
      "$FONT" => Self::Font,
      #[cfg(not(target_os = "android"))]
      "$RUNTIME" => Self::Runtime,
      #[cfg(not(target_os = "android"))]
      "$TEMPLATE" => Self::Template,

      _ => return None,
    };
    Some(res)
  }
}

impl<R: Runtime> PathResolver<R> {
  /// Resolves the path with the base directory.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::{path::BaseDirectory, Manager};
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let path = app.path().resolve("path/to/something", BaseDirectory::Config)?;
  ///     assert_eq!(path.to_str().unwrap(), "/home/${whoami}/.config/path/to/something");
  ///     Ok(())
  ///   });
  /// ```
  pub fn resolve<P: AsRef<Path>>(&self, path: P, base_directory: BaseDirectory) -> Result<PathBuf> {
    resolve_path::<R>(self, base_directory, Some(path.as_ref().to_path_buf()))
  }

  /// Parse the given path, resolving a [`BaseDirectory`] variable if the path starts with one.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::Manager;
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
          p.push(resolve_path::<R>(self, base_directory, None)?);
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

fn resolve_path<R: Runtime>(
  resolver: &PathResolver<R>,
  directory: BaseDirectory,
  path: Option<PathBuf>,
) -> Result<PathBuf> {
  let resolve_resource = matches!(directory, BaseDirectory::Resource);
  let mut base_dir_path = match directory {
    BaseDirectory::Audio => resolver.audio_dir(),
    BaseDirectory::Cache => resolver.cache_dir(),
    BaseDirectory::Config => resolver.config_dir(),
    BaseDirectory::Data => resolver.data_dir(),
    BaseDirectory::LocalData => resolver.local_data_dir(),
    BaseDirectory::Document => resolver.document_dir(),
    BaseDirectory::Download => resolver.download_dir(),
    BaseDirectory::Picture => resolver.picture_dir(),
    BaseDirectory::Public => resolver.public_dir(),
    BaseDirectory::Video => resolver.video_dir(),
    BaseDirectory::Resource => resolver.resource_dir(),
    BaseDirectory::Temp => resolver.temp_dir(),
    BaseDirectory::AppConfig => resolver.app_config_dir(),
    BaseDirectory::AppData => resolver.app_data_dir(),
    BaseDirectory::AppLocalData => resolver.app_local_data_dir(),
    BaseDirectory::AppCache => resolver.app_cache_dir(),
    BaseDirectory::AppLog => resolver.app_log_dir(),
    BaseDirectory::Home => resolver.home_dir(),
    #[cfg(not(target_os = "android"))]
    BaseDirectory::Desktop => resolver.desktop_dir(),
    #[cfg(not(target_os = "android"))]
    BaseDirectory::Executable => resolver.executable_dir(),
    #[cfg(not(target_os = "android"))]
    BaseDirectory::Font => resolver.font_dir(),
    #[cfg(not(target_os = "android"))]
    BaseDirectory::Runtime => resolver.runtime_dir(),
    #[cfg(not(target_os = "android"))]
    BaseDirectory::Template => resolver.template_dir(),
  }?;

  if let Some(path) = path {
    // use the same path resolution mechanism as the bundler's resource injection algorithm
    if resolve_resource {
      let mut resource_path = PathBuf::new();
      for component in path.components() {
        match component {
          Component::Prefix(_) => {}
          Component::RootDir => resource_path.push("_root_"),
          Component::CurDir => {}
          Component::ParentDir => resource_path.push("_up_"),
          Component::Normal(p) => resource_path.push(p),
        }
      }
      base_dir_path.push(resource_path);
    } else {
      base_dir_path.push(path);
    }
  }

  Ok(base_dir_path)
}

#[cfg(test)]
mod test {
  use super::SafePathBuf;
  use quickcheck::{Arbitrary, Gen};

  use std::path::PathBuf;

  impl Arbitrary for SafePathBuf {
    fn arbitrary(g: &mut Gen) -> Self {
      Self(PathBuf::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
      Box::new(self.0.shrink().map(SafePathBuf))
    }
  }
}
