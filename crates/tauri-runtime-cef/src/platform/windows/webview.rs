// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cef::ImplBrowserHost;
use tauri_runtime::dpi::{PhysicalPosition, PhysicalSize, Rect};
use tauri_utils::config::Color;
use windows::Win32::{
  Foundation::{HWND, POINT, RECT},
  Graphics::Gdi::MapWindowPoints,
  UI::WindowsAndMessaging::{
    GetParent, GetWindowRect, SW_HIDE, SW_SHOW, SWP_NOACTIVATE, SWP_NOZORDER, SetParent,
    SetWindowPos, ShowWindow,
  },
};

use crate::{webview::AppWebview, window::AppWindow};

impl AppWebview {
  pub(crate) fn hwnd(&self) -> HWND {
    let hwnd = self.host.window_handle();
    HWND(hwnd.0 as _)
  }

  pub(crate) fn set_background_color(&self, _color: Option<Color>) {
    // TODO: might not be supported on Windows
  }

  pub(crate) fn bounds(&self) -> Option<Rect> {
    let hwnd = self.hwnd();

    let mut rect = RECT::default();
    unsafe {
      let parent = GetParent(hwnd).ok()?;
      if parent.0.is_null() {
        return None;
      }

      GetWindowRect(hwnd, &mut rect).ok()?;

      let mut points = [
        POINT {
          x: rect.left,
          y: rect.top,
        },
        POINT {
          x: rect.right,
          y: rect.bottom,
        },
      ];
      if MapWindowPoints(None, Some(parent), &mut points) == 0 {
        return None;
      }

      let x = points[0].x;
      let y = points[0].y;
      let width = (points[1].x - points[0].x).max(0) as u32;
      let height = (points[1].y - points[0].y).max(0) as u32;
      Some(Rect {
        position: PhysicalPosition::new(x, y).into(),
        size: PhysicalSize::new(width, height).into(),
      })
    }
  }

  pub(crate) fn reparent(&self, parent: &AppWindow) {
    let parent = parent.hwnd();
    let _ = unsafe { SetParent(self.hwnd(), Some(parent)) };
  }

  pub(crate) fn apply_visible(&self, visible: bool) {
    let _ = unsafe { ShowWindow(self.hwnd(), if visible { SW_SHOW } else { SW_HIDE }) };
  }

  pub(crate) fn apply_physical_bounds(&self, _scale: f64, x: i32, y: i32, width: i32, height: i32) {
    unsafe {
      let _ = SetWindowPos(
        self.hwnd(),
        None,
        x,
        y,
        width,
        height,
        SWP_NOZORDER | SWP_NOACTIVATE,
      );
    }
  }
}
