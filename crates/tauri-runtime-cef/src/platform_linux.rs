//! Linux / *BSD (X11) implementation of the platform window helpers.

#![allow(clippy::missing_safety_doc)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::{c_char, c_long, c_uint, c_ulong};

use cef::{ImplView, ImplWindow};
use tauri_runtime::dpi::{PhysicalPosition, Position};
use tauri_runtime::window::CursorIcon;
use tauri_runtime::{ProgressBarState, ProgressBarStatus, ResizeDirection, UserAttentionType};
use x11_dl::xlib;

// EWMH / X11 constants not exported by x11-dl.
const _NET_WM_STATE_REMOVE: c_long = 0;
const _NET_WM_STATE_ADD: c_long = 1;
const CLIENT_MESSAGE: i32 = 33;
const SUBSTRUCTURE_REDIRECT_MASK: c_long = 1 << 20;
const SUBSTRUCTURE_NOTIFY_MASK: c_long = 1 << 19;
const CURRENT_TIME: c_ulong = 0;
const SHAPE_INPUT: i32 = 2;
// _NET_WM_MOVERESIZE directions.
const MOVERESIZE_SIZE_TOPLEFT: c_long = 0;
const MOVERESIZE_SIZE_TOP: c_long = 1;
const MOVERESIZE_SIZE_TOPRIGHT: c_long = 2;
const MOVERESIZE_SIZE_RIGHT: c_long = 3;
const MOVERESIZE_SIZE_BOTTOMRIGHT: c_long = 4;
const MOVERESIZE_SIZE_BOTTOM: c_long = 5;
const MOVERESIZE_SIZE_BOTTOMLEFT: c_long = 6;
const MOVERESIZE_SIZE_LEFT: c_long = 7;
// X cursor font shapes (X11/cursorfont.h).
const XC_ARROW: c_uint = 2;
const XC_CROSSHAIR: c_uint = 34;
const XC_HAND2: c_uint = 60;
const XC_XTERM: c_uint = 152;
const XC_WATCH: c_uint = 150;
const XC_QUESTION_ARROW: c_uint = 92;
const XC_FLEUR: c_uint = 52;
const XC_SB_H_DOUBLE_ARROW: c_uint = 108;
const XC_SB_V_DOUBLE_ARROW: c_uint = 116;

struct Display(*mut xlib::Display);

thread_local! {
  // A persistent X11 connection. A transient connection (open/close per call)
  // cannot hold a pointer grab — the grab is released when the connection is
  // closed — so we keep one alive for the lifetime of the thread. All window
  // messages are handled on the UI thread, so a thread-local is sufficient.
  static DISPLAY: RefCell<Option<Display>> = const { RefCell::new(None) };
}

