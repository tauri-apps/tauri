// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use gtk::prelude::*;
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
use tao::platform::unix::WindowExtUnix;

impl super::WindowExt for tao::window::Window {
  fn set_enabled(&self, enabled: bool) {
    self.gtk_window().set_sensitive(enabled);
  }

  fn is_enabled(&self) -> bool {
    self.gtk_window().is_sensitive()
  }

  fn center(&self) {
    if let Some(monitor) = self.current_monitor() {
      let window_size = self.outer_size();
      let new_pos = super::calculate_window_center_position(window_size, monitor);
      self.set_outer_position(new_pos);
    }
  }
}
