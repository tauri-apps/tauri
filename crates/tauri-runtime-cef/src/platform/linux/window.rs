// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::os::raw::{c_int, c_ulong};
use tauri_runtime::ProgressBarState;
use tauri_utils::config::Color;

use crate::window::AppWindow;

use super::{taskbar, utils::set_wm_state};

impl AppWindow {
  fn xid(&self) -> Option<c_ulong> {
    let handle = self.window.window_handle().ok()?;
    match handle.as_raw() {
      RawWindowHandle::Xlib(handle) => Some(handle.window as c_ulong),
      RawWindowHandle::Xcb(handle) => Some(handle.window.get() as c_ulong),
      _ => None,
    }
  }

  pub(crate) fn set_enabled(&self, enabled: bool) {
    let _ = (self, enabled);
    // TODO: implement native window enabled state on Linux/BSD.
  }

  pub(crate) fn is_enabled(&self) -> bool {
    let _ = self;
    // TODO: query native window enabled state on Linux/BSD.
    true
  }

  pub(crate) fn set_focusable(&self, focusable: bool) {
    if let Some(xid) = self.xid() {
      super::utils::with_x11((), |xlib, display| unsafe {
        let mut hints = (xlib.XGetWMHints)(display, xid);
        if hints.is_null() {
          hints = (xlib.XAllocWMHints)();
        }
        if hints.is_null() {
          return;
        }

        (*hints).flags |= x11_dl::xlib::InputHint;
        (*hints).input = focusable as c_int;
        (xlib.XSetWMHints)(display, xid, hints);
        (xlib.XFree)(hints.cast());
      });
    }
  }

  pub(crate) fn set_background_color(&self, color: Option<Color>) {
    let Some(xid) = self.xid() else {
      return;
    };
    let Some(color) = color else {
      return;
    };

    super::utils::with_x11((), |xlib, display| unsafe {
      let screen = (xlib.XDefaultScreen)(display);
      let colormap = (xlib.XDefaultColormap)(display, screen);
      let mut xcolor = x11_dl::xlib::XColor {
        pixel: 0,
        red: u16::from(color.0) * 257,
        green: u16::from(color.1) * 257,
        blue: u16::from(color.2) * 257,
        flags: x11_dl::xlib::DoRed | x11_dl::xlib::DoGreen | x11_dl::xlib::DoBlue,
        pad: 0,
      };

      if (xlib.XAllocColor)(display, colormap, &mut xcolor) != 0 {
        (xlib.XSetWindowBackground)(display, xid, xcolor.pixel);
        (xlib.XClearWindow)(display, xid);
      }
    });
  }

  pub(crate) fn set_skip_taskbar(&self, skip: bool) {
    if let Some(xid) = self.xid() {
      set_wm_state(xid, skip, "_NET_WM_STATE_SKIP_TASKBAR", None);
    }
  }

  pub(crate) fn set_visible_on_all_workspaces(&self, visible: bool) {
    if let Some(xid) = self.xid() {
      set_wm_state(xid, visible, "_NET_WM_STATE_STICKY", None);
    }
  }

  pub(crate) fn set_progress_bar(&self, state: ProgressBarState) {
    taskbar::set_progress_bar(state);
  }
}