/// Runs `f` with the X11 library and a live display connection, returning
/// `default` when X11 is unavailable.
fn with_x11<R>(default: R, f: impl FnOnce(&xlib::Xlib, *mut xlib::Display) -> R) -> R {
  static XLIB: std::sync::LazyLock<Option<xlib::Xlib>> =
    std::sync::LazyLock::new(|| xlib::Xlib::open().ok());

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

fn atom(xlib: &xlib::Xlib, display: *mut xlib::Display, name: &str) -> c_ulong {
  let cname = CString::new(name).unwrap();
  unsafe { (xlib.XInternAtom)(display, cname.as_ptr(), 0) }
}

/// Sends a `_NET_WM_STATE` client message to the root window to add/remove a
/// state on a mapped window.
fn set_wm_state(
  xlib: &xlib::Xlib,
  display: *mut xlib::Display,
  xid: c_ulong,
  add: bool,
  atom1: &str,
  atom2: Option<&str>,
) {
  let wm_state = atom(xlib, display, "_NET_WM_STATE");
  let a1 = atom(xlib, display, atom1);
  let a2 = atom2.map(|n| atom(xlib, display, n)).unwrap_or(0);
  let action = if add {
    _NET_WM_STATE_ADD
  } else {
    _NET_WM_STATE_REMOVE
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
}

/// Enables or disables user interaction with the window.
///
/// On X11 there is no `EnableWindow`-style call, and CEF runs on its own
/// Aura/X11 windows rather than a GTK widget we could mark insensitive (the way
/// `tao` does via `set_sensitive`). The best available mechanism is CEF's own
/// `View::set_enabled`; note it disables the window's root view but does not
/// necessarily block input to the child browser, so this is best-effort on
/// Linux.
pub fn set_enabled(window: &cef::Window, enabled: bool) {
  window.set_enabled(if enabled { 1 } else { 0 });
}

/// Reports whether the window's root view is enabled.
pub fn is_enabled(window: &cef::Window) -> bool {
  window.is_enabled() == 1
}

/// Only macOS needs to break the run loop explicitly; elsewhere
/// `cef::quit_message_loop` suffices, so this is a no-op.
pub fn stop_event_loop() {}

pub fn set_skip_taskbar(window: &cef::Window, skip: bool) {
  let xid = window.window_handle() as c_ulong;
  // GTK's `set_skip_taskbar_hint` (used by tao) only toggles the taskbar hint.
  with_x11((), |xlib, display| {
    set_wm_state(xlib, display, xid, skip, "_NET_WM_STATE_SKIP_TASKBAR", None);
  });
}

pub fn set_always_on_bottom(window: &cef::Window, on_bottom: bool) {
  let xid = window.window_handle() as c_ulong;
  with_x11((), |xlib, display| {
    set_wm_state(xlib, display, xid, on_bottom, "_NET_WM_STATE_BELOW", None);
  });
}

pub fn set_visible_on_all_workspaces(window: &cef::Window, visible: bool) {
  let xid = window.window_handle() as c_ulong;
  // GTK's `stick`/`unstick` (used by tao) maps to the `_NET_WM_STATE_STICKY` hint.
  with_x11((), |xlib, display| {
    set_wm_state(xlib, display, xid, visible, "_NET_WM_STATE_STICKY", None);
  });
}

/// X11 has no standard way to toggle the window-manager drop shadow, so this
/// is a no-op (matching `tao`'s behavior on Linux).
pub fn set_shadow(_window: &cef::Window, _enable: bool) {}

pub fn request_user_attention(window: &cef::Window, request_type: Option<UserAttentionType>) {
  let xid = window.window_handle() as c_ulong;
  let demand = request_type.is_some();
  with_x11((), |xlib, display| unsafe {
    set_wm_state(
      xlib,
      display,
      xid,
      demand,
      "_NET_WM_STATE_DEMANDS_ATTENTION",
      None,
    );
    // Also toggle the ICCCM urgency hint for window managers that honor it.
    let hints = (xlib.XAllocWMHints)();
    if !hints.is_null() {
      (*hints).flags = xlib::XUrgencyHint;
      (*hints).input = demand as i32;
      (xlib.XSetWMHints)(display, xid, hints);
      (xlib.XFree)(hints as *mut _);
    }
  });
}

pub fn global_cursor_position() -> Option<PhysicalPosition<f64>> {
  with_x11(None, |xlib, display| unsafe {
    let root = (xlib.XDefaultRootWindow)(display);
    let mut root_return: c_ulong = 0;
    let mut child_return: c_ulong = 0;
    let mut root_x = 0;
    let mut root_y = 0;
    let mut win_x = 0;
    let mut win_y = 0;
    let mut mask: c_uint = 0;
    let ok = (xlib.XQueryPointer)(
      display,
      root,
      &mut root_return,
      &mut child_return,
      &mut root_x,
      &mut root_y,
      &mut win_x,
      &mut win_y,
      &mut mask,
    );
    if ok == 0 {
      None
    } else {
      Some(PhysicalPosition::new(root_x as f64, root_y as f64))
    }
  })
}

pub fn set_cursor_position(window: &cef::Window, position: Position, scale_factor: f64) {
  // The position is relative to the window (like tao). Warping with the window
  // as the destination interprets the coordinates relative to its origin.
  let xid = window.window_handle() as c_ulong;
  let physical = position.to_physical::<i32>(scale_factor);
  with_x11((), |xlib, display| unsafe {
    (xlib.XWarpPointer)(display, 0, xid, 0, 0, 0, 0, physical.x, physical.y);
  });
}

pub fn set_cursor_visible(window: &cef::Window, visible: bool) {
  let xid = window.window_handle() as c_ulong;
  static XFIXES: std::sync::LazyLock<Option<x11_dl::xfixes::Xlib>> =
    std::sync::LazyLock::new(|| x11_dl::xfixes::Xlib::open().ok());
  let Some(xfixes) = XFIXES.as_ref() else {
    return;
  };
  with_x11((), |_xlib, display| unsafe {
    if visible {
      (xfixes.XFixesShowCursor)(display, xid);
    } else {
      (xfixes.XFixesHideCursor)(display, xid);
    }
  });
}

/// `tao` does not implement cursor grabbing on Linux (its GTK backend leaves it
/// unimplemented), so this is a no-op for parity.
pub fn set_cursor_grab(_window: &cef::Window, _grab: bool) {}

pub fn set_ignore_cursor_events(window: &cef::Window, ignore: bool) {
  let xid = window.window_handle() as c_ulong;
  static XFIXES: std::sync::LazyLock<Option<x11_dl::xfixes::Xlib>> =
    std::sync::LazyLock::new(|| x11_dl::xfixes::Xlib::open().ok());
  let Some(xfixes) = XFIXES.as_ref() else {
    return;
  };
  with_x11((), |_xlib, display| unsafe {
    if ignore {
      // An empty input region makes the window transparent to pointer events.
      let region = (xfixes.XFixesCreateRegion)(display, std::ptr::null_mut(), 0);
      (xfixes.XFixesSetWindowShapeRegion)(display, xid, SHAPE_INPUT, 0, 0, region);
      (xfixes.XFixesDestroyRegion)(display, region);
    } else {
      // Region 0 (None) restores the default whole-window input region.
      (xfixes.XFixesSetWindowShapeRegion)(display, xid, SHAPE_INPUT, 0, 0, 0);
    }
  });
}

pub fn set_cursor_icon(window: &cef::Window, icon: CursorIcon) {
  let xid = window.window_handle() as c_ulong;
  // Prefer themed cursors via libXcursor, falling back to the X cursor font.
  static XCURSOR: std::sync::LazyLock<Option<x11_dl::xcursor::Xcursor>> =
    std::sync::LazyLock::new(|| x11_dl::xcursor::Xcursor::open().ok());

  with_x11((), |xlib, display| unsafe {
    let mut cursor: c_ulong = 0;
    if let Some(xcursor) = XCURSOR.as_ref()
      && let Ok(name) = CString::new(cursor_icon_name(icon))
    {
      cursor = (xcursor.XcursorLibraryLoadCursor)(display, name.as_ptr());
    }
    if cursor == 0 {
      cursor = (xlib.XCreateFontCursor)(display, cursor_icon_font_shape(icon));
    }
    if cursor != 0 {
      (xlib.XDefineCursor)(display, xid, cursor);
      (xlib.XFreeCursor)(display, cursor);
    }
  });
}

/// Maps a [`CursorIcon`] to a freedesktop cursor name for libXcursor.
fn cursor_icon_name(icon: CursorIcon) -> &'static str {
  use CursorIcon::*;
  match icon {
    Default => "left_ptr",
    Crosshair => "crosshair",
    Hand | Grab => "hand2",
    Grabbing => "grabbing",
    Arrow => "arrow",
    Move => "fleur",
    Text => "xterm",
    Wait => "watch",
    Help => "question_arrow",
    Progress => "left_ptr_watch",
    NotAllowed | NoDrop => "crossed_circle",
    ContextMenu => "context-menu",
    Cell => "plus",
    VerticalText => "vertical-text",
    Alias => "dnd-link",
    Copy => "dnd-copy",
    AllScroll => "all-scroll",
    ZoomIn => "zoom-in",
    ZoomOut => "zoom-out",
    EResize | WResize | EwResize | ColResize => "sb_h_double_arrow",
    NResize | SResize | NsResize | RowResize => "sb_v_double_arrow",
    NeResize | SwResize | NeswResize => "fd_double_arrow",
    NwResize | SeResize | NwseResize => "bd_double_arrow",
    _ => "left_ptr",
  }
}

/// Maps a [`CursorIcon`] to an X cursor-font shape (the fallback).
fn cursor_icon_font_shape(icon: CursorIcon) -> c_uint {
  use CursorIcon::*;
  match icon {
    Crosshair => XC_CROSSHAIR,
    Hand | Grab | Grabbing => XC_HAND2,
    Move | AllScroll => XC_FLEUR,
    Text | VerticalText => XC_XTERM,
    Wait | Progress => XC_WATCH,
    Help => XC_QUESTION_ARROW,
    EResize | WResize | EwResize | ColResize => XC_SB_H_DOUBLE_ARROW,
    NResize | SResize | NsResize | RowResize => XC_SB_V_DOUBLE_ARROW,
    _ => XC_ARROW,
  }
}

pub fn set_progress_bar(_window: &cef::Window, state: ProgressBarState) {
  let Some(desktop_filename) = state.desktop_filename.as_deref() else {
    return;
  };
  unity::with_entry(desktop_filename, |lib, entry| unsafe {
    if let Some(progress) = state.progress {
      let progress = progress.min(100) as f64 / 100.0;
      lib.unity_launcher_entry_set_progress(entry, progress);
    }
    if let Some(status) = state.status {
      let visible = !matches!(status, ProgressBarStatus::None);
      lib.unity_launcher_entry_set_progress_visible(entry, if visible { 1 } else { 0 });
    }
  });
}

pub fn set_badge_count(
  _window: &cef::Window,
  count: Option<i64>,
  desktop_filename: Option<String>,
) {
  let Some(desktop_filename) = desktop_filename.as_deref() else {
    return;
  };
  unity::with_entry(desktop_filename, |lib, entry| unsafe {
    match count {
      Some(count) => {
        lib.unity_launcher_entry_set_count(entry, count);
        lib.unity_launcher_entry_set_count_visible(entry, true);
      }
      // Removes the count.
      None => {
        lib.unity_launcher_entry_set_count(entry, 0);
        lib.unity_launcher_entry_set_count_visible(entry, false);
      }
    }
  });
}

/// Unity `LauncherEntry` taskbar integration, loaded dynamically (when present)
/// from `libunity`. Mirrors `tao`'s Linux taskbar implementation, including the
/// `unity_inspector` "is Unity running" gate and library load order.
mod unity {
  use super::*;
  use dlopen2::wrapper::{Container, WrapperApi};

  #[derive(WrapperApi)]
  pub struct UnityLib {
    unity_launcher_entry_get_for_desktop_id:
      unsafe extern "C" fn(id: *const c_char) -> *const isize,
    unity_inspector_get_default: unsafe extern "C" fn() -> *const isize,
    unity_inspector_get_unity_running: unsafe extern "C" fn(inspector: *const isize) -> i32,
    unity_launcher_entry_set_progress: unsafe extern "C" fn(entry: *const isize, value: f64) -> i32,
    unity_launcher_entry_set_progress_visible:
      unsafe extern "C" fn(entry: *const isize, value: i32) -> i32,
    unity_launcher_entry_set_count: unsafe extern "C" fn(entry: *const isize, value: i64) -> i32,
    unity_launcher_entry_set_count_visible:
      unsafe extern "C" fn(entry: *const isize, value: bool) -> bool,
  }

  struct State {
    lib: Option<Container<UnityLib>>,
    inspector: Option<*const isize>,
    /// Cache of `desktop_id -> UnityLauncherEntry*` (owned by libunity).
    entries: HashMap<String, *const isize>,
  }

  thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
  }

  fn load() -> State {
    let lib: Option<Container<UnityLib>> = unsafe {
      Container::load("libunity.so.4")
        .or_else(|_| Container::load("libunity.so.6"))
        .or_else(|_| Container::load("libunity.so.9"))
        .ok()
    };
    let inspector = lib.as_ref().and_then(|lib| {
      let handle = unsafe { lib.unity_inspector_get_default() };
      (!handle.is_null()).then_some(handle)
    });
    State {
      lib,
      inspector,
      entries: HashMap::new(),
    }
  }

  fn is_unity_running(state: &State) -> bool {
    match (state.lib.as_ref(), state.inspector) {
      (Some(lib), Some(inspector)) => unsafe {
        lib.unity_inspector_get_unity_running(inspector) == 1
      },
      _ => false,
    }
  }

  /// Resolves (and caches) the launcher entry for `desktop_filename`, then runs
  /// `f` with it. No-op unless libunity is present and Unity is running.
  pub fn with_entry(desktop_filename: &str, f: impl FnOnce(&UnityLib, *const isize)) {
    STATE.with(|cell| {
      let mut guard = cell.borrow_mut();
      if guard.is_none() {
        *guard = Some(load());
      }
      let state = guard.as_mut().unwrap();

      if !is_unity_running(state) {
        return;
      }

      let entry = match state.entries.get(desktop_filename).copied() {
        Some(entry) => entry,
        None => {
          let Some(lib) = state.lib.as_ref() else {
            return;
          };
          let id = CString::new(desktop_filename).unwrap_or_default();
          let handle = unsafe { lib.unity_launcher_entry_get_for_desktop_id(id.as_ptr()) };
          if handle.is_null() {
            return;
          }
          state.entries.insert(desktop_filename.to_string(), handle);
          handle
        }
      };

      if let Some(lib) = state.lib.as_ref() {
        f(lib, entry);
      }
    });
  }
}

