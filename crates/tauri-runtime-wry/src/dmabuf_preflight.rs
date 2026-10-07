// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Advisory startup check for the WebKitGTK DMA-BUF renderer launch failure
//! tracked in tauri-apps/tauri#10702 (same failure family as #9304): on
//! Wayland sessions with webkit2gtk 2.44.x, WebKitGTK's DMA-BUF renderer can
//! violate the Wayland protocol, the compositor disconnects the client, and
//! GDK aborts the process with a single cryptic line (`Gdk-Message: Error 71
//! (Protocol error) dispatching to Wayland display`) before any window
//! appears. The bug is upstream of Tauri; this check only names the escape
//! hatch before the app can die.

use std::process::Command;

/// WebKitGTK reads this at library init; setting it skips the DMA-BUF renderer.
const DMABUF_RENDERER_DISABLE_ENV: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";

/// The DMA-BUF renderer became the default in webkit2gtk 2.44 and the
/// Wayland protocol violations were reworked after that series, so only
/// 2.44.x is worth naming.
fn is_affected(version: (u32, u32, u32)) -> bool {
  version.0 == 2 && version.1 == 44
}

fn parse_modversion(output: &str) -> Option<(u32, u32, u32)> {
  let mut parts = output.trim().split('.');
  Some((
    parts.next()?.parse().ok()?,
    parts.next()?.parse().ok()?,
    parts.next()?.parse().ok()?,
  ))
}

/// Version of the system webkit2gtk, queried through pkg-config so a distro
/// update is seen at run time; a build-time constant would go stale. Returns
/// `None` whenever detection is not possible: the warning is advisory and
/// stays silent rather than guess.
fn system_webkit2gtk_version() -> Option<(u32, u32, u32)> {
  for module in ["webkit2gtk-4.1", "webkit2gtk-4.0"] {
    if let Ok(output) = Command::new("pkg-config")
      .args(["--modversion", module])
      .output()
    {
      if output.status.success() {
        if let Some(version) = parse_modversion(&String::from_utf8_lossy(&output.stdout)) {
          return Some(version);
        }
      }
    }
  }
  None
}

fn should_warn(
  wayland_session: bool,
  already_disabled: bool,
  version: Option<(u32, u32, u32)>,
) -> bool {
  wayland_session && !already_disabled && version.is_some_and(is_affected)
}

const WARNING: &str = "warning: webkit2gtk 2.44.x on Wayland can be killed at launch by a \
                       WebKitGTK DMA-BUF renderer protocol violation (`Gdk-Message: Error 71 \
                       (Protocol error) dispatching to Wayland display`, see \
                       tauri-apps/tauri#10702). If that happens, re-run this app with \
                       WEBKIT_DISABLE_DMABUF_RENDERER=1";

/// The advisory line, or `None` when the session does not match the known
/// failure mode. Reads the environment and pkg-config only; never fails.
pub(crate) fn dmabuf_preflight() -> Option<&'static str> {
  let wayland_session = std::env::var_os("WAYLAND_DISPLAY").is_some();
  let already_disabled = std::env::var_os(DMABUF_RENDERER_DISABLE_ENV).is_some();
  should_warn(
    wayland_session,
    already_disabled,
    system_webkit2gtk_version(),
  )
  .then_some(WARNING)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn only_the_2_44_series_is_affected() {
    assert!(!is_affected((2, 43, 4)));
    assert!(is_affected((2, 44, 0)));
    assert!(is_affected((2, 44, 3)));
    assert!(!is_affected((2, 45, 1)));
    assert!(!is_affected((2, 46, 0)));
  }

  #[test]
  fn modversion_parses_distro_output() {
    assert_eq!(parse_modversion("2.44.3\n"), Some((2, 44, 3)));
    assert_eq!(parse_modversion("  2.46.8 "), Some((2, 46, 8)));
    assert_eq!(parse_modversion("not-a-version"), None);
    assert_eq!(parse_modversion("2.44"), None);
    assert_eq!(parse_modversion(""), None);
  }

  #[test]
  fn decision_table_stays_silent_when_not_matching() {
    let affected = Some((2, 44, 3));
    assert!(should_warn(true, false, affected));
    assert!(!should_warn(false, false, affected)); // X11 session
    assert!(!should_warn(true, true, affected)); // user already mitigated
    assert!(!should_warn(true, false, Some((2, 46, 0)))); // fixed series
    assert!(!should_warn(true, false, None)); // undetectable: stay silent
  }

  // Runs the real check against a shimmed pkg-config and a fake Wayland
  // session. The only test allowed to touch PATH and these env vars.
  #[cfg(unix)]
  #[test]
  fn end_to_end_with_shimmed_pkg_config_and_wayland_env() {
    let dir = std::env::temp_dir().join(format!("tauri-dmabuf-shim-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let shim = dir.join("pkg-config");
    std::fs::write(&shim, "#!/bin/sh\necho 2.44.3\n").unwrap();
    {
      use std::os::unix::fs::PermissionsExt;
      let mut perms = std::fs::metadata(&shim).unwrap().permissions();
      perms.set_mode(0o755);
      std::fs::set_permissions(&shim, perms).unwrap();
    }

    let old_path = std::env::var("PATH").unwrap_or_default();
    unsafe {
      std::env::set_var("PATH", format!("{}:{old_path}", dir.display()));
      std::env::set_var("WAYLAND_DISPLAY", "wayland-0");
      std::env::remove_var(DMABUF_RENDERER_DISABLE_ENV);
    }

    let warning = dmabuf_preflight();
    assert!(
      warning
        .is_some_and(|w| w.contains("Error 71") && w.contains("WEBKIT_DISABLE_DMABUF_RENDERER=1"))
    );

    unsafe {
      std::env::set_var(DMABUF_RENDERER_DISABLE_ENV, "1");
    }
    assert!(dmabuf_preflight().is_none());

    unsafe {
      std::env::remove_var(DMABUF_RENDERER_DISABLE_ENV);
      std::env::remove_var("WAYLAND_DISPLAY");
    }
    assert!(dmabuf_preflight().is_none());

    unsafe {
      std::env::set_var("PATH", old_path);
    }
    let _ = std::fs::remove_dir_all(&dir);
  }
}
