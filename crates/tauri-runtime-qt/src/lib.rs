// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A [Tauri](https://tauri.app) runtime backed by Qt Widgets and QtWebEngine (Chromium).
//!
//! It replaces `tauri-runtime-wry` (WebKitGTK on Linux) with a Chromium engine that the
//! distribution already ships, so WebRTC, screen sharing, codecs and WebGL behave like Chrome,
//! and windows are native Wayland (or X11) windows owned by Qt.
//!
//! ```ignore
//! tauri::Builder::default()
//!   .runtime(tauri_runtime_qt::Qt::default())
//!   .run(tauri::generate_context!())
//!   .expect("error while running tauri application");
//! ```
//!
//! The runtime doesn't use GTK. Qt runs on the GLib event loop, so GTK-based pieces an app adds
//! (e.g. `rfd` dialogs) still get their events. Window menus and context menus are Qt menus:
//! the runtime enables `tauri`'s `qt` feature, which drives muda's Qt backend through
//! `WindowDispatch::qt_window`. `gtk_window`/`default_vbox` return an error.
//!
//! None of the exposed API of this crate is stable, and it may break semver
//! compatibility in the future. The major version only signifies the intended Tauri version.

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]

use std::{
  any::Any,
  cell::{Cell, RefCell},
  collections::{BTreeMap, HashMap, VecDeque},
  fmt,
  marker::PhantomData,
  rc::Rc,
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU32, Ordering},
    mpsc::{channel, sync_channel},
  },
  thread::ThreadId,
};

use qtshim::{
  app::{self, Event, EventKind, Reply},
  webengine::{self, Profile, SchemeFlags, View, ViewOptions},
};
use raw_window_handle::{DisplayHandle, RawDisplayHandle};
use tauri_runtime::{
  DeviceEventFilter, Error, EventLoopProxy, ExitRequestedEventAction, Result, RunEvent, Runtime,
  RuntimeHandle, RuntimeInitArgs, UserEvent,
  dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize},
  monitor::Monitor,
  webview::{
    DetachedWebview, DocumentTitleChangedHandler, DownloadEvent, DownloadHandler,
    NavigationHandler, NewWindowFeatures, NewWindowHandler, NewWindowResponse, OnPageLoadHandler,
    OnWebContentProcessTerminateHandler, PageLoadEvent, PendingWebview, PermissionKind,
    PermissionResponse, UriSchemeProtocolHandler, WebContentProcessTermination,
    WebContentProcessTerminationReason, WebviewIpcHandler,
  },
  window::{
    DetachedWindow, DetachedWindowWebview, DragDropEvent, PendingWindow, RawWindow, WebviewEvent,
    WindowEvent, WindowId,
  },
};
use tauri_utils::Theme;
use url::Url;

mod dispatch;
mod util;
mod window_builder;

pub use dispatch::{QtWebviewDispatcher, QtWindowDispatcher};

/// Debugging helpers for the Qt menubar of a window. Main thread only.
///
/// Turn the pointer from `tauri::Window::qt_window` into a [`Widget`](debug::Widget) with
/// [`Widget::from_ptr`](debug::Widget::from_ptr).
pub mod debug {
  pub use qtshim::menu::{
    Widget,
    debug::{menubar_tree, take_popup_menu, trigger_menu_item},
  };
}
pub use window_builder::QtWindowBuilder;

/// Selects and configures the Qt runtime. Pass it to `tauri::Builder::runtime`.
///
/// ```ignore
/// tauri::Builder::default().runtime(tauri_runtime_qt::Qt::default().chromium_flags("--ignore-gpu-blocklist"));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Qt {
  /// Extra Chromium command-line flags, appended to `QTWEBENGINE_CHROMIUM_FLAGS`.
  pub chromium_flags: Vec<String>,
  /// Expose the Chromium DevTools protocol on this port (`--remote-debugging-port`).
  pub remote_debugging_port: Option<u16>,
}

impl Qt {
  /// Append Chromium command-line flags (space separated).
  #[must_use]
  pub fn chromium_flags(mut self, flags: impl Into<String>) -> Self {
    self.chromium_flags.push(flags.into());
    self
  }

  /// Expose the Chromium DevTools protocol on `port`.
  #[must_use]
  pub fn remote_debugging_port(mut self, port: u16) -> Self {
    self.remote_debugging_port = Some(port);
    self
  }
}

impl<T: UserEvent> tauri_runtime::RuntimeInitAttrs<T> for Qt {
  type Runtime = QtRuntime<T>;
}

impl<T: UserEvent> From<Qt> for tauri_runtime::dynamic::DynRuntimeInitAttrs<T> {
  fn from(attrs: Qt) -> Self {
    Self::new(attrs)
  }
}

/// Qt-specific webview attributes (`Runtime::RuntimeWebviewAttributes`).
#[derive(Debug, Clone, Default)]
pub struct QtWebviewAttributes {}

/// The webview that asked for a new window; pass it to the new webview's builder
/// (`WebviewWindowBuilder::window_features`) so the popup is connected to its opener.
#[derive(Debug, Clone)]
pub struct QtWindowOpener {
  /// The opener's webview id.
  pub webview_id: u32,
  /// The URL the opener asked to open.
  pub url: String,
}

/// Schemes Tauri uses on Linux. `RuntimeInitArgs::custom_schemes` lists every scheme the app
/// registers; these are added in case an older `tauri` leaves some out (QtWebEngine needs every
/// scheme before the engine starts).
const DEFAULT_SCHEMES: &[&str] = &["tauri", "ipc", "asset", "isolation"];

/// Runs the app's event callback.
type RunCallback<T> = Box<dyn FnMut(RunEvent<T>)>;

// ---------------------------------------------------------------------------------------------
// shared (thread-safe) context

pub(crate) struct Shared {
  main_thread: ThreadId,
  alive: AtomicBool,
  next_window_id: AtomicU32,
  next_webview_id: AtomicU32,
  /// Unity launcher state: badge count, progress, desktop file.
  launcher: Mutex<(Option<i64>, Option<f64>, Option<String>)>,
  app_id: Option<String>,
}

/// Thread-safe handle to the runtime internals shared by every dispatcher.
pub struct Context<T: UserEvent> {
  pub(crate) shared: Arc<Shared>,
  _marker: PhantomData<fn() -> T>,
}

impl<T: UserEvent> Clone for Context<T> {
  fn clone(&self) -> Self {
    Self {
      shared: self.shared.clone(),
      _marker: PhantomData,
    }
  }
}

impl<T: UserEvent> fmt::Debug for Context<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Context")
      .field("main_thread", &self.shared.main_thread)
      .finish_non_exhaustive()
  }
}

impl<T: UserEvent> Context<T> {
  fn is_main_thread(&self) -> bool {
    std::thread::current().id() == self.shared.main_thread
  }

