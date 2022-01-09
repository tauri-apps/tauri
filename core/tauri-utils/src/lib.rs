// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Tauri utility helpers
#![warn(missing_docs, rust_2018_idioms)]

pub mod assets;
pub mod config;
pub mod html;
pub mod platform;

/// `tauri::App` package information.
#[derive(Debug, Clone)]
pub struct PackageInfo {
  /// App name
  pub name: String,
  /// App version
  pub version: String,
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

/// Information about environment variables.
#[derive(Debug, Clone)]
pub struct Env {
  /// The APPIMAGE environment variable.
  #[cfg(target_os = "linux")]
  pub appimage: Option<std::ffi::OsString>,
  /// The APPDIR environment variable.
  #[cfg(target_os = "linux")]
  pub appdir: Option<std::ffi::OsString>,
}

impl Default for Env {
  fn default() -> Self {
    Self {
      #[cfg(target_os = "linux")]
      appimage: std::env::var_os("APPIMAGE"),
      #[cfg(target_os = "linux")]
      appdir: std::env::var_os("APPDIR"),
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
}
