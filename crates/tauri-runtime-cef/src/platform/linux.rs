// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  platform::{EventLoopExt, MonitorExt},
  webview::AppWebview,
  window::AppWindow,
};
use tauri_runtime::{
  Error, Result,
  dpi::{PhysicalPosition, PhysicalRect, Rect},
};
use tauri_utils::config::Color;
use winit::{event_loop::ActiveEventLoop, monitor::MonitorHandle};

impl MonitorExt for MonitorHandle {
  fn work_area(&self) -> PhysicalRect<i32, u32> {
    // TODO: implement native Linux/BSD work-area lookup via winit-gtk4 when it is available.
    super::monitor_bounds(self)
  }
}

impl AppWindow {
  pub(crate) fn set_background_color(&self, color: Option<Color>) {
    let _ = (self, color);
    // TODO: implement native window background color on Linux/BSD.
  }
}

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

impl AppWebview {
  pub(crate) fn set_background_color(&self, color: Option<Color>) {
    let _ = (self, color);
    // Native child-window background is not equivalent to Chromium's rendered
    // background. Creation still applies BrowserSettings.
  }

  pub(crate) fn bounds(&self) -> Option<Rect> {
    let _ = self;
    todo!("TODO: implement CEF/winit child bounds on Linux")
  }

  pub(crate) fn reparent(&self, parent: &AppWindow) {
    let _ = (self, parent);
    todo!("TODO: implement CEF/winit child reparenting on Linux")
  }

  pub(crate) fn apply_visible(&self, visible: bool) {
    let _ = visible;
    todo!("TODO: implement CEF/winit child visibility on Linux")
  }

  pub(crate) fn apply_physical_bounds(&self, scale: f64, x: i32, y: i32, width: i32, height: i32) {
    let _ = (scale, x, y, width, height);
    todo!("TODO: implement CEF/winit child layout on Linux")
  }
}