  /// Run `f` on the main thread and wait for its result. Runs inline on the main thread.
  pub(crate) fn run_on_main<R, F>(&self, f: F) -> Result<R>
  where
    R: Send + 'static,
    F: FnOnce(&MainState<T>) -> R + Send + 'static,
  {
    if self.is_main_thread() {
      let st = state::<T>().ok_or(Error::EventLoopClosed)?;
      return Ok(f(&st));
    }
    if !self.shared.alive.load(Ordering::Acquire) {
      return Err(Error::EventLoopClosed);
    }
    let (tx, rx) = sync_channel(1);
    let queued = app::dispatch(move || {
      if let Some(st) = state::<T>() {
        let _ = tx.send(f(&st));
      }
    });
    if !queued {
      return Err(Error::EventLoopClosed);
    }
    // If the app shuts down before running `f`, it's dropped with `tx` and this fails.
    rx.recv().map_err(|_| Error::FailedToReceiveMessage)
  }

  /// Run `f` now on the main thread, or queue it from other threads without waiting. Like wry,
  /// so a setter followed by a getter in the same main-thread call observes the change.
  pub(crate) fn run_or_post<F>(&self, f: F) -> Result<()>
  where
    F: FnOnce(&MainState<T>) + Send + 'static,
  {
    if self.is_main_thread() {
      let st = state::<T>().ok_or(Error::EventLoopClosed)?;
      f(&st);
      Ok(())
    } else {
      self.post(f)
    }
  }

  /// Queue `f` on the main loop without waiting (also when called from the main thread).
  pub(crate) fn post<F>(&self, f: F) -> Result<()>
  where
    F: FnOnce(&MainState<T>) + Send + 'static,
  {
    if !self.shared.alive.load(Ordering::Acquire) {
      return Err(Error::EventLoopClosed);
    }
    let queued = app::dispatch(move || {
      if let Some(st) = state::<T>() {
        f(&st);
      }
    });
    if queued {
      Ok(())
    } else {
      Err(Error::EventLoopClosed)
    }
  }

  pub(crate) fn update_launcher(
    &self,
    count: Option<Option<i64>>,
    progress: Option<Option<f64>>,
    desktop: Option<String>,
  ) {
    let mut l = self.shared.launcher.lock().unwrap();
    if let Some(c) = count {
      l.0 = c;
    }
    if let Some(p) = progress {
      l.1 = p;
    }
    if desktop.is_some() {
      l.2 = desktop;
    }
    let (count, progress) = (l.0, l.1);
    let desktop = l.2.clone().or_else(|| self.shared.app_id.clone());
    drop(l);
    let _ = self.post(move |_| {
      app::launcher_update(desktop.as_deref(), count, progress);
    });
  }
}

// ---------------------------------------------------------------------------------------------
// main-thread state

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WebviewKind {
  /// The webview filling a window created with `PendingWindow::webview`.
  WindowContent,
  /// A child webview added with `create_webview`.
  WindowChild,
}

/// Position/size of a child webview relative to its window, for `auto_resize`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Rates {
  x: f64,
  y: f64,
  width: f64,
  height: f64,
}

pub(crate) struct WebviewEntry {
  pub id: u32,
  pub kind: WebviewKind,
  pub profile: Profile,
  pub auto_resize: Cell<Option<Rates>>,
  pub view: View,
}

pub(crate) struct WindowEntry {
  pub label: String,
  /// Declared before `window` so views are destroyed before their window.
  pub webviews: Vec<WebviewEntry>,
  pub window: webengine::Window,
}

impl WindowEntry {
  pub(crate) fn webview(&self, id: u32) -> Option<&WebviewEntry> {
    self.webviews.iter().find(|w| w.id == id)
  }
}

/// Per-webview callbacks from `PendingWebview`. Only used on the main thread.
pub(crate) struct WebviewMeta<T: UserEvent> {
  label: String,
  window_id: Arc<Mutex<WindowId>>,
  ipc_handler: Option<WebviewIpcHandler<T, QtRuntime<T>>>,
  navigation_handler: Option<Box<NavigationHandler>>,
  new_window_handler: Option<Box<NewWindowHandler<T, QtRuntime<T>>>>,
  title_handler: Option<Box<DocumentTitleChangedHandler>>,
  page_load_handler: Option<Box<OnPageLoadHandler>>,
  download_handler: Option<Arc<DownloadHandler>>,
  permission_handler: Option<Box<dyn Fn(PermissionKind) -> PermissionResponse + Send + Sync>>,
  terminate_handler: Option<Box<OnWebContentProcessTerminateHandler>>,
}

pub(crate) struct MainState<T: UserEvent> {
  pub(crate) context: Context<T>,
  pub(crate) windows: RefCell<BTreeMap<WindowId, WindowEntry>>,
  /// Qt window id -> Tauri window id.
  window_ids: RefCell<HashMap<u64, WindowId>>,
  webviews: RefCell<HashMap<u32, Rc<WebviewMeta<T>>>>,
  protocols: RefCell<HashMap<u32, HashMap<String, Arc<UriSchemeProtocolHandler>>>>,
  callback: RefCell<Option<RunCallback<T>>>,
  queue: RefCell<VecDeque<RunEvent<T>>>,
}

thread_local! {
  static STATE: RefCell<Option<Rc<dyn Any>>> = const { RefCell::new(None) };
}

pub(crate) fn state<T: UserEvent>() -> Option<Rc<MainState<T>>> {
  STATE
    .with(|s| s.borrow().clone())?
    .downcast::<MainState<T>>()
    .ok()
}

impl<T: UserEvent> MainState<T> {
  fn callback_busy(&self) -> bool {
    self.callback.try_borrow_mut().is_err()
  }

  /// Deliver a run event to the app callback. Re-entrant calls are queued and delivered
  /// once the outer callback returns.
  pub(crate) fn emit(&self, event: RunEvent<T>) {
    self.queue.borrow_mut().push_back(event);
    self.flush();
  }

  /// Deliver queued events in order, unless the callback is already running (the outer
  /// invocation will pick them up) or the loop hasn't started (kept until `run`).
  fn flush(&self) {
    loop {
      let Ok(mut cb) = self.callback.try_borrow_mut() else {
        return;
      };
      let Some(cb) = cb.as_mut() else {
        return;
      };
      let next = self.queue.borrow_mut().pop_front();
      match next {
        Some(ev) => cb(ev),
        None => return,
      }
    }
  }

  pub(crate) fn window_id(&self, qt_id: u64) -> Option<WindowId> {
    self.window_ids.borrow().get(&qt_id).copied()
  }

  fn window_label(&self, id: WindowId) -> Option<String> {
    self.windows.borrow().get(&id).map(|w| w.label.clone())
  }