fn moveresize_direction(direction: ResizeDirection) -> c_long {
  match direction {
    ResizeDirection::NorthWest => MOVERESIZE_SIZE_TOPLEFT,
    ResizeDirection::North => MOVERESIZE_SIZE_TOP,
    ResizeDirection::NorthEast => MOVERESIZE_SIZE_TOPRIGHT,
    ResizeDirection::East => MOVERESIZE_SIZE_RIGHT,
    ResizeDirection::SouthEast => MOVERESIZE_SIZE_BOTTOMRIGHT,
    ResizeDirection::South => MOVERESIZE_SIZE_BOTTOM,
    ResizeDirection::SouthWest => MOVERESIZE_SIZE_BOTTOMLEFT,
    ResizeDirection::West => MOVERESIZE_SIZE_LEFT,
  }
}

pub fn start_resize_dragging(window: &cef::Window, direction: ResizeDirection) {
  let xid = window.window_handle() as c_ulong;
  let dir = moveresize_direction(direction);
  with_x11((), |xlib, display| unsafe {
    let root = (xlib.XDefaultRootWindow)(display);
    // Query the current pointer location to seed the move/resize gesture.
    let mut root_return: c_ulong = 0;
    let mut child_return: c_ulong = 0;
    let mut root_x = 0;
    let mut root_y = 0;
    let mut win_x = 0;
    let mut win_y = 0;
    let mut mask: c_uint = 0;
    (xlib.XQueryPointer)(
      display,
      root,
      &mut root_return,
      &mut child_return,
      &mut root_x,
      &mut root_y,
      &mut win_x,
      &mut win_y,
      &mut mask,
    );
    // The window manager needs the pointer to be ungrabbed to take over.
    (xlib.XUngrabPointer)(display, CURRENT_TIME);

    let moveresize = atom(xlib, display, "_NET_WM_MOVERESIZE");
    let mut event: xlib::XEvent = std::mem::zeroed();
    event.client_message = xlib::XClientMessageEvent {
      type_: CLIENT_MESSAGE,
      serial: 0,
      send_event: 1,
      display,
      window: xid,
      message_type: moveresize,
      format: 32,
      data: xlib::ClientMessageData::from([
        root_x as c_long,
        root_y as c_long,
        dir,
        1, // left mouse button
        1, // source: application
      ]),
    };
    (xlib.XSendEvent)(
      display,
      root,
      0,
      SUBSTRUCTURE_REDIRECT_MASK | SUBSTRUCTURE_NOTIFY_MASK,
      &mut event,
    );
  });
}

