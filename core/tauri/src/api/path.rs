// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to file system path operations.

use std::path::{Component, Path, PathBuf};

use crate::{Config, Env, PackageInfo};

use serde_repr::{Deserialize_repr, Serialize_repr};

/// A base directory to be used in [`resolve_path`].
///
/// The base directory is the optional root of a file system operation.
/// If informed by the API call, all paths will be relative to the path of the given directory.
///
/// For more information, check the [`dirs_next` documentation](https://docs.rs/dirs_next/).
#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
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
  /// The Desktop directory.
  Desktop,
  /// The Document directory.
  Document,
  /// The Download directory.
  Download,
  /// The Executable directory.
  Executable,
  /// The Font directory.
  Font,
  /// The Home directory.
  Home,
  /// The Picture directory.
  Picture,
  /// The Public directory.
  Public,
  /// The Runtime directory.
  Runtime,
  /// The Template directory.
  Template,
  /// The Video directory.
  Video,
  /// The Resource directory.
  Resource,
  /// The default App config directory.
  /// Resolves to [`BaseDirectory::Config`].
  App,
  /// The Log directory.
  /// Resolves to `BaseDirectory::Home/Library/Logs/{bundle_identifier}` on macOS
  /// and `BaseDirectory::Config/{bundle_identifier}/logs` on linux and windows.
  Log,
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
      Self::Desktop => "$DESKTOP",
      Self::Document => "$DOCUMENT",
      Self::Download => "$DOWNLOAD",
      Self::Executable => "$EXE",
      Self::Font => "$FONT",
      Self::Home => "$HOME",
      Self::Picture => "$PICTURE",
      Self::Public => "$PUBLIC",
      Self::Runtime => "$RUNTIME",
      Self::Template => "$TEMPLATE",
      Self::Video => "$VIDEO",
      Self::Resource => "$RESOURCE",
      Self::App => "$APP",
      Self::Log => "$LOG",
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
      "$DESKTOP" => Self::Desktop,
      "$DOCUMENT" => Self::Document,
      "$DOWNLOAD" => Self::Download,
      "$EXE" => Self::Executable,
      "$FONT" => Self::Font,
      "$HOME" => Self::Home,
      "$PICTURE" => Self::Picture,
      "$PUBLIC" => Self::Public,
      "$RUNTIME" => Self::Runtime,
      "$TEMPLATE" => Self::Template,
      "$VIDEO" => Self::Video,
      "$RESOURCE" => Self::Resource,
      "$APP" => Self::App,
      "$LOG" => Self::Log,
      _ => return None,
    };
    Some(res)
  }
}

/// Parse the given path, resolving a [`BaseDirectory`] variable if the path starts with one.
///
/// # Examples
///
/// ```rust,no_run
/// use tauri::Manager;
/// tauri::Builder::default()
///   .setup(|app| {
///     let path = tauri::api::path::parse(&app.config(), app.package_info(), &app.env(), "$HOME/.bashrc").unwrap();
///     assert_eq!(path.to_str().unwrap(), "/home/${whoami}/.bashrc");
///     Ok(())
///   });
/// ```
pub fn parse<P: AsRef<Path>>(
  config: &Config,
  package_info: &PackageInfo,
  env: &Env,
  path: P,
) -> crate::api::Result<PathBuf> {
  let mut p = PathBuf::new();
  let mut components = path.as_ref().components();
  if let Some(Component::Normal(str)) = components.next() {
    if let Some(base_directory) = BaseDirectory::from_variable(&str.to_string_lossy()) {
      p.push(resolve_path(
        config,
        package_info,
        env,
        "",
        Some(base_directory),
      )?);
    } else {
      p.push(str);
    }
  }

  for component in components {
    if let Component::ParentDir = component {
      continue;
    }
    p.push(component);
  }

  Ok(p)
}

