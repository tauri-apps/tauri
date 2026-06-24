// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
mod linux;
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
pub use linux::*;

pub trait EventLoopExt {
  #[cfg(target_os = "macos")]
  fn set_activation_policy(&self, policy: tauri_runtime::ActivationPolicy);
  #[cfg(target_os = "macos")]
  fn set_dock_visibility(&self, visible: bool);
  #[cfg(target_os = "macos")]
  fn show_application(&self);
  #[cfg(target_os = "macos")]
  fn hide_application(&self);
  fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>);
  fn set_badge_label(&self, label: Option<String>);
  fn cursor_position(&self) -> tauri_runtime::Result<tauri_runtime::dpi::PhysicalPosition<f64>>;
}
