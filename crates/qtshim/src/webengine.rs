// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Windows hosting `QWebEngineView`s, web profiles, custom URL schemes and IPC.
//!
//! Sizes and positions passed to and from [`Window`] and [`View`] are Qt *logical* pixels;
//! events carry *physical* pixels.

use std::{
  ffi::{CStr, c_char, c_int, c_void},
  marker::PhantomData,
  ptr::NonNull,
};

use qtshim_sys as sys;

use crate::{
  app::{NativeHandle, Screen},
  util::{boxed, cstring, drop_box, str_arg, take_cstring},
};

fn headers_string(headers: &http::HeaderMap, skip_content_type: bool) -> String {
  let mut out = String::new();
  for (k, v) in headers {
    if skip_content_type && k == http::header::CONTENT_TYPE {
      continue;
    }
    if let Ok(v) = v.to_str() {
      out.push_str(k.as_str());
      out.push_str(": ");
      out.push_str(v);
      out.push('\n');
    }
  }
  out
}

// ---------------------------------------------------------------------------------------------
// engine

/// Flags for [`register_scheme`].
#[derive(Debug, Clone, Copy, Default)]
pub struct SchemeFlags {
  /// Treat the scheme as secure (a secure context, like `https`).
  pub secure: bool,
  /// Allow CORS requests to the scheme.
  pub cors: bool,
  /// Allow `fetch()` to the scheme.
  pub fetch_api: bool,
  /// Let pages on this scheme load local (`file:`) resources.
  pub local_access: bool,
  /// Allow service workers on this scheme.
  pub service_workers: bool,
  /// Ignore the page's Content Security Policy for this scheme.
  pub csp_bypass: bool,
}

impl SchemeFlags {
  /// Defaults for an app-asset / IPC scheme like `tauri://` or `ipc://`.
  pub const APP: Self = Self {
    secure: true,
    cors: true,
    fetch_api: true,
    local_access: true,
    service_workers: false,
    csp_bypass: false,
  };
  fn bits(self) -> u32 {
    (self.secure as u32)
      | (self.cors as u32) << 1
      | (self.fetch_api as u32) << 2
      | (self.local_access as u32) << 3
      | (self.service_workers as u32) << 4
      | (self.csp_bypass as u32) << 5
  }
}

/// Register a custom URL scheme. Returns `false` if the web engine already started (schemes
/// must be known before the first [`Profile`] is created).
pub fn register_scheme(name: &str, flags: SchemeFlags) -> bool {
  crate::lifecycle::ensure();
  unsafe { sys::qs_register_scheme(cstring(name).as_ptr(), flags.bits()) != 0 }
}

/// Append Chromium command-line flags. Only effective before the web engine starts.
pub fn add_chromium_flags(flags: &str) {
  crate::lifecycle::ensure();
  unsafe { sys::qs_engine_add_flags(cstring(flags).as_ptr()) }
}

/// Whether the web engine has started (the first [`Profile`] was created). Schemes, Chromium
/// flags and the proxy can only be set before that.
pub fn engine_started() -> bool {
  crate::lifecycle::ensure();
  unsafe { sys::qs_engine_started() != 0 }
}

/// The engine version, e.g. `"6.11.2 (Chromium 140.0.7339.225)"`.
pub fn engine_version() -> String {
  crate::lifecycle::ensure();
  unsafe { CStr::from_ptr(sys::qs_engine_version()) }
    .to_string_lossy()
    .into_owned()
}

#[derive(Debug, Clone, Copy)]
/// The proxy protocol for [`set_proxy`].
pub enum ProxyKind {
  /// An HTTP proxy.
  Http,
  /// A SOCKS5 proxy.
  Socks5,
}

/// Route all web traffic through a proxy. Only effective before the web engine starts.
pub fn set_proxy(kind: ProxyKind, host: &str, port: u16) {
  crate::lifecycle::ensure();
  let t = match kind {
    ProxyKind::Http => 1,
    ProxyKind::Socks5 => 2,
  };
  unsafe { sys::qs_set_proxy(t, cstring(host).as_ptr(), port) }
}

// ---------------------------------------------------------------------------------------------
// custom schemes

/// Completes a custom-scheme request. `Send`, so it can be answered from another thread.
/// Dropping it unanswered fails the request.
pub struct Responder(Option<*mut sys::QsJob>);
unsafe impl Send for Responder {}

