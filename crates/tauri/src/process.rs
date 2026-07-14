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

    if let Err(e) = Command::new(path).args(env.args_os.iter().skip(1)).spawn() {
      log::error!("failed to restart app: {e}");
    }
  }

  exit(0);
}

#[cfg(target_os = "macos")]
fn restart_macos_app(current_binary: &std::path::Path, env: &Env) {
  use std::process::{exit, Command};

  if let Some(macos_directory) = current_binary.parent() {
    if macos_directory.components().next_back()
      != Some(std::path::Component::Normal(std::ffi::OsStr::new("MacOS")))
    {
      return;
    }

    if let Some(contents_directory) = macos_directory.parent() {
      if contents_directory.components().next_back()
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
            .args(env.args_os.iter().skip(1))
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

/// Kill a process and its descendant process tree (best-effort).
///
/// On Windows this function prefers the built-in `taskkill /T /PID <pid> /F`
/// utility which can terminate a process tree. If `taskkill` is unavailable
/// or returns a non-zero exit status (for example due to permissions), the
/// function falls back to a PowerShell-based recursive traversal which mirrors
/// the previous implementation.
///
/// On Unix-like systems a small shell function using `pgrep -P` is used to
/// collect child PIDs and send `SIGKILL` to descendants and the root PID.
///
/// Note: terminating processes is inherently best-effort and may fail for
/// protected or system processes, or when the caller lacks sufficient
/// privileges. Callers should handle and log any errors returned by this
/// function.

// TODO: Move this helper into the `process` plugin in the plugins-workspace repo.
pub fn kill_process_tree(pid: u32) -> std::io::Result<()> {
  #[cfg(windows)]
  {
    use std::process::Command;

    // Prefer the built-in `taskkill` utility on Windows which can terminate a process
    // tree with `/T`. If that fails (permissions, not found, or non-zero exit), fall
    // back to a PowerShell-based recursive stop that mirrors the previous behavior.
    let pid_s = pid.to_string();

    if let Ok(status) = Command::new("taskkill")
      .args(&["/T", "/PID", &pid_s, "/F"]) // /F to force termination
      .status()
    {
      if status.success() {
        return Ok(());
      }
      // If taskkill returned non-zero, fall through to try PowerShell.
    }

    // Fallback: Use PowerShell to recursively find and stop child processes, then stop the root.
    // This mirrors the approach used elsewhere in the project (tauri-cli) and preserves
    // behavior on systems where taskkill isn't available or failed due to permissions.
    let ps = format!(
      "function Kill-Tree {{ Param([int]$ppid); Get-CimInstance Win32_Process | Where-Object {{ $_.ParentProcessId -eq $ppid }} | ForEach-Object {{ Kill-Tree $_.ProcessId }}; Stop-Process -Id $ppid -ErrorAction SilentlyContinue }}; Kill-Tree {}",
      pid
    );

    let status = Command::new("powershell")
      .arg("-NoProfile")
      .arg("-Command")
      .arg(ps)
      .status()?;

    if status.success() {
      Ok(())
    } else {
      Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("kill-tree failed: powershell exited with status: {}", status),
      ))
    }
  }

  #[cfg(not(windows))]
  {
    use std::process::Command;

    // On Unix, recursively collect children via pgrep -P and kill them. We use a small
    // shell function to traverse descendants and then kill them. Use SIGKILL to ensure
    // termination (best effort).
    let sh = format!(r#"
getcpid() {{
  for cpid in $(pgrep -P "$1" 2>/dev/null || true); do
    getcpid "$cpid"
    echo "$cpid"
  done
}}
for p in $(getcpid {pid}); do
  kill -9 "$p" 2>/dev/null || true
done
kill -9 {pid} 2>/dev/null || true
"#, pid = pid);

    let status = Command::new("sh").arg("-c").arg(sh).status()?;

    if status.success() {
      Ok(())
    } else {
      Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("sh kill-tree failed with status: {}", status),
      ))
    }
  }
}

