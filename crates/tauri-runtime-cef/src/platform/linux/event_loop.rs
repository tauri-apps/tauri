// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_runtime::{Error, Result, dpi::PhysicalPosition};
use winit::event_loop::ActiveEventLoop;

use crate::platform::EventLoopExt;

impl EventLoopExt for dyn ActiveEventLoop + '_ {
  fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>) {
    let _ = (count, desktop_filename);
    // TODO
  }

  fn set_badge_label(&self, label: Option<String>) {
    let _ = label;
    // TODO
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    // TODO: implement CEF/winit global cursor position on Linux/BSD.
    Err(Error::FailedToGetCursorPosition)
  }
}