  fn meta(&self, webview_id: u32) -> Option<Rc<WebviewMeta<T>>> {
    self.webviews.borrow().get(&webview_id).cloned()
  }

  fn emit_window_event(&self, id: WindowId, event: WindowEvent) {
    let Some(label) = self.window_label(id) else {
      return;
    };
    self.emit(RunEvent::WindowEvent { label, event });
  }

  fn emit_webview_event(&self, webview_id: u32, event: WebviewEvent) {
    let Some(meta) = self.meta(webview_id) else {
      return;
    };
    self.emit(RunEvent::WebviewEvent {
      label: meta.label.clone(),
      event,
    });
  }

  fn dispatcher_for(&self, webview_id: u32, meta: &WebviewMeta<T>) -> QtWebviewDispatcher<T> {
    QtWebviewDispatcher {
      window_id: meta.window_id.clone(),
      webview_id,
      context: self.context.clone(),
    }
  }
}

// ---------------------------------------------------------------------------------------------
// lifecycle: close / destroy / exit

pub(crate) fn close_requested<T: UserEvent>(st: &MainState<T>, id: WindowId) {
  if st.callback_busy() {
    let _ = st.context.post(move |st| close_requested(st, id));
    return;
  }
  if st.window_label(id).is_none() {
    return;
  }
  let (tx, rx) = channel();
  st.emit_window_event(id, WindowEvent::CloseRequested { signal_tx: tx });
  if !matches!(rx.try_recv(), Ok(true)) {
    destroy_window(st, id);
  }
}

pub(crate) fn destroy_window<T: UserEvent>(st: &MainState<T>, id: WindowId) {
  let Ok(mut windows) = st.windows.try_borrow_mut() else {
    let _ = st.context.post(move |st| destroy_window(st, id));
    return;
  };
  let Some(entry) = windows.remove(&id) else {
    return;
  };
  let now_empty = windows.is_empty();
  drop(windows);
  st.window_ids.borrow_mut().retain(|_, v| *v != id);

  let webview_ids: Vec<u32> = entry.webviews.iter().map(|w| w.id).collect();
  for wid in &webview_ids {
    st.webviews.borrow_mut().remove(wid);
    st.protocols.borrow_mut().remove(wid);
  }
  let label = entry.label.clone();
  drop(entry); // destroys the Qt window and its views

  st.emit(RunEvent::WindowEvent {
    label,
    event: WindowEvent::Destroyed,
  });

  if now_empty {
    exit_requested(st, None);
  }
}

pub(crate) fn exit_requested<T: UserEvent>(st: &MainState<T>, code: Option<i32>) {
  if st.callback_busy() {
    let _ = st.context.post(move |st| exit_requested(st, code));
    return;
  }
  let (tx, rx) = channel();
  st.emit(RunEvent::ExitRequested { code, tx });
  if !matches!(rx.try_recv(), Ok(ExitRequestedEventAction::Prevent)) {
    app::quit(code.unwrap_or(0));
  }
}

// ---------------------------------------------------------------------------------------------
// window / webview creation (main thread)

pub(crate) fn create_window<T: UserEvent>(
  st: &MainState<T>,
  pending: PendingWindow<T, QtRuntime<T>>,
) -> Result<DetachedWindow<T, QtRuntime<T>>> {
  let PendingWindow {
    label,
    window_builder: b,
    webview,
  } = pending;

  let raw_id = st
    .context
    .shared
    .next_window_id
    .fetch_add(1, Ordering::Relaxed);
  let id = WindowId::from(raw_id);
  let window = webengine::Window::new(raw_id as u64, b.transparent);
  window.set_title(&b.title);

  let scale = window.scale_factor();
  let screen = window.screen();
  let c = &b.constraints;
  let to_l = |u: Option<tauri_runtime::dpi::PixelUnit>| u.map(|u| util::unit_to_logical(u, scale));
  if let (Some(w), Some(h)) = (to_l(c.min_width), to_l(c.min_height)) {
    window.set_min_size(Some((util::round(w), util::round(h))));
  } else if c.min_width.is_some() || c.min_height.is_some() {
    window.set_min_size(Some((
      to_l(c.min_width).map_or(0, util::round),
      to_l(c.min_height).map_or(0, util::round),
    )));
  }
  if c.max_width.is_some() || c.max_height.is_some() {
    window.set_max_size(Some((
      to_l(c.max_width).map_or(i32::MAX / 2, util::round),
      to_l(c.max_height).map_or(i32::MAX / 2, util::round),
    )));
  }

  let (mut w, mut h) = b.inner_size.unwrap_or((800.0, 600.0));
  let margin = match &b.prevent_overflow {
    window_builder::PreventOverflow::Off => None,
    window_builder::PreventOverflow::On => Some((0.0, 0.0)),
    window_builder::PreventOverflow::Margin(m) => {
      let m: LogicalSize<f64> = m.to_logical(scale);
      Some((m.width, m.height))
    }
  };
  if let Some((mw, mh)) = margin {
    w = w.min(screen.available_width as f64 - mw);
    h = h.min(screen.available_height as f64 - mh);
  }
  window.resize(util::round(w), util::round(h));
  window.set_resizable(b.resizable);

  use qtshim::webengine::WindowFlag;
  if !b.decorations {
    window.set_flag(WindowFlag::Decorations, false);
  }
  if b.always_on_top {
    window.set_flag(WindowFlag::AlwaysOnTop, true);
  }
  if b.always_on_bottom {
    window.set_flag(WindowFlag::AlwaysOnBottom, true);
  }
  if !b.focusable {
    window.set_flag(WindowFlag::Focusable, false);
  }
  if b.skip_taskbar {
    window.set_flag(WindowFlag::SkipTaskbar, true);
  }
  if !b.closable {
    window.set_flag(WindowFlag::Closable, false);
  }
  if !b.minimizable {
    window.set_flag(WindowFlag::Minimizable, false);
  }
  if !b.maximizable {
    window.set_flag(WindowFlag::Maximizable, false);
  }
  if let Some(icon) = &b.icon {
    window.set_icon(&icon.rgba, icon.width, icon.height);
  }
  // `tauri` takes `transient_parent` from the parent window's `qt_window()`; it's validated
  // here anyway.
  if let Some(parent) = b
    .transient_parent
    .and_then(|p| qtshim::menu::Widget::from_ptr(p as *mut std::ffi::c_void))
  {
    window.set_transient_for_widget(&parent);
  }
  if let Some(c) = b.background_color {
    window.set_background(Some(c.into()));
  }
  if let Some(theme) = b.theme {
    apply_theme(Some(theme));
  }
  if let Some((x, y)) = b.position {
    window.move_to(util::round(x), util::round(y));
  } else if b.center {
    window.center();
  }

  st.windows.borrow_mut().insert(
    id,
    WindowEntry {
      label: label.clone(),
      webviews: Vec::new(),
      window,
    },
  );
  st.window_ids.borrow_mut().insert(raw_id as u64, id);

  let mut detached_webview = None;
  if let Some(pending_webview) = webview {
    let use_https_scheme = pending_webview.webview_attributes.use_https_scheme;
    let devtools = pending_webview.webview_attributes.devtools;
    match create_webview(st, id, WebviewKind::WindowContent, pending_webview) {
      Ok(webview) => {
        detached_webview = Some(DetachedWindowWebview {
          webview,
          use_https_scheme,
          devtools,
        })
      }
      Err(e) => {
        st.windows.borrow_mut().remove(&id);
        st.window_ids.borrow_mut().remove(&(raw_id as u64));
        return Err(e);
      }
    }
  }

  if let Some(entry) = st.windows.borrow().get(&id) {
    let window = &entry.window;
    if b.maximized {
      window.set_maximized(true);
    }
    if b.fullscreen {
      window.set_fullscreen(true);
    }
    if b.visible {
      window.set_visible(true);
      if b.center && b.position.is_none() {
        window.center();
      }
      if b.focused {
        window.focus();
      }
    }
  }

  Ok(DetachedWindow {
    id,
    label,
    dispatcher: QtWindowDispatcher {
      window_id: id,
      context: st.context.clone(),
    },
    webview: detached_webview,
  })
}