/// Watches CEF Views toplevel windows for hidden→visible transitions and
/// "repairs" their child browser windows when the toplevel is shown again.
///
/// When a window manager hides a window (workspace switch in i3/sway,
/// iconification, ...), Chromium pauses the toplevel's compositing, and once
/// shown again the child browsers' compositors stay paused, leaving every
/// webview blank until something else (a refocus, a resize) forces a new
/// frame. CEF offers no callback for any of this — it is only observable at
/// the X11 level — so a small watcher thread with its own X connection
/// listens for it.
///
/// Two distinct signals cover the WM landscape:
/// - `MapNotify`: WMs that unmap the client window directly.
/// - `PropertyNotify` for `WM_STATE` / `_NET_WM_STATE`: reparenting WMs like
///   i3 never unmap the client — they hide the frame and only flip
///   `WM_STATE` to Iconic and add `_NET_WM_STATE_HIDDEN` on the client. The
///   watcher tracks the combined hidden state and reacts to the
///   hidden→visible transition.
///
/// On either signal the watcher, for every registered child browser window:
/// 1. raises it above the Views window's own full-size content surface
///    (which Chromium restacks on top of the children when the window is
///    shown), and
/// 2. cycles the `_NET_WM_STATE_HIDDEN` property off/on — the cefclient
///    `SetXWindowVisible` protocol, which CEF's `CefWindowX11` forwards to
///    the inner Chromium widget, pausing and resuming its compositor so a
///    fresh frame is produced.
pub mod map_watcher {
  use super::*;
  use std::os::unix::io::RawFd;
  use std::sync::{Mutex, OnceLock, mpsc};

