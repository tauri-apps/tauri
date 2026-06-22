// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::ffi::c_void;

use tauri_runtime::dpi::Rect;
use winit::window::Window;

#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(windows)]
use windows as imp;

#[cfg(target_os = "macos")]
use macos as imp;

#[cfg(target_os = "linux")]
use linux as imp;

pub(crate) fn raw_handle(window: &dyn Window) -> *mut c_void {
  #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
  {
    imp::raw_handle(window)
  }

  #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
  {
    let _ = window;
    todo!("CEF/winit host handles are TODO for this platform")
  }
}

pub(crate) fn set_child_bounds(handle: *mut c_void, bounds: Rect, scale: f64) {
  let position = bounds.position.to_physical::<i32>(scale);
  let size = bounds.size.to_physical::<u32>(scale);

  #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
  {
    imp::set_child_bounds(
      handle,
      position.x,
      position.y,
      size.width as i32,
      size.height as i32,
    );
  }

  #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
  {
    let _ = (handle, position, size);
    todo!("CEF/winit child layout is TODO for this platform")
  }
}

pub(crate) fn child_bounds(handle: *mut c_void) -> Option<Rect> {
  #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
  {
    imp::child_bounds(handle)
  }

  #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
  {
    let _ = handle;
    todo!("CEF/winit child bounds are TODO for this platform")
  }
}
