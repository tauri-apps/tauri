// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  cell::RefCell,
  ffi::CString,
  os::raw::{c_long, c_ulong},
  sync::LazyLock,
};
use winit::window::ResizeDirection;
use x11_dl::xlib;

const NET_WM_STATE_REMOVE: c_long = 0;
const NET_WM_STATE_ADD: c_long = 1;
const NET_ACTIVE_WINDOW_SOURCE_PAGER: c_long = 2;
const CURRENT_TIME: c_long = 0;
const CLIENT_MESSAGE: i32 = 33;
const SUBSTRUCTURE_REDIRECT_MASK: c_long = 1 << 20;
const SUBSTRUCTURE_NOTIFY_MASK: c_long = 1 << 19;
const NET_WM_MOVERESIZE_RESIZE_NW: c_long = 0;
const NET_WM_MOVERESIZE_RESIZE_N: c_long = 1;
const NET_WM_MOVERESIZE_RESIZE_NE: c_long = 2;
const NET_WM_MOVERESIZE_RESIZE_E: c_long = 3;
const NET_WM_MOVERESIZE_RESIZE_SE: c_long = 4;
const NET_WM_MOVERESIZE_RESIZE_S: c_long = 5;
const NET_WM_MOVERESIZE_RESIZE_SW: c_long = 6;
const NET_WM_MOVERESIZE_RESIZE_W: c_long = 7;
const NET_WM_MOVERESIZE_MOVE: c_long = 8;

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

/// Ask the window manager to make `xid` the active window.
///
/// The source indication is `2` ("pager"): EWMH tells window managers to treat
/// those requests as if they came from the user, which is what gets past the
/// focus-stealing prevention that would otherwise turn the request into a
/// taskbar highlight. `1` ("application") would need a valid user input
/// timestamp we do not have when a window is created.
pub(super) fn activate_window(xid: c_ulong) {
  with_x11((), |xlib, display| {
    let net_active_window = atom(xlib, display, "_NET_ACTIVE_WINDOW");

    unsafe {
      (xlib.XRaiseWindow)(display, xid);

      let root = (xlib.XDefaultRootWindow)(display);
      let mut event: xlib::XEvent = std::mem::zeroed();
      event.client_message = xlib::XClientMessageEvent {
        type_: CLIENT_MESSAGE,
        serial: 0,
        send_event: 1,
        display,
        window: xid,
        message_type: net_active_window,
        format: 32,
        data: xlib::ClientMessageData::from([
          NET_ACTIVE_WINDOW_SOURCE_PAGER,
          CURRENT_TIME,
          // No requestor window: the request is about our own window.
          0,
          0,
          0,
        ]),
      };
      (xlib.XSendEvent)(
        display,
        root,
        0,
        SUBSTRUCTURE_REDIRECT_MASK | SUBSTRUCTURE_NOTIFY_MASK,
        &mut event,
      );
    }
  });
}

pub(super) fn set_wm_state(xid: c_ulong, add: bool, atom1: &str, atom2: Option<&str>) {
  with_x11((), |xlib, display| {
    let wm_state = atom(xlib, display, "_NET_WM_STATE");
    let a1 = atom(xlib, display, atom1);
    let a2 = atom2.map(|name| atom(xlib, display, name)).unwrap_or(0);
    let action = if add {
      NET_WM_STATE_ADD
    } else {
      NET_WM_STATE_REMOVE
    };

    unsafe {
      let root = (xlib.XDefaultRootWindow)(display);
      let mut event: xlib::XEvent = std::mem::zeroed();
      event.client_message = xlib::XClientMessageEvent {
        type_: CLIENT_MESSAGE,
        serial: 0,
        send_event: 1,
        display,
        window: xid,
        message_type: wm_state,
        format: 32,
        data: xlib::ClientMessageData::from([action, a1 as c_long, a2 as c_long, 1, 0]),
      };
      (xlib.XSendEvent)(
        display,
        root,
        0,
        SUBSTRUCTURE_REDIRECT_MASK | SUBSTRUCTURE_NOTIFY_MASK,
        &mut event,
      );
    }
  });
}

/// Ask the window manager to move/resize `xid` with the held pointer button.
pub(super) fn window_move_resize(
  xid: c_ulong,
  cef_xid: c_ulong,
  direction: Option<ResizeDirection>,
) {
  with_cef_display((), |xlib, display| unsafe {
    let root = (xlib.XDefaultRootWindow)(display);

    // Get root window position
    let mut x = 0;
    let mut y = 0;
    let mut mask = 0;
    // Different typed dummy variables are needed to satisfy `xlib.XQueryPointer`
    let mut dummy_u64: u64 = 0;
    let mut dummy_i32: i32 = 0;
    (xlib.XQueryPointer)(
      display,
      root,
      &mut dummy_u64, // root
      &mut dummy_u64, // child
      &mut x,
      &mut y,
      &mut dummy_i32, // child X pos
      &mut dummy_i32, // child Y pos
      &mut mask,      // pointer mask
    );
    let button = if mask & xlib::Button1Mask != 0 {
      1
    } else if mask & xlib::Button2Mask != 0 {
      2
    } else if mask & xlib::Button3Mask != 0 {
      3
    } else {
      return;
    };

    // According to spec, "The Client MUST release all grabs prior to sending _NET_WM_MOVERESIZE"
    // Chromium's internal connection holds the pointer, and since a client can only release it's own,
    // an unmap/remap of the CEF host forces the release of the pointer for the WM.
    (xlib.XUnmapWindow)(display, cef_xid);
    (xlib.XMapWindow)(display, cef_xid);

    let mut event: xlib::XEvent = std::mem::zeroed();
    event.client_message = xlib::XClientMessageEvent {
      type_: CLIENT_MESSAGE,
      serial: 0,
      send_event: 1,
      display,
      window: xid,
      message_type: atom(xlib, display, "_NET_WM_MOVERESIZE"),
      format: 32,
      data: xlib::ClientMessageData::from([
        x as c_long,
        y as c_long,
        match direction {
          // direction is converted to WM respective numbers
          Some(dir) => match dir {
            ResizeDirection::East => NET_WM_MOVERESIZE_RESIZE_E,
            ResizeDirection::North => NET_WM_MOVERESIZE_RESIZE_N,
            ResizeDirection::NorthEast => NET_WM_MOVERESIZE_RESIZE_NE,
            ResizeDirection::NorthWest => NET_WM_MOVERESIZE_RESIZE_NW,
            ResizeDirection::South => NET_WM_MOVERESIZE_RESIZE_S,
            ResizeDirection::SouthEast => NET_WM_MOVERESIZE_RESIZE_SE,
            ResizeDirection::SouthWest => NET_WM_MOVERESIZE_RESIZE_SW,
            ResizeDirection::West => NET_WM_MOVERESIZE_RESIZE_W,
          },
          // MOVE code is used for normal dragging
          None => NET_WM_MOVERESIZE_MOVE,
        },
        button,
        0,
      ]),
    };
    (xlib.XSendEvent)(
      display,
      root,
      0,
      SUBSTRUCTURE_REDIRECT_MASK | SUBSTRUCTURE_NOTIFY_MASK,
      &mut event,
    );
  })
}
