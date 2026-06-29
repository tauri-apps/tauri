// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_runtime::{Error, Result, dpi::PhysicalPosition};
use windows::Win32::{Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos};
use winit::event_loop::ActiveEventLoop;

use crate::platform::EventLoopExt;

impl EventLoopExt for dyn ActiveEventLoop + '_ {
  fn set_badge_count(&self, _count: Option<i64>, _desktop_filename: Option<String>) {
    // Unsupported on Windows
  }

  fn set_badge_label(&self, _label: Option<String>) {
    // Unsupported on Windows
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point) }.map_err(|_| Error::FailedToGetCursorPosition)?;
    Ok(PhysicalPosition::new(point.x as f64, point.y as f64))
  }
}