pub(crate) fn create_webview<T: UserEvent>(
  st: &MainState<T>,
  window_id: WindowId,
  kind: WebviewKind,
  pending: PendingWebview<T, QtRuntime<T>>,
) -> Result<DetachedWebview<T, QtRuntime<T>>> {
  let PendingWebview {
    label,
    webview_attributes: attrs,
    uri_scheme_protocols,
    ipc_handler,
    navigation_handler,
    new_window_handler,
    document_title_changed_handler,
    url,
    on_page_load_handler,
    download_handler,
    permission_request_handler,
    on_web_content_process_terminate_handler,
    opener,
    ..
  } = pending;

  for scheme in uri_scheme_protocols.keys() {
    if !webengine::register_scheme(scheme, SchemeFlags::APP) {
      log::warn!(
        "custom scheme `{scheme}` was registered after the web engine started; QtWebEngine \
         will not route it (register it on the first webview)"
      );
    }
  }
  if !webengine::engine_started() {
    if let Some(args) = &attrs.additional_browser_args {
      webengine::add_chromium_flags(args);
    }
    if let Some(proxy) = &attrs.proxy_url {
      let kind = match proxy.scheme() {
        "http" => Some(webengine::ProxyKind::Http),
        "socks5" => Some(webengine::ProxyKind::Socks5),
        _ => None,
      };
      match (kind, proxy.host_str()) {
        (Some(kind), Some(host)) => {
          webengine::set_proxy(kind, host, proxy.port_or_known_default().unwrap_or(80))
        }
        _ => return Err(Error::InvalidProxyUrl),
      }
    }
  } else if attrs.additional_browser_args.is_some() || attrs.proxy_url.is_some() {
    log::warn!(
      "additional_browser_args / proxy_url only apply to the first webview with the Qt runtime"
    );
  }

  let profile = Profile::get(
    attrs.data_directory.as_deref().and_then(|p| p.to_str()),
    attrs.incognito,
  );
  if let Some(ua) = &attrs.user_agent {
    profile.set_user_agent(ua);
  }

  let webview_id = st
    .context
    .shared
    .next_webview_id
    .fetch_add(1, Ordering::Relaxed);
  let fill = kind == WebviewKind::WindowContent || attrs.bounds.is_none();

  let (view, auto_resize) = {
    let windows = st.windows.borrow();
    let entry = windows.get(&window_id).ok_or(Error::WindowNotFound)?;
    let view = View::new(
      webview_id as u64,
      &entry.window,
      profile,
      ViewOptions {
        devtools: attrs.devtools.unwrap_or(true),
        transparent: attrs.transparent,
        fill,
        javascript_disabled: attrs.javascript_disabled,
        clipboard: attrs.clipboard,
        drag_handler: attrs.drag_drop_handler_enabled,
        focus: attrs.focus,
      },
    );
    let mut auto_resize = None;
    if let (false, Some(bounds)) = (fill, attrs.bounds) {
      let scale = entry.window.scale_factor();
      let (x, y) = util::position_to_logical(bounds.position, scale);
      let (w, h) = util::size_to_logical(bounds.size, scale);
      view.set_bounds(x, y, w, h);
      if attrs.auto_resize {
        let (ww, wh) = entry.window.content_size();
        auto_resize = rates(x, y, w, h, ww, wh);
      }
    }
    (view, auto_resize)
  };

  if let Some(color) = attrs.background_color {
    view.set_background_color(color.into());
  }
  for script in &attrs.initialization_scripts {
    view.add_init_script(&script.script, script.for_main_frame_only);
  }
  // QtWebEngine binds no DevTools shortcut of its own, so (like wry) inject Tauri's toggle.
  #[cfg(any(debug_assertions, feature = "devtools"))]
  if attrs.devtools.unwrap_or(true) {
    view.add_init_script(&tauri_runtime::webview::devtools_shortcut_script(), true);
  }

  let window_id_cell = Arc::new(Mutex::new(window_id));
  st.webviews.borrow_mut().insert(
    webview_id,
    Rc::new(WebviewMeta {
      label: label.clone(),
      window_id: window_id_cell.clone(),
      ipc_handler,
      navigation_handler,
      new_window_handler,
      title_handler: document_title_changed_handler,
      page_load_handler: on_page_load_handler,
      download_handler,
      permission_handler: permission_request_handler,
      terminate_handler: on_web_content_process_terminate_handler,
    }),
  );
  st.protocols.borrow_mut().insert(
    webview_id,
    uri_scheme_protocols
      .into_iter()
      .map(|(k, v)| (k, Arc::<UriSchemeProtocolHandler>::from(v)))
      .collect(),
  );

  log::debug!("webview {webview_id} ({label}) loading {url}");
  // A popup created for an opener gets its content through `QWebEngineNewWindowRequest::openIn`
  // when the new-window handler returns `Create`.
  if let Some(opener) = &opener {
    log::debug!(
      "webview {webview_id} ({label}) opened by webview {}",
      opener.webview_id
    );
  } else if url != "about:blank" {
    view.load_url(&url, None);
  }

  {
    let mut windows = st.windows.borrow_mut();
    let entry = windows.get_mut(&window_id).ok_or(Error::WindowNotFound)?;
    entry.webviews.push(WebviewEntry {
      id: webview_id,
      kind,
      profile,
      auto_resize: Cell::new(auto_resize),
      view,
    });
  }

  Ok(DetachedWebview {
    label,
    dispatcher: QtWebviewDispatcher {
      window_id: window_id_cell,
      webview_id,
      context: st.context.clone(),
    },
  })
}

