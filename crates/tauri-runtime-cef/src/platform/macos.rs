// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::ffi::c_void;
use tauri_runtime::dpi::Rect;
use winit::window::Window;

pub(crate) fn raw_handle(window: &dyn Window) -> *mut c_void {
  let _ = window;
  todo!("TODO: implement CEF/winit host handles on macOS")
}

pub(crate) fn set_child_bounds(handle: *mut c_void, x: i32, y: i32, width: i32, height: i32) {
  let _ = (handle, x, y, width, height);
  todo!("TODO: implement CEF/winit child layout on macOS")
}

pub(crate) fn child_bounds(handle: *mut c_void) -> Option<Rect> {
  let _ = handle;
  todo!("TODO: implement CEF/winit child bounds on macOS")
}
