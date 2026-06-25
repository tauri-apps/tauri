// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_runtime::dpi::Rect;
use tauri_utils::config::Color;

use crate::{webview::AppWebview, window::AppWindow};

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
