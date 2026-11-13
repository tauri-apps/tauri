// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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

fn affected_system_webkit2gtk() -> bool {
  Command::new("pkg-config")
    .args(["--modversion", "webkit2gtk-4.1"])
    .output()
    .is_ok_and(|output| {
      output.status.success()
        && parse_modversion(&String::from_utf8_lossy(&output.stdout))
          .is_some_and(|v| v.0 == 2 && v.1 == 44)
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
      std::env::set_var(DMABUF_RENDERER_DISABLE_ENV, "1");
    }
    assert!(dmabuf_preflight().is_none());

    unsafe {
      std::env::remove_var(DMABUF_RENDERER_DISABLE_ENV);
      std::env::remove_var("WAYLAND_DISPLAY");
    }
    assert!(dmabuf_preflight().is_none());

    unsafe {
      std::env::set_var("WAYLAND_DISPLAY", "wayland-0");
      std::env::set_var("SHIM_WEBKIT_VERSION", "2.46.0");
    }
    assert!(dmabuf_preflight().is_none());

    unsafe {
      std::env::set_var("SHIM_WEBKIT_VERSION", "junk");
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
