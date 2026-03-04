use cef::*;
use std::sync::LazyLock;
use x11_dl::xlib;

use crate::cef_webview::CefBrowserExt;

static X11: LazyLock<Option<xlib::Xlib>> = LazyLock::new(|| xlib::Xlib::open().ok());

impl CefBrowserExt for cef::Browser {
  fn xid(&self) -> Option<u64> {
    let host = self.host()?;
    let xid = host.window_handle();
    Some(xid)
  }

  fn bounds(&self) -> cef::Rect {
    let Some(xid) = self.xid() else {
      return cef::Rect::default();
    };

    let Some(xlib) = X11.as_ref() else {
      return cef::Rect::default();
    };

    unsafe {
      let display = (xlib.XOpenDisplay)(std::ptr::null());
      if display.is_null() {
        return cef::Rect::default();
      }

      let mut root: xlib::Window = 0;
      let mut x: i32 = 0;
      let mut y: i32 = 0;
      let mut width: u32 = 0;
      let mut height: u32 = 0;
      let mut border_width: u32 = 0;
      let mut depth: u32 = 0;

      let status = (xlib.XGetGeometry)(
        display,
        xid as xlib::Window,
        &mut root,
        &mut x,
        &mut y,
        &mut width,
        &mut height,
        &mut border_width,
        &mut depth,
      );

      (xlib.XCloseDisplay)(display);

      if status == 0 {
        return cef::Rect::default();
      }

      // XGetGeometry returns position relative to parent, which is what we need
      cef::Rect {
        x,
        y,
        width: width as i32,
        height: height as i32,
      }
    }
  }

  fn set_bounds(&self, rect: Option<&cef::Rect>) {
    let Some(rect) = rect else {
      return;
    };

    let Some(xid) = self.xid() else {
      return;
    };

    let Some(xlib) = X11.as_ref() else {
      return;
    };

    unsafe {
      let display = (xlib.XOpenDisplay)(std::ptr::null());
      if display.is_null() {
        return;
      }

      (xlib.XMoveResizeWindow)(
        display,
        xid as xlib::Window,
        rect.x,
        rect.y,
        rect.width as u32,
        rect.height as u32,
      );
      // Ensure window is mapped and raised after setting bounds
      (xlib.XMapRaised)(display, xid as xlib::Window);
      (xlib.XFlush)(display);
      (xlib.XCloseDisplay)(display);
    }
  }

  fn scale_factor(&self) -> f64 {
    // Get scale factor from primary display
    // CEF on Linux doesn't provide direct access to the window's display,
    // so we use the primary display as a reasonable default
    cef::display_get_primary()
      .map(|d| d.device_scale_factor() as f64)
      .unwrap_or(1.0)
  }

  fn set_visible(&self, visible: i32) {
    let Some(xid) = self.xid() else {
      return;
    };

    let Some(xlib) = X11.as_ref() else {
      return;
    };

    unsafe {
      let display = (xlib.XOpenDisplay)(std::ptr::null());
      if display.is_null() {
        return;
      }

      if visible != 0 {
        (xlib.XMapWindow)(display, xid as xlib::Window);
      } else {
        (xlib.XUnmapWindow)(display, xid as xlib::Window);
      }
      (xlib.XFlush)(display);
      (xlib.XCloseDisplay)(display);
    }
  }

  fn close(&self) {
    let Some(xid) = self.xid() else {
      return;
    };

    let Some(xlib) = X11.as_ref() else {
      return;
    };

    unsafe {
      let display = (xlib.XOpenDisplay)(std::ptr::null());
      if display.is_null() {
        return;
      }

      (xlib.XDestroyWindow)(display, xid as xlib::Window);
      (xlib.XFlush)(display);
      (xlib.XCloseDisplay)(display);
    }
  }

  fn set_parent(&self, parent: &cef::Window) {
    let Some(xid) = self.xid() else {
      return;
    };

    let parent_xid = parent.window_handle();
    if parent_xid == 0 {
      return;
    }

    let Some(xlib) = X11.as_ref() else {
      return;
    };

    unsafe {
      let display = (xlib.XOpenDisplay)(std::ptr::null());
      if display.is_null() {
        return;
      }

      // Check if window exists before reparenting
      let mut root: xlib::Window = 0;
      let mut parent_window: xlib::Window = 0;
      let mut children: *mut xlib::Window = std::ptr::null_mut();
      let mut nchildren: u32 = 0;
      let status = (xlib.XQueryTree)(
        display,
        xid as xlib::Window,
        &mut root,
        &mut parent_window,
        &mut children,
        &mut nchildren,
      );

      if status != 0 && !children.is_null() {
        (xlib.XFree)(children as *mut std::ffi::c_void);
      }

      (xlib.XReparentWindow)(
        display,
        xid as xlib::Window,
        parent_xid as xlib::Window,
        0,
        0,
      );

      // Ensure window is mapped and raised after reparenting
      (xlib.XMapRaised)(display, xid as xlib::Window);
      (xlib.XFlush)(display);
      (xlib.XCloseDisplay)(display);
    }
  }
}
