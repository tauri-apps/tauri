// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The application: Qt's event loop, the event channel, and app-wide queries.
//!
//! Call [`init`] once on the main thread, install a handler with [`set_event_handler`], then run
//! [`exec`]. On Linux Qt runs on the GLib event dispatcher, so [`exec`] also drives GTK/GLib
//! sources.

use std::{
  cell::{Cell, RefCell},
  collections::BTreeMap,
  ffi::{CStr, c_char, c_int, c_void},
  ptr::NonNull,
  rc::Rc,
  sync::{
    Mutex, PoisonError,
    atomic::{AtomicUsize, Ordering},
  },
};

use qtshim_sys as sys;

use crate::util::{catch_panic, cstring, str_arg};

// ---------------------------------------------------------------------------------------------
// events

/// What an [`Event`] is about. The meaning of the event's fields depends on the kind; it's
/// documented on each variant (`a`–`d` are integers, `s1`/`s2` strings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EventKind {
  /// A window's client area was resized. `a`, `b`: physical width and height.
  WindowResized,
  /// A window moved. `a`, `b`: physical position of its frame.
  WindowMoved,
  /// The user asked to close a window. The close is always blocked; destroy the window to
  /// close it.
  WindowCloseRequested,
  /// A window gained (`a == 1`) or lost (`a == 0`) focus.
  WindowFocus,
  /// A window's scale factor changed. `f`: the new scale; `a`, `b`: physical inner size.
  WindowScale,
  /// The app-wide color scheme changed. `a`: `1` light, `2` dark. `window` is `0`.
  Theme,
  /// A page called `window.ipc.postMessage`. `s1`: the message; `s2`: the page URL.
  ViewIpc,
  /// A page's title changed. `s1`: the title.
  ViewTitle,
  /// A page started (`a == 0`) or finished (`a == 1`) loading. `s1`: the URL.
  ViewLoad,
  /// A main-frame navigation is about to happen. `s1`: the URL. Reply with code `0` to block it.
  ViewNavigate,
  /// A page asked for a new window. `s1`: the URL; `a`–`d`: requested logical geometry (zero if
  /// unset). Reply `0` to deny, `1` to open a plain popup, `2` to open it in the view whose id is
  /// in [`Reply::int`].
  ViewNewWindow,
  /// A page asked for a permission. `s1`: the origin; `a`: the `QWebEnginePermission` type.
  /// Reply `1` to grant, `0` to deny, `-1` to show a prompt.
  ViewPermission,
  /// A download is starting. `s1`: the URL; `s2`: the suggested path. Reply `0` to cancel or `1`
  /// to accept, optionally with a destination path in [`Reply::string`].
  ViewDownload,
  /// A download finished. `s1`: the URL; `s2`: the path; `a`: `1` on success.
  ViewDownloadDone,
  /// A file drag over a view. `a`: `0` enter, `1` over, `2` drop, `3` leave; `b`, `c`: physical
  /// position; `s1`: newline-separated paths. Reply `1` to keep the page from handling it.
  ViewDrag,
  /// A view's renderer process died. `a`: the termination status; `b`: the exit code. Reply `1`
  /// if handled, otherwise the page reloads.
  ViewCrashed,
  /// The event loop is about to wait for new events.
  AboutToBlock,
  /// An event kind this version doesn't know.
  Unknown(i32),
}

impl EventKind {
  fn from_raw(kind: i32) -> Self {
    match kind {
      sys::QS_EV_WINDOW_RESIZED => Self::WindowResized,
      sys::QS_EV_WINDOW_MOVED => Self::WindowMoved,
      sys::QS_EV_WINDOW_CLOSE_REQ => Self::WindowCloseRequested,
      sys::QS_EV_WINDOW_FOCUS => Self::WindowFocus,
      sys::QS_EV_WINDOW_SCALE => Self::WindowScale,
      sys::QS_EV_THEME => Self::Theme,
      sys::QS_EV_VIEW_IPC => Self::ViewIpc,
      sys::QS_EV_VIEW_TITLE => Self::ViewTitle,
      sys::QS_EV_VIEW_LOAD => Self::ViewLoad,
      sys::QS_EV_VIEW_NAVIGATE => Self::ViewNavigate,
      sys::QS_EV_VIEW_NEW_WINDOW => Self::ViewNewWindow,
      sys::QS_EV_VIEW_PERMISSION => Self::ViewPermission,
      sys::QS_EV_VIEW_DOWNLOAD => Self::ViewDownload,
      sys::QS_EV_VIEW_DOWNLOAD_DONE => Self::ViewDownloadDone,
      sys::QS_EV_VIEW_DRAG => Self::ViewDrag,
      sys::QS_EV_VIEW_CRASHED => Self::ViewCrashed,
      sys::QS_EV_ABOUT_TO_BLOCK => Self::AboutToBlock,
      other => Self::Unknown(other),
    }
  }
}

