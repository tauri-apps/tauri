// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to shell.

use crate::ShellScope;
use std::str::FromStr;

/// Program to use on the [`open()`] call.
pub enum Program {
  /// Use the `open` program.
  Open,
  /// Use the `start` program.
  Start,
  /// Use the `xdg-open` program.
  XdgOpen,
  /// Use the `gio` program.
  Gio,
  /// Use the `gnome-open` program.
  GnomeOpen,
  /// Use the `kde-open` program.
  KdeOpen,
  /// Use the `wslview` program.
  WslView,
  /// Use the `Firefox` program.
  Firefox,
  /// Use the `Google Chrome` program.
  Chrome,
  /// Use the `Chromium` program.
  Chromium,
  /// Use the `Safari` program.
  Safari,
}

impl FromStr for Program {
  type Err = super::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let p = match s.to_lowercase().as_str() {
      "open" => Self::Open,
      "start" => Self::Start,
      "xdg-open" => Self::XdgOpen,
      "gio" => Self::Gio,
      "gnome-open" => Self::GnomeOpen,
      "kde-open" => Self::KdeOpen,
      "wslview" => Self::WslView,
      "firefox" => Self::Firefox,
      "chrome" | "google chrome" => Self::Chrome,
      "chromium" => Self::Chromium,
      "safari" => Self::Safari,
      _ => return Err(super::Error::UnknownProgramName(s.to_string())),
    };
    Ok(p)
  }
}

impl Program {
  pub(crate) fn name(self) -> &'static str {
    match self {
      Self::Open => "open",
      Self::Start => "start",
      Self::XdgOpen => "xdg-open",
      Self::Gio => "gio",
      Self::GnomeOpen => "gnome-open",
      Self::KdeOpen => "kde-open",
      Self::WslView => "wslview",

      #[cfg(target_os = "macos")]
      Self::Firefox => "Firefox",
      #[cfg(not(target_os = "macos"))]
      Self::Firefox => "firefox",

      #[cfg(target_os = "macos")]
      Self::Chrome => "Google Chrome",
      #[cfg(not(target_os = "macos"))]
      Self::Chrome => "google-chrome",

      #[cfg(target_os = "macos")]
      Self::Chromium => "Chromium",
      #[cfg(not(target_os = "macos"))]
      Self::Chromium => "chromium",

      #[cfg(target_os = "macos")]
      Self::Safari => "Safari",
      #[cfg(not(target_os = "macos"))]
      Self::Safari => "safari",
    }
  }
}

/// Opens path or URL with the program specified in `with`, or system default if `None`.
///
/// The path will be matched against the shell open validation regex, defaulting to `^https?://`.
/// A custom validation regex may be supplied in the config in `tauri > allowlist > scope > open`.
///
/// # Examples
///
/// ```rust,no_run
/// use tauri::{api::shell::open, Manager};
/// tauri::Builder::default()
///   .setup(|app| {
///     // open the given URL on the system default browser
///     open(&app.shell_scope(), "https://github.com/tauri-apps/tauri", None).unwrap();
///     Ok(())
///   });
/// ```
pub fn open<P: AsRef<str>>(
  scope: &ShellScope,
  path: P,
  with: Option<Program>,
) -> crate::api::Result<()> {
  scope
    .open(path.as_ref(), with)
    .map_err(|err| crate::api::Error::Shell(format!("failed to open: {}", err)))
}
