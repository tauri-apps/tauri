// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Tauri utility helpers
#![warn(missing_docs, rust_2018_idioms)]

use std::fmt::Display;

use semver::Version;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod assets;
pub mod config;
pub mod html;
pub mod io;
pub mod platform;
/// Prepare application resources and sidecars.
#[cfg(feature = "resources")]
pub mod resources;

/// Application pattern.
pub mod pattern;

/// `tauri::App` package information.
#[derive(Debug, Clone)]
pub struct PackageInfo {
  /// App name
  pub name: String,
  /// App version
  pub version: Version,
  /// The crate authors.
  pub authors: &'static str,
  /// The crate description.
  pub description: &'static str,
}

impl PackageInfo {
  /// Returns the application package name.
  /// On macOS and Windows it's the `name` field, and on Linux it's the `name` in `kebab-case`.
  pub fn package_name(&self) -> String {
    #[cfg(target_os = "linux")]
    {
      use heck::ToKebabCase;
      self.name.clone().to_kebab_case()
    }
    #[cfg(not(target_os = "linux"))]
    self.name.clone()
  }
}

/// System theme.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum Theme {
  /// Light theme.
  Light,
  /// Dark theme.
  Dark,
}

impl Serialize for Theme {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}

impl<'de> Deserialize<'de> for Theme {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Ok(match s.to_lowercase().as_str() {
      "dark" => Self::Dark,
      _ => Self::Light,
    })
  }
}

impl Display for Theme {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Light => "light",
        Self::Dark => "dark",
      }
    )
  }
}

/// Information about environment variables.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Env {
  /// The APPIMAGE environment variable.
  #[cfg(target_os = "linux")]
  pub appimage: Option<std::ffi::OsString>,
  /// The APPDIR environment variable.
  #[cfg(target_os = "linux")]
  pub appdir: Option<std::ffi::OsString>,
}

#[allow(clippy::derivable_impls)]
impl Default for Env {
  fn default() -> Self {
    #[cfg(target_os = "linux")]
    {
      let env = Self {
        #[cfg(target_os = "linux")]
        appimage: std::env::var_os("APPIMAGE"),
        #[cfg(target_os = "linux")]
        appdir: std::env::var_os("APPDIR"),
      };
      if env.appimage.is_some() || env.appdir.is_some() {
        // validate that we're actually running on an AppImage
        // an AppImage is mounted to `/$TEMPDIR/.mount_${appPrefix}${hash}`
        // see https://github.com/AppImage/AppImageKit/blob/1681fd84dbe09c7d9b22e13cdb16ea601aa0ec47/src/runtime.c#L501
        // note that it is safe to use `std::env::current_exe` here since we just loaded an AppImage.
        let is_temp = std::env::current_exe()
          .map(|p| {
            p.display()
              .to_string()
              .starts_with(&format!("{}/.mount_", std::env::temp_dir().display()))
          })
          .unwrap_or(true);

        if !is_temp {
          panic!("`APPDIR` or `APPIMAGE` environment variable found but this application was not detected as an AppImage; this might be a security issue.");
        }
      }
      env
    }
    #[cfg(not(target_os = "linux"))]
    {
      Self {}
    }
  }
}

/// The result type of `tauri-utils`.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type of `tauri-utils`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
  /// Target triple architecture error
  #[error("Unable to determine target-architecture")]
  Architecture,
  /// Target triple OS error
  #[error("Unable to determine target-os")]
  Os,
  /// Target triple environment error
  #[error("Unable to determine target-environment")]
  Environment,
  /// Tried to get resource on an unsupported platform
  #[error("Unsupported platform for reading resources")]
  UnsupportedPlatform,
  /// Get parent process error
  #[error("Could not get parent process")]
  ParentProcess,
  /// Get parent process PID error
  #[error("Could not get parent PID")]
  ParentPid,
  /// Get child process error
  #[error("Could not get child process")]
  ChildProcess,
  /// IO error
  #[error("{0}")]
  Io(#[from] std::io::Error),
  /// Invalid pattern.
  #[error("invalid pattern `{0}`. Expected either `brownfield` or `isolation`.")]
  InvalidPattern(String),
  /// Invalid glob pattern.
  #[cfg(feature = "resources")]
  #[error("{0}")]
  GlobPattern(#[from] glob::PatternError),
  /// Failed to use glob pattern.
  #[cfg(feature = "resources")]
  #[error("`{0}`")]
  Glob(#[from] glob::GlobError),
  /// Glob pattern did not find any results.
  #[cfg(feature = "resources")]
  #[error("path matching {0} not found.")]
  GlobPathNotFound(String),
  /// Error walking directory.
  #[cfg(feature = "resources")]
  #[error("{0}")]
  WalkdirError(#[from] walkdir::Error),
  /// Not allowed to walk dir.
  #[cfg(feature = "resources")]
  #[error("could not walk directory `{0}`, try changing `allow_walk` to true on the `ResourcePaths` constructor.")]
  NotAllowedToWalkDir(std::path::PathBuf),
}