/// An event from Qt. What the fields mean depends on [`EventKind`].
#[derive(Debug)]
pub struct Event<'a> {
  /// What happened.
  pub kind: EventKind,
  /// The id of the window it happened to (`0` for app-wide events).
  pub window: u64,
  /// The id of the view it happened to (`0` for window and app events).
  pub view: u64,
  /// First integer payload.
  pub a: i64,
  /// Second integer payload.
  pub b: i64,
  /// Third integer payload.
  pub c: i64,
  /// Fourth integer payload.
  pub d: i64,
  /// Floating-point payload.
  pub f: f64,
  /// First string payload.
  pub s1: &'a str,
  /// Second string payload.
  pub s2: &'a str,
}

/// The answer to an [`Event`]. The meaning of `code` depends on the event kind; `-1` (the
/// default) means "no opinion, do the default".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
  /// The decision.
  pub code: i32,
  /// An integer payload (e.g. a view id).
  pub int: i64,
  /// A string payload (e.g. a download path).
  pub string: Option<String>,
}

impl Default for Reply {
  fn default() -> Self {
    Self {
      code: -1,
      int: 0,
      string: None,
    }
  }
}

impl Reply {
  /// A reply carrying only a decision code.
  pub fn code(code: i32) -> Self {
    Self {
      code,
      ..Self::default()
    }
  }
}

