// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{cell::RefCell, ffi::CString, os::raw::c_ulong, sync::LazyLock};
use x11_dl::xlib;

static XLIB: LazyLock<Option<xlib::Xlib>> = LazyLock::new(|| xlib::Xlib::open().ok());

struct Display(*mut xlib::Display);

thread_local! {
  static DISPLAY: RefCell<Option<Display>> = const { RefCell::new(None) };
}

pub(super) fn with_cef_display<R>(
  default: R,
  f: impl FnOnce(&xlib::Xlib, *mut xlib::Display) -> R,
) -> R {
  let Some(xlib) = XLIB.as_ref() else {
    return default;
  };
  let display = cef::get_xdisplay() as *mut xlib::Display;
  if display.is_null() {
    return default;
  }

  let result = f(xlib, display);
  unsafe {
    (xlib.XFlush)(display);
  }
  result
}

pub(super) fn with_x11<R>(default: R, f: impl FnOnce(&xlib::Xlib, *mut xlib::Display) -> R) -> R {
  let Some(xlib) = XLIB.as_ref() else {
    return default;
  };

  DISPLAY.with(|cell| {
    let mut guard = cell.borrow_mut();
    if guard.is_none() {
      let display = unsafe { (xlib.XOpenDisplay)(std::ptr::null()) };
      if display.is_null() {
        return default;
      }
      *guard = Some(Display(display));
    }

    let display = guard.as_ref().unwrap().0;
    let result = f(xlib, display);
    unsafe {
      (xlib.XFlush)(display);
    }
    result
  })
}

pub(super) fn atom(xlib: &xlib::Xlib, display: *mut xlib::Display, name: &str) -> c_ulong {
  let cname = CString::new(name).unwrap();
  unsafe { (xlib.XInternAtom)(display, cname.as_ptr(), 0) }
}
