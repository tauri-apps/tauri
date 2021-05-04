// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  env,
  path::{Path, PathBuf},
};

use crate::Config;

use serde_repr::{Deserialize_repr, Serialize_repr};

/// A Base Directory to use.
/// The base directory is the optional root of a FS operation.
/// If informed by the API call, all paths will be relative to the path of the given directory.
///
/// For more information, check the [dirs_next documentation](https://docs.rs/dirs_next/).
#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
#[repr(u16)]
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
  /// Resolves to ${BaseDirectory::Config}/${config.tauri.bundle.identifier}
  App,
  /// The current working directory.
  Current,
}

/// Resolves the path with the optional base directory.
///
/// # Example
/// ```
/// use tauri::api::path::{resolve_path, BaseDirectory};
/// let path = resolve_path("path/to/something", Some(BaseDirectory::Config))
///   .expect("failed to resolve path");
/// // path is equal to "/home/${whoami}/.config/path/to/something" on Linux
/// ```
pub fn resolve_path<P: AsRef<Path>>(
  config: &Config,
  path: P,
  dir: Option<BaseDirectory>,
) -> crate::api::Result<PathBuf> {
  if let Some(base_dir) = dir {
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
      BaseDirectory::Resource => resource_dir(),
      BaseDirectory::App => app_dir(config),
      BaseDirectory::Current => Some(env::current_dir()?),
    };
    if let Some(mut base_dir_path_value) = base_dir_path {
      base_dir_path_value.push(path);
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
pub fn resource_dir() -> Option<PathBuf> {
  crate::api::platform::resource_dir().ok()
}

/// Returns the path to the suggested directory for your app config files.
pub fn app_dir(config: &Config) -> Option<PathBuf> {
  dirs_next::config_dir().map(|dir| dir.join(&config.tauri.bundle.identifier))
}
