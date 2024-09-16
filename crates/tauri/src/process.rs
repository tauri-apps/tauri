// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to child processes management.

use crate::Env;

use std::path::PathBuf;

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
/// use tauri::{process::current_binary, Env, Manager};
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
/// use tauri::{process::restart, Env, Manager};
///
/// tauri::Builder::default()
///   .setup(|app| {
///     restart(&app.env());
///     Ok(())
///   });
/// ```
pub fn restart(env: &Env) -> ! {
  use std::process::{exit, Command};

  if let Ok(path) = current_binary(env) {
    // on macOS on updates the binary name might have changed
    // so we'll read the Contents/Info.plist file to determine the binary path
    #[cfg(target_os = "macos")]
    restart_macos_app(&path, env);

    if let Err(e) = Command::new(path)
      .args(env.args_os.iter().skip(1).collect::<Vec<_>>())
      .spawn()
    {
      log::error!("failed to restart app: {e}");
    }
  }

  exit(0);
}

#[cfg(target_os = "macos")]
fn restart_macos_app(current_binary: &std::path::Path, env: &Env) {
  use std::process::{exit, Command};

  if let Some(macos_directory) = current_binary.parent() {
    if macos_directory.components().last()
      != Some(std::path::Component::Normal(std::ffi::OsStr::new("MacOS")))
    {
      return;
    }

    if let Some(contents_directory) = macos_directory.parent() {
      if contents_directory.components().last()
        != Some(std::path::Component::Normal(std::ffi::OsStr::new(
          "Contents",
        )))
      {
        return;
      }

      if let Ok(info_plist) =
        plist::from_file::<_, plist::Dictionary>(contents_directory.join("Info.plist"))
      {
        if let Some(binary_name) = info_plist
          .get("CFBundleExecutable")
          .and_then(|v| v.as_string())
        {
          if let Err(e) = Command::new(macos_directory.join(binary_name))
            .args(env.args_os.iter().skip(1).collect::<Vec<_>>())
            .spawn()
          {
            log::error!("failed to restart app: {e}");
          }

          exit(0);
        }
      }
    }
  }
}