pub(crate) fn rates(x: i32, y: i32, w: i32, h: i32, ww: i32, wh: i32) -> Option<Rates> {
  if ww <= 0 || wh <= 0 {
    return None;
  }
  Some(Rates {
    x: x as f64 / ww as f64,
    y: y as f64 / wh as f64,
    width: w as f64 / ww as f64,
    height: h as f64 / wh as f64,
  })
}

pub(crate) fn color_scheme(theme: Theme) -> app::ColorScheme {
  match theme {
    Theme::Dark => app::ColorScheme::Dark,
    _ => app::ColorScheme::Light,
  }
}

thread_local! {
  /// The theme the app asked for with `set_theme`, if any.
  static THEME_OVERRIDE: std::cell::Cell<Option<Theme>> = const { std::cell::Cell::new(None) };
}

/// Ask Qt for `theme` (`None`: follow the system) and remember it: Qt's color scheme is only a
/// request, which platform themes may ignore (offscreen, some Wayland desktops), and `theme()`
/// must still report what the app set.
pub(crate) fn apply_theme(theme: Option<Theme>) {
  THEME_OVERRIDE.with(|t| t.set(theme));
  app::set_color_scheme(theme.map(color_scheme));
}

pub(crate) fn current_theme() -> Theme {
  if let Some(theme) = THEME_OVERRIDE.with(std::cell::Cell::get) {
    return theme;
  }
  match app::color_scheme() {
    Some(app::ColorScheme::Dark) => Theme::Dark,
    _ => Theme::Light,
  }
}

// ---------------------------------------------------------------------------------------------
// Qt event routing

fn handle_event<T: UserEvent>(ev: &Event) -> Reply {
  let Some(st) = state::<T>() else {
    return Reply::default();
  };
  match ev.kind {
    EventKind::WindowResized
    | EventKind::WindowMoved
    | EventKind::WindowFocus
    | EventKind::WindowScale
    | EventKind::Theme => {
      let owned = WindowEv {
        kind: ev.kind,
        window: ev.window,
        a: ev.a,
        b: ev.b,
        f: ev.f,
      };
      // Fired synchronously from inside a Qt call we made while holding the windows map:
      // handle it once that call has returned.
      if st.windows.try_borrow_mut().is_err() {
        let _ = st.context.post(move |st| handle_window_event(st, owned));
      } else {
        handle_window_event(&st, owned);
      }
      Reply::default()
    }
    EventKind::WindowCloseRequested => {
      if let Some(id) = st.window_id(ev.window) {
        close_requested(&st, id);
      }
      Reply::default()
    }
    EventKind::AboutToBlock => {
      if !st.callback_busy() {
        st.flush();
        st.emit(RunEvent::MainEventsCleared);
      }
      Reply::default()
    }
    _ => handle_view_event(&st, ev),
  }
}

#[derive(Clone, Copy)]
struct WindowEv {
  kind: EventKind,
  window: u64,
  a: i64,
  b: i64,
  f: f64,
}

fn handle_window_event<T: UserEvent>(st: &MainState<T>, ev: WindowEv) {
  if ev.kind == EventKind::Theme {
    let theme = if ev.a == 2 { Theme::Dark } else { Theme::Light };
    let ids: Vec<WindowId> = st.windows.borrow().keys().copied().collect();
    for id in ids {
      st.emit_window_event(id, WindowEvent::ThemeChanged(theme));
    }
    return;
  }
  let Some(id) = st.window_id(ev.window) else {
    return;
  };
  let event = match ev.kind {
    EventKind::WindowResized => {
      apply_auto_resize(st, id);
      WindowEvent::Resized(PhysicalSize::new(ev.a.max(0) as u32, ev.b.max(0) as u32))
    }
    EventKind::WindowMoved => WindowEvent::Moved(PhysicalPosition::new(ev.a as i32, ev.b as i32)),
    EventKind::WindowFocus => WindowEvent::Focused(ev.a != 0),
    EventKind::WindowScale => WindowEvent::ScaleFactorChanged {
      scale_factor: ev.f,
      new_inner_size: PhysicalSize::new(ev.a.max(0) as u32, ev.b.max(0) as u32),
    },
    _ => return,
  };
  st.emit_window_event(id, event);
}

fn apply_auto_resize<T: UserEvent>(st: &MainState<T>, id: WindowId) {
  let windows = st.windows.borrow();
  let Some(entry) = windows.get(&id) else {
    return;
  };
  let (ww, wh) = entry.window.content_size();
  for wv in &entry.webviews {
    if let Some(r) = wv.auto_resize.get() {
      wv.view.set_bounds(
        util::round(r.x * ww as f64),
        util::round(r.y * wh as f64),
        util::round(r.width * ww as f64),
        util::round(r.height * wh as f64),
      );
    }
  }
}