type EventHandler = Rc<dyn Fn(&Event<'_>) -> Reply>;

thread_local! {
  static EVENT_HANDLER: RefCell<Option<EventHandler>> = const { RefCell::new(None) };
}

unsafe extern "C" fn event_tramp(
  _ud: *mut c_void,
  ev: *const sys::QsEvent,
  reply: *mut sys::QsReply,
) -> c_int {
  let _guard = crate::lifecycle::enter_callback();
  let ev = unsafe { &*ev };
  // Clone the handler out so a re-entrant event (fired while handling this one) works.
  let Some(handler) = EVENT_HANDLER.with(|h| h.borrow().clone()) else {
    return -1;
  };
  let event = Event {
    kind: EventKind::from_raw(ev.kind),
    window: ev.window,
    view: ev.view,
    a: ev.a,
    b: ev.b,
    c: ev.c,
    d: ev.d,
    f: ev.f,
    s1: unsafe { str_arg(ev.s1, ev.s1_len) },
    s2: unsafe { str_arg(ev.s2, ev.s2_len) },
  };
  let Some(r) = catch_panic(&format!("the event handler for {:?}", event.kind), || {
    handler(&event)
  }) else {
    return -1;
  };
  unsafe {
    sys::qs_reply_set_int(reply, r.int);
    if let Some(s) = &r.string {
      sys::qs_reply_set_str(reply, s.as_ptr() as *const c_char, s.len());
    }
  }
  r.code
}

/// Install the handler that receives every window, view and app event, replacing the previous
/// one. Main thread only.
pub fn set_event_handler(f: impl Fn(&Event<'_>) -> Reply + 'static) {
  crate::lifecycle::ensure();
  EVENT_HANDLER.with(|h| *h.borrow_mut() = Some(Rc::new(f)));
  unsafe { sys::qs_set_event_handler(event_tramp, std::ptr::null_mut()) }
}

// ---------------------------------------------------------------------------------------------
// lifecycle

/// Create the `QApplication` on the current thread, which becomes the main thread. `app_id`
/// becomes the application name and the Wayland app id / desktop file name. Calling it again on
/// the main thread does nothing.
///
/// Returns a description such as `"QPAEventDispatcherGlib on wayland"`.
///
/// # Panics
///
/// If another thread is already the main thread.
pub fn init(app_id: Option<&str>) -> String {
  crate::lifecycle::start();
  let id = app_id.map(cstring);
  let description = unsafe {
    CStr::from_ptr(sys::qs_app_init(
      id.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
    ))
  }
  .to_string_lossy()
  .into_owned();
  crate::lifecycle::set_alive();
  description
}

/// Run the Qt (and GLib) event loop until [`quit`] is called; returns the exit code.
pub fn exec() -> i32 {
  crate::lifecycle::ensure();
  unsafe { sys::qs_app_exec() }
}

/// Make [`exec`] return `code`.
pub fn quit(code: i32) {
  crate::lifecycle::ensure();
  unsafe { sys::qs_app_quit(code) }
}

/// Process pending events once without blocking.
pub fn process_events() {
  crate::lifecycle::ensure();
  unsafe { sys::qs_app_process_events() }
}

/// Tear down whatever is left (windows, views, web profiles) and destroy the `QApplication`.
/// Call once after [`exec`] returns and after dropping your own windows and views. Closures
/// still queued by [`dispatch`] or [`post`] are dropped without running. The application can't
/// be created again afterwards.
///
/// # Panics
///
/// If called from inside the event loop or a Qt callback.
pub fn shutdown() {
  crate::lifecycle::ensure();
  // From here on no other thread can hand work to Qt, and Qt objects left on the Rust side are
  // inert: their methods panic and their `Drop` does nothing.
  crate::lifecycle::stop();
  EVENT_HANDLER.with(|h| h.borrow_mut().take());
  // Qt discards the queued calls; drop their closures (and whatever they captured, such as a
  // channel another thread waits on) instead of leaking them.
  let dispatched = std::mem::take(&mut *lock_dispatched());
  let posted = POSTED.with(|p| std::mem::take(&mut *p.borrow_mut()));
  // Their `Drop` may panic (e.g. by using Qt, which is stopped now): finish the shutdown anyway.
  catch_panic("dropping closures queued at shutdown", move || {
    drop((dispatched, posted))
  });
  unsafe { sys::qs_app_shutdown() }
}

/// Whether this is the main thread and the application is running (between [`init`] and
/// [`shutdown`]), i.e. whether the rest of this crate may be used here.
pub fn is_running() -> bool {
  crate::lifecycle::is_running_here()
}

// Queued closures are kept here, not handed to Qt, so `shutdown` can drop the ones Qt never ran.
// Qt only gets their id.

type SendTask = Box<dyn FnOnce() + Send>;
type Task = Box<dyn FnOnce()>;

static DISPATCHED: Mutex<BTreeMap<usize, SendTask>> = Mutex::new(BTreeMap::new());
static NEXT_DISPATCHED: AtomicUsize = AtomicUsize::new(0);

thread_local! {
  static POSTED: RefCell<BTreeMap<usize, Task>> = const { RefCell::new(BTreeMap::new()) };
  static NEXT_POSTED: Cell<usize> = const { Cell::new(0) };
}

fn lock_dispatched() -> std::sync::MutexGuard<'static, BTreeMap<usize, SendTask>> {
  DISPATCHED.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Run `f` on the main thread from the event loop. May be called from any thread.
///
/// Returns `false` (and drops `f`) if the application isn't running.
pub fn dispatch<F: FnOnce() + Send + 'static>(f: F) -> bool {
  unsafe extern "C" fn tramp(id: *mut c_void) {
    let _guard = crate::lifecycle::enter_callback();
    // Take it out before running it: it may dispatch again.
    let task = lock_dispatched().remove(&(id as usize));
    if let Some(task) = task {
      catch_panic("a dispatched closure", task);
    }
  }
  crate::lifecycle::while_alive(move || {
    let id = NEXT_DISPATCHED.fetch_add(1, Ordering::Relaxed);
    lock_dispatched().insert(id, Box::new(f));
    unsafe { sys::qs_dispatch(tramp, id as *mut c_void) }
  })
  .is_some()
}

/// Run `f` from the event loop, on the main thread. Unlike [`dispatch`], `f` need not be `Send`,
/// but this must be called on the main thread.
pub fn post<F: FnOnce() + 'static>(f: F) {
  crate::lifecycle::ensure();
  unsafe extern "C" fn tramp(id: *mut c_void) {
    let _guard = crate::lifecycle::enter_callback();
    let task = POSTED.with(|p| p.borrow_mut().remove(&(id as usize)));
    if let Some(task) = task {
      catch_panic("a posted closure", task);
    }
  }
  let id = NEXT_POSTED.with(|n| n.replace(n.get() + 1));
  POSTED.with(|p| p.borrow_mut().insert(id, Box::new(f)));
  unsafe { sys::qs_post(tramp, id as *mut c_void) }
}

// ---------------------------------------------------------------------------------------------
// app-wide queries

/// Light or dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorScheme {
  /// Light colors.
  Light,
  /// Dark colors.
  Dark,
}

/// The app-wide color scheme, or `None` if Qt doesn't know it.
pub fn color_scheme() -> Option<ColorScheme> {
  crate::lifecycle::ensure();
  match unsafe { sys::qs_app_color_scheme() } {
    1 => Some(ColorScheme::Light),
    2 => Some(ColorScheme::Dark),
    _ => None,
  }
}

