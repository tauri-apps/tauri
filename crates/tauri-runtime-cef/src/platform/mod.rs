// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
mod linux;

use tauri_runtime::dpi::PhysicalRect;
use winit::monitor::MonitorHandle;

pub(crate) trait MonitorExt {
  /// Get the work area of this monitor.
  ///
  /// TODO: upstream work-area support into winit and replace this native shim.
  fn work_area(&self) -> PhysicalRect<i32, u32>;
}

fn monitor_bounds(monitor: &MonitorHandle) -> PhysicalRect<i32, u32> {
  PhysicalRect {
    position: monitor.position().unwrap_or_default(),
    size: monitor
      .current_video_mode()
      .map(|video_mode| video_mode.size())
      .unwrap_or_default(),
  }
}

pub trait EventLoopExt {
  #[cfg(target_os = "macos")]
  fn set_activation_policy(&self, policy: tauri_runtime::ActivationPolicy);
  #[cfg(target_os = "macos")]
  fn set_dock_visibility(&self, visible: bool);
  #[cfg(target_os = "macos")]
  fn show_application(&self);
  #[cfg(target_os = "macos")]
  fn hide_application(&self);
  #[cfg(target_os = "macos")]
  fn set_progress_bar(&self, state: tauri_runtime::ProgressBarState);
  fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>);
  fn set_badge_label(&self, label: Option<String>);
  fn cursor_position(&self) -> tauri_runtime::Result<tauri_runtime::dpi::PhysicalPosition<f64>>;
}
