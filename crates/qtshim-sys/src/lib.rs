// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Raw FFI bindings to `qtshim`, a small C++ library over Qt Widgets and QtWebEngine.
//!
//! This crate compiles the C++ shim (`cpp/`) against the system's Qt 6 and declares its C API
//! (`cpp/qtshim.h`). Everything here is `unsafe` and mirrors the header one to one; use the
//! [`qtshim`](https://docs.rs/qtshim) crate for a safe API.
//!
//! # Features
//!
//! - *(always)*: the application, event loop and event channel, screens, menus.
//! - `webengine`: windows hosting `QWebEngineView`s, profiles, custom schemes and IPC.
//!
//! # Build requirements
//!
//! A C++17 compiler, `pkg-config`, and the Qt 6 development files (`Qt6Widgets`, `Qt6DBus`; with
//! `webengine` also `Qt6WebEngineWidgets`, `Qt6WebChannel`, `Qt6PrintSupport`). Set `QT_MOC` to
//! override the location of Qt's `moc` tool.
#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]
#![allow(non_camel_case_types)]
#![no_std]

use core::ffi::{c_char, c_int, c_void};

macro_rules! opaque {
  ($($(#[$m:meta])* $name:ident),* $(,)?) => {$(
    $(#[$m])*
    #[repr(C)]
    pub struct $name {
      _private: [u8; 0],
    }
  )*};
}

opaque!(
  /// Answer to an event, filled by the Rust event handler.
  QsReply,
);
#[cfg(feature = "webengine")]
opaque!(
  /// A top-level window hosting views.
  QsWindow,
  /// A `QWebEngineView` inside a window.
  QsView,
  /// A web profile (storage, cookies, cache).
  QsProfile,
  /// A pending custom-scheme request.
  QsJob,
);

/// An event from Qt; field meanings depend on `kind` (see `QS_EV_*` in `qtshim.h`).
#[repr(C)]
pub struct QsEvent {
  pub kind: i32,
  pub window: u64,
  pub view: u64,
  pub a: i64,
  pub b: i64,
  pub c: i64,
  pub d: i64,
  pub f: f64,
  pub s1: *const c_char,
  pub s1_len: usize,
  pub s2: *const c_char,
  pub s2_len: usize,
}

/// A screen; geometry in logical pixels.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct QsScreen {
  pub name: [c_char; 128],
  pub x: i32,
  pub y: i32,
  pub w: i32,
  pub h: i32,
  pub ax: i32,
  pub ay: i32,
  pub aw: i32,
  pub ah: i32,
  pub scale: f64,
  pub primary: i32,
}

pub type qs_fn = unsafe extern "C" fn(*mut c_void);
pub type qs_drop_fn = unsafe extern "C" fn(*mut c_void);
pub type qs_str_fn = unsafe extern "C" fn(*mut c_void, *const c_char, usize);
pub type qs_event_fn = unsafe extern "C" fn(*mut c_void, *const QsEvent, *mut QsReply) -> c_int;
#[cfg(feature = "webengine")]
pub type qs_scheme_fn = unsafe extern "C" fn(
  *mut c_void,
  *mut QsJob,
  u64,
  *const c_char,
  *const c_char,
  *const c_char,
  *const u8,
  usize,
);

// Event kinds (`QsEvent::kind`).
pub const QS_EV_WINDOW_RESIZED: i32 = 1;
pub const QS_EV_WINDOW_MOVED: i32 = 2;
pub const QS_EV_WINDOW_CLOSE_REQ: i32 = 3;
pub const QS_EV_WINDOW_FOCUS: i32 = 4;
pub const QS_EV_WINDOW_SCALE: i32 = 5;
pub const QS_EV_THEME: i32 = 6;
pub const QS_EV_VIEW_IPC: i32 = 20;
pub const QS_EV_VIEW_TITLE: i32 = 21;
pub const QS_EV_VIEW_LOAD: i32 = 22;
pub const QS_EV_VIEW_NAVIGATE: i32 = 23;
pub const QS_EV_VIEW_NEW_WINDOW: i32 = 24;
pub const QS_EV_VIEW_PERMISSION: i32 = 25;
pub const QS_EV_VIEW_DOWNLOAD: i32 = 26;
pub const QS_EV_VIEW_DOWNLOAD_DONE: i32 = 27;
pub const QS_EV_VIEW_DRAG: i32 = 28;
pub const QS_EV_VIEW_CRASHED: i32 = 29;
pub const QS_EV_ABOUT_TO_BLOCK: i32 = 40;

// Standard actions (`qs_standard_action`).
pub const QS_ACTION_COPY: c_int = 1;
pub const QS_ACTION_CUT: c_int = 2;
pub const QS_ACTION_PASTE: c_int = 3;
pub const QS_ACTION_SELECT_ALL: c_int = 4;
pub const QS_ACTION_UNDO: c_int = 5;
pub const QS_ACTION_REDO: c_int = 6;
pub const QS_ACTION_DELETE: c_int = 7;
pub const QS_ACTION_MINIMIZE: c_int = 8;
pub const QS_ACTION_MAXIMIZE: c_int = 9;
pub const QS_ACTION_FULLSCREEN: c_int = 10;
pub const QS_ACTION_HIDE: c_int = 11;
pub const QS_ACTION_CLOSE_WINDOW: c_int = 12;
pub const QS_ACTION_QUIT: c_int = 13;

// Application, event channel and app-wide queries.
unsafe extern "C" {
  pub fn qs_app_init(app_id: *const c_char) -> *const c_char;
  pub fn qs_set_event_handler(f: qs_event_fn, ud: *mut c_void);
  pub fn qs_reply_set_int(r: *mut QsReply, v: i64);
  pub fn qs_reply_set_str(r: *mut QsReply, s: *const c_char, len: usize);
  pub fn qs_app_exec() -> c_int;
  pub fn qs_app_quit(code: c_int);
  pub fn qs_app_process_events();
  pub fn qs_app_shutdown();
  pub fn qs_dispatch(f: qs_fn, ud: *mut c_void);
  pub fn qs_app_color_scheme() -> c_int;
  pub fn qs_app_set_color_scheme(scheme: c_int);
  pub fn qs_screens(out: *mut QsScreen, max: c_int) -> c_int;
  pub fn qs_cursor_pos(x: *mut i32, y: *mut i32, scale: *mut f64) -> c_int;
  pub fn qs_native_display(kind: *mut c_int, display: *mut *mut c_void) -> c_int;
  pub fn qs_launcher_update(desktop: *const c_char, count: i64, progress: f64);
  pub fn qs_free(p: *mut c_void);
  pub fn qs_post(f: qs_fn, ud: *mut c_void);
}

// Menus.
unsafe extern "C" {
  pub fn qs_widget_find(ptr: *mut c_void) -> *mut c_void;
  pub fn qs_widget_track(widget: *mut c_void) -> *mut c_void;
  pub fn qs_widget_get(tracker: *mut c_void) -> *mut c_void;
  pub fn qs_widget_untrack(tracker: *mut c_void);
  pub fn qs_menubar_for(widget: *mut c_void, create: c_int) -> *mut c_void;
  pub fn qs_menubar_remove(widget: *mut c_void);
  pub fn qs_menubar_set_visible(widget: *mut c_void, visible: c_int);
  pub fn qs_menubar_is_visible(widget: *mut c_void) -> c_int;
  pub fn qs_container_clear(container: *mut c_void);
  pub fn qs_container_add_menu(
    container: *mut c_void,
    text: *const c_char,
    enabled: c_int,
    rgba: *const u8,
    w: c_int,
    h: c_int,
  ) -> *mut c_void;
  pub fn qs_container_add_action(
    container: *mut c_void,
    text: *const c_char,
    enabled: c_int,
    checkable: c_int,
    checked: c_int,
    shortcut: *const c_char,
    shortcut_hint: *const c_char,
    rgba: *const u8,
    w: c_int,
    h: c_int,
    on_click: Option<qs_fn>,
    ud: *mut c_void,
    drop: Option<qs_drop_fn>,
  );
  pub fn qs_container_add_separator(container: *mut c_void);
  pub fn qs_popup_menu_new(widget: *mut c_void) -> *mut c_void;
  pub fn qs_popup_menu_show(
    menu: *mut c_void,
    widget: *mut c_void,
    has_pos: c_int,
    x: f64,
    y: f64,
    physical: c_int,
  );
  pub fn qs_standard_action(kind: c_int);
  pub fn qs_show_about(
    title: *const c_char,
    html: *const c_char,
    rgba: *const u8,
    w: c_int,
    h: c_int,
  );
  pub fn qs_debug_menubar(widget: *mut c_void) -> *mut c_char;
  pub fn qs_debug_trigger(widget: *mut c_void, path: *const c_char) -> c_int;
  pub fn qs_debug_take_popup() -> *mut c_char;
}

// Windows, profiles, views, custom schemes.
#[cfg(feature = "webengine")]
unsafe extern "C" {
  pub fn qs_set_scheme_handler(f: qs_scheme_fn, ud: *mut c_void);
  pub fn qs_register_scheme(name: *const c_char, flags: u32) -> c_int;
  pub fn qs_engine_add_flags(flags: *const c_char);
  pub fn qs_engine_started() -> c_int;
  pub fn qs_engine_version() -> *const c_char;
  pub fn qs_set_proxy(kind: c_int, host: *const c_char, port: u16);

  pub fn qs_window_new(id: u64, flags: u32) -> *mut QsWindow;
  pub fn qs_window_destroy(w: *mut QsWindow);
  pub fn qs_window_set_title(w: *mut QsWindow, t: *const c_char);
  pub fn qs_window_title(w: *mut QsWindow) -> *mut c_char;
  pub fn qs_window_set_visible(w: *mut QsWindow, v: c_int);
  pub fn qs_window_geometry(w: *mut QsWindow, out: *mut i32);
  pub fn qs_window_scale(w: *mut QsWindow) -> f64;
  pub fn qs_window_screen(w: *mut QsWindow, out: *mut QsScreen);
  pub fn qs_window_resize(w: *mut QsWindow, width: i32, height: i32);
  pub fn qs_window_move(w: *mut QsWindow, x: i32, y: i32);
  pub fn qs_window_set_min_size(w: *mut QsWindow, width: i32, height: i32);
  pub fn qs_window_set_max_size(w: *mut QsWindow, width: i32, height: i32);
  pub fn qs_window_set_resizable(w: *mut QsWindow, r: c_int);
  pub fn qs_window_state(w: *mut QsWindow) -> u32;
  pub fn qs_window_set_state(w: *mut QsWindow, which: u32, on: c_int);
  pub fn qs_window_set_flag(w: *mut QsWindow, flag: u32, on: c_int);
  pub fn qs_window_set_enabled(w: *mut QsWindow, e: c_int);
  pub fn qs_window_focus(w: *mut QsWindow);
  pub fn qs_window_center(w: *mut QsWindow);
  pub fn qs_window_set_icon(w: *mut QsWindow, rgba: *const u8, width: i32, height: i32);
  pub fn qs_window_set_background(w: *mut QsWindow, has: c_int, r: u8, g: u8, b: u8, a: u8);
  pub fn qs_window_set_cursor(w: *mut QsWindow, shape: c_int);
  pub fn qs_window_set_cursor_visible(w: *mut QsWindow, v: c_int);
  pub fn qs_window_set_cursor_grab(w: *mut QsWindow, g: c_int);
  pub fn qs_window_set_cursor_pos(w: *mut QsWindow, x: i32, y: i32);
  pub fn qs_window_start_move(w: *mut QsWindow) -> c_int;
  pub fn qs_window_start_resize(w: *mut QsWindow, edges: u32) -> c_int;
  pub fn qs_window_alert(w: *mut QsWindow, critical: c_int);
  pub fn qs_window_fullscreen_on(w: *mut QsWindow, x: i32, y: i32);
  pub fn qs_window_set_transient_for(w: *mut QsWindow, parent: *mut QsWindow);
  pub fn qs_window_native(w: *mut QsWindow, kind: *mut c_int, handle: *mut *mut c_void) -> c_int;
  pub fn qs_window_qwidget(w: *mut QsWindow) -> *mut c_void;
  pub fn qs_window_content_size(w: *mut QsWindow, width: *mut i32, height: *mut i32);
  pub fn qs_debug_window_hover(w: *mut QsWindow, x: i32, y: i32) -> c_int;
  pub fn qs_window_set_transient_for_widget(w: *mut QsWindow, parent: *mut c_void);

  pub fn qs_profile_get(dir: *const c_char, incognito: c_int) -> *mut QsProfile;
  pub fn qs_profile_set_user_agent(p: *mut QsProfile, ua: *const c_char);
  pub fn qs_profile_cookies(p: *mut QsProfile, f: qs_str_fn, ud: *mut c_void);
  pub fn qs_profile_set_cookie(p: *mut QsProfile, c: *const c_char, origin: *const c_char);
  pub fn qs_profile_delete_cookie(p: *mut QsProfile, c: *const c_char, origin: *const c_char);
  pub fn qs_profile_clear_data(p: *mut QsProfile);

  pub fn qs_view_new(
    id: u64,
    parent: *mut QsWindow,
    profile: *mut QsProfile,
    flags: u32,
  ) -> *mut QsView;
  pub fn qs_view_destroy(v: *mut QsView);
  pub fn qs_view_reparent(v: *mut QsView, parent: *mut QsWindow);
  pub fn qs_view_set_fill(v: *mut QsView, fill: c_int);
  pub fn qs_view_set_bounds(v: *mut QsView, x: i32, y: i32, w: i32, h: i32);
  pub fn qs_view_bounds(v: *mut QsView, out: *mut i32);
  pub fn qs_view_set_visible(v: *mut QsView, visible: c_int);
  pub fn qs_view_focus(v: *mut QsView);
  pub fn qs_view_load_url(v: *mut QsView, url: *const c_char, headers: *const c_char);
  pub fn qs_view_load_html(v: *mut QsView, html: *const c_char, base: *const c_char);
  pub fn qs_view_reload(v: *mut QsView);
  pub fn qs_view_back(v: *mut QsView);
  pub fn qs_view_forward(v: *mut QsView);
  pub fn qs_view_can_go_back(v: *mut QsView) -> c_int;
  pub fn qs_view_can_go_forward(v: *mut QsView) -> c_int;
  pub fn qs_view_set_zoom(v: *mut QsView, f: f64);
  pub fn qs_view_set_background(v: *mut QsView, r: u8, g: u8, b: u8, a: u8);
  pub fn qs_view_add_init_script(v: *mut QsView, js: *const c_char, main_only: c_int);
  pub fn qs_view_set_user_agent(v: *mut QsView, ua: *const c_char);
  pub fn qs_view_eval(
    v: *mut QsView,
    js: *const c_char,
    cb: Option<qs_str_fn>,
    ud: *mut c_void,
    drop: Option<qs_drop_fn>,
  );
  pub fn qs_view_url(v: *mut QsView) -> *mut c_char;
  pub fn qs_view_open_devtools(v: *mut QsView);
  pub fn qs_view_close_devtools(v: *mut QsView);
  pub fn qs_view_is_devtools_open(v: *mut QsView) -> c_int;
  pub fn qs_view_print(v: *mut QsView);
  pub fn qs_view_qt_view(v: *mut QsView) -> *mut c_void;
  pub fn qs_view_qt_page(v: *mut QsView) -> *mut c_void;

  pub fn qs_job_respond(
    job: *mut QsJob,
    status: c_int,
    content_type: *const c_char,
    headers: *const c_char,
    data: *const u8,
    len: usize,
    range_start: i64,
    range_total: i64,
  );
}
