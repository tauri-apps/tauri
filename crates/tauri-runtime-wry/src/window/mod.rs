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

  /// Clears the window sufrace. i.e make it it transparent.
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
  #[cfg(windows)]
  {
    use ::windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFO};
    use tao::platform::windows::MonitorHandleExtWindows;

    let mut monitor_info = MONITORINFO {
      cbSize: std::mem::size_of::<MONITORINFO>() as u32,
      ..Default::default()
    };
    let hmonitor = target_monitor.hmonitor();
    let status = unsafe { GetMonitorInfoW(HMONITOR(hmonitor as _), &mut monitor_info) };
    if status.into() {
      let available_width = monitor_info.rcWork.right - monitor_info.rcWork.left;
      let available_height = monitor_info.rcWork.bottom - monitor_info.rcWork.top;
      let x = (available_width - window_size.width as i32) / 2 + monitor_info.rcWork.left;
      let y = (available_height - window_size.height as i32) / 2 + monitor_info.rcWork.top;
      return tao::dpi::PhysicalPosition::new(x, y);
    }
  }

  let screen_size = target_monitor.size();
  let monitor_pos = target_monitor.position();
  let x = (screen_size.width as i32 - window_size.width as i32) / 2 + monitor_pos.x;
  let y = (screen_size.height as i32 - window_size.height as i32) / 2 + monitor_pos.y;
  tao::dpi::PhysicalPosition::new(x, y)
}
