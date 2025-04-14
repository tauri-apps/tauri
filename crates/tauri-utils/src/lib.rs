// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! This crate contains common code that is reused in many places and offers useful utilities like parsing configuration files, detecting platform triples, injecting the CSP, and managing assets.

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]
#![warn(missing_docs, rust_2018_idioms)]
#![allow(clippy::deprecated_semver)]

use std::{
  ffi::OsString,
  fmt::Display,
  path::{Path, PathBuf},
};

use semver::Version;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod acl;
pub mod assets;
pub mod config;
pub mod html;
pub mod io;
pub mod mime_type;
pub mod platform;
pub mod plugin;
/// Prepare application resources and sidecars.
#[cfg(feature = "resources")]
pub mod resources;
#[cfg(feature = "build")]
pub mod tokens;

#[cfg(feature = "build")]
pub mod build;

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
  /// The crate name.
  pub crate_name: &'static str,
}

#[allow(deprecated)]
mod window_effects {
  use super::*;

  #[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
  #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
  #[serde(rename_all = "camelCase")]
  /// Platform-specific window effects
  pub enum WindowEffect {
    /// A default material appropriate for the view's effectiveAppearance. **macOS 10.14-**
    #[deprecated(
      since = "macOS 10.14",
      note = "You should instead choose an appropriate semantic material."
    )]
    AppearanceBased,
    /// **macOS 10.14-**
    #[deprecated(since = "macOS 10.14", note = "Use a semantic material instead.")]
    Light,
    /// **macOS 10.14-**
    #[deprecated(since = "macOS 10.14", note = "Use a semantic material instead.")]
    Dark,
    /// **macOS 10.14-**
    #[deprecated(since = "macOS 10.14", note = "Use a semantic material instead.")]
    MediumLight,
    /// **macOS 10.14-**
    #[deprecated(since = "macOS 10.14", note = "Use a semantic material instead.")]
    UltraDark,
    /// **macOS 10.10+**
    Titlebar,
    /// **macOS 10.10+**
    Selection,
    /// **macOS 10.11+**
    Menu,
    /// **macOS 10.11+**
    Popover,
    /// **macOS 10.11+**
    Sidebar,
    /// **macOS 10.14+**
    HeaderView,
    /// **macOS 10.14+**
    Sheet,
    /// **macOS 10.14+**
    WindowBackground,
    /// **macOS 10.14+**
    HudWindow,
    /// **macOS 10.14+**
    FullScreenUI,
    /// **macOS 10.14+**
    Tooltip,
    /// **macOS 10.14+**
    ContentBackground,
    /// **macOS 10.14+**
    UnderWindowBackground,
    /// **macOS 10.14+**
    UnderPageBackground,
    /// Mica effect that matches the system dark perefence **Windows 11 Only**
    Mica,
    /// Mica effect with dark mode but only if dark mode is enabled on the system **Windows 11 Only**
    MicaDark,
    /// Mica effect with light mode **Windows 11 Only**
    MicaLight,
    /// Tabbed effect that matches the system dark perefence **Windows 11 Only**
    Tabbed,
    /// Tabbed effect with dark mode but only if dark mode is enabled on the system **Windows 11 Only**
    TabbedDark,
    /// Tabbed effect with light mode **Windows 11 Only**
    TabbedLight,
    /// **Windows 7/10/11(22H1) Only**
    ///
    /// ## Notes
    ///
    /// This effect has bad performance when resizing/dragging the window on Windows 11 build 22621.
    Blur,
    /// **Windows 10/11 Only**
    ///
    /// ## Notes
    ///
    /// This effect has bad performance when resizing/dragging the window on Windows 10 v1903+ and Windows 11 build 22000.
    Acrylic,
  }

  /// Window effect state **macOS only**
  ///
  /// <https://developer.apple.com/documentation/appkit/nsvisualeffectview/state>
  #[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
  #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
  #[serde(rename_all = "camelCase")]
  pub enum WindowEffectState {
    /// Make window effect state follow the window's active state
    FollowsWindowActiveState,
    /// Make window effect state always active
    Active,
    /// Make window effect state always inactive
    Inactive,
  }
}

pub use window_effects::{WindowEffect, WindowEffectState};

