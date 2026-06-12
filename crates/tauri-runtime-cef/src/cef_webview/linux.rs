use cef::*;
use std::os::raw::c_ulong;
use std::sync::LazyLock;
use x11_dl::xlib;

use crate::cef_webview::CefBrowserExt;

static X11: LazyLock<Option<xlib::Xlib>> = LazyLock::new(|| xlib::Xlib::open().ok());

/// Runs `f` with the Xlib handle and CEF's own X11 display connection.
///
/// cefclient performs every operation on a windowed browser's X window through
/// the display returned by `cef_get_xdisplay()` (see
/// `tests/cefclient/browser/browser_window_std_gtk.cc`). Using CEF's
/// connection keeps our requests ordered with Chromium's own X11 traffic and
/// avoids the cost of opening (and the races of flushing/closing) a throwaway
/// connection per call.
///
/// Must only be called on the CEF UI thread — which is where all webview
/// messages are handled.
fn with_cef_display<R>(default: R, f: impl FnOnce(&xlib::Xlib, *mut xlib::Display) -> R) -> R {
  let Some(xlib) = X11.as_ref() else {
    return default;
  };
  let display = cef::get_xdisplay() as *mut xlib::Display;
  if display.is_null() {
    return default;
  }
  let result = f(xlib, display);
  // Flush so the request is processed now; never close CEF's display.
  unsafe { (xlib.XFlush)(display) };
  result
}

fn atom(xlib: &xlib::Xlib, display: *mut xlib::Display, name: &str) -> c_ulong {
  let cname = std::ffi::CString::new(name).unwrap();
  unsafe { (xlib.XInternAtom)(display, cname.as_ptr(), 0) }
}

/// Repairs a browser X window after its toplevel was hidden and shown again
/// by the window manager: raises it above the Views content surface and
/// forces the renderer to produce a fresh frame.
///
/// Must run on the CEF UI thread — that serializes the repair with
/// Chromium's own processing of the X events that accompany the show
/// transition, so the jiggle cannot race ahead of the visibility handling.
pub(crate) fn repair_browser_window(xid: u64) {
  with_cef_display((), |xlib, display| {
    unsafe { (xlib.XRaiseWindow)(display, xid as xlib::Window) };
    resize_jiggle(xlib, display, xid as xlib::Window);
  });
}

/// Resizes the browser's X window by one pixel and back.
///
/// Forces the renderer to redraw: in Chromium's surface synchronization,
/// every size change allocates a new viz `LocalSurfaceId`, which obligates
/// the renderer to submit a fresh `CompositorFrame` — for both the shrink
/// and the restore. This recovers from the stalled state window managers
/// leave child browsers in after hiding the toplevel (workspace switch,
/// iconify): the GPU presentation surfaces are recreated on show, but no new
/// frame is produced until the renderer redraws, which otherwise happens
/// only on focus or input. CEF's `CefWindowX11` observes both
/// `ConfigureNotify` events and propagates the sizes to the inner Chromium
/// widget.
pub(crate) fn resize_jiggle(xlib: &xlib::Xlib, display: *mut xlib::Display, window: xlib::Window) {
  unsafe {
    let mut root: xlib::Window = 0;
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut border_width: u32 = 0;
    let mut depth: u32 = 0;
    if (xlib.XGetGeometry)(
      display,
      window,
      &mut root,
      &mut x,
      &mut y,
      &mut width,
      &mut height,
      &mut border_width,
      &mut depth,
    ) == 0
      || width == 0
      || height == 0
    {
      return;
    }

    (xlib.XResizeWindow)(display, window, width, height.max(2) - 1);
    (xlib.XFlush)(display);
    (xlib.XResizeWindow)(display, window, width, height);
    (xlib.XFlush)(display);
  }
}

/// The browser's X window lives in raw X11 coordinates (physical pixels) while
/// the `CefBrowserExt` contract — like the CEF Views APIs used everywhere else
/// in this runtime — is DIP. These helpers convert between the two.
fn logical_to_physical(rect: &cef::Rect, scale: f64) -> cef::Rect {
  cef::Rect {
    x: (rect.x as f64 * scale).round() as i32,
    y: (rect.y as f64 * scale).round() as i32,
    width: (rect.width as f64 * scale).round() as i32,
    height: (rect.height as f64 * scale).round() as i32,
  }
}