impl Responder {
  /// Send `response`. A `206 Partial Content` response must carry a `Content-Range` header; Qt
  /// can't send other status codes, so error responses with an empty body fail the request and
  /// other bodies are served as `200`.
  pub fn respond<B: AsRef<[u8]>>(mut self, response: http::Response<B>) {
    let job = self.0.take().unwrap();
    let (parts, body) = response.into_parts();
    let body = body.as_ref();
    let content_type = parts
      .headers
      .get(http::header::CONTENT_TYPE)
      .and_then(|v| v.to_str().ok())
      .unwrap_or("text/plain");
    let (range_start, range_total) = if parts.status == http::StatusCode::PARTIAL_CONTENT {
      parts
        .headers
        .get(http::header::CONTENT_RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(parse_content_range)
        .unwrap_or((-1, -1))
    } else {
      (-1, -1)
    };
    let ct = cstring(content_type);
    let h = cstring(&headers_string(&parts.headers, true));
    // Without the application the request is gone too; the job handle is then leaked.
    crate::lifecycle::while_alive(|| unsafe {
      sys::qs_job_respond(
        job,
        parts.status.as_u16() as c_int,
        ct.as_ptr(),
        h.as_ptr(),
        body.as_ptr(),
        body.len(),
        range_start,
        range_total,
      )
    });
  }
}

/// `bytes <start>-<end>/<total>` -> (start, total)
fn parse_content_range(v: &str) -> Option<(i64, i64)> {
  let rest = v.trim().strip_prefix("bytes")?.trim();
  let (range, total) = rest.split_once('/')?;
  let (start, _) = range.split_once('-')?;
  Some((start.trim().parse().ok()?, total.trim().parse().ok()?))
}

impl Drop for Responder {
  fn drop(&mut self) {
    if let Some(job) = self.0.take() {
      crate::lifecycle::while_alive(|| unsafe {
        sys::qs_job_respond(
          job,
          500,
          std::ptr::null(),
          std::ptr::null(),
          std::ptr::null(),
          0,
          -1,
          -1,
        )
      });
    }
  }
}

/// A request to a registered custom scheme. `view` identifies the requesting [`View`].
pub struct SchemeRequest {
  /// The id of the [`View`] that made the request (`0` if unknown, e.g. a service worker).
  pub view: u64,
  /// The request, with the full URL, method, headers and body.
  pub request: http::Request<Vec<u8>>,
}

type SchemeCb = Box<dyn Fn(SchemeRequest, Responder)>;

/// Serve every registered custom scheme. Main thread, after [`crate::app::init`].
pub fn set_scheme_handler(f: impl Fn(SchemeRequest, Responder) + 'static) {
  crate::lifecycle::ensure();
  unsafe extern "C" fn tramp(
    ud: *mut c_void,
    job: *mut sys::QsJob,
    view: u64,
    method: *const c_char,
    url: *const c_char,
    headers: *const c_char,
    body: *const u8,
    body_len: usize,
  ) {
    let _guard = crate::lifecycle::enter_callback();
    let f = unsafe { &*(ud as *const SchemeCb) };
    let cs = |p: *const c_char| unsafe { CStr::from_ptr(p).to_string_lossy().into_owned() };
    let mut req = http::Request::builder()
      .method(cs(method).as_str())
      .uri(cs(url).as_str());
    for line in cs(headers).lines() {
      if let Some((k, v)) = line.split_once(':') {
        req = req.header(k.trim(), v.trim());
      }
    }
    let body = if body.is_null() || body_len == 0 {
      Vec::new()
    } else {
      unsafe { std::slice::from_raw_parts(body, body_len) }.to_vec()
    };
    let responder = Responder(Some(job));
    match req.body(body) {
      Ok(request) => {
        crate::util::catch_panic("the scheme handler", || {
          f(SchemeRequest { view, request }, responder)
        });
      }
      Err(_) => drop(responder),
    }
  }
  let f: SchemeCb = Box::new(f);
  // Intentionally leaked: lives for the whole application.
  unsafe { sys::qs_set_scheme_handler(tramp, boxed(f)) }
}

// ---------------------------------------------------------------------------------------------
// profiles

/// A browsing profile (storage + cookies + cache). Lives until [`crate::app::shutdown`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Profile(NonNull<sys::QsProfile>);

impl Profile {
  /// Get (or create) the profile for a data directory. The first call starts the web engine:
  /// register every custom scheme before it.
  pub fn get(data_directory: Option<&str>, incognito: bool) -> Self {
    crate::lifecycle::ensure();
    let d = data_directory.map(cstring);
    let p = unsafe {
      sys::qs_profile_get(
        d.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
        incognito as c_int,
      )
    };
    Self(NonNull::new(p).expect("wq_profile_get returned null"))
  }
  /// Set the user agent of every view using this profile.
  pub fn set_user_agent(&self, ua: &str) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_profile_set_user_agent(self.0.as_ptr(), cstring(ua).as_ptr()) }
  }
  /// All cookies, in `Set-Cookie` form.
  pub fn cookies(&self) -> Vec<String> {
    crate::lifecycle::ensure();
    unsafe extern "C" fn push(ud: *mut c_void, s: *const c_char, len: usize) {
      let v = unsafe { &mut *(ud as *mut Vec<String>) };
      v.push(unsafe { str_arg(s, len) }.to_string());
    }
    let mut out: Vec<String> = Vec::new();
    unsafe {
      sys::qs_profile_cookies(
        self.0.as_ptr(),
        push,
        &mut out as *mut Vec<String> as *mut c_void,
      )
    };
    out
  }
  /// Add or replace a cookie given in `Set-Cookie` form. `origin` is needed when the cookie has no
  /// `Domain` attribute.
  pub fn set_cookie(&self, set_cookie: &str, origin: Option<&str>) {
    crate::lifecycle::ensure();
    let o = origin.map(cstring);
    unsafe {
      sys::qs_profile_set_cookie(
        self.0.as_ptr(),
        cstring(set_cookie).as_ptr(),
        o.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
      )
    }
  }
  /// Delete a cookie given in `Set-Cookie` form (name, domain and path identify it).
  pub fn delete_cookie(&self, set_cookie: &str, origin: Option<&str>) {
    crate::lifecycle::ensure();
    let o = origin.map(cstring);
    unsafe {
      sys::qs_profile_delete_cookie(
        self.0.as_ptr(),
        cstring(set_cookie).as_ptr(),
        o.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
      )
    }
  }
  /// Clear the HTTP cache, visited links and cookies.
  pub fn clear_data(&self) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_profile_clear_data(self.0.as_ptr()) }
  }
}

