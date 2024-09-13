// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to child processes management.

use crate::Env;

use std::path::PathBuf;

#[cfg(feature = "process-command-api")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "process-command-api")))]
mod command;
#[cfg(feature = "process-command-api")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "process-command-api")))]
pub use command::*;

/// Finds the current running binary's path.
///
/// With exception to any following platform-specific behavior, the path is cached as soon as
/// possible, and then used repeatedly instead of querying for a new path every time this function
/// is called.
///
/// # Platform-specific behavior
///
/// ## Linux
///
/// On Linux, this function will **attempt** to detect if it's currently running from a
/// valid [AppImage] and use that path instead.
///
/// ## macOS
///
/// On `macOS`, this function will return an error if the original path contained any symlinks
/// due to less protection on macOS regarding symlinks. This behavior can be disabled by setting the
/// `process-relaunch-dangerous-allow-symlink-macos` feature, although it is *highly discouraged*.
///
/// # Security
///
/// See [`tauri_utils::platform::current_exe`] for possible security implications.
///
/// # Examples
///
/// ```rust,no_run
/// use tauri::{api::process::current_binary, Env, Manager};
/// let current_binary_path = current_binary(&Env::default()).unwrap();
///
/// tauri::Builder::default()
///   .setup(|app| {
///     let current_binary_path = current_binary(&app.env())?;
///     Ok(())
///   });
/// ```
///
/// [AppImage]: https://appimage.org/
pub fn current_binary(_env: &Env) -> std::io::Result<PathBuf> {
  // if we are running from an AppImage, we ONLY want the set AppImage path
  #[cfg(target_os = "linux")]
  if let Some(app_image_path) = &_env.appimage {
    return Ok(PathBuf::from(app_image_path));
  }

  tauri_utils::platform::current_exe()
}

/// Restarts the currently running binary.
///
/// See [`current_binary`] for platform specific behavior, and
/// [`tauri_utils::platform::current_exe`] for possible security implications.
///
/// # Examples
///
/// ```rust,no_run
/// use tauri::{api::process::restart, Env, Manager};
///
/// tauri::Builder::default()
///   .setup(|app| {
///     restart(&app.env());
///     Ok(())
///   });
/// ```
pub fn restart(env: &Env) {
  use std::process::{exit, Command};

  if let Ok(path) = current_binary(env) {
    // on macOS on updates the binary name might have changed
    // so we'll read the Contents/MacOS folder instead to infer the actual binary path
    #[cfg(target_os = "macos")]
    if let Some(parent) = path.parent() {
      if parent.components().last()
        == Some(std::path::Component::Normal(std::ffi::OsStr::new("MacOS")))
      {
        let macos_binaries = std::fs::read_dir(parent)
          .map(|dir| {
            dir
              .into_iter()
              .flatten()
              .map(|entry| entry.path())
              .collect::<Vec<_>>()
          })
          .unwrap_or_default();
        match macos_binaries.len() {
          0 => {
            // should never happen, but let's not panic here since it's a crucial feature for updates
            exit(1);
          }
          1 => {
            // we have one binary (no sidecar) so we should use it to restart
            if let Err(e) = Command::new(macos_binaries.first().unwrap())
              .args(&env.args)
              .spawn()
            {
              eprintln!("failed to restart app: {e}");
            }

            exit(0);
          }
          _ => {
            // in case of sidecars we don't have enough information here to decide what's the right binary name
            // so let's hope the binary name didn't change by running the Command::spawn below
          }
        }
      }
    }

    if let Err(e) = Command::new(path).args(&env.args).spawn() {
      eprintln!("failed to restart app: {e}");
    }
  }

  exit(0);
}
