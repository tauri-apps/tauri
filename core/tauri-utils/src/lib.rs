// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Tauri utility helpers
#![warn(missing_docs, rust_2018_idioms)]

use std::{
  fmt::{self, Display},
  path::{Path, PathBuf},
  sync::Arc,
};

use assets::Assets;
use config::Config;
use pattern::Pattern;
use semver::Version;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod assets;
pub mod config;
pub mod html;
pub mod io;
pub mod mime_type;
pub mod platform;
/// Prepare application resources and sidecars.
#[cfg(feature = "resources")]
pub mod resources;

/// Application pattern.
pub mod pattern;

#[cfg(target_os = "macos")]
#[doc(hidden)]
pub use embed_plist;

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

/// How the window title bar should be displayed on macOS.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
  pub args: Vec<String>,
}

#[allow(clippy::derivable_impls)]
impl Default for Env {
  fn default() -> Self {
    let args = std::env::args().skip(1).collect();
    #[cfg(target_os = "linux")]
    {
      let env = Self {
        #[cfg(target_os = "linux")]
        appimage: std::env::var_os("APPIMAGE"),
        #[cfg(target_os = "linux")]
        appdir: std::env::var_os("APPDIR"),
        args,
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
      Self { args }
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

/// Suppresses the unused-variable warnings of the given inputs.
///
/// This does not move any values. Instead, it just suppresses the warning by taking a
/// reference to the value.
#[macro_export]
macro_rules! consume_unused_variable {
  ($($arg:expr),*) => {
    $(
      let _ = &$arg;
    )*
    ()
  };
}

/// Prints to the standard error, with a newline.
///
/// Equivalent to the [`eprintln!`] macro, except that it's only effective for debug builds.
#[macro_export]
macro_rules! debug_eprintln {
  () => ($crate::debug_eprintln!(""));
  ($($arg:tt)*) => {
    #[cfg(debug_assertions)]
    eprintln!($($arg)*);
    #[cfg(not(debug_assertions))]
    $crate::consume_unused_variable!($($arg)*);
  };
}

/// Reconstructs a path from its components using the platform separator then converts it to String
pub fn display_path<P: AsRef<Path>>(p: P) -> String {
  p.as_ref()
    .components()
    .collect::<PathBuf>()
    .display()
    .to_string()
}

/// Window icon.
#[derive(Debug, Clone)]
pub struct RawIcon {
  /// RGBA bytes of the icon.
  pub rgba: Vec<u8>,
  /// Icon width.
  pub width: u32,
  /// Icon height.
  pub height: u32,
}

/// A icon definition.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Icon {
  /// Icon from file path.
  #[cfg(any(feature = "icon-ico", feature = "icon-png"))]
  #[cfg_attr(doc_cfg, doc(cfg(any(feature = "icon-ico", feature = "icon-png"))))]
  File(std::path::PathBuf),
  /// Icon from raw RGBA bytes. Width and height is parsed at runtime.
  #[cfg(any(feature = "icon-ico", feature = "icon-png"))]
  #[cfg_attr(doc_cfg, doc(cfg(any(feature = "icon-ico", feature = "icon-png"))))]
  Raw(Vec<u8>),
  /// Icon from raw RGBA bytes.
  Rgba {
    /// RGBA bytes of the icon image.
    rgba: Vec<u8>,
    /// Icon width.
    width: u32,
    /// Icon height.
    height: u32,
  },
}

/// Errors that can happen while loading an Icon.
#[derive(Debug, thiserror::Error)]
pub enum IconError {
  /// Any IO error.
  #[error(transparent)]
  Io(#[from] std::io::Error),

  /// Invalid PNG.
  #[cfg(feature = "icon-png")]
  #[error(transparent)]
  Png(#[from] png::DecodingError),
}

impl TryFrom<Icon> for RawIcon {
  type Error = IconError;

  fn try_from(icon: Icon) -> ::std::result::Result<Self, Self::Error> {
    #[allow(irrefutable_let_patterns)]
    if let Icon::Rgba {
      rgba,
      width,
      height,
    } = icon
    {
      Ok(Self {
        rgba,
        width,
        height,
      })
    } else {
      #[cfg(not(any(feature = "icon-ico", feature = "icon-png")))]
      panic!("unexpected Icon variant");
      #[cfg(any(feature = "icon-ico", feature = "icon-png"))]
      {
        let bytes = match icon {
          Icon::File(p) => std::fs::read(p)?,
          Icon::Raw(r) => r,
          Icon::Rgba { .. } => unreachable!(),
        };
        let extension = infer::get(&bytes)
          .expect("could not determine icon extension")
          .extension();
        match extension {
        #[cfg(feature = "icon-ico")]
        "ico" => {
          let icon_dir = ico::IconDir::read(std::io::Cursor::new(bytes))?;
          let entry = &icon_dir.entries()[0];
          Ok(Self {
            rgba: entry.decode()?.rgba_data().to_vec(),
            width: entry.width(),
            height: entry.height(),
          })
        }
        #[cfg(feature = "icon-png")]
        "png" => {
          let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
          let mut reader = decoder.read_info()?;
          let mut buffer = Vec::new();
          while let Ok(Some(row)) = reader.next_row() {
            buffer.extend(row.data());
          }
          Ok(Self {
            rgba: buffer,
            width: reader.info().width,
            height: reader.info().height,
          })
        }
        _ => panic!(
          "image `{extension}` extension not supported; please file a Tauri feature request. `png` or `ico` icons are supported with the `icon-png` and `icon-ico` feature flags"
        ),
      }
      }
    }
  }
}

/// User supplied data required inside of a Tauri application.
///
/// # Stability
///
/// This is the output of Tauri codegen, and is not considered part of the stable API.
/// Unless you know what you are doing and are prepared for this type to have breaking changes, do not create it yourself.
pub struct Context<A: Assets> {
  #[doc(hidden)]
  pub config: Config,

  #[doc(hidden)]
  pub assets: Arc<A>,

  #[doc(hidden)]
  pub default_window_icon: Option<Icon>,

  #[doc(hidden)]
  pub app_icon: Option<Vec<u8>>,

  #[doc(hidden)]
  pub system_tray_icon: Option<Icon>,

  #[doc(hidden)]
  pub package_info: PackageInfo,

  #[doc(hidden)]
  pub pattern: Pattern,
}

impl<A: Assets> fmt::Debug for Context<A> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Context")
      .field("config", &self.config)
      .field("default_window_icon", &self.default_window_icon)
      .field("app_icon", &self.app_icon)
      .field("package_info", &self.package_info)
      .field("pattern", &self.pattern)
      .field("system_tray_icon", &self.system_tray_icon)
      .finish()
  }
}

impl<A: Assets> Context<A> {
  /// The config the application was prepared with.
  #[inline(always)]
  pub fn config(&self) -> &Config {
    &self.config
  }

  /// A mutable reference to the config the application was prepared with.
  #[inline(always)]
  pub fn config_mut(&mut self) -> &mut Config {
    &mut self.config
  }

  /// The assets to be served directly by Tauri.
  #[inline(always)]
  pub fn assets(&self) -> Arc<A> {
    self.assets.clone()
  }

  /// A mutable reference to the assets to be served directly by Tauri.
  #[inline(always)]
  pub fn assets_mut(&mut self) -> &mut Arc<A> {
    &mut self.assets
  }

  /// The default window icon Tauri should use when creating windows.
  #[inline(always)]
  pub fn default_window_icon(&self) -> Option<&Icon> {
    self.default_window_icon.as_ref()
  }

  /// A mutable reference to the default window icon Tauri should use when creating windows.
  #[inline(always)]
  pub fn default_window_icon_mut(&mut self) -> &mut Option<Icon> {
    &mut self.default_window_icon
  }

  /// The icon to use on the system tray UI.
  #[inline(always)]
  pub fn system_tray_icon(&self) -> Option<&Icon> {
    self.system_tray_icon.as_ref()
  }

  /// A mutable reference to the icon to use on the system tray UI.
  #[inline(always)]
  pub fn system_tray_icon_mut(&mut self) -> &mut Option<Icon> {
    &mut self.system_tray_icon
  }

  /// Package information.
  #[inline(always)]
  pub fn package_info(&self) -> &PackageInfo {
    &self.package_info
  }

  /// A mutable reference to the package information.
  #[inline(always)]
  pub fn package_info_mut(&mut self) -> &mut PackageInfo {
    &mut self.package_info
  }

  /// The application pattern.
  #[inline(always)]
  pub fn pattern(&self) -> &Pattern {
    &self.pattern
  }

  /// Create a new [`Context`] from the minimal required items.
  #[inline(always)]
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    config: Config,
    assets: Arc<A>,
    default_window_icon: Option<Icon>,
    app_icon: Option<Vec<u8>>,
    package_info: PackageInfo,
    pattern: Pattern,
  ) -> Self {
    Self {
      config,
      assets,
      default_window_icon,
      app_icon,
      system_tray_icon: None,
      package_info,
      pattern,
    }
  }

  /// Sets the app tray icon.
  #[inline(always)]
  pub fn set_system_tray_icon(&mut self, icon: Icon) {
    self.system_tray_icon.replace(icon);
  }
}