fn handle_view_event<T: UserEvent>(st: &MainState<T>, ev: &Event) -> Reply {
  let webview_id = ev.view as u32;
  log::trace!(
    "view event {:?} webview={webview_id} s1={:.120}",
    ev.kind,
    ev.s1
  );
  let Some(meta) = st.meta(webview_id) else {
    return Reply::default();
  };
  match ev.kind {
    EventKind::ViewIpc => {
      if let Some(handler) = &meta.ipc_handler {
        let uri = if ev.s2.is_empty() {
          "about:blank"
        } else {
          ev.s2
        };
        if let Ok(request) = http::Request::builder().uri(uri).body(ev.s1.to_string()) {
          handler(
            DetachedWebview {
              label: meta.label.clone(),
              dispatcher: st.dispatcher_for(webview_id, &meta),
            },
            request,
          );
        }
      }
      Reply::default()
    }
    EventKind::ViewTitle => {
      if let Some(h) = &meta.title_handler {
        h(ev.s1.to_string());
      }
      Reply::default()
    }
    EventKind::ViewLoad => {
      if let (Some(h), Ok(url)) = (&meta.page_load_handler, Url::parse(ev.s1)) {
        h(
          url,
          if ev.a == 0 {
            PageLoadEvent::Started
          } else {
            PageLoadEvent::Finished
          },
        );
      }
      Reply::default()
    }
    EventKind::ViewNavigate => match (&meta.navigation_handler, Url::parse(ev.s1)) {
      (Some(h), Ok(url)) => Reply::code(h(&url) as i32),
      _ => Reply::default(),
    },
    EventKind::ViewNewWindow => {
      let (Some(h), Ok(url)) = (&meta.new_window_handler, Url::parse(ev.s1)) else {
        return Reply::default();
      };
      let size = (ev.c > 0 && ev.d > 0).then(|| LogicalSize::new(ev.c as f64, ev.d as f64));
      let position = (ev.c > 0 && ev.d > 0).then(|| LogicalPosition::new(ev.a as f64, ev.b as f64));
      let features = NewWindowFeatures::new(
        size,
        position,
        QtWindowOpener {
          webview_id,
          url: ev.s1.to_string(),
        },
      );
      match h(url, features) {
        NewWindowResponse::Allow => Reply::code(1),
        NewWindowResponse::Deny => Reply::code(0),
        NewWindowResponse::Create { window_id } => {
          let target = st
            .windows
            .borrow()
            .get(&window_id)
            .and_then(|w| w.webviews.first().map(|v| v.id));
          match target {
            Some(id) => Reply {
              code: 2,
              int: id as i64,
              string: None,
            },
            None => Reply::code(0),
          }
        }
      }
    }
    EventKind::ViewPermission => {
      let Some(h) = &meta.permission_handler else {
        return Reply::default();
      };
      let kinds: &[PermissionKind] = match ev.a {
        1 => &[PermissionKind::Microphone],
        2 => &[PermissionKind::Camera],
        3 => &[PermissionKind::Camera, PermissionKind::Microphone],
        4 | 5 => &[PermissionKind::DisplayCapture],
        6 => &[PermissionKind::PointerLock],
        7 => &[PermissionKind::Notifications],
        8 => &[PermissionKind::Geolocation],
        9 => &[PermissionKind::ClipboardRead],
        10 => &[PermissionKind::LocalFonts],
        _ => &[PermissionKind::Other],
      };
      let mut result = PermissionResponse::Allow;
      for k in kinds {
        match h(*k) {
          PermissionResponse::Deny => return Reply::code(0),
          PermissionResponse::Default => result = PermissionResponse::Default,
          PermissionResponse::Allow => {}
        }
      }
      Reply::code(match result {
        PermissionResponse::Allow => 1,
        PermissionResponse::Deny => 0,
        PermissionResponse::Default => -1,
      })
    }
    EventKind::ViewDownload => {
      let Some(h) = &meta.download_handler else {
        return Reply::code(1);
      };
      let Ok(url) = Url::parse(ev.s1) else {
        return Reply::code(1);
      };
      let mut destination = std::path::PathBuf::from(ev.s2);
      let allow = h(DownloadEvent::Requested {
        url,
        destination: &mut destination,
      });
      Reply {
        code: allow as i32,
        int: 0,
        string: Some(destination.to_string_lossy().into_owned()),
      }
    }
    EventKind::ViewDownloadDone => {
      if let (Some(h), Ok(url)) = (&meta.download_handler, Url::parse(ev.s1)) {
        h(DownloadEvent::Finished {
          url,
          path: (!ev.s2.is_empty()).then(|| ev.s2.into()),
          success: ev.a != 0,
        });
      }
      Reply::default()
    }
    EventKind::ViewCrashed => {
      let Some(h) = &meta.terminate_handler else {
        return Reply::default();
      };
      let reason = match ev.a {
        0 => WebContentProcessTerminationReason::Normal,
        1 => WebContentProcessTerminationReason::Abnormal,
        2 => WebContentProcessTerminationReason::Crashed,
        3 => WebContentProcessTerminationReason::Killed,
        _ => WebContentProcessTerminationReason::Unknown,
      };
      h(WebContentProcessTermination {
        reason,
        error_code: Some(ev.b as i32),
        error_string: None,
      });
      Reply::code(1)
    }
    EventKind::ViewDrag => {
      let position = PhysicalPosition::new(ev.b as f64, ev.c as f64);
      let paths = || -> Vec<std::path::PathBuf> {
        ev.s1
          .split('\n')
          .filter(|p| !p.is_empty())
          .map(Into::into)
          .collect()
      };
      let event = match ev.a {
        0 => DragDropEvent::Enter {
          paths: paths(),
          position,
        },
        1 => DragDropEvent::Over { position },
        2 => DragDropEvent::Drop {
          paths: paths(),
          position,
        },
        _ => DragDropEvent::Leave,
      };
      let window_id = *meta.window_id.lock().unwrap();
      let is_content = st
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|w| {
          w.webview(webview_id)
            .map(|v| v.kind == WebviewKind::WindowContent)
        })
        .unwrap_or(false);
      if is_content {
        st.emit_window_event(window_id, WindowEvent::DragDrop(event));
      } else {
        st.emit_webview_event(webview_id, WebviewEvent::DragDrop(event));
      }
      // Like wry: the runtime consumes file drops, the page doesn't see them.
      Reply::code(1)
    }
    _ => Reply::default(),
  }
}

fn handle_scheme<T: UserEvent>(req: webengine::SchemeRequest, responder: webengine::Responder) {
  let Some(st) = state::<T>() else {
    return;
  };
  let scheme = req
    .request
    .uri()
    .scheme_str()
    .unwrap_or_default()
    .to_string();
  let webview_id = req.view as u32;
  let found = {
    let protocols = st.protocols.borrow();
    protocols
      .get(&webview_id)
      .and_then(|m| m.get(&scheme))
      .map(|h| (webview_id, h.clone()))
      // Requests without a known webview (service workers, devtools): use any handler.
      .or_else(|| {
        protocols
          .iter()
          .find_map(|(id, m)| m.get(&scheme).map(|h| (*id, h.clone())))
      })
  };
  log::debug!(
    "scheme request {} {} from webview {webview_id} -> handler: {}",
    req.request.method(),
    req.request.uri(),
    found.is_some()
  );
  let Some((owner, handler)) = found else {
    drop(responder);
    return;
  };
  let label = st.meta(owner).map(|m| m.label.clone()).unwrap_or_default();
  drop(st);
  let uri = req.request.uri().to_string();
  handler(
    &label,
    req.request,
    Box::new(move |response| {
      log::trace!(
        "scheme response {uri} -> {} {:?} {:.200}",
        response.status(),
        response.headers(),
        String::from_utf8_lossy(response.body())
      );
      responder.respond(response)
    }),
  );
}

// ---------------------------------------------------------------------------------------------
// Runtime

/// The Qt runtime. Selected with [`Qt`]; `T` is Tauri's event loop message type.
pub struct QtRuntime<T: UserEvent> {
  context: Context<T>,
  _not_send: PhantomData<*const ()>,
}

impl<T: UserEvent> fmt::Debug for QtRuntime<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("QtRuntime")
      .field("context", &self.context)
      .finish()
  }
}

/// A [`Send`] handle to the runtime.
pub struct QtHandle<T: UserEvent> {
  context: Context<T>,
}

impl<T: UserEvent> Clone for QtHandle<T> {
  fn clone(&self) -> Self {
    Self {
      context: self.context.clone(),
    }
  }
}

impl<T: UserEvent> fmt::Debug for QtHandle<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("QtHandle").finish_non_exhaustive()
  }
}

/// Sends user events to the main loop.
pub struct QtEventProxy<T: UserEvent> {
  context: Context<T>,
}