// ---------------------------------------------------------------------------------------------
// windows

/// Bits returned by [`Window::state`].
pub mod window_state {
  /// The window is minimized.
  pub const MINIMIZED: u32 = 1;
  /// The window is maximized.
  pub const MAXIMIZED: u32 = 2;
  /// The window is fullscreen.
  pub const FULLSCREEN: u32 = 4;
  /// The window is the active (focused) window.
  pub const ACTIVE: u32 = 8;
  /// The window is visible.
  pub const VISIBLE: u32 = 16;
  /// The window accepts input.
  pub const ENABLED: u32 = 32;
  /// The window has decorations (title bar and borders).
  pub const DECORATED: u32 = 64;
  /// The window stays on top of others.
  pub const ON_TOP: u32 = 128;
  /// The window can be resized.
  pub const RESIZABLE: u32 = 256;
  /// The window has a close button.
  pub const CLOSABLE: u32 = 512;
  /// The window has a minimize button.
  pub const MINIMIZABLE: u32 = 1024;
  /// The window has a maximize button.
  pub const MAXIMIZABLE: u32 = 2048;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A window property toggled with [`Window::set_flag`].
pub enum WindowFlag {
  /// Title bar and borders.
  Decorations = 1,
  /// Keep the window above other windows.
  AlwaysOnTop = 2,
  /// Keep the window below other windows.
  AlwaysOnBottom = 3,
  /// Let the window take keyboard focus.
  Focusable = 4,
  /// Show a close button.
  Closable = 5,
  /// Show a minimize button.
  Minimizable = 6,
  /// Show a maximize button.
  Maximizable = 7,
  /// Let mouse events pass through the window.
  IgnoreCursorEvents = 8,
  /// Hide the window from the taskbar (makes it a Qt tool window; X11 only).
  SkipTaskbar = 9,
}

/// Logical geometry of a window: client area and frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowGeometry {
  /// Client area: `(x, y, width, height)`.
  pub inner: (i32, i32, i32, i32),
  /// Frame including decorations: `(x, y, width, height)`.
  pub outer: (i32, i32, i32, i32),
}

