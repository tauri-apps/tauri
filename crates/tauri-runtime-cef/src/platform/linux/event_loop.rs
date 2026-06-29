// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::os::raw::{c_uint, c_ulong};
use tauri_runtime::{Error, Result, dpi::PhysicalPosition};
use winit::event_loop::ActiveEventLoop;

use crate::platform::EventLoopExt;

use super::{taskbar, utils::with_x11};

impl EventLoopExt for dyn ActiveEventLoop + '_ {
  fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>) {
    taskbar::set_badge_count(count, desktop_filename);
  }

  fn set_badge_label(&self, _label: Option<String>) {
    // Unsupported on Linux/BSD
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    with_x11(None, |xlib, display| unsafe {
      let root = (xlib.XDefaultRootWindow)(display);
      let mut root_return: c_ulong = 0;
      let mut child_return: c_ulong = 0;
      let mut root_x = 0;
      let mut root_y = 0;
      let mut win_x = 0;
      let mut win_y = 0;
      let mut mask: c_uint = 0;

      let ok = (xlib.XQueryPointer)(
        display,
        root,
        &mut root_return,
        &mut child_return,
        &mut root_x,
        &mut root_y,
        &mut win_x,
        &mut win_y,
        &mut mask,
      );

      (ok != 0).then_some(PhysicalPosition::new(root_x as f64, root_y as f64))
    })
    .ok_or(Error::FailedToGetCursorPosition)
  }
}