impl<T: UserEvent> Clone for QtEventProxy<T> {
  fn clone(&self) -> Self {
    Self {
      context: self.context.clone(),
    }
  }
}

impl<T: UserEvent> fmt::Debug for QtEventProxy<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("QtEventProxy").finish_non_exhaustive()
  }
}

impl<T: UserEvent> EventLoopProxy<T> for QtEventProxy<T> {
  fn send_event(&self, event: T) -> Result<()> {
    self
      .context
      .post(move |st| st.emit(RunEvent::UserEvent(event)))
  }
}

/// Information about the platform webview passed to `with_webview`.
#[derive(Debug, Clone, Copy)]
pub struct QtWebview {
  /// `QWebEngineView*`
  pub view: *mut std::ffi::c_void,
  /// `QWebEnginePage*`
  pub page: *mut std::ffi::c_void,
}

impl<T: UserEvent> QtRuntime<T> {
  fn init(args: RuntimeInitArgs<Qt>) -> Result<Self> {
    let RuntimeInitArgs {
      app_id,
      identifier,
      custom_schemes,
      runtime_init_attrs: attrs,
    } = args;
    let app_id = app_id.or_else(|| (!identifier.is_empty()).then_some(identifier));
    let info = app::init(app_id.as_deref());
    log::debug!("Qt runtime: {info}");
    if !info.contains("Glib") {
      log::warn!(
        "Qt is not using the GLib event dispatcher ({info}); GTK-based plugins won't receive events"
      );
    }
    for scheme in DEFAULT_SCHEMES
      .iter()
      .copied()
      .chain(custom_schemes.iter().map(String::as_str))
    {
      webengine::register_scheme(scheme, SchemeFlags::APP);
    }
    for flags in &attrs.chromium_flags {
      webengine::add_chromium_flags(flags);
    }
    if let Some(port) = attrs.remote_debugging_port {
      webengine::add_chromium_flags(&format!("--remote-debugging-port={port}"));
    }

    let context = Context {
      shared: Arc::new(Shared {
        main_thread: std::thread::current().id(),
        alive: AtomicBool::new(true),
        next_window_id: AtomicU32::new(1),
        next_webview_id: AtomicU32::new(1),
        launcher: Default::default(),
        app_id,
      }),
      _marker: PhantomData,
    };
    let st: Rc<MainState<T>> = Rc::new(MainState {
      context: context.clone(),
      windows: Default::default(),
      window_ids: Default::default(),
      webviews: Default::default(),
      protocols: Default::default(),
      callback: RefCell::new(None),
      queue: Default::default(),
    });
    STATE.with(|s| *s.borrow_mut() = Some(st as Rc<dyn Any>));
    app::set_event_handler(handle_event::<T>);
    webengine::set_scheme_handler(handle_scheme::<T>);

    Ok(Self {
      context,
      _not_send: PhantomData,
    })
  }

  fn teardown(&self) {
    if let Some(st) = state::<T>() {
      let windows = std::mem::take(&mut *st.windows.borrow_mut());
      drop(windows);
      st.webviews.borrow_mut().clear();
      st.protocols.borrow_mut().clear();
      st.queue.borrow_mut().clear();
      let cb = st.callback.borrow_mut().take();
      drop(cb);
    }
    self.context.shared.alive.store(false, Ordering::Release);
    app::shutdown();
    STATE.with(|s| s.borrow_mut().take());
  }
}

pub(crate) fn display_handle_raw() -> Option<RawDisplayHandle> {
  use raw_window_handle::{WaylandDisplayHandle, XlibDisplayHandle};
  match app::native_display()? {
    app::NativeHandle::Wayland(p) => Some(WaylandDisplayHandle::new(p).into()),
    app::NativeHandle::X11(d) => {
      Some(XlibDisplayHandle::new(std::ptr::NonNull::new(d as *mut std::ffi::c_void), 0).into())
    }
  }
}

/// `RawDisplayHandle` isn't `Send`; carry it across threads as plain integers.
#[derive(Clone, Copy)]
pub(crate) struct SendHandle(pub(crate) Option<(u8, usize)>);

pub(crate) fn display_handle_send() -> SendHandle {
  SendHandle(app::native_display().map(|h| match h {
    app::NativeHandle::Wayland(p) => (1, p.as_ptr() as usize),
    app::NativeHandle::X11(d) => (2, d),
  }))
}

pub(crate) fn display_from_send(h: SendHandle) -> Option<RawDisplayHandle> {
  use raw_window_handle::{WaylandDisplayHandle, XlibDisplayHandle};
  let (kind, v) = h.0?;
  let p = std::ptr::NonNull::new(v as *mut std::ffi::c_void);
  match kind {
    1 => p.map(|p| WaylandDisplayHandle::new(p).into()),
    _ => Some(XlibDisplayHandle::new(p, 0).into()),
  }
}

impl<T: UserEvent> RuntimeHandle<T> for QtHandle<T> {
  type Runtime = QtRuntime<T>;

  fn create_proxy(&self) -> QtEventProxy<T> {
    QtEventProxy {
      context: self.context.clone(),
    }
  }

  fn request_exit(&self, code: i32) -> Result<()> {
    self.context.post(move |st| exit_requested(st, Some(code)))
  }

  fn create_window<F: Fn(RawWindow) + Send + 'static>(
    &self,
    pending: PendingWindow<T, QtRuntime<T>>,
    _after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, QtRuntime<T>>> {
    let pending = SendWrapper(pending);
    self
      .context
      .run_on_main(move |st| create_window(st, pending.into_inner()))?
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, QtRuntime<T>>,
  ) -> Result<DetachedWebview<T, QtRuntime<T>>> {
    let pending = SendWrapper(pending);
    self.context.run_on_main(move |st| {
      create_webview(
        st,
        window_id,
        WebviewKind::WindowChild,
        pending.into_inner(),
      )
    })?
  }

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    if self.context.is_main_thread() {
      f();
      Ok(())
    } else {
      self.context.post(move |_| f())
    }
  }

  fn display_handle(
    &self,
  ) -> std::result::Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
    let h = self
      .context
      .run_on_main(|_| display_handle_send())
      .map_err(|_| raw_window_handle::HandleError::Unavailable)?;
    display_from_send(h)
      .map(|raw| unsafe { DisplayHandle::borrow_raw(raw) })
      .ok_or(raw_window_handle::HandleError::Unavailable)
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    self.context.run_on_main(|_| primary_monitor())
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Result<Option<Monitor>> {
    self
      .context
      .run_on_main(move |_| util::screen_at_physical(x, y).map(|s| util::monitor_from_screen(&s)))
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    self.context.run_on_main(|_| available_monitors())
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    self.context.run_on_main(|_| cursor_position())?
  }

  fn set_theme(&self, theme: Option<Theme>) {
    let _ = self.context.post(move |_| apply_theme(theme));
  }

  fn set_device_event_filter(&self, _filter: DeviceEventFilter) {}

  /// Custom schemes are served as `<scheme>://localhost` (QtWebEngine registers them natively).
  fn custom_scheme_url(&self, scheme: &str, _https: bool) -> String {
    format!("{scheme}://localhost")
  }

  fn webview_version(&self) -> Result<String> {
    Ok(webengine::engine_version())
  }
}