/// Force the app-wide color scheme, or follow the system with `None`.
pub fn set_color_scheme(scheme: Option<ColorScheme>) {
  crate::lifecycle::ensure();
  let value = match scheme {
    None => 0,
    Some(ColorScheme::Light) => 1,
    Some(ColorScheme::Dark) => 2,
  };
  unsafe { sys::qs_app_set_color_scheme(value) }
}

/// A screen. Geometry is in logical pixels; multiply by `scale` for physical pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct Screen {
  /// The screen's name (e.g. `"HDMI-A-1"`).
  pub name: String,
  /// Left edge.
  pub x: i32,
  /// Top edge.
  pub y: i32,
  /// Width.
  pub width: i32,
  /// Height.
  pub height: i32,
  /// Left edge of the area not covered by panels.
  pub available_x: i32,
  /// Top edge of the area not covered by panels.
  pub available_y: i32,
  /// Width of the area not covered by panels.
  pub available_width: i32,
  /// Height of the area not covered by panels.
  pub available_height: i32,
  /// Device pixel ratio.
  pub scale: f64,
  /// Whether this is the primary screen.
  pub primary: bool,
}

impl From<&sys::QsScreen> for Screen {
  fn from(s: &sys::QsScreen) -> Self {
    Self {
      name: unsafe { CStr::from_ptr(s.name.as_ptr()) }
        .to_string_lossy()
        .into_owned(),
      x: s.x,
      y: s.y,
      width: s.w,
      height: s.h,
      available_x: s.ax,
      available_y: s.ay,
      available_width: s.aw,
      available_height: s.ah,
      scale: s.scale,
      primary: s.primary != 0,
    }
  }
}

/// All screens.
pub fn screens() -> Vec<Screen> {
  crate::lifecycle::ensure();
  const MAX: usize = 16;
  let mut buf: Vec<sys::QsScreen> = vec![unsafe { std::mem::zeroed() }; MAX];
  let n = unsafe { sys::qs_screens(buf.as_mut_ptr(), MAX as c_int) } as usize;
  buf.truncate(n.min(MAX));
  buf.iter().map(Screen::from).collect()
}

/// The cursor position in logical pixels, with the scale of the screen under it.
pub fn cursor_position() -> Option<(i32, i32, f64)> {
  crate::lifecycle::ensure();
  let (mut x, mut y, mut scale) = (0, 0, 1.0);
  (unsafe { sys::qs_cursor_pos(&mut x, &mut y, &mut scale) } != 0).then_some((x, y, scale))
}

/// A native windowing-system handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeHandle {
  /// A `wl_display*` (for the display) or `wl_surface*` (for a window).
  Wayland(NonNull<c_void>),
  /// A `Display*` (for the display) or an X11 window id (for a window).
  X11(usize),
}

/// The native display connection, if the platform exposes one.
pub fn native_display() -> Option<NativeHandle> {
  crate::lifecycle::ensure();
  let mut kind = 0;
  let mut ptr = std::ptr::null_mut();
  if unsafe { sys::qs_native_display(&mut kind, &mut ptr) } == 0 {
    return None;
  }
  match kind {
    1 => NonNull::new(ptr).map(NativeHandle::Wayland),
    // A null `Display*` (e.g. the offscreen platform) is no display.
    2 if !ptr.is_null() => Some(NativeHandle::X11(ptr as usize)),
    _ => None,
  }
}

/// Update the Unity LauncherEntry (taskbar badge and progress) of a desktop file; `None` hides
/// the badge / progress. Without `desktop_file`, the application's desktop file name is used.
pub fn launcher_update(desktop_file: Option<&str>, count: Option<i64>, progress: Option<f64>) {
  crate::lifecycle::ensure();
  let desktop_file = desktop_file.map(cstring);
  unsafe {
    sys::qs_launcher_update(
      desktop_file
        .as_ref()
        .map_or(std::ptr::null(), |s| s.as_ptr()),
      count.unwrap_or(-1),
      progress.unwrap_or(-1.0),
    )
  }
}

