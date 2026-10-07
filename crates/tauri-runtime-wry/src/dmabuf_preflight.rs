// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Startup check for the WebKitGTK DMA-BUF Wayland launch failure
//! (tauri-apps/tauri#10702, also #9304). Upstream bug, this only names the
//! workaround.

use std::process::Command;

const DMABUF_RENDERER_DISABLE_ENV: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";

const WARNING: &str = "warning: webkit2gtk 2.44.x on Wayland can be killed at launch by a \
                       WebKitGTK DMA-BUF renderer protocol violation (`Gdk-Message: Error 71 \
                       (Protocol error) dispatching to Wayland display`, see \
                       tauri-apps/tauri#10702). If that happens, re-run this app with \
                       WEBKIT_DISABLE_DMABUF_RENDERER=1";

fn parse_modversion(output: &str) -> Option<(u32, u32, u32)> {
  let mut parts = output.trim().split('.');
  Some((
    parts.next()?.parse().ok()?,
    parts.next()?.parse().ok()?,
    parts.next()?.parse().ok()?,
  ))
}

// pkg-config because the system webkit2gtk can be updated under the app.
fn affected_system_webkit2gtk() -> bool {
  ["webkit2gtk-4.1", "webkit2gtk-4.0"].iter().any(|module| {
    Command::new("pkg-config")
      .args(["--modversion", module])
      .output()
      .is_ok_and(|output| {
        output.status.success()
          && parse_modversion(&String::from_utf8_lossy(&output.stdout))
            .is_some_and(|v| v.0 == 2 && v.1 == 44)
      })
  })
}

pub(crate) fn dmabuf_preflight() -> Option<&'static str> {
  let known_failure_mode = std::env::var_os("WAYLAND_DISPLAY").is_some()
    && std::env::var_os(DMABUF_RENDERER_DISABLE_ENV).is_none()
    && affected_system_webkit2gtk();
  known_failure_mode.then_some(WARNING)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn only_the_2_44_series_parses_as_affected() {
    assert!(parse_modversion("2.44.3\n").is_some_and(|v| v.0 == 2 && v.1 == 44));
    assert!(parse_modversion("  2.44.0 ").is_some_and(|v| v.0 == 2 && v.1 == 44));
    assert!(!parse_modversion("2.46.8").is_some_and(|v| v.0 == 2 && v.1 == 44));
    assert!(!parse_modversion("2.43.4").is_some_and(|v| v.0 == 2 && v.1 == 44));
    assert_eq!(parse_modversion("not-a-version"), None);
    assert_eq!(parse_modversion("2.44"), None);
    assert_eq!(parse_modversion(""), None);
  }

  // The only test allowed to touch PATH and these env vars.
  #[cfg(unix)]
  #[test]
  fn fires_only_for_wayland_affected_and_unmitigated() {
    let dir = std::env::temp_dir().join(format!("tauri-dmabuf-shim-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let shim = dir.join("pkg-config");
    std::fs::write(&shim, "#!/bin/sh\necho \"$SHIM_WEBKIT_VERSION\"\n").unwrap();
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
      std::env::set_var("SHIM_WEBKIT_VERSION", "2.44.3");
      std::env::remove_var(DMABUF_RENDERER_DISABLE_ENV);
    }

    assert!(
      dmabuf_preflight()
        .is_some_and(|w| w.contains("Error 71") && w.contains("WEBKIT_DISABLE_DMABUF_RENDERER=1"))
    );

    unsafe {
      std::env::set_var(DMABUF_RENDERER_DISABLE_ENV, "1"); // already mitigated
    }
    assert!(dmabuf_preflight().is_none());

    unsafe {
      std::env::remove_var(DMABUF_RENDERER_DISABLE_ENV);
      std::env::remove_var("WAYLAND_DISPLAY"); // X11
    }
    assert!(dmabuf_preflight().is_none());

    unsafe {
      std::env::set_var("WAYLAND_DISPLAY", "wayland-0");
      std::env::set_var("SHIM_WEBKIT_VERSION", "2.46.0"); // fixed
    }
    assert!(dmabuf_preflight().is_none());

    unsafe {
      std::env::set_var("SHIM_WEBKIT_VERSION", "junk"); // undetectable
    }
    assert!(dmabuf_preflight().is_none());

    unsafe {
      std::env::set_var("PATH", old_path);
      std::env::remove_var("WAYLAND_DISPLAY");
      std::env::remove_var("SHIM_WEBKIT_VERSION");
    }
    let _ = std::fs::remove_dir_all(&dir);
  }
}
