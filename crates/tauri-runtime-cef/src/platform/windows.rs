// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_runtime::dpi::{PhysicalPosition, PhysicalSize, Rect};
use windows::Win32::{
  Foundation::{HWND, POINT, RECT},
  Graphics::Gdi::MapWindowPoints,
  UI::WindowsAndMessaging::{GetParent, GetWindowRect, SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos},
};
use winit::{
  raw_window_handle::{HasWindowHandle, RawWindowHandle},
  window::Window,
};

use std::ffi::c_void;

pub(crate) fn raw_handle(window: &dyn Window) -> *mut c_void {
  let handle = window.window_handle().expect("failed to get window handle");
  match handle.as_raw() {
    RawWindowHandle::Win32(handle) => handle.hwnd.get() as usize as *mut c_void,
    other => panic!("expected Win32 window handle, got {other:?}"),
  }
}

pub(crate) fn set_child_bounds(handle: *mut c_void, x: i32, y: i32, width: i32, height: i32) {
  unsafe {
    let _ = SetWindowPos(
      HWND(handle),
      None,
      x,
      y,
      width,
      height,
      SWP_NOZORDER | SWP_NOACTIVATE,
    );
  }
}

pub(crate) fn child_bounds(handle: *mut c_void) -> Option<Rect> {
  let mut rect = RECT::default();
  unsafe {
    let parent = GetParent(HWND(handle)).ok()?;
    if parent.0.is_null() {
      return None;
    }

    GetWindowRect(HWND(handle), &mut rect).ok()?;

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
