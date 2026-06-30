// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cef::ImplBrowserHost;
use std::os::raw::c_ulong;
use tauri_runtime::dpi::{PhysicalPosition, PhysicalSize, Rect};
use tauri_utils::config::Color;
use x11_dl::xlib;

use crate::{webview::AppWebview, window::AppWindow};

use super::utils::{atom, with_cef_display};

impl AppWebview {
  fn xid(&self) -> xlib::Window {
    let xid = self.host.window_handle();
    assert_ne!(xid, 0, "failed to get XID");
    xid as xlib::Window
  }

  pub(crate) fn set_background_color(&self, color: Option<Color>) {
    let _ = (self, color);
    // Native child-window background is not equivalent to Chromium's rendered
    // background. Creation still applies BrowserSettings.
  }

  pub(crate) fn bounds(&self) -> Option<Rect> {
    let xid = self.xid();

    with_cef_display(None, |xlib, display| unsafe {
      let mut root: xlib::Window = 0;
      let mut x: i32 = 0;
      let mut y: i32 = 0;
      let mut width: u32 = 0;
      let mut height: u32 = 0;
      let mut border_width: u32 = 0;
      let mut depth: u32 = 0;

      if (xlib.XGetGeometry)(
        display,
        xid,
        &mut root,
        &mut x,
        &mut y,
        &mut width,
        &mut height,
        &mut border_width,
        &mut depth,
      ) == 0
      {
        return None;
      }

      Some(Rect {
        position: PhysicalPosition::new(x, y).into(),
        size: PhysicalSize::new(width, height).into(),
      })
    })
  }

  pub(crate) fn reparent(&self, parent: &AppWindow) {
    let xid = self.xid();
    let parent_xid = parent.xid();

    with_cef_display((), |xlib, display| unsafe {
      (xlib.XReparentWindow)(display, xid, parent_xid as xlib::Window, 0, 0);
      (xlib.XMapRaised)(display, xid);
    });
  }

  pub(crate) fn apply_visible(&self, visible: bool) {
    let xid = self.xid();

    with_cef_display((), |xlib, display| unsafe {
      let net_wm_state = atom(xlib, display, "_NET_WM_STATE");
      const PROP_MODE_REPLACE: i32 = 0;

      if visible {
        (xlib.XChangeProperty)(
          display,
          xid,
          net_wm_state,
          xlib::XA_ATOM,
          32,
          PROP_MODE_REPLACE,
          std::ptr::null(),
          0,
        );
        (xlib.XMapWindow)(display, xid);
      } else {
        let hidden: [c_ulong; 1] = [atom(xlib, display, "_NET_WM_STATE_HIDDEN")];
        (xlib.XChangeProperty)(
          display,
          xid,
          net_wm_state,
          xlib::XA_ATOM,
          32,
          PROP_MODE_REPLACE,
          hidden.as_ptr() as *const u8,
          1,
        );
        (xlib.XUnmapWindow)(display, xid);
      }
    });
  }

  pub(crate) fn apply_physical_bounds(&self, _scale: f64, x: i32, y: i32, width: i32, height: i32) {
    let xid = self.xid();

    with_cef_display((), |xlib, display| unsafe {
      (xlib.XMoveResizeWindow)(
        display,
        xid,
        x,
        y,
        width.max(1) as u32,
        height.max(1) as u32,
      );
      // `with_cef_display` issues an `XFlush` once the closure returns, so a
      // blocking `XSync` round-trip here just stalls every resize frame.
    });
  }
}