  enum Control {
    Watch {
      toplevel: c_ulong,
    },
    Unwatch {
      toplevel: c_ulong,
    },
    SetChildren {
      toplevel: c_ulong,
      children: Vec<c_ulong>,
    },
  }

  struct Watcher {
    tx: Mutex<mpsc::Sender<Control>>,
    wake_fd: RawFd,
  }

  static WATCHER: OnceLock<Option<Watcher>> = OnceLock::new();

  /// Starts watching `toplevel` for map events.
  pub fn watch(toplevel: u64) {
    send(Control::Watch {
      toplevel: toplevel as c_ulong,
    });
  }

  /// Stops watching `toplevel` (e.g. when the window is destroyed).
  pub fn unwatch(toplevel: u64) {
    send(Control::Unwatch {
      toplevel: toplevel as c_ulong,
    });
  }

  /// Replaces the set of child browser windows repaired on `MapNotify`.
  pub fn set_children(toplevel: u64, children: Vec<u64>) {
    send(Control::SetChildren {
      toplevel: toplevel as c_ulong,
      children: children.into_iter().map(|c| c as c_ulong).collect(),
    });
  }

  fn send(msg: Control) {
    let Some(watcher) = WATCHER.get_or_init(spawn).as_ref() else {
      return;
    };
    if watcher.tx.lock().unwrap().send(msg).is_ok() {
      // Wake the watcher thread out of poll().
      unsafe {
        let byte = 1u8;
        libc::write(watcher.wake_fd, &byte as *const u8 as *const _, 1);
      }
    }
  }