/// A top-level window (a `QWidget`). Destroyed on drop. Not `Send`.
pub struct Window {
  raw: NonNull<sys::QsWindow>,
  _not_send: PhantomData<*const ()>,
}

impl std::fmt::Debug for Window {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Window").field("raw", &self.raw).finish()
  }
}

impl Window {
  /// `id` is echoed back in every event of this window.
  pub fn new(id: u64, transparent: bool) -> Self {
    crate::lifecycle::ensure();
    let raw = unsafe { sys::qs_window_new(id, if transparent { 1 } else { 0 }) };
    Self {
      raw: NonNull::new(raw).unwrap(),
      _not_send: PhantomData,
    }
  }
  fn p(&self) -> *mut sys::QsWindow {
    self.raw.as_ptr()
  }
  /// Set the title.
  pub fn set_title(&self, title: &str) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_set_title(self.p(), cstring(title).as_ptr()) }
  }
  /// The title.
  pub fn title(&self) -> String {
    crate::lifecycle::ensure();
    unsafe { take_cstring(sys::qs_window_title(self.p())) }
  }
  /// Show or hide the window.
  pub fn set_visible(&self, visible: bool) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_set_visible(self.p(), visible as c_int) }
  }
  /// The client-area and frame geometry.
  pub fn geometry(&self) -> WindowGeometry {
    crate::lifecycle::ensure();
    let mut o = [0i32; 8];
    unsafe { sys::qs_window_geometry(self.p(), o.as_mut_ptr()) };
    WindowGeometry {
      inner: (o[0], o[1], o[2], o[3]),
      outer: (o[4], o[5], o[6], o[7]),
    }
  }
  /// The device pixel ratio of the window's screen.
  pub fn scale_factor(&self) -> f64 {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_scale(self.p()) }
  }
  /// The screen the window is on.
  pub fn screen(&self) -> Screen {
    crate::lifecycle::ensure();
    let mut s: sys::QsScreen = unsafe { std::mem::zeroed() };
    unsafe { sys::qs_window_screen(self.p(), &mut s) };
    Screen::from(&s)
  }
  /// Resize the client area.
  pub fn resize(&self, w: i32, h: i32) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_resize(self.p(), w, h) }
  }
  /// Move the window (X11 only: Wayland doesn't let clients position windows).
  pub fn move_to(&self, x: i32, y: i32) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_move(self.p(), x, y) }
  }
  /// Set the minimum client size, or remove it with `None`.
  pub fn set_min_size(&self, size: Option<(i32, i32)>) {
    crate::lifecycle::ensure();
    let (w, h) = size.unwrap_or((-1, -1));
    unsafe { sys::qs_window_set_min_size(self.p(), w, h) }
  }
  /// Set the maximum client size, or remove it with `None`.
  pub fn set_max_size(&self, size: Option<(i32, i32)>) {
    crate::lifecycle::ensure();
    let (w, h) = size.unwrap_or((-1, -1));
    unsafe { sys::qs_window_set_max_size(self.p(), w, h) }
  }
  /// Allow or prevent resizing.
  pub fn set_resizable(&self, resizable: bool) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_set_resizable(self.p(), resizable as c_int) }
  }
  /// Bitmask of [`window_state`] constants.
  pub fn state(&self) -> u32 {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_state(self.p()) }
  }
  /// Minimize or restore the window.
  pub fn set_minimized(&self, on: bool) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_set_state(self.p(), window_state::MINIMIZED, on as c_int) }
  }
  /// Maximize or restore the window.
  pub fn set_maximized(&self, on: bool) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_set_state(self.p(), window_state::MAXIMIZED, on as c_int) }
  }
  /// Enter or leave fullscreen.
  pub fn set_fullscreen(&self, on: bool) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_set_state(self.p(), window_state::FULLSCREEN, on as c_int) }
  }
  /// Turn a [`WindowFlag`] on or off.
  pub fn set_flag(&self, flag: WindowFlag, on: bool) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_set_flag(self.p(), flag as u32, on as c_int) }
  }
  /// Enable or disable input.
  pub fn set_enabled(&self, enabled: bool) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_set_enabled(self.p(), enabled as c_int) }
  }
  /// Raise and focus the window (restoring it if minimized).
  pub fn focus(&self) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_focus(self.p()) }
  }
  /// Center the window on its screen (X11 only).
  pub fn center(&self) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_center(self.p()) }
  }
  /// Set the window icon from `width * height * 4` bytes of RGBA (ignored if the buffer is too
  /// small).
  pub fn set_icon(&self, rgba: &[u8], width: u32, height: u32) {
    crate::lifecycle::ensure();
    if let Some((w, h)) = crate::util::rgba_dims(rgba.len(), width, height) {
      unsafe { sys::qs_window_set_icon(self.p(), rgba.as_ptr(), w, h) }
    }
  }
  /// Set the window's background color (`(r, g, b, a)`), or reset it with `None`.
  pub fn set_background(&self, color: Option<(u8, u8, u8, u8)>) {
    crate::lifecycle::ensure();
    let (r, g, b, a) = color.unwrap_or_default();
    unsafe { sys::qs_window_set_background(self.p(), color.is_some() as c_int, r, g, b, a) }
  }
  /// `shape` is a `Qt::CursorShape` value.
  pub fn set_cursor(&self, shape: i32) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_set_cursor(self.p(), shape) }
  }
  /// Show or hide the cursor over the window.
  pub fn set_cursor_visible(&self, visible: bool) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_set_cursor_visible(self.p(), visible as c_int) }
  }
  /// Grab or release the mouse.
  pub fn set_cursor_grab(&self, grab: bool) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_set_cursor_grab(self.p(), grab as c_int) }
  }
  /// Move the cursor to a position relative to the window (X11 only).
  pub fn set_cursor_position(&self, x: i32, y: i32) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_set_cursor_pos(self.p(), x, y) }
  }
  /// Start an interactive move, as if the title bar were dragged. Returns `false` if the platform
  /// refused.
  pub fn start_move(&self) -> bool {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_start_move(self.p()) != 0 }
  }
  /// `edges`: Qt::Edges bitmask (Top=1, Left=2, Right=4, Bottom=8).
  pub fn start_resize(&self, edges: u32) -> bool {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_start_resize(self.p(), edges) != 0 }
  }
  /// Flash the taskbar entry; `critical` keeps it flashing until the window is focused.
  pub fn alert(&self, critical: bool) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_alert(self.p(), critical as c_int) }
  }
  /// Go fullscreen on the screen containing the logical point `(x, y)`.
  pub fn fullscreen_on(&self, x: i32, y: i32) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_fullscreen_on(self.p(), x, y) }
  }
  /// Keep this window above `parent` and group it with it.
  pub fn set_transient_for(&self, parent: &Window) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_window_set_transient_for(self.p(), parent.p()) }
  }
  /// The window's `QWidget*`, valid while this `Window` exists. Hand it to code that takes a
  /// widget pointer (it can come back in through [`crate::menu::Widget::from_ptr`]).
  pub fn widget_ptr(&self) -> NonNull<c_void> {
    crate::lifecycle::ensure();
    NonNull::new(unsafe { sys::qs_window_qwidget(self.p()) }).expect("window has no QWidget")
  }
  /// The window's `QWidget`, e.g. for [`crate::menu::MenuBar`].
  pub fn widget(&self) -> crate::menu::Widget {
    crate::lifecycle::ensure();
    // SAFETY: the window is alive while `self` exists.
    unsafe { crate::menu::Widget::track(self.widget_ptr()) }
  }
  /// Debugging: move the mouse to logical `(x, y)` in the window through Qt's input path, like a
  /// real pointer, and return the application's override cursor shape (a `Qt::CursorShape`;
  /// `None` without one). `Err(())` if the platform build can't inject input.
  #[doc(hidden)]
  #[allow(clippy::result_unit_err)]
  pub fn debug_hover(&self, x: i32, y: i32) -> Result<Option<i32>, ()> {
    crate::lifecycle::ensure();
    match unsafe { sys::qs_debug_window_hover(self.p(), x, y) } {
      -2 => Err(()),
      -1 => Ok(None),
      shape => Ok(Some(shape)),
    }
  }
  /// Logical size of the area holding the views (excludes a menubar).
  pub fn content_size(&self) -> (i32, i32) {
    crate::lifecycle::ensure();
    let (mut w, mut h) = (0, 0);
    unsafe { sys::qs_window_content_size(self.p(), &mut w, &mut h) };
    (w, h)
  }
  /// Keep this window above the window containing `parent` and group it with it. Does nothing
  /// if `parent` is gone.
  pub fn set_transient_for_widget(&self, parent: &crate::menu::Widget) {
    if let Some(parent) = parent.as_ptr() {
      unsafe { sys::qs_window_set_transient_for_widget(self.p(), parent.as_ptr()) }
    }
  }
  /// The native handle of the window (`wl_surface*` on Wayland, the window id on X11), if
  /// available.
  pub fn native_handle(&self) -> Option<NativeHandle> {
    crate::lifecycle::ensure();
    let mut kind = 0;
    let mut ptr = std::ptr::null_mut();
    if unsafe { sys::qs_window_native(self.p(), &mut kind, &mut ptr) } == 0 {
      return None;
    }
    match kind {
      1 => NonNull::new(ptr).map(NativeHandle::Wayland),
      2 => Some(NativeHandle::X11(ptr as usize)),
      _ => None,
    }
  }
}