// Unit tests run on libtest worker threads of one process: they never create the
// `QApplication` (see `tests/qt` for that), so they only cover what works without it.
#[cfg(test)]
mod tests {
  use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  };

  use super::*;

  #[test]
  fn event_kinds_from_raw() {
    let known = [
      (sys::QS_EV_WINDOW_RESIZED, EventKind::WindowResized),
      (sys::QS_EV_WINDOW_MOVED, EventKind::WindowMoved),
      (sys::QS_EV_WINDOW_CLOSE_REQ, EventKind::WindowCloseRequested),
      (sys::QS_EV_WINDOW_FOCUS, EventKind::WindowFocus),
      (sys::QS_EV_WINDOW_SCALE, EventKind::WindowScale),
      (sys::QS_EV_THEME, EventKind::Theme),
      (sys::QS_EV_VIEW_IPC, EventKind::ViewIpc),
      (sys::QS_EV_VIEW_TITLE, EventKind::ViewTitle),
      (sys::QS_EV_VIEW_LOAD, EventKind::ViewLoad),
      (sys::QS_EV_VIEW_NAVIGATE, EventKind::ViewNavigate),
      (sys::QS_EV_VIEW_NEW_WINDOW, EventKind::ViewNewWindow),
      (sys::QS_EV_VIEW_PERMISSION, EventKind::ViewPermission),
      (sys::QS_EV_VIEW_DOWNLOAD, EventKind::ViewDownload),
      (sys::QS_EV_VIEW_DOWNLOAD_DONE, EventKind::ViewDownloadDone),
      (sys::QS_EV_VIEW_DRAG, EventKind::ViewDrag),
      (sys::QS_EV_VIEW_CRASHED, EventKind::ViewCrashed),
      (sys::QS_EV_ABOUT_TO_BLOCK, EventKind::AboutToBlock),
    ];
    for (raw, kind) in known {
      assert_eq!(EventKind::from_raw(raw), kind);
    }
    // Every raw value maps to a distinct kind.
    let mut kinds: Vec<_> = known.iter().map(|(_, k)| format!("{k:?}")).collect();
    kinds.sort();
    kinds.dedup();
    assert_eq!(kinds.len(), known.len());
    assert_eq!(EventKind::from_raw(9999), EventKind::Unknown(9999));
    assert_eq!(EventKind::from_raw(-1), EventKind::Unknown(-1));
  }

  #[test]
  fn reply_defaults() {
    let r = Reply::default();
    assert_eq!((r.code, r.int, r.string), (-1, 0, None));
    assert_eq!(
      Reply::code(1),
      Reply {
        code: 1,
        int: 0,
        string: None
      }
    );
  }

  #[test]
  fn screen_from_raw() {
    let mut raw: sys::QsScreen = unsafe { std::mem::zeroed() };
    for (dst, src) in raw.name.iter_mut().zip(b"HDMI-A-1") {
      *dst = *src as c_char;
    }
    (raw.x, raw.y, raw.w, raw.h) = (10, 20, 1920, 1080);
    (raw.ax, raw.ay, raw.aw, raw.ah) = (10, 50, 1920, 1050);
    raw.scale = 1.5;
    raw.primary = 1;
    let s = Screen::from(&raw);
    assert_eq!(
      s,
      Screen {
        name: "HDMI-A-1".into(),
        x: 10,
        y: 20,
        width: 1920,
        height: 1080,
        available_x: 10,
        available_y: 50,
        available_width: 1920,
        available_height: 1050,
        scale: 1.5,
        primary: true,
      }
    );
    raw.primary = 0;
    raw.name = [0; 128];
    let s = Screen::from(&raw);
    assert!(!s.primary);
    assert_eq!(s.name, "");
  }

  #[test]
  fn not_running_on_test_threads() {
    assert!(!is_running());
  }

  struct DropCount(Arc<AtomicUsize>);
  impl Drop for DropCount {
    fn drop(&mut self) {
      self.0.fetch_add(1, Ordering::SeqCst);
    }
  }

  #[test]
  fn dispatch_without_app_drops_the_closure_at_once() {
    let drops = Arc::new(AtomicUsize::new(0));
    let guard = DropCount(drops.clone());
    assert!(!dispatch(move || {
      let _g = guard;
      unreachable!()
    }));
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!(lock_dispatched().is_empty());
  }

  macro_rules! panics_without_app {
    ($($name:ident: $call:expr;)*) => {$(
      #[test]
      #[should_panic(expected = "Qt can only be used on the thread that called `app::init`")]
      fn $name() {
        let _ = $call;
      }
    )*};
  }

  panics_without_app! {
    exec_without_app: exec();
    quit_without_app: quit(0);
    process_events_without_app: process_events();
    shutdown_without_app: shutdown();
    post_without_app: post(|| {});
    set_event_handler_without_app: set_event_handler(|_| Reply::default());
    color_scheme_without_app: color_scheme();
    set_color_scheme_without_app: set_color_scheme(None);
    cursor_position_without_app: cursor_position();
    native_display_without_app: native_display();
    launcher_update_without_app: launcher_update(None, None, None);
  }
}