pub(crate) fn primary_monitor() -> Option<Monitor> {
  app::screens()
    .iter()
    .find(|s| s.primary)
    .map(util::monitor_from_screen)
}

pub(crate) fn available_monitors() -> Vec<Monitor> {
  app::screens()
    .iter()
    .map(util::monitor_from_screen)
    .collect()
}

pub(crate) fn cursor_position() -> Result<PhysicalPosition<f64>> {
  let (x, y, s) = app::cursor_position().ok_or(Error::FailedToGetCursorPosition)?;
  Ok(PhysicalPosition::new(x as f64 * s, y as f64 * s))
}

/// Moves a non-`Send` value onto the main thread. Only sound because the value is created on
/// the caller's thread and exclusively used on the main thread afterwards; the pending window
/// and webview types are `!Send` only because they may contain platform objects that are
/// unused on Linux-with-Qt.
pub(crate) struct SendWrapper<V>(V);
unsafe impl<V> Send for SendWrapper<V> {}
impl<V> SendWrapper<V> {
  pub(crate) fn into_inner(self) -> V {
    self.0
  }
}

impl<T: UserEvent> Runtime<T> for QtRuntime<T> {
  type WindowDispatcher = QtWindowDispatcher<T>;
  type WebviewDispatcher = QtWebviewDispatcher<T>;
  type Handle = QtHandle<T>;
  type EventLoopProxy = QtEventProxy<T>;
  type RuntimeWebviewAttributes = QtWebviewAttributes;
  type Webview = QtWebview;
  type RuntimeInitAttrs = Qt;
  type WindowOpener = QtWindowOpener;

  fn new(args: RuntimeInitArgs<Qt>) -> Result<Self> {
    Self::init(args)
  }

  fn new_any_thread(args: RuntimeInitArgs<Qt>) -> Result<Self> {
    Self::init(args)
  }

  fn create_proxy(&self) -> QtEventProxy<T> {
    QtEventProxy {
      context: self.context.clone(),
    }
  }

  fn handle(&self) -> QtHandle<T> {
    QtHandle {
      context: self.context.clone(),
    }
  }

  fn create_window<F: Fn(RawWindow) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self>,
    _after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self>> {
    let st = state::<T>().ok_or(Error::EventLoopClosed)?;
    create_window(&st, pending)
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self>,
  ) -> Result<DetachedWebview<T, Self>> {
    let st = state::<T>().ok_or(Error::EventLoopClosed)?;
    create_webview(&st, window_id, WebviewKind::WindowChild, pending)
  }

  fn primary_monitor(&self) -> Option<Monitor> {
    primary_monitor()
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Option<Monitor> {
    util::screen_at_physical(x, y).map(|s| util::monitor_from_screen(&s))
  }

  fn available_monitors(&self) -> Vec<Monitor> {
    available_monitors()
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    cursor_position()
  }

  fn set_theme(&self, theme: Option<Theme>) {
    apply_theme(theme);
  }

  fn set_device_event_filter(&mut self, _filter: DeviceEventFilter) {}

  fn run_iteration<F: FnMut(RunEvent<T>) + 'static>(&mut self, callback: F) {
    if let Some(st) = state::<T>() {
      *st.callback.borrow_mut() = Some(Box::new(callback));
      st.flush();
    }
    app::process_events();
  }

  fn run_return<F: FnMut(RunEvent<T>) + 'static>(self, callback: F) -> i32 {
    let Some(st) = state::<T>() else {
      return 1;
    };
    *st.callback.borrow_mut() = Some(Box::new(callback));
    // Ready goes first: delivered now, ahead of the events queued while windows were built
    // before `run` (flushed right after it) and of closures other threads already queued with
    // Qt (they run once the loop starts).
    st.queue.borrow_mut().push_front(RunEvent::Ready);
    st.flush();
    drop(st);
    let code = app::exec();
    if let Some(st) = state::<T>() {
      st.emit(RunEvent::Exit);
    }
    self.teardown();
    code
  }

  fn run<F: FnMut(RunEvent<T>) + 'static>(self, callback: F) {
    let code = self.run_return(callback);
    std::process::exit(code);
  }
}

// Silence "unused" for helpers only used by some cfgs.
#[allow(dead_code)]
fn _assert_traits() {
  fn send_sync<S: Send + Sync>() {}
  send_sync::<QtHandle<()>>();
  send_sync::<QtEventProxy<()>>();
  let _ = display_handle_raw;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn qt_init_attrs_builder() {
    let qt = Qt::default()
      .chromium_flags("--a --b")
      .chromium_flags("--c")
      .remote_debugging_port(9222);
    assert_eq!(qt.chromium_flags, vec!["--a --b", "--c"]);
    assert_eq!(qt.remote_debugging_port, Some(9222));
    assert_eq!(Qt::default(), Qt::default());
    // Selectable as the type-erased runtime's init attributes.
    let _: tauri_runtime::dynamic::DynRuntimeInitAttrs<()> = qt.into();
  }

  #[test]
  fn auto_resize_rates() {
    let r = rates(10, 20, 100, 50, 200, 100).unwrap();
    assert_eq!((r.x, r.y, r.width, r.height), (0.05, 0.2, 0.5, 0.5));
    assert!(rates(0, 0, 10, 10, 0, 100).is_none());
    assert!(rates(0, 0, 10, 10, 100, -1).is_none());
  }

  #[test]
  fn theme_to_color_scheme() {
    assert_eq!(color_scheme(Theme::Dark), app::ColorScheme::Dark);
    assert_eq!(color_scheme(Theme::Light), app::ColorScheme::Light);
  }

  #[test]
  fn display_handles_survive_the_send_round_trip() {
    use raw_window_handle::RawDisplayHandle as R;
    assert!(display_from_send(SendHandle(None)).is_none());
    assert!(matches!(
      display_from_send(SendHandle(Some((1, 0x1000)))),
      Some(R::Wayland(h)) if h.display.as_ptr() as usize == 0x1000
    ));
    // A null Wayland display is no handle.
    assert!(display_from_send(SendHandle(Some((1, 0)))).is_none());
    assert!(matches!(
      display_from_send(SendHandle(Some((2, 0x2000)))),
      Some(R::Xlib(h)) if h.display.map(|d| d.as_ptr() as usize) == Some(0x2000)
    ));
  }
}