impl Drop for Window {
  fn drop(&mut self) {
    // After `app::shutdown` Qt has already destroyed the window.
    if crate::lifecycle::is_running_here() {
      unsafe { sys::qs_window_destroy(self.p()) }
    }
  }
}

// ---------------------------------------------------------------------------------------------
// views

/// Options for [`View::new`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ViewOptions {
  /// Allow opening DevTools.
  pub devtools: bool,
  /// Give the page a transparent background.
  pub transparent: bool,
  /// Track the window's client area.
  pub fill: bool,
  /// Disable JavaScript.
  pub javascript_disabled: bool,
  /// Let JavaScript read and write the clipboard.
  pub clipboard: bool,
  /// Report file drags as [`crate::app::EventKind::ViewDrag`].
  pub drag_handler: bool,
  /// Give the view keyboard focus once created.
  pub focus: bool,
}

impl ViewOptions {
  fn bits(self) -> u32 {
    (self.devtools as u32)
      | (self.transparent as u32) << 1
      | (self.fill as u32) << 2
      | (self.javascript_disabled as u32) << 3
      | (self.clipboard as u32) << 4
      | (self.drag_handler as u32) << 5
      | (self.focus as u32) << 6
  }
}

type StrCb = Box<dyn Fn(&str)>;

unsafe extern "C" fn call_str(ud: *mut c_void, s: *const c_char, len: usize) {
  let _guard = crate::lifecycle::enter_callback();
  let f = unsafe { &*(ud as *const StrCb) };
  let s = unsafe { str_arg(s, len) };
  crate::util::catch_panic("a JavaScript result callback", || f(s));
}