fn physical_to_logical(rect: &cef::Rect, scale: f64) -> cef::Rect {
  cef::Rect {
    x: (rect.x as f64 / scale).round() as i32,
    y: (rect.y as f64 / scale).round() as i32,
    width: (rect.width as f64 / scale).round() as i32,
    height: (rect.height as f64 / scale).round() as i32,
  }
}

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

    let physical = with_cef_display(cef::Rect::default(), |xlib, display| unsafe {
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
    });

    physical_to_logical(&physical, self.scale_factor())
  }

  fn set_bounds(&self, rect: Option<&cef::Rect>) {
    let Some(rect) = rect else {
      return;
    };

    let Some(xid) = self.xid() else {
      return;
    };

    let physical = logical_to_physical(rect, self.scale_factor());

    with_cef_display((), |xlib, display| unsafe {
      // CEF's `CefWindowX11` observes the resulting ConfigureNotify and
      // resizes the inner Chromium widget to match. No map/raise here: doing
      // that on every resize churns the z-order between sibling webviews.
      (xlib.XMoveResizeWindow)(
        display,
        xid as xlib::Window,
        physical.x,
        physical.y,
        physical.width.max(1) as u32,
        physical.height.max(1) as u32,
      );
    });
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

    with_cef_display((), |xlib, display| unsafe {
      // Mirror cefclient's `SetXWindowVisible`: toggle `_NET_WM_STATE_HIDDEN`
      // on the browser window. CEF's `CefWindowX11` forwards the state to the
      // inner Chromium widget so rendering resources are released while
      // hidden. The window must additionally be mapped/unmapped for the
      // visual effect, since its parent stays mapped.
      let net_wm_state = atom(xlib, display, "_NET_WM_STATE");
      const PROP_MODE_REPLACE: i32 = 0;

      if visible != 0 {
        (xlib.XChangeProperty)(
          display,
          xid as xlib::Window,
          net_wm_state,
          xlib::XA_ATOM,
          32,
          PROP_MODE_REPLACE,
          std::ptr::null(),
          0,
        );
        (xlib.XMapWindow)(display, xid as xlib::Window);
      } else {
        let hidden: [c_ulong; 1] = [atom(xlib, display, "_NET_WM_STATE_HIDDEN")];
        (xlib.XChangeProperty)(
          display,
          xid as xlib::Window,
          net_wm_state,
          xlib::XA_ATOM,
          32,
          PROP_MODE_REPLACE,
          hidden.as_ptr() as *const u8,
          1,
        );
        (xlib.XUnmapWindow)(display, xid as xlib::Window);
      }
    });
  }

  fn close(&self) {
    // Destroying the X window behind CEF's back (the previous behavior,
    // `XDestroyWindow`) leaves `CefWindowX11` unaware — it has no
    // DestroyNotify handling — so the browser never goes through
    // `OnBeforeClose`, leaking the renderer process and hanging
    // `cef::shutdown` on exit. Drive CEF's regular close lifecycle instead,
    // matching the macOS implementation: CEF itself sends WM_DELETE_WINDOW to
    // its `CefWindowX11` and tears the window down.
    let Some(host) = self.host() else {
      return;
    };
    let _ = host.try_close_browser();
  }

  fn raise(&self) {
    let Some(xid) = self.xid() else {
      return;
    };

    with_cef_display((), |xlib, display| unsafe {
      (xlib.XRaiseWindow)(display, xid as xlib::Window);
    });
  }

  fn refresh_render_target(&self) {
    let Some(xid) = self.xid() else {
      return;
    };

    with_cef_display((), |xlib, display| {
      resize_jiggle(xlib, display, xid as xlib::Window);
    });
  }

  fn set_parent(&self, parent: &cef::Window) {
    let Some(xid) = self.xid() else {
      return;
    };

    let parent_xid = parent.window_handle();
    if parent_xid == 0 {
      return;
    }

    with_cef_display((), |xlib, display| unsafe {
      // cefclient's `ShowPopup` pattern: reparent, then make sure the window
      // is mapped (the caller re-applies bounds afterwards).
      (xlib.XReparentWindow)(
        display,
        xid as xlib::Window,
        parent_xid as xlib::Window,
        0,
        0,
      );
      (xlib.XMapRaised)(display, xid as xlib::Window);
    });
  }
}
