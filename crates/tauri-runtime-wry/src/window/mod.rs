// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

use crate::monitor::MonitorExt;

pub trait WindowExt {
  /// Enable or disable the window
  ///
  /// ## Platform-specific:
  ///
  /// - **Android / iOS**: Unsupported.
  fn set_enabled(&self, enabled: bool);

  /// Whether the window is enabled or disabled.
  ///
  /// ## Platform-specific:
  ///
  /// - **Android / iOS**: Unsupported, always returns `true`.
  fn is_enabled(&self) -> bool;

  /// Center the window
  ///
  /// ## Platform-specific:
  ///
  /// - **Android / iOS**: Unsupported.
  fn center(&self) {}

  /// Clears the window surface. i.e make it it transparent.
  #[cfg(windows)]
  fn draw_surface(
    &self,
    surface: &mut softbuffer::Surface<
      std::sync::Arc<tao::window::Window>,
      std::sync::Arc<tao::window::Window>,
    >,
    background_color: Option<tao::window::RGBA>,
  );
}

#[cfg(mobile)]
impl WindowExt for tao::window::Window {
  fn set_enabled(&self, _: bool) {}
  fn is_enabled(&self) -> bool {
    true
  }
}

pub fn calculate_window_center_position(
  window_size: tao::dpi::PhysicalSize<u32>,
  target_monitor: tao::monitor::MonitorHandle,
) -> tao::dpi::PhysicalPosition<i32> {
  let work_area = target_monitor.work_area();

  tao::dpi::PhysicalPosition::new(
    (work_area.size.width as i32 - window_size.width as i32) / 2 + work_area.position.x,
    (work_area.size.height as i32 - window_size.height as i32) / 2 + work_area.position.y,
  )
}