/// A `QWebEngineView` inside a [`Window`]. Destroyed on drop. Not `Send`.
///
/// If its window is dropped first the view becomes inert (all calls are no-ops).
pub struct View {
  raw: NonNull<sys::QsView>,
  _not_send: PhantomData<*const ()>,
}

impl std::fmt::Debug for View {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("View").field("raw", &self.raw).finish()
  }
}

impl View {
  /// `id` is echoed back in every event of this view and identifies its scheme requests.
  pub fn new(id: u64, window: &Window, profile: Profile, options: ViewOptions) -> Self {
    crate::lifecycle::ensure();
    let raw = unsafe { sys::qs_view_new(id, window.p(), profile.0.as_ptr(), options.bits()) };
    Self {
      raw: NonNull::new(raw).unwrap(),
      _not_send: PhantomData,
    }
  }
  fn p(&self) -> *mut sys::QsView {
    self.raw.as_ptr()
  }
  /// Move the view into another window.
  pub fn reparent(&self, window: &Window) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_reparent(self.p(), window.p()) }
  }
  /// Make the view follow the window's content area (or stop following it).
  pub fn set_fill(&self, fill: bool) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_set_fill(self.p(), fill as c_int) }
  }
  /// Logical bounds relative to the window. Disables fill.
  pub fn set_bounds(&self, x: i32, y: i32, w: i32, h: i32) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_set_bounds(self.p(), x, y, w, h) }
  }
  /// Logical bounds relative to the window's content area: `(x, y, width, height)`.
  pub fn bounds(&self) -> (i32, i32, i32, i32) {
    crate::lifecycle::ensure();
    let mut o = [0i32; 4];
    unsafe { sys::qs_view_bounds(self.p(), o.as_mut_ptr()) };
    (o[0], o[1], o[2], o[3])
  }
  /// Show or hide the view.
  pub fn set_visible(&self, visible: bool) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_set_visible(self.p(), visible as c_int) }
  }
  /// Focus the view (and raise its window).
  pub fn focus(&self) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_focus(self.p()) }
  }
  /// Load a URL, optionally with extra request headers.
  pub fn load_url(&self, url: &str, headers: Option<&http::HeaderMap>) {
    crate::lifecycle::ensure();
    let h = headers.map(|h| cstring(&headers_string(h, false)));
    unsafe {
      sys::qs_view_load_url(
        self.p(),
        cstring(url).as_ptr(),
        h.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
      )
    }
  }
  /// Load HTML directly, resolving relative URLs against `base_url`.
  pub fn load_html(&self, html: &str, base_url: Option<&str>) {
    crate::lifecycle::ensure();
    let base = base_url.map(cstring);
    unsafe {
      sys::qs_view_load_html(
        self.p(),
        cstring(html).as_ptr(),
        base.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()),
      )
    }
  }
  /// Reload the page.
  pub fn reload(&self) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_reload(self.p()) }
  }
  /// Go back in history (like the page calling `history.back()`).
  pub fn go_back(&self) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_back(self.p()) }
  }
  /// Go forward in history (like the page calling `history.forward()`).
  pub fn go_forward(&self) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_forward(self.p()) }
  }
  /// Whether there's a history entry to go back to.
  pub fn can_go_back(&self) -> bool {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_can_go_back(self.p()) != 0 }
  }
  /// Whether there's a history entry to go forward to.
  pub fn can_go_forward(&self) -> bool {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_can_go_forward(self.p()) != 0 }
  }
  /// Set the zoom factor (`1.0` is 100%).
  pub fn set_zoom(&self, factor: f64) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_set_zoom(self.p(), factor) }
  }
  /// Set the background color shown before and behind the page.
  pub fn set_background_color(&self, (r, g, b, a): (u8, u8, u8, u8)) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_set_background(self.p(), r, g, b, a) }
  }
  /// Inject `js` at document creation for every subsequent navigation.
  pub fn add_init_script(&self, js: &str, main_frame_only: bool) {
    crate::lifecycle::ensure();
    unsafe {
      sys::qs_view_add_init_script(self.p(), cstring(js).as_ptr(), main_frame_only as c_int)
    }
  }
  /// Evaluate `js` without waiting for the result.
  pub fn eval(&self, js: &str) {
    crate::lifecycle::ensure();
    unsafe {
      sys::qs_view_eval(
        self.p(),
        cstring(js).as_ptr(),
        None,
        std::ptr::null_mut(),
        None,
      )
    }
  }
  /// Evaluate `js`; `callback` receives the JSON-serialised result.
  pub fn eval_with_callback(&self, js: &str, callback: impl Fn(&str) + 'static) {
    crate::lifecycle::ensure();
    let cb: StrCb = Box::new(callback);
    unsafe {
      sys::qs_view_eval(
        self.p(),
        cstring(js).as_ptr(),
        Some(call_str),
        boxed(cb),
        Some(drop_box::<StrCb>),
      )
    }
  }
  /// The current URL.
  pub fn url(&self) -> String {
    crate::lifecycle::ensure();
    unsafe { take_cstring(sys::qs_view_url(self.p())) }
  }
  /// Open DevTools in a separate window.
  pub fn open_devtools(&self) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_open_devtools(self.p()) }
  }
  /// Hide the DevTools window.
  pub fn close_devtools(&self) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_close_devtools(self.p()) }
  }
  /// Whether the DevTools window is open.
  pub fn is_devtools_open(&self) -> bool {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_is_devtools_open(self.p()) != 0 }
  }
  /// Show the print dialog (blocks in a nested event loop).
  pub fn print(&self) {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_print(self.p()) }
  }
  /// The underlying `QWebEngineView*`.
  pub fn qt_view(&self) -> *mut c_void {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_qt_view(self.p()) }
  }
  /// The underlying `QWebEnginePage*`.
  pub fn qt_page(&self) -> *mut c_void {
    crate::lifecycle::ensure();
    unsafe { sys::qs_view_qt_page(self.p()) }
  }
}