/// How the window title bar should be displayed on macOS.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum TitleBarStyle {
  /// A normal title bar.
  Visible,
  /// Makes the title bar transparent, so the window background color is shown instead.
  ///
  /// Useful if you don't need to have actual HTML under the title bar. This lets you avoid the caveats of using `TitleBarStyle::Overlay`. Will be more useful when Tauri lets you set a custom window background color.
  Transparent,
  /// Shows the title bar as a transparent overlay over the window's content.
  ///
  /// Keep in mind:
  /// - The height of the title bar is different on different OS versions, which can lead to window the controls and title not being where you don't expect.
  /// - You need to define a custom drag region to make your window draggable, however due to a limitation you can't drag the window when it's not in focus <https://github.com/tauri-apps/tauri/issues/4316>.
  /// - The color of the window title depends on the system theme.
  Overlay,
}

impl Default for TitleBarStyle {
  fn default() -> Self {
    Self::Visible
  }
}

impl Serialize for TitleBarStyle {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}

impl<'de> Deserialize<'de> for TitleBarStyle {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Ok(match s.to_lowercase().as_str() {
      "transparent" => Self::Transparent,
      "overlay" => Self::Overlay,
      _ => Self::Visible,
    })
  }
}

impl Display for TitleBarStyle {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Visible => "Visible",
        Self::Transparent => "Transparent",
        Self::Overlay => "Overlay",
      }
    )
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
  /// The command line arguments of the current process.
  pub args_os: Vec<OsString>,
}

#[allow(clippy::derivable_impls)]
impl Default for Env {
  fn default() -> Self {
    let args_os = std::env::args_os().collect();
    #[cfg(target_os = "linux")]
    {
      let env = Self {
        #[cfg(target_os = "linux")]
        appimage: std::env::var_os("APPIMAGE"),
        #[cfg(target_os = "linux")]
        appdir: std::env::var_os("APPDIR"),
        args_os,
      };
      if env.appimage.is_some() || env.appdir.is_some() {
        // validate that we're actually running on an AppImage
        // an AppImage is mounted to `/$TEMPDIR/.mount_${appPrefix}${hash}`
        // see <https://github.com/AppImage/AppImageKit/blob/1681fd84dbe09c7d9b22e13cdb16ea601aa0ec47/src/runtime.c#L501>
        // note that it is safe to use `std::env::current_exe` here since we just loaded an AppImage.
        let is_temp = std::env::current_exe()
          .map(|p| {
            p.display()
              .to_string()
              .starts_with(&format!("{}/.mount_", std::env::temp_dir().display()))
          })
          .unwrap_or(true);

        if !is_temp {
          log::warn!("`APPDIR` or `APPIMAGE` environment variable found but this application was not detected as an AppImage; this might be a security issue.");
        }
      }
      env
    }
    #[cfg(not(target_os = "linux"))]
    {
      Self { args_os }
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
  #[error("glob pattern {0} path not found or didn't match any files.")]
  GlobPathNotFound(String),
  /// Error walking directory.
  #[cfg(feature = "resources")]
  #[error("{0}")]
  WalkdirError(#[from] walkdir::Error),
  /// Not allowed to walk dir.
  #[cfg(feature = "resources")]
  #[error("could not walk directory `{0}`, try changing `allow_walk` to true on the `ResourcePaths` constructor.")]
  NotAllowedToWalkDir(std::path::PathBuf),
  /// Resourece path doesn't exist
  #[cfg(feature = "resources")]
  #[error("resource path `{0}` doesn't exist")]
  ResourcePathNotFound(std::path::PathBuf),
}

/// Reconstructs a path from its components using the platform separator then converts it to String and removes UNC prefixes on Windows if it exists.
pub fn display_path<P: AsRef<Path>>(p: P) -> String {
  dunce::simplified(&p.as_ref().components().collect::<PathBuf>())
    .display()
    .to_string()
}

/// Write the file only if the content of the existing file (if any) is different.
///
/// This will always write unless the file exists with identical content.
pub fn write_if_changed<P, C>(path: P, content: C) -> std::io::Result<()>
where
  P: AsRef<Path>,
  C: AsRef<[u8]>,
{
  if let Ok(existing) = std::fs::read(&path) {
    if existing == content.as_ref() {
      return Ok(());
    }
  }

  std::fs::write(path, content)
}