/// Resolves the path with the optional base directory.
///
/// This is a low level API. If the application has been built,
/// prefer the [path resolver API](`crate::AppHandle#method.path_resolver`).
///
/// # Examples
///
/// ## Before initializing the application
///
/// ```rust,no_run
/// use tauri::{api::path::{BaseDirectory, resolve_path}, Env};
/// // on an actual app, remove the string argument
/// let context = tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json");
/// let path = resolve_path(
///   context.config(),
///   context.package_info(),
///   &Env::default(),
///   "db/tauri.sqlite",
///   Some(BaseDirectory::App))
/// .expect("failed to resolve path");
/// assert_eq!(path.to_str().unwrap(), "/home/${whoami}/.config/com.tauri.app/db/tauri.sqlite");
///
/// tauri::Builder::default().run(context).expect("error while running tauri application");
/// ```
///
/// ## With an initialized app
/// ```rust,no_run
/// use tauri::{api::path::{BaseDirectory, resolve_path}, Manager};
/// tauri::Builder::default()
///   .setup(|app| {
///     let path = resolve_path(
///       &app.config(),
///       app.package_info(),
///       &app.env(),
///       "path/to/something",
///       Some(BaseDirectory::Config)
///     ).expect("failed to resolve path");
///     assert_eq!(path.to_str().unwrap(), "/home/${whoami}/.config/path/to/something");
///     Ok(())
///   });
/// ```
pub fn resolve_path<P: AsRef<Path>>(
  config: &Config,
  package_info: &PackageInfo,
  env: &Env,
  path: P,
  dir: Option<BaseDirectory>,
) -> crate::api::Result<PathBuf> {
  if let Some(base_dir) = dir {
    let resolve_resource = matches!(base_dir, BaseDirectory::Resource);
    let base_dir_path = match base_dir {
      BaseDirectory::Audio => audio_dir(),
      BaseDirectory::Cache => cache_dir(),
      BaseDirectory::Config => config_dir(),
      BaseDirectory::Data => data_dir(),
      BaseDirectory::LocalData => local_data_dir(),
      BaseDirectory::Desktop => desktop_dir(),
      BaseDirectory::Document => document_dir(),
      BaseDirectory::Download => download_dir(),
      BaseDirectory::Executable => executable_dir(),
      BaseDirectory::Font => font_dir(),
      BaseDirectory::Home => home_dir(),
      BaseDirectory::Picture => picture_dir(),
      BaseDirectory::Public => public_dir(),
      BaseDirectory::Runtime => runtime_dir(),
      BaseDirectory::Template => template_dir(),
      BaseDirectory::Video => video_dir(),
      BaseDirectory::Resource => resource_dir(package_info, env),
      BaseDirectory::App => app_dir(config),
      BaseDirectory::Log => log_dir(config),
    };
    if let Some(mut base_dir_path_value) = base_dir_path {
      // use the same path resolution mechanism as the bundler's resource injection algorithm
      if resolve_resource {
        let mut resource_path = PathBuf::new();
        for component in path.as_ref().components() {
          match component {
            Component::Prefix(_) => {}
            Component::RootDir => resource_path.push("_root_"),
            Component::CurDir => {}
            Component::ParentDir => resource_path.push("_up_"),
            Component::Normal(p) => resource_path.push(p),
          }
        }
        base_dir_path_value.push(resource_path);
      } else {
        base_dir_path_value.push(path);
      }
      Ok(base_dir_path_value)
    } else {
      Err(crate::api::Error::Path(
        "unable to determine base dir path".to_string(),
      ))
    }
  } else {
    let mut dir_path = PathBuf::new();
    dir_path.push(path);
    Ok(dir_path)
  }
}

/// Returns the path to the user's audio directory.
pub fn audio_dir() -> Option<PathBuf> {
  dirs_next::audio_dir()
}

/// Returns the path to the user's cache directory.
pub fn cache_dir() -> Option<PathBuf> {
  dirs_next::cache_dir()
}

/// Returns the path to the user's config directory.
pub fn config_dir() -> Option<PathBuf> {
  dirs_next::config_dir()
}

/// Returns the path to the user's data directory.
pub fn data_dir() -> Option<PathBuf> {
  dirs_next::data_dir()
}

/// Returns the path to the user's local data directory.
pub fn local_data_dir() -> Option<PathBuf> {
  dirs_next::data_local_dir()
}

/// Returns the path to the user's desktop directory.
pub fn desktop_dir() -> Option<PathBuf> {
  dirs_next::desktop_dir()
}

/// Returns the path to the user's document directory.
pub fn document_dir() -> Option<PathBuf> {
  dirs_next::document_dir()
}

/// Returns the path to the user's download directory.
pub fn download_dir() -> Option<PathBuf> {
  dirs_next::download_dir()
}

/// Returns the path to the user's executable directory.
pub fn executable_dir() -> Option<PathBuf> {
  dirs_next::executable_dir()
}

/// Returns the path to the user's font directory.
pub fn font_dir() -> Option<PathBuf> {
  dirs_next::font_dir()
}

/// Returns the path to the user's home directory.
pub fn home_dir() -> Option<PathBuf> {
  dirs_next::home_dir()
}

/// Returns the path to the user's picture directory.
pub fn picture_dir() -> Option<PathBuf> {
  dirs_next::picture_dir()
}

/// Returns the path to the user's public directory.
pub fn public_dir() -> Option<PathBuf> {
  dirs_next::public_dir()
}

/// Returns the path to the user's runtime directory.
pub fn runtime_dir() -> Option<PathBuf> {
  dirs_next::runtime_dir()
}

/// Returns the path to the user's template directory.
pub fn template_dir() -> Option<PathBuf> {
  dirs_next::template_dir()
}

/// Returns the path to the user's video dir
pub fn video_dir() -> Option<PathBuf> {
  dirs_next::video_dir()
}

/// Returns the path to the resource directory of this app.
pub fn resource_dir(package_info: &PackageInfo, env: &Env) -> Option<PathBuf> {
  crate::utils::platform::resource_dir(package_info, env).ok()
}

/// Returns the path to the suggested directory for your app config files.
pub fn app_dir(config: &Config) -> Option<PathBuf> {
  dirs_next::config_dir().map(|dir| dir.join(&config.tauri.bundle.identifier))
}

/// Returns the path to the suggested log directory.
pub fn log_dir(config: &Config) -> Option<PathBuf> {
  #[cfg(target_os = "macos")]
  let path = dirs_next::home_dir().map(|dir| {
    dir
      .join("Library/Logs")
      .join(&config.tauri.bundle.identifier)
  });

  #[cfg(not(target_os = "macos"))]
  let path =
    dirs_next::config_dir().map(|dir| dir.join(&config.tauri.bundle.identifier).join("logs"));

  path
}