impl Drop for View {
  fn drop(&mut self) {
    // After `app::shutdown` Qt has already destroyed the view.
    if crate::lifecycle::is_running_here() {
      unsafe { sys::qs_view_destroy(self.p()) }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{SchemeFlags, headers_string, parse_content_range};

  #[test]
  fn content_range() {
    assert_eq!(parse_content_range("bytes 2-4/10"), Some((2, 10)));
    assert_eq!(parse_content_range("bytes 0-0/*"), None);
  }

  #[test]
  fn content_range_whitespace_and_shapes() {
    assert_eq!(parse_content_range("  bytes 0-99/100 "), Some((0, 100)));
    assert_eq!(parse_content_range("bytes  5 - 9 / 20"), Some((5, 20)));
    assert_eq!(parse_content_range("bytes=5-9/20"), None);
    assert_eq!(parse_content_range("bytes 7-/20"), Some((7, 20)));
    assert_eq!(
      parse_content_range("bytes 0-1/9000000000"),
      Some((0, 9_000_000_000))
    );
  }

  #[test]
  fn content_range_rejects_malformed() {
    for v in [
      "",
      "bytes",
      "items 0-1/2",
      "bytes 0-1",
      "bytes */100",
      "bytes a-1/2",
      "bytes 0-1/x",
      "bytes -1-2/3",
    ] {
      assert_eq!(parse_content_range(v), None, "{v:?}");
    }
  }

  #[test]
  fn headers_string_formats_one_line_per_value() {
    let mut h = http::HeaderMap::new();
    h.insert("x-one", "1".parse().unwrap());
    h.append("x-multi", "a".parse().unwrap());
    h.append("x-multi", "b".parse().unwrap());
    let s = headers_string(&h, false);
    let mut lines: Vec<_> = s.lines().collect();
    lines.sort_unstable();
    assert_eq!(lines, ["x-multi: a", "x-multi: b", "x-one: 1"]);
    assert!(s.ends_with('\n'));
    assert_eq!(headers_string(&http::HeaderMap::new(), false), "");
  }

  #[test]
  fn headers_string_skips_content_type_on_request() {
    let mut h = http::HeaderMap::new();
    h.insert(http::header::CONTENT_TYPE, "text/html".parse().unwrap());
    h.insert("x-a", "1".parse().unwrap());
    assert_eq!(headers_string(&h, true), "x-a: 1\n");
    assert!(headers_string(&h, false).contains("content-type: text/html\n"));
  }

  #[test]
  fn headers_string_skips_non_utf8_values() {
    let mut h = http::HeaderMap::new();
    h.insert("x-bin", http::HeaderValue::from_bytes(b"caf\xe9").unwrap());
    h.insert("x-ok", "fine".parse().unwrap());
    assert_eq!(headers_string(&h, false), "x-ok: fine\n");
  }

  #[test]
  fn scheme_flag_bits() {
    assert_eq!(SchemeFlags::default().bits(), 0);
    assert_eq!(SchemeFlags::APP.bits(), 0b1111);
    let all = SchemeFlags {
      secure: true,
      cors: true,
      fetch_api: true,
      local_access: true,
      service_workers: true,
      csp_bypass: true,
    };
    assert_eq!(all.bits(), 0b11_1111);
    let one = |f: fn(&mut SchemeFlags)| {
      let mut flags = SchemeFlags::default();
      f(&mut flags);
      flags.bits()
    };
    assert_eq!(one(|f| f.secure = true), 1);
    assert_eq!(one(|f| f.cors = true), 2);
    assert_eq!(one(|f| f.fetch_api = true), 4);
    assert_eq!(one(|f| f.local_access = true), 8);
    assert_eq!(one(|f| f.service_workers = true), 16);
    assert_eq!(one(|f| f.csp_bypass = true), 32);
  }
}