  fn spawn() -> Option<Watcher> {
    let xlib = xlib::Xlib::open().ok()?;
    let mut fds = [0; 2];
    if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
      return None;
    }
    let (read_fd, wake_fd) = (fds[0], fds[1]);

    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
      .name("cef-x11-map-watcher".into())
      .spawn(move || run(xlib, rx, read_fd))
      .ok()?;

    Some(Watcher {
      tx: Mutex::new(tx),
      wake_fd,
    })
  }

  fn run(xlib: xlib::Xlib, rx: mpsc::Receiver<Control>, wake_rx: RawFd) {
    let display = unsafe { (xlib.XOpenDisplay)(std::ptr::null()) };
    if display.is_null() {
      return;
    }
    let x_fd = unsafe { (xlib.XConnectionNumber)(display) };

    struct Watched {
      children: Vec<c_ulong>,
      hidden: bool,
      /// Rate limiter for `VisibilityNotify`-driven repairs.
      last_visibility_repair: Option<std::time::Instant>,
    }

    let net_wm_state = atom(&xlib, display, "_NET_WM_STATE");
    let net_wm_state_hidden = atom(&xlib, display, "_NET_WM_STATE_HIDDEN");
    let wm_state = atom(&xlib, display, "WM_STATE");

    let mut watched: HashMap<c_ulong, Watched> = HashMap::new();
    // Repairs scheduled for watched toplevels after a show transition.
    // Chromium recreates its presentation surfaces asynchronously, so a
    // single fixed delay races with it — run several passes instead. The
    // cycle is idempotent: extra passes on an already-repaired window are
    // no-ops visually.
    const REPAIR_DELAYS: [std::time::Duration; 4] = [
      std::time::Duration::from_millis(50),
      std::time::Duration::from_millis(250),
      std::time::Duration::from_millis(750),
      std::time::Duration::from_millis(1500),
    ];
    let mut pending_repairs: Vec<(std::time::Instant, c_ulong)> = Vec::new();
    let schedule_repairs = |pending: &mut Vec<(std::time::Instant, c_ulong)>, toplevel: c_ulong| {
      let now = std::time::Instant::now();
      pending.retain(|(_, t)| *t != toplevel);
      pending.extend(REPAIR_DELAYS.iter().map(|d| (now + *d, toplevel)));
    };

    loop {
      // Apply registration changes.
      while let Ok(msg) = rx.try_recv() {
        match msg {
          Control::Watch { toplevel } => {
            unsafe {
              (xlib.XSelectInput)(
                display,
                toplevel,
                xlib::StructureNotifyMask | xlib::PropertyChangeMask | xlib::VisibilityChangeMask,
              );
              (xlib.XFlush)(display);
            }
            let hidden = window_hidden(
              &xlib,
              display,
              toplevel,
              net_wm_state,
              net_wm_state_hidden,
              wm_state,
            );
            log::debug!("cef map watcher: watching 0x{toplevel:x}, hidden={hidden}");
            watched.entry(toplevel).or_insert(Watched {
              children: Vec::new(),
              hidden,
              last_visibility_repair: None,
            });
          }
          Control::Unwatch { toplevel } => {
            watched.remove(&toplevel);
            pending_repairs.retain(|(_, t)| *t != toplevel);
          }
          Control::SetChildren { toplevel, children } => {
            watched
              .entry(toplevel)
              .or_insert(Watched {
                children: Vec::new(),
                hidden: false,
                last_visibility_repair: None,
              })
              .children = children;
          }
        }
      }

      // Drain X events.
      while unsafe { (xlib.XPending)(display) } > 0 {
        let mut event: xlib::XEvent = unsafe { std::mem::zeroed() };
        unsafe { (xlib.XNextEvent)(display, &mut event) };
        match unsafe { event.type_ } {
          // WMs that unmap the client window directly.
          xlib::MapNotify => {
            let map: &xlib::XMapEvent = unsafe { &event.map };
            if let Some(state) = watched.get_mut(&map.window) {
              state.hidden = false;
              schedule_repairs(&mut pending_repairs, map.window);
            }
          }
          // Reparenting WMs (i3, ...) never unmap the client — they only
          // flip WM_STATE to Iconic / add _NET_WM_STATE_HIDDEN while the
          // window's workspace is not visible.
          xlib::PropertyNotify => {
            let property: &xlib::XPropertyEvent = unsafe { &event.property };
            if (property.atom == net_wm_state || property.atom == wm_state)
              && let Some(state) = watched.get_mut(&property.window)
            {
              let hidden = window_hidden(
                &xlib,
                display,
                property.window,
                net_wm_state,
                net_wm_state_hidden,
                wm_state,
              );
              log::trace!(
                "cef map watcher: WM state change on 0x{:x}, hidden {} -> {}",
                property.window,
                state.hidden,
                hidden
              );
              if state.hidden && !hidden {
                schedule_repairs(&mut pending_repairs, property.window);
              }
              state.hidden = hidden;
            }
          }
          // The catch-all: X always generates a VisibilityNotify when a
          // window transitions back to viewable, even when the window
          // manager hid it without unmapping the client or flipping its WM
          // state (i3 workspace switches hide only the frame). Becoming
          // unviewable generates no event, so this cannot track the hidden
          // state — instead, any non-fully-obscured visibility event
          // triggers a (rate-limited) repair. Spurious repairs from plain
          // uncover events are harmless: the resize jiggle is a 1px
          // resize-and-restore.
          xlib::VisibilityNotify => {
            let visibility: &xlib::XVisibilityEvent = unsafe { &event.visibility };
            if visibility.state != xlib::VisibilityFullyObscured
              && let Some(state) = watched.get_mut(&visibility.window)
            {
              let now = std::time::Instant::now();
              let recently_repaired = state
                .last_visibility_repair
                .is_some_and(|last| now.duration_since(last) < std::time::Duration::from_secs(1));
              log::trace!(
                "cef map watcher: visibility change on 0x{:x} (state {}), recently_repaired={recently_repaired}",
                visibility.window,
                visibility.state,
              );
              if !recently_repaired {
                state.last_visibility_repair = Some(now);
                state.hidden = false;
                schedule_repairs(&mut pending_repairs, visibility.window);
              }
            }
          }
          _ => {}
        }
      }

      // Run due repairs.
      let now = std::time::Instant::now();
      pending_repairs.retain(|(deadline, toplevel)| {
        if *deadline <= now {
          if let Some(state) = watched.get(toplevel) {
            repair_children(&state.children);
          }
          false
        } else {
          true
        }
      });

      // Sleep until the X connection or the control pipe has data, or the
      // next scheduled repair is due.
      let timeout = pending_repairs
        .iter()
        .map(|(deadline, _)| *deadline)
        .min()
        .map(|deadline| {
          deadline
            .saturating_duration_since(std::time::Instant::now())
            .as_millis()
            .min(i32::MAX as u128) as i32
            + 1
        })
        .unwrap_or(-1);
      let mut poll_fds = [
        libc::pollfd {
          fd: x_fd,
          events: libc::POLLIN,
          revents: 0,
        },
        libc::pollfd {
          fd: wake_rx,
          events: libc::POLLIN,
          revents: 0,
        },
      ];
      unsafe {
        libc::poll(poll_fds.as_mut_ptr(), 2, timeout);
      }
      if poll_fds[1].revents & libc::POLLIN != 0 {
        let mut buf = [0u8; 32];
        unsafe {
          libc::read(wake_rx, buf.as_mut_ptr() as *mut _, buf.len());
        }
      }
    }
  }

  /// Reports whether the window manager currently considers `window` hidden:
  /// either `_NET_WM_STATE` contains `_NET_WM_STATE_HIDDEN` (EWMH) or
  /// `WM_STATE` is anything other than Normal (ICCCM). The latter covers
  /// i3, which sets `WM_STATE` to Withdrawn — not Iconic — for windows on
  /// invisible workspaces, without adding `_NET_WM_STATE_HIDDEN`.
  fn window_hidden(
    xlib: &xlib::Xlib,
    display: *mut xlib::Display,
    window: c_ulong,
    net_wm_state: c_ulong,
    net_wm_state_hidden: c_ulong,
    wm_state: c_ulong,
  ) -> bool {
    const NORMAL_STATE: c_ulong = 1;

    unsafe fn read_property(
      xlib: &xlib::Xlib,
      display: *mut xlib::Display,
      window: c_ulong,
      property: c_ulong,
    ) -> Option<Vec<c_ulong>> {
      let mut actual_type: c_ulong = 0;
      let mut actual_format: i32 = 0;
      let mut nitems: c_ulong = 0;
      let mut bytes_after: c_ulong = 0;
      let mut prop: *mut u8 = std::ptr::null_mut();
      let status = unsafe {
        (xlib.XGetWindowProperty)(
          display,
          window,
          property,
          0,
          1024,
          xlib::False,
          0, // AnyPropertyType
          &mut actual_type,
          &mut actual_format,
          &mut nitems,
          &mut bytes_after,
          &mut prop,
        )
      };
      if status != 0 || prop.is_null() {
        return None;
      }
      let values = if actual_format == 32 {
        unsafe { std::slice::from_raw_parts(prop as *const c_ulong, nitems as usize).to_vec() }
      } else {
        Vec::new()
      };
      unsafe { (xlib.XFree)(prop as *mut _) };
      Some(values)
    }

    if let Some(atoms) = unsafe { read_property(xlib, display, window, net_wm_state) }
      && atoms.contains(&net_wm_state_hidden)
    {
      return true;
    }

    if let Some(state) = unsafe { read_property(xlib, display, window, wm_state) }
      && let Some(&value) = state.first()
      && value != NORMAL_STATE
    {
      return true;
    }

    false
  }

  fn repair_children(children: &[c_ulong]) {
    log::debug!(
      "cef map watcher: posting repair task for {} webviews",
      children.len()
    );
    // Run the actual repair on the CEF UI thread so it is serialized with
    // Chromium's processing of the show transition's X events; doing the X
    // work from this thread's connection can race ahead of Chromium's
    // visibility handling and intermittently have no effect.
    crate::cef_impl::post_repair_webviews_task(children.to_vec());
  }
}
