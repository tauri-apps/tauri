// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{platform::EventLoopExt, webview::AppWebview, window::AppWindow};
use tauri_runtime::dpi::Rect;
use winit::event_loop::ActiveEventLoop;

impl EventLoopExt for dyn ActiveEventLoop + '_ {
  fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>) {
    let _ = (count, desktop_filename);
    // TODO
  }

  fn set_badge_label(&self, label: Option<String>) {
    let _ = label;
    // TODO
  }
}

impl AppWebview {
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
