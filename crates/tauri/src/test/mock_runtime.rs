// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(dead_code)]
#![allow(missing_docs)]

use tauri_runtime::{
  dpi::{
    LogicalUnit, PhysicalPosition, PhysicalRect, PhysicalSize, PhysicalUnit, PixelUnit, Position,
    Rect, Size,
  },
  monitor::Monitor,
  webview::{DetachedWebview, PendingWebview},
  window::{
    CursorIcon, DetachedWindow, DetachedWindowWebview, PendingWindow, RawWindow, WebviewEvent,
    WindowBuilder, WindowBuilderBase, WindowEvent, WindowId, WindowSizeConstraints,
  },
  Cookie, DeviceEventFilter, Error, EventLoopProxy, ExitRequestedEventAction, Icon,
  ProgressBarState, ProgressBarStatus, Result, RunEvent, Runtime, RuntimeHandle, RuntimeInitArgs,
  UserAttentionType, UserEvent, WebviewDispatch, WebviewEventId, WindowDispatch, WindowEventId,
};

use tauri_utils::{
  config::{Color, WindowConfig},
  Theme, TitleBarStyle,
};
use url::Url;

#[cfg(windows)]
use windows::Win32::Foundation::HWND;

use std::{
  any::Any,
  borrow::Cow,
  collections::HashMap,
  fmt,
  sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    mpsc::{channel, sync_channel, Receiver, RecvTimeoutError, SyncSender},
    Arc, Mutex,
  },
  time::Duration,
};

/// The resolution of the monitor mocked by this runtime.
const MONITOR_SIZE: PhysicalSize<u32> = PhysicalSize::new(1920, 1080);
/// The work area of the monitor mocked by this runtime (resolution minus a fake taskbar).
const MONITOR_WORK_AREA_SIZE: PhysicalSize<u32> = PhysicalSize::new(1920, 1040);

fn mock_monitor() -> Monitor {
  Monitor {
    name: Some("MockMonitor".into()),
    size: MONITOR_SIZE,
    position: PhysicalPosition::new(0, 0),
    work_area: PhysicalRect {
      position: PhysicalPosition::new(0, 0),
      size: MONITOR_WORK_AREA_SIZE,
    },
    scale_factor: 1.0,
  }
}

fn monitor_from_point(x: f64, y: f64) -> Option<Monitor> {
  (x >= 0.0 && x < MONITOR_SIZE.width as f64 && y >= 0.0 && y < MONITOR_SIZE.height as f64)
    .then(mock_monitor)
}

fn owned_icon(icon: Icon<'_>) -> Icon<'static> {
  Icon {
    rgba: Cow::Owned(icon.rgba.into_owned()),
    width: icon.width,
    height: icon.height,
  }
}

type WindowEventListeners = Arc<Mutex<HashMap<WindowEventId, Box<dyn Fn(&WindowEvent) + Send>>>>;
type WebviewEventListeners = Arc<Mutex<HashMap<WebviewEventId, Box<dyn Fn(&WebviewEvent) + Send>>>>;

enum Message {
  Task(Box<dyn FnOnce() + Send>),
  CloseWindow(WindowId),
  DestroyWindow(WindowId),
  RequestExit(i32),
  UserEvent(Box<dyn Any + Send>),
  /// Forward an already dispatched window event to the event loop callback.
  WindowEvent(WindowId, WindowEvent),
}

/// A snapshot of [`ProgressBarState`], which is not `Clone`.
#[derive(Debug, Clone)]
struct ProgressBarSnapshot {
  status: Option<ProgressBarStatus>,
  progress: Option<u64>,
  desktop_filename: Option<String>,
}

/// The state of a mock window, tracked by the setters and returned by the getters
/// so tests can assert the result of window operations.
#[derive(Debug, Clone)]
struct WindowState {
  title: String,
  position: PhysicalPosition<i32>,
  inner_size: PhysicalSize<u32>,
  size_before_fullscreen: Option<PhysicalSize<u32>>,
  size_before_maximized: Option<PhysicalSize<u32>>,
  size_constraints: WindowSizeConstraints,
  resizable: bool,
  maximizable: bool,
  minimizable: bool,
  closable: bool,
  fullscreen: bool,
  focused: bool,
  focusable: bool,
  minimized: bool,
  maximized: bool,
  visible: bool,
  enabled: bool,
  decorated: bool,
  shadow: bool,
  transparent: bool,
  always_on_top: bool,
  always_on_bottom: bool,
  visible_on_all_workspaces: bool,
  content_protected: bool,
  skip_taskbar: bool,
  theme: Option<Theme>,
  icon: Option<Icon<'static>>,
  overlay_icon: Option<Icon<'static>>,
  badge_count: Option<i64>,
  badge_label: Option<String>,
  progress_bar: Option<ProgressBarSnapshot>,
  cursor_grab: bool,
  cursor_visible: bool,
  cursor_icon: CursorIcon,
  ignore_cursor_events: bool,
  user_attention: Option<UserAttentionType>,
  title_bar_style: TitleBarStyle,
  traffic_light_position: Option<Position>,
  background_color: Option<Color>,
  scale_factor: f64,
}

impl Default for WindowState {
  fn default() -> Self {
    Self {
      title: String::new(),
      position: PhysicalPosition::new(0, 0),
      inner_size: PhysicalSize::new(800, 600),
      size_before_fullscreen: None,
      size_before_maximized: None,
      size_constraints: WindowSizeConstraints::default(),
      resizable: true,
      maximizable: true,
      minimizable: true,
      closable: true,
      fullscreen: false,
      focused: true,
      focusable: true,
      minimized: false,
      maximized: false,
      visible: true,
      enabled: true,
      decorated: true,
      shadow: true,
      transparent: false,
      always_on_top: false,
      always_on_bottom: false,
      visible_on_all_workspaces: false,
      content_protected: false,
      skip_taskbar: false,
      theme: None,
      icon: None,
      overlay_icon: None,
      badge_count: None,
      badge_label: None,
      progress_bar: None,
      cursor_grab: false,
      cursor_visible: true,
      cursor_icon: CursorIcon::default(),
      ignore_cursor_events: false,
      user_attention: None,
      title_bar_style: TitleBarStyle::default(),
      traffic_light_position: None,
      background_color: None,
      scale_factor: 1.0,
    }
  }
}

/// The state of a mock webview, tracked by the setters and returned by the getters
/// so tests can assert the result of webview operations.
#[derive(Debug)]
struct WebviewState {
  url: String,
  bounds: Rect,
  auto_resize: bool,
  focused: bool,
  visible: bool,
  zoom: f64,
  background_color: Option<Color>,
  devtools_open: bool,
  cookies: Vec<Cookie<'static>>,
  evaluated_scripts: Vec<String>,
  parent_window: WindowId,
}

struct Webview {
  id: u32,
  label: String,
  state: Arc<Mutex<WebviewState>>,
  listeners: WebviewEventListeners,
}

struct Window {
  label: String,
  state: Arc<Mutex<WindowState>>,
  listeners: WindowEventListeners,
  webviews: Vec<Webview>,
}

/// Invokes every listener with the given event.
///
/// Each listener is temporarily removed from the map while it runs so a listener
/// that calls back into the runtime (e.g. registering another listener or triggering
/// another event) does not deadlock or infinitely recurse.
fn dispatch_window_event(listeners: &WindowEventListeners, event: &WindowEvent) {
  let ids = listeners
    .lock()
    .unwrap()
    .keys()
    .copied()
    .collect::<Vec<_>>();
  for id in ids {
    let listener = listeners.lock().unwrap().remove(&id);
    if let Some(listener) = listener {
      listener(event);
      listeners.lock().unwrap().insert(id, listener);
    }
  }
}

#[derive(Clone)]
pub struct RuntimeContext {
  is_running: Arc<AtomicBool>,
  windows: Arc<Mutex<HashMap<WindowId, Window>>>,
  app_theme: Arc<Mutex<Option<Theme>>>,
  cursor_position: Arc<Mutex<PhysicalPosition<f64>>>,
  device_event_filter: Arc<Mutex<DeviceEventFilter>>,
  run_tx: SyncSender<Message>,
  main_thread_id: std::thread::ThreadId,
  next_window_id: Arc<AtomicU32>,
  next_webview_id: Arc<AtomicU32>,
  next_window_event_id: Arc<AtomicU32>,
  next_webview_event_id: Arc<AtomicU32>,
}

impl RuntimeContext {
  fn send_message(&self, message: Message) -> Result<()> {
    if std::thread::current().id() == self.main_thread_id {
      if let Message::Task(task) = message {
        task();
        return Ok(());
      }
    }
    if self.is_running.load(Ordering::Relaxed) {
      self
        .run_tx
        .send(message)
        .map_err(|_| Error::FailedToSendMessage)
    } else {
      // the event loop is not running; process the message inline (without an event loop callback)
      process_message::<(), _>(self, message, &mut |_| {});
      Ok(())
    }
  }

  fn next_window_id(&self) -> WindowId {
    self.next_window_id.fetch_add(1, Ordering::Relaxed).into()
  }

  fn next_webview_id(&self) -> u32 {
    self.next_webview_id.fetch_add(1, Ordering::Relaxed)
  }

  fn next_window_event_id(&self) -> WindowEventId {
    self.next_window_event_id.fetch_add(1, Ordering::Relaxed)
  }

  fn next_webview_event_id(&self) -> WebviewEventId {
    self.next_webview_event_id.fetch_add(1, Ordering::Relaxed)
  }

  /// Dispatches an event to the window listeners and, when the event loop is running,
  /// forwards it to the loop so the `run` callback receives it as a [`RunEvent::WindowEvent`].
  fn emit_window_event(&self, id: WindowId, listeners: &WindowEventListeners, event: WindowEvent) {
    dispatch_window_event(listeners, &event);
    if self.is_running.load(Ordering::Relaxed) {
      // try_send: never block the caller; the callback forward is best-effort
      let _ = self.run_tx.try_send(Message::WindowEvent(id, event));
    }
  }

  fn set_app_theme(&self, theme: Option<Theme>) {
    *self.app_theme.lock().unwrap() = theme;
    let windows = self
      .windows
      .lock()
      .unwrap()
      .iter()
      .map(|(id, w)| (*id, w.state.clone(), w.listeners.clone()))
      .collect::<Vec<_>>();
    for (id, state, listeners) in windows {
      state.lock().unwrap().theme = theme;
      self.emit_window_event(
        id,
        &listeners,
        WindowEvent::ThemeChanged(theme.unwrap_or(Theme::Light)),
      );
    }
  }

  fn create_window_impl<T: UserEvent>(
    &self,
    pending: PendingWindow<T, MockRuntime>,
  ) -> Result<DetachedWindow<T, MockRuntime>> {
    let id = self.next_window_id();

    let MockWindowBuilder { mut state, center } = pending.window_builder;
    if state.theme.is_none() {
      state.theme = *self.app_theme.lock().unwrap();
    }
    if center {
      state.position = PhysicalPosition::new(
        (MONITOR_SIZE.width.saturating_sub(state.inner_size.width) / 2) as i32,
        (MONITOR_SIZE.height.saturating_sub(state.inner_size.height) / 2) as i32,
      );
    }
    // a window can only be focused when it is visible
    state.focused = state.focused && state.visible;
    let inner_size = state.inner_size;

    let window_state = Arc::new(Mutex::new(state));
    let listeners = WindowEventListeners::default();

    let (webview_entry, detached_webview) = match pending.webview {
      Some(pending_webview) => {
        let use_https_scheme = pending_webview.webview_attributes.use_https_scheme;
        let (entry, detached) = self.prepare_webview(id, inner_size, pending_webview);
        (Some(entry), Some((detached, use_https_scheme)))
      }
      None => (None, None),
    };

    self.windows.lock().unwrap().insert(
      id,
      Window {
        label: pending.label.clone(),
        state: window_state.clone(),
        listeners: listeners.clone(),
        webviews: webview_entry.into_iter().collect(),
      },
    );

    Ok(DetachedWindow {
      id,
      label: pending.label,
      dispatcher: MockWindowDispatcher {
        id,
        context: self.clone(),
        state: window_state,
        listeners,
      },
      webview: detached_webview.map(|(webview, use_https_scheme)| DetachedWindowWebview {
        webview,
        use_https_scheme,
      }),
    })
  }

  fn create_webview_impl<T: UserEvent>(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, MockRuntime>,
  ) -> Result<DetachedWebview<T, MockRuntime>> {
    let window_size = self
      .windows
      .lock()
      .unwrap()
      .get(&window_id)
      .ok_or(Error::WindowNotFound)?
      .state
      .lock()
      .unwrap()
      .inner_size;

    let (entry, detached) = self.prepare_webview(window_id, window_size, pending);

    if let Some(window) = self.windows.lock().unwrap().get_mut(&window_id) {
      window.webviews.push(entry);
    }

    Ok(detached)
  }

  fn prepare_webview<T: UserEvent>(
    &self,
    window_id: WindowId,
    window_size: PhysicalSize<u32>,
    pending: PendingWebview<T, MockRuntime>,
  ) -> (Webview, DetachedWebview<T, MockRuntime>) {
    let id = self.next_webview_id();

    let bounds = pending.webview_attributes.bounds.unwrap_or(Rect {
      position: Position::Physical(PhysicalPosition::new(0, 0)),
      size: Size::Physical(window_size),
    });

    let state = Arc::new(Mutex::new(WebviewState {
      url: pending.url.clone(),
      bounds,
      auto_resize: pending.webview_attributes.auto_resize,
      focused: pending.webview_attributes.focus,
      visible: true,
      zoom: 1.0,
      background_color: pending.webview_attributes.background_color,
      devtools_open: false,
      cookies: Vec::new(),
      evaluated_scripts: Vec::new(),
      parent_window: window_id,
    }));
    let listeners = WebviewEventListeners::default();

    let entry = Webview {
      id,
      label: pending.label.clone(),
      state: state.clone(),
      listeners: listeners.clone(),
    };

    let detached = DetachedWebview {
      label: pending.label,
      dispatcher: MockWebviewDispatcher {
        id,
        context: self.clone(),
        state,
        listeners,
      },
    };

    (entry, detached)
  }
}

/// Processes a runtime message, forwarding resulting events to `callback`.
///
/// Returns `Some(exit_code)` when the message leads to an unprevented exit request,
/// signaling the event loop to stop.
fn process_message<T: UserEvent, F: FnMut(RunEvent<T>)>(
  context: &RuntimeContext,
  message: Message,
  callback: &mut F,
) -> Option<i32> {
  match message {
    Message::Task(task) => {
      task();
      None
    }
    Message::CloseWindow(id) => {
      if handle_close_requested(context, id, callback) {
        handle_empty_windows(context, callback)
      } else {
        None
      }
    }
    Message::DestroyWindow(id) => {
      if handle_destroy(context, id, callback) {
        handle_empty_windows(context, callback)
      } else {
        None
      }
    }
    Message::RequestExit(code) => {
      let (tx, rx) = channel();
      callback(RunEvent::ExitRequested {
        code: Some(code),
        tx,
      });
      let should_prevent = matches!(rx.try_recv(), Ok(ExitRequestedEventAction::Prevent));
      (!should_prevent).then_some(code)
    }
    Message::UserEvent(event) => {
      if let Ok(event) = event.downcast::<T>() {
        callback(RunEvent::UserEvent(*event));
      }
      None
    }
    Message::WindowEvent(id, event) => {
      let label = context
        .windows
        .lock()
        .unwrap()
        .get(&id)
        .map(|w| w.label.clone());
      if let Some(label) = label {
        callback(RunEvent::WindowEvent { label, event });
      }
      None
    }
  }
}

/// Emits `CloseRequested` to the window listeners and the loop callback and,
/// unless prevented, destroys the window. Returns whether the window was destroyed.
fn handle_close_requested<T: UserEvent, F: FnMut(RunEvent<T>)>(
  context: &RuntimeContext,
  id: WindowId,
  callback: &mut F,
) -> bool {
  let Some((label, listeners)) = context
    .windows
    .lock()
    .unwrap()
    .get(&id)
    .map(|w| (w.label.clone(), w.listeners.clone()))
  else {
    return false;
  };

  let (tx, rx) = channel();
  let event = WindowEvent::CloseRequested { signal_tx: tx };
  dispatch_window_event(&listeners, &event);
  callback(RunEvent::WindowEvent { label, event });

  let should_prevent = matches!(rx.try_recv(), Ok(true));
  if should_prevent {
    false
  } else {
    handle_destroy(context, id, callback)
  }
}

/// Emits `Destroyed` and removes the window. Returns whether the window existed.
fn handle_destroy<T: UserEvent, F: FnMut(RunEvent<T>)>(
  context: &RuntimeContext,
  id: WindowId,
  callback: &mut F,
) -> bool {
  let Some(window) = context.windows.lock().unwrap().remove(&id) else {
    return false;
  };
  dispatch_window_event(&window.listeners, &WindowEvent::Destroyed);
  callback(RunEvent::WindowEvent {
    label: window.label,
    event: WindowEvent::Destroyed,
  });
  true
}

/// When all windows are closed, requests an exit from the loop callback.
/// Returns the exit code unless the exit was prevented.
fn handle_empty_windows<T: UserEvent, F: FnMut(RunEvent<T>)>(
  context: &RuntimeContext,
  callback: &mut F,
) -> Option<i32> {
  if !context.windows.lock().unwrap().is_empty() {
    return None;
  }
  let (tx, rx) = channel();
  callback(RunEvent::ExitRequested { code: None, tx });
  let should_prevent = matches!(rx.try_recv(), Ok(ExitRequestedEventAction::Prevent));
  (!should_prevent).then_some(0)
}

impl fmt::Debug for RuntimeContext {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("RuntimeContext").finish()
  }
}

#[derive(Debug, Clone)]
pub struct MockRuntimeHandle {
  context: RuntimeContext,
}

impl<T: UserEvent> RuntimeHandle<T> for MockRuntimeHandle {
  type Runtime = MockRuntime;

  fn create_proxy(&self) -> EventProxy {
    EventProxy {
      context: self.context.clone(),
    }
  }

  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn set_activation_policy(
    &self,
    activation_policy: tauri_runtime::ActivationPolicy,
  ) -> Result<()> {
    Ok(())
  }

  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn set_dock_visibility(&self, visible: bool) -> Result<()> {
    Ok(())
  }

  fn request_exit(&self, code: i32) -> Result<()> {
    self.context.send_message(Message::RequestExit(code))
  }

  /// Create a new webview window.
  fn create_window<F: Fn(RawWindow<'_>) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self::Runtime>,
    _after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    self.context.create_window_impl(pending)
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>> {
    self.context.create_webview_impl(window_id, pending)
  }

  /// Run a task on the main thread.
  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.context.send_message(Message::Task(Box::new(f)))
  }

  fn display_handle(
    &self,
  ) -> std::result::Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
    #[cfg(target_os = "linux")]
    return Ok(unsafe {
      raw_window_handle::DisplayHandle::borrow_raw(raw_window_handle::RawDisplayHandle::Xlib(
        raw_window_handle::XlibDisplayHandle::new(None, 0),
      ))
    });
    #[cfg(target_os = "macos")]
    return Ok(unsafe {
      raw_window_handle::DisplayHandle::borrow_raw(raw_window_handle::RawDisplayHandle::AppKit(
        raw_window_handle::AppKitDisplayHandle::new(),
      ))
    });
    #[cfg(windows)]
    return Ok(unsafe {
      raw_window_handle::DisplayHandle::borrow_raw(raw_window_handle::RawDisplayHandle::Windows(
        raw_window_handle::WindowsDisplayHandle::new(),
      ))
    });
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    unimplemented!();
  }

  fn primary_monitor(&self) -> Option<Monitor> {
    Some(mock_monitor())
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Option<Monitor> {
    monitor_from_point(x, y)
  }

  fn available_monitors(&self) -> Vec<Monitor> {
    vec![mock_monitor()]
  }

  fn set_theme(&self, theme: Option<Theme>) {
    self.context.set_app_theme(theme);
  }

  /// Shows the application, but does not automatically focus it.
  #[cfg(target_os = "macos")]
  fn show(&self) -> Result<()> {
    Ok(())
  }

  /// Hides the application.
  #[cfg(target_os = "macos")]
  fn hide(&self) -> Result<()> {
    Ok(())
  }

  fn set_device_event_filter(&self, filter: DeviceEventFilter) {
    *self.context.device_event_filter.lock().unwrap() = filter;
  }

  #[cfg(target_os = "android")]
  fn find_class<'a>(
    &self,
    env: &mut jni::JNIEnv<'a>,
    activity: &jni::objects::JObject<'_>,
    name: impl Into<String>,
  ) -> std::result::Result<jni::objects::JClass<'a>, jni::errors::Error> {
    todo!()
  }

  #[cfg(target_os = "android")]
  fn run_on_android_context<F>(&self, f: F)
  where
    F: FnOnce(&mut jni::JNIEnv<'_>, &jni::objects::JObject<'_>, &jni::objects::JObject<'_>)
      + Send
      + 'static,
  {
    todo!()
  }

  #[cfg(any(target_os = "macos", target_os = "ios"))]
  fn fetch_data_store_identifiers<F: FnOnce(Vec<[u8; 16]>) + Send + 'static>(
    &self,
    cb: F,
  ) -> Result<()> {
    cb(Vec::new());
    Ok(())
  }

  #[cfg(any(target_os = "macos", target_os = "ios"))]
  fn remove_data_store<F: FnOnce(Result<()>) + Send + 'static>(
    &self,
    uuid: [u8; 16],
    cb: F,
  ) -> Result<()> {
    cb(Ok(()));
    Ok(())
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    Ok(*self.context.cursor_position.lock().unwrap())
  }
}

#[derive(Clone)]
pub struct MockWebviewDispatcher {
  id: u32,
  context: RuntimeContext,
  state: Arc<Mutex<WebviewState>>,
  listeners: WebviewEventListeners,
}

impl fmt::Debug for MockWebviewDispatcher {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("MockWebviewDispatcher")
      .field("id", &self.id)
      .field("state", &self.state)
      .finish()
  }
}

impl MockWebviewDispatcher {
  /// The last JavaScript script evaluated on this webview.
  pub fn last_evaluated_script(&self) -> Option<String> {
    self.state.lock().unwrap().evaluated_scripts.last().cloned()
  }

  /// All JavaScript scripts evaluated on this webview, in evaluation order.
  pub fn evaluated_scripts(&self) -> Vec<String> {
    self.state.lock().unwrap().evaluated_scripts.clone()
  }

  /// The current zoom level, as set by [`WebviewDispatch::set_zoom`].
  pub fn zoom(&self) -> f64 {
    self.state.lock().unwrap().zoom
  }
}

#[derive(Clone)]
pub struct MockWindowDispatcher {
  id: WindowId,
  context: RuntimeContext,
  state: Arc<Mutex<WindowState>>,
  listeners: WindowEventListeners,
}

impl fmt::Debug for MockWindowDispatcher {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("MockWindowDispatcher")
      .field("id", &self.id)
      .field("state", &self.state)
      .finish()
  }
}

impl MockWindowDispatcher {
  fn emit(&self, event: WindowEvent) {
    self
      .context
      .emit_window_event(self.id, &self.listeners, event);
  }
}

#[derive(Debug, Clone)]
pub struct MockWindowBuilder {
  state: WindowState,
  center: bool,
}

impl WindowBuilderBase for MockWindowBuilder {}

impl WindowBuilder for MockWindowBuilder {
  fn new() -> Self {
    Self {
      state: WindowState::default(),
      center: false,
    }
  }

  fn with_config(config: &WindowConfig) -> Self {
    let mut builder = Self::new()
      .title(config.title.as_str())
      .inner_size(config.width, config.height)
      .visible(config.visible)
      .resizable(config.resizable)
      .fullscreen(config.fullscreen)
      .decorations(config.decorations)
      .maximized(config.maximized)
      .always_on_bottom(config.always_on_bottom)
      .always_on_top(config.always_on_top)
      .visible_on_all_workspaces(config.visible_on_all_workspaces)
      .content_protected(config.content_protected)
      .skip_taskbar(config.skip_taskbar)
      .shadow(config.shadow)
      .focused(config.focus)
      .focusable(config.focusable)
      .maximizable(config.maximizable)
      .minimizable(config.minimizable)
      .closable(config.closable)
      .theme(config.theme);

    builder.state.transparent = config.transparent;
    builder.state.title_bar_style = config.title_bar_style;
    builder.state.traffic_light_position = config
      .traffic_light_position
      .as_ref()
      .map(|p| Position::Logical(tauri_runtime::dpi::LogicalPosition::new(p.x, p.y)));
    builder.state.background_color = config.background_color;
    builder.state.size_constraints = WindowSizeConstraints {
      min_width: config
        .min_width
        .map(|v| PixelUnit::Logical(LogicalUnit::new(v))),
      min_height: config
        .min_height
        .map(|v| PixelUnit::Logical(LogicalUnit::new(v))),
      max_width: config
        .max_width
        .map(|v| PixelUnit::Logical(LogicalUnit::new(v))),
      max_height: config
        .max_height
        .map(|v| PixelUnit::Logical(LogicalUnit::new(v))),
    };

    if let (Some(x), Some(y)) = (config.x, config.y) {
      builder = builder.position(x, y);
    }
    if config.center {
      builder = builder.center();
    }

    builder
  }

  fn center(mut self) -> Self {
    self.center = true;
    self
  }

  fn position(mut self, x: f64, y: f64) -> Self {
    self.state.position = PhysicalPosition::new(x as i32, y as i32);
    self
  }

  fn inner_size(mut self, width: f64, height: f64) -> Self {
    self.state.inner_size = PhysicalSize::new(width as u32, height as u32);
    self
  }

  fn min_inner_size(mut self, min_width: f64, min_height: f64) -> Self {
    self.state.size_constraints.min_width = Some(PixelUnit::Logical(LogicalUnit::new(min_width)));
    self.state.size_constraints.min_height = Some(PixelUnit::Logical(LogicalUnit::new(min_height)));
    self
  }

  fn max_inner_size(mut self, max_width: f64, max_height: f64) -> Self {
    self.state.size_constraints.max_width = Some(PixelUnit::Logical(LogicalUnit::new(max_width)));
    self.state.size_constraints.max_height = Some(PixelUnit::Logical(LogicalUnit::new(max_height)));
    self
  }

  fn inner_size_constraints(mut self, constraints: WindowSizeConstraints) -> Self {
    self.state.size_constraints = constraints;
    self
  }

  fn prevent_overflow(mut self) -> Self {
    self.state.inner_size.width = self
      .state
      .inner_size
      .width
      .min(MONITOR_WORK_AREA_SIZE.width);
    self.state.inner_size.height = self
      .state
      .inner_size
      .height
      .min(MONITOR_WORK_AREA_SIZE.height);
    self
  }

  fn prevent_overflow_with_margin(mut self, margin: tauri_runtime::dpi::Size) -> Self {
    let margin = margin.to_physical::<u32>(1.0);
    let max_size = PhysicalSize::new(
      MONITOR_WORK_AREA_SIZE
        .width
        .saturating_sub(margin.width)
        .max(1),
      MONITOR_WORK_AREA_SIZE
        .height
        .saturating_sub(margin.height)
        .max(1),
    );
    self.state.inner_size.width = self.state.inner_size.width.min(max_size.width);
    self.state.inner_size.height = self.state.inner_size.height.min(max_size.height);
    self
  }

  fn resizable(mut self, resizable: bool) -> Self {
    self.state.resizable = resizable;
    self
  }

  fn maximizable(mut self, maximizable: bool) -> Self {
    self.state.maximizable = maximizable;
    self
  }

  fn minimizable(mut self, minimizable: bool) -> Self {
    self.state.minimizable = minimizable;
    self
  }

  fn closable(mut self, closable: bool) -> Self {
    self.state.closable = closable;
    self
  }

  fn title<S: Into<String>>(mut self, title: S) -> Self {
    self.state.title = title.into();
    self
  }

  fn fullscreen(mut self, fullscreen: bool) -> Self {
    self.state.fullscreen = fullscreen;
    self
  }

  fn focused(mut self, focused: bool) -> Self {
    self.state.focused = focused;
    self
  }

  fn focusable(mut self, focusable: bool) -> Self {
    self.state.focusable = focusable;
    self
  }

  fn maximized(mut self, maximized: bool) -> Self {
    self.state.maximized = maximized;
    self
  }

  fn visible(mut self, visible: bool) -> Self {
    self.state.visible = visible;
    self
  }

  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  #[cfg_attr(
    docsrs,
    doc(cfg(any(not(target_os = "macos"), feature = "macos-private-api")))
  )]
  fn transparent(mut self, transparent: bool) -> Self {
    self.state.transparent = transparent;
    self
  }

  fn decorations(mut self, decorations: bool) -> Self {
    self.state.decorated = decorations;
    self
  }

  fn always_on_bottom(mut self, always_on_bottom: bool) -> Self {
    self.state.always_on_bottom = always_on_bottom;
    self
  }

  fn always_on_top(mut self, always_on_top: bool) -> Self {
    self.state.always_on_top = always_on_top;
    self
  }

  fn visible_on_all_workspaces(mut self, visible_on_all_workspaces: bool) -> Self {
    self.state.visible_on_all_workspaces = visible_on_all_workspaces;
    self
  }

  fn content_protected(mut self, protected: bool) -> Self {
    self.state.content_protected = protected;
    self
  }

  fn icon(mut self, icon: Icon<'_>) -> Result<Self> {
    self.state.icon = Some(owned_icon(icon));
    Ok(self)
  }

  fn skip_taskbar(mut self, skip: bool) -> Self {
    self.state.skip_taskbar = skip;
    self
  }

  fn window_classname<S: Into<String>>(self, classname: S) -> Self {
    self
  }

  fn no_redirection_bitmap(self, enable: bool) -> Self {
    self
  }

  fn shadow(mut self, enable: bool) -> Self {
    self.state.shadow = enable;
    self
  }

  #[cfg(windows)]
  fn owner(self, owner: HWND) -> Self {
    self
  }

  #[cfg(windows)]
  fn parent(self, parent: HWND) -> Self {
    self
  }

  #[cfg(target_os = "macos")]
  fn parent(self, parent: *mut std::ffi::c_void) -> Self {
    self
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn transient_for(self, parent: &impl gtk::glib::IsA<gtk::Window>) -> Self {
    self
  }

  #[cfg(windows)]
  fn drag_and_drop(self, enabled: bool) -> Self {
    self
  }

  #[cfg(target_os = "macos")]
  fn title_bar_style(mut self, style: TitleBarStyle) -> Self {
    self.state.title_bar_style = style;
    self
  }

  #[cfg(target_os = "macos")]
  fn traffic_light_position<P: Into<Position>>(mut self, position: P) -> Self {
    self.state.traffic_light_position = Some(position.into());
    self
  }

  #[cfg(target_os = "macos")]
  fn hidden_title(self, transparent: bool) -> Self {
    self
  }

  #[cfg(target_os = "macos")]
  fn tabbing_identifier(self, identifier: &str) -> Self {
    self
  }

  fn theme(mut self, theme: Option<Theme>) -> Self {
    self.state.theme = theme;
    self
  }

  fn has_icon(&self) -> bool {
    self.state.icon.is_some()
  }

  fn get_theme(&self) -> Option<Theme> {
    self.state.theme
  }

  fn background_color(mut self, color: Color) -> Self {
    self.state.background_color = Some(color);
    self
  }

  #[cfg(target_os = "android")]
  fn activity_name<S: Into<String>>(self, _class_name: S) -> Self {
    self
  }

  #[cfg(target_os = "android")]
  fn created_by_activity_name<S: Into<String>>(self, _class_name: S) -> Self {
    self
  }

  #[cfg(target_os = "ios")]
  fn requested_by_scene_identifier<S: Into<String>>(self, _identifier: S) -> Self {
    self
  }
}

impl<T: UserEvent> WebviewDispatch<T> for MockWebviewDispatcher {
  type Runtime = MockRuntime;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.context.send_message(Message::Task(Box::new(f)))
  }

  fn on_webview_event<F: Fn(&WebviewEvent) + Send + 'static>(&self, f: F) -> WebviewEventId {
    let id = self.context.next_webview_event_id();
    self.listeners.lock().unwrap().insert(id, Box::new(f));
    id
  }

  fn with_webview<F: FnOnce(Box<dyn std::any::Any>) + Send + 'static>(&self, f: F) -> Result<()> {
    f(Box::new(()));
    Ok(())
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn open_devtools(&self) {
    self.state.lock().unwrap().devtools_open = true;
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn close_devtools(&self) {
    self.state.lock().unwrap().devtools_open = false;
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn is_devtools_open(&self) -> Result<bool> {
    Ok(self.state.lock().unwrap().devtools_open)
  }

  fn set_zoom(&self, scale_factor: f64) -> Result<()> {
    self.state.lock().unwrap().zoom = scale_factor;
    Ok(())
  }

  fn eval_script<S: Into<String>>(&self, script: S) -> Result<()> {
    self
      .state
      .lock()
      .unwrap()
      .evaluated_scripts
      .push(script.into());
    Ok(())
  }

  fn eval_script_with_callback<S: Into<String>>(
    &self,
    script: S,
    callback: impl Fn(String) + Send + 'static,
  ) -> Result<()> {
    self
      .state
      .lock()
      .unwrap()
      .evaluated_scripts
      .push(script.into());
    // the mock runtime does not execute scripts; every evaluation "returns" null
    callback("null".into());
    Ok(())
  }

  fn url(&self) -> Result<String> {
    Ok(self.state.lock().unwrap().url.clone())
  }

  fn bounds(&self) -> Result<Rect> {
    Ok(self.state.lock().unwrap().bounds)
  }

  fn position(&self) -> Result<PhysicalPosition<i32>> {
    Ok(self.state.lock().unwrap().bounds.position.to_physical(1.0))
  }

  fn size(&self) -> Result<PhysicalSize<u32>> {
    Ok(self.state.lock().unwrap().bounds.size.to_physical(1.0))
  }

  fn navigate(&self, url: Url) -> Result<()> {
    self.state.lock().unwrap().url = url.to_string();
    Ok(())
  }

  fn reload(&self) -> Result<()> {
    Ok(())
  }

  fn print(&self) -> Result<()> {
    Ok(())
  }

  fn close(&self) -> Result<()> {
    let parent = self.state.lock().unwrap().parent_window;
    if let Some(window) = self.context.windows.lock().unwrap().get_mut(&parent) {
      window.webviews.retain(|webview| webview.id != self.id);
    }
    Ok(())
  }

  fn set_bounds(&self, bounds: Rect) -> Result<()> {
    self.state.lock().unwrap().bounds = bounds;
    Ok(())
  }

  fn set_size(&self, size: Size) -> Result<()> {
    self.state.lock().unwrap().bounds.size = size;
    Ok(())
  }

  fn set_position(&self, position: Position) -> Result<()> {
    self.state.lock().unwrap().bounds.position = position;
    Ok(())
  }

  fn set_focus(&self) -> Result<()> {
    self.state.lock().unwrap().focused = true;
    Ok(())
  }

  fn reparent(&self, window_id: WindowId) -> Result<()> {
    let current_parent = self.state.lock().unwrap().parent_window;
    if current_parent == window_id {
      return Ok(());
    }
    {
      let mut windows = self.context.windows.lock().unwrap();
      if !windows.contains_key(&window_id) {
        return Err(Error::WindowNotFound);
      }
      let entry = windows.get_mut(&current_parent).and_then(|window| {
        window
          .webviews
          .iter()
          .position(|webview| webview.id == self.id)
          .map(|i| window.webviews.remove(i))
      });
      if let Some(entry) = entry {
        windows
          .get_mut(&window_id)
          .expect("target window was checked above")
          .webviews
          .push(entry);
      }
    }
    self.state.lock().unwrap().parent_window = window_id;
    Ok(())
  }

  fn cookies(&self) -> Result<Vec<Cookie<'static>>> {
    Ok(self.state.lock().unwrap().cookies.clone())
  }

  fn cookies_for_url(&self, url: Url) -> Result<Vec<Cookie<'static>>> {
    let host = url.host_str().unwrap_or_default().to_string();
    Ok(
      self
        .state
        .lock()
        .unwrap()
        .cookies
        .iter()
        .filter(|cookie| match cookie.domain() {
          Some(domain) => {
            let domain = domain.trim_start_matches('.');
            host == domain || host.ends_with(&format!(".{domain}"))
          }
          None => true,
        })
        .cloned()
        .collect(),
    )
  }

  fn set_cookie(&self, cookie: Cookie<'_>) -> Result<()> {
    let cookie = cookie.into_owned();
    let mut state = self.state.lock().unwrap();
    state.cookies.retain(|c| {
      !(c.name() == cookie.name() && c.domain() == cookie.domain() && c.path() == cookie.path())
    });
    state.cookies.push(cookie);
    Ok(())
  }

  fn delete_cookie(&self, cookie: Cookie<'_>) -> Result<()> {
    self.state.lock().unwrap().cookies.retain(|c| {
      !(c.name() == cookie.name() && c.domain() == cookie.domain() && c.path() == cookie.path())
    });
    Ok(())
  }

  fn set_auto_resize(&self, auto_resize: bool) -> Result<()> {
    self.state.lock().unwrap().auto_resize = auto_resize;
    Ok(())
  }

  fn clear_all_browsing_data(&self) -> Result<()> {
    self.state.lock().unwrap().cookies.clear();
    Ok(())
  }

  fn hide(&self) -> Result<()> {
    self.state.lock().unwrap().visible = false;
    Ok(())
  }

  fn show(&self) -> Result<()> {
    self.state.lock().unwrap().visible = true;
    Ok(())
  }

  fn set_background_color(&self, color: Option<Color>) -> Result<()> {
    self.state.lock().unwrap().background_color = color;
    Ok(())
  }
}

impl<T: UserEvent> WindowDispatch<T> for MockWindowDispatcher {
  type Runtime = MockRuntime;

  type WindowBuilder = MockWindowBuilder;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.context.send_message(Message::Task(Box::new(f)))
  }

  fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) -> WindowEventId {
    let id = self.context.next_window_event_id();
    self.listeners.lock().unwrap().insert(id, Box::new(f));
    id
  }

  fn scale_factor(&self) -> Result<f64> {
    Ok(self.state.lock().unwrap().scale_factor)
  }

  fn inner_position(&self) -> Result<PhysicalPosition<i32>> {
    Ok(self.state.lock().unwrap().position)
  }

  fn outer_position(&self) -> Result<PhysicalPosition<i32>> {
    Ok(self.state.lock().unwrap().position)
  }

  fn inner_size(&self) -> Result<PhysicalSize<u32>> {
    Ok(self.state.lock().unwrap().inner_size)
  }

  fn outer_size(&self) -> Result<PhysicalSize<u32>> {
    Ok(self.state.lock().unwrap().inner_size)
  }

  fn is_fullscreen(&self) -> Result<bool> {
    Ok(self.state.lock().unwrap().fullscreen)
  }

  fn is_minimized(&self) -> Result<bool> {
    Ok(self.state.lock().unwrap().minimized)
  }

  fn is_maximized(&self) -> Result<bool> {
    Ok(self.state.lock().unwrap().maximized)
  }

  fn is_focused(&self) -> Result<bool> {
    Ok(self.state.lock().unwrap().focused)
  }

  fn is_decorated(&self) -> Result<bool> {
    Ok(self.state.lock().unwrap().decorated)
  }

  fn is_resizable(&self) -> Result<bool> {
    Ok(self.state.lock().unwrap().resizable)
  }

  fn is_maximizable(&self) -> Result<bool> {
    let state = self.state.lock().unwrap();
    Ok(state.maximizable && state.resizable)
  }

  fn is_minimizable(&self) -> Result<bool> {
    Ok(self.state.lock().unwrap().minimizable)
  }

  fn is_closable(&self) -> Result<bool> {
    Ok(self.state.lock().unwrap().closable)
  }

  fn is_visible(&self) -> Result<bool> {
    Ok(self.state.lock().unwrap().visible)
  }

  fn title(&self) -> Result<String> {
    Ok(self.state.lock().unwrap().title.clone())
  }

  fn current_monitor(&self) -> Result<Option<Monitor>> {
    Ok(Some(mock_monitor()))
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    Ok(Some(mock_monitor()))
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Result<Option<Monitor>> {
    Ok(monitor_from_point(x, y))
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    Ok(vec![mock_monitor()])
  }

  fn theme(&self) -> Result<Theme> {
    Ok(self.state.lock().unwrap().theme.unwrap_or(Theme::Light))
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn gtk_window(&self) -> Result<gtk::ApplicationWindow> {
    unimplemented!()
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn default_vbox(&self) -> Result<gtk::Box> {
    unimplemented!()
  }

  #[cfg(target_os = "android")]
  fn activity_name(&self) -> Result<String> {
    unimplemented!()
  }

  #[cfg(target_os = "ios")]
  fn scene_identifier(&self) -> Result<String> {
    unimplemented!()
  }

  fn window_handle(
    &self,
  ) -> std::result::Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
    #[cfg(target_os = "linux")]
    return unsafe {
      Ok(raw_window_handle::WindowHandle::borrow_raw(
        raw_window_handle::RawWindowHandle::Xlib(raw_window_handle::XlibWindowHandle::new(0)),
      ))
    };
    #[cfg(target_os = "macos")]
    return unsafe {
      Ok(raw_window_handle::WindowHandle::borrow_raw(
        raw_window_handle::RawWindowHandle::AppKit(raw_window_handle::AppKitWindowHandle::new(
          std::ptr::NonNull::from(&()).cast(),
        )),
      ))
    };
    #[cfg(windows)]
    return unsafe {
      Ok(raw_window_handle::WindowHandle::borrow_raw(
        raw_window_handle::RawWindowHandle::Win32(raw_window_handle::Win32WindowHandle::new(
          std::num::NonZeroIsize::MIN,
        )),
      ))
    };
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    unimplemented!();
  }

  fn center(&self) -> Result<()> {
    let new_position = {
      let mut state = self.state.lock().unwrap();
      state.position = PhysicalPosition::new(
        (MONITOR_SIZE.width.saturating_sub(state.inner_size.width) / 2) as i32,
        (MONITOR_SIZE.height.saturating_sub(state.inner_size.height) / 2) as i32,
      );
      state.position
    };
    self.emit(WindowEvent::Moved(new_position));
    Ok(())
  }

  fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> Result<()> {
    self.state.lock().unwrap().user_attention = request_type;
    Ok(())
  }

  fn create_window<F: Fn(RawWindow<'_>) + Send + 'static>(
    &mut self,
    pending: PendingWindow<T, Self::Runtime>,
    _after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    self.context.create_window_impl(pending)
  }

  fn create_webview(
    &mut self,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>> {
    self.context.create_webview_impl(self.id, pending)
  }

  fn set_resizable(&self, resizable: bool) -> Result<()> {
    self.state.lock().unwrap().resizable = resizable;
    Ok(())
  }

  fn set_maximizable(&self, maximizable: bool) -> Result<()> {
    self.state.lock().unwrap().maximizable = maximizable;
    Ok(())
  }

  fn set_minimizable(&self, minimizable: bool) -> Result<()> {
    self.state.lock().unwrap().minimizable = minimizable;
    Ok(())
  }

  fn set_closable(&self, closable: bool) -> Result<()> {
    self.state.lock().unwrap().closable = closable;
    Ok(())
  }

  fn set_title<S: Into<String>>(&self, title: S) -> Result<()> {
    self.state.lock().unwrap().title = title.into();
    Ok(())
  }

  fn maximize(&self) -> Result<()> {
    let new_size = {
      let mut state = self.state.lock().unwrap();
      if state.maximized || !(state.maximizable && state.resizable) {
        return Ok(());
      }
      state.maximized = true;
      state.minimized = false;
      state.size_before_maximized = Some(state.inner_size);
      state.inner_size = MONITOR_WORK_AREA_SIZE;
      state.inner_size
    };
    self.emit(WindowEvent::Resized(new_size));
    Ok(())
  }

  fn unmaximize(&self) -> Result<()> {
    let new_size = {
      let mut state = self.state.lock().unwrap();
      if !state.maximized {
        return Ok(());
      }
      state.maximized = false;
      if let Some(size) = state.size_before_maximized.take() {
        state.inner_size = size;
      }
      state.inner_size
    };
    self.emit(WindowEvent::Resized(new_size));
    Ok(())
  }

  fn minimize(&self) -> Result<()> {
    let was_focused = {
      let mut state = self.state.lock().unwrap();
      if !state.minimizable {
        return Ok(());
      }
      state.minimized = true;
      std::mem::replace(&mut state.focused, false)
    };
    if was_focused {
      self.emit(WindowEvent::Focused(false));
    }
    Ok(())
  }

  fn unminimize(&self) -> Result<()> {
    self.state.lock().unwrap().minimized = false;
    Ok(())
  }

  fn show(&self) -> Result<()> {
    self.state.lock().unwrap().visible = true;
    Ok(())
  }

  fn hide(&self) -> Result<()> {
    let was_focused = {
      let mut state = self.state.lock().unwrap();
      state.visible = false;
      std::mem::replace(&mut state.focused, false)
    };
    if was_focused {
      self.emit(WindowEvent::Focused(false));
    }
    Ok(())
  }

  fn close(&self) -> Result<()> {
    self.context.send_message(Message::CloseWindow(self.id))?;
    Ok(())
  }

  fn destroy(&self) -> Result<()> {
    self.context.send_message(Message::DestroyWindow(self.id))?;
    Ok(())
  }

  fn set_decorations(&self, decorations: bool) -> Result<()> {
    self.state.lock().unwrap().decorated = decorations;
    Ok(())
  }

  fn set_shadow(&self, shadow: bool) -> Result<()> {
    self.state.lock().unwrap().shadow = shadow;
    Ok(())
  }

  fn set_always_on_bottom(&self, always_on_bottom: bool) -> Result<()> {
    self.state.lock().unwrap().always_on_bottom = always_on_bottom;
    Ok(())
  }

  fn set_always_on_top(&self, always_on_top: bool) -> Result<()> {
    self.state.lock().unwrap().always_on_top = always_on_top;
    Ok(())
  }

  fn set_visible_on_all_workspaces(&self, visible_on_all_workspaces: bool) -> Result<()> {
    self.state.lock().unwrap().visible_on_all_workspaces = visible_on_all_workspaces;
    Ok(())
  }

  fn set_content_protected(&self, protected: bool) -> Result<()> {
    self.state.lock().unwrap().content_protected = protected;
    Ok(())
  }

  fn set_size(&self, size: Size) -> Result<()> {
    let new_size = {
      let mut state = self.state.lock().unwrap();
      state.inner_size = size.to_physical(state.scale_factor);
      state.inner_size
    };
    self.emit(WindowEvent::Resized(new_size));
    Ok(())
  }

  fn set_min_size(&self, size: Option<Size>) -> Result<()> {
    let mut state = self.state.lock().unwrap();
    match size {
      Some(size) => {
        let size = size.to_physical::<u32>(state.scale_factor);
        state.size_constraints.min_width = Some(PixelUnit::Physical(PhysicalUnit(
          size.width.try_into().unwrap_or(i32::MAX),
        )));
        state.size_constraints.min_height = Some(PixelUnit::Physical(PhysicalUnit(
          size.height.try_into().unwrap_or(i32::MAX),
        )));
      }
      None => {
        state.size_constraints.min_width = None;
        state.size_constraints.min_height = None;
      }
    }
    Ok(())
  }

  fn set_max_size(&self, size: Option<Size>) -> Result<()> {
    let mut state = self.state.lock().unwrap();
    match size {
      Some(size) => {
        let size = size.to_physical::<u32>(state.scale_factor);
        state.size_constraints.max_width = Some(PixelUnit::Physical(PhysicalUnit(
          size.width.try_into().unwrap_or(i32::MAX),
        )));
        state.size_constraints.max_height = Some(PixelUnit::Physical(PhysicalUnit(
          size.height.try_into().unwrap_or(i32::MAX),
        )));
      }
      None => {
        state.size_constraints.max_width = None;
        state.size_constraints.max_height = None;
      }
    }
    Ok(())
  }

  fn set_size_constraints(&self, constraints: WindowSizeConstraints) -> Result<()> {
    self.state.lock().unwrap().size_constraints = constraints;
    Ok(())
  }

  fn set_position(&self, position: Position) -> Result<()> {
    let new_position = {
      let mut state = self.state.lock().unwrap();
      state.position = position.to_physical(state.scale_factor);
      state.position
    };
    self.emit(WindowEvent::Moved(new_position));
    Ok(())
  }

  fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
    let new_size = {
      let mut state = self.state.lock().unwrap();
      if state.fullscreen == fullscreen {
        return Ok(());
      }
      state.fullscreen = fullscreen;
      if fullscreen {
        state.size_before_fullscreen = Some(state.inner_size);
        state.inner_size = MONITOR_SIZE;
      } else if let Some(size) = state.size_before_fullscreen.take() {
        state.inner_size = size;
      }
      state.inner_size
    };
    self.emit(WindowEvent::Resized(new_size));
    Ok(())
  }

  #[cfg(target_os = "macos")]
  fn set_simple_fullscreen(&self, enable: bool) -> Result<()> {
    <Self as WindowDispatch<T>>::set_fullscreen(self, enable)
  }

  fn set_focus(&self) -> Result<()> {
    let focus_changed = {
      let mut state = self.state.lock().unwrap();
      if !state.focusable {
        return Ok(());
      }
      let focus_changed = !state.focused;
      state.focused = true;
      state.minimized = false;
      focus_changed
    };
    if focus_changed {
      self.emit(WindowEvent::Focused(true));
    }
    Ok(())
  }

  fn set_focusable(&self, focusable: bool) -> Result<()> {
    self.state.lock().unwrap().focusable = focusable;
    Ok(())
  }

  fn set_icon(&self, icon: Icon<'_>) -> Result<()> {
    self.state.lock().unwrap().icon = Some(owned_icon(icon));
    Ok(())
  }

  fn set_skip_taskbar(&self, skip: bool) -> Result<()> {
    self.state.lock().unwrap().skip_taskbar = skip;
    Ok(())
  }

  fn set_cursor_grab(&self, grab: bool) -> Result<()> {
    self.state.lock().unwrap().cursor_grab = grab;
    Ok(())
  }

  fn set_cursor_visible(&self, visible: bool) -> Result<()> {
    self.state.lock().unwrap().cursor_visible = visible;
    Ok(())
  }

  fn set_cursor_icon(&self, icon: CursorIcon) -> Result<()> {
    self.state.lock().unwrap().cursor_icon = icon;
    Ok(())
  }

  fn set_cursor_position<Pos: Into<Position>>(&self, position: Pos) -> Result<()> {
    let scale_factor = self.state.lock().unwrap().scale_factor;
    *self.context.cursor_position.lock().unwrap() = position.into().to_physical(scale_factor);
    Ok(())
  }

  fn set_ignore_cursor_events(&self, ignore: bool) -> Result<()> {
    self.state.lock().unwrap().ignore_cursor_events = ignore;
    Ok(())
  }

  fn start_dragging(&self) -> Result<()> {
    Ok(())
  }

  fn start_resize_dragging(&self, direction: tauri_runtime::ResizeDirection) -> Result<()> {
    Ok(())
  }

  fn set_progress_bar(&self, progress_state: ProgressBarState) -> Result<()> {
    self.state.lock().unwrap().progress_bar = Some(ProgressBarSnapshot {
      status: progress_state.status,
      progress: progress_state.progress,
      desktop_filename: progress_state.desktop_filename,
    });
    Ok(())
  }

  fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>) -> Result<()> {
    self.state.lock().unwrap().badge_count = count;
    Ok(())
  }

  fn set_badge_label(&self, label: Option<String>) -> Result<()> {
    self.state.lock().unwrap().badge_label = label;
    Ok(())
  }

  fn set_overlay_icon(&self, icon: Option<Icon<'_>>) -> Result<()> {
    self.state.lock().unwrap().overlay_icon = icon.map(owned_icon);
    Ok(())
  }

  fn set_title_bar_style(&self, style: TitleBarStyle) -> Result<()> {
    self.state.lock().unwrap().title_bar_style = style;
    Ok(())
  }

  fn set_traffic_light_position(&self, position: Position) -> Result<()> {
    self.state.lock().unwrap().traffic_light_position = Some(position);
    Ok(())
  }

  fn set_theme(&self, theme: Option<Theme>) -> Result<()> {
    self.state.lock().unwrap().theme = theme;
    self.emit(WindowEvent::ThemeChanged(theme.unwrap_or(Theme::Light)));
    Ok(())
  }

  fn set_enabled(&self, enabled: bool) -> Result<()> {
    self.state.lock().unwrap().enabled = enabled;
    Ok(())
  }

  fn is_enabled(&self) -> Result<bool> {
    Ok(self.state.lock().unwrap().enabled)
  }

  fn is_always_on_top(&self) -> Result<bool> {
    Ok(self.state.lock().unwrap().always_on_top)
  }

  fn set_background_color(&self, color: Option<Color>) -> Result<()> {
    self.state.lock().unwrap().background_color = color;
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct EventProxy {
  context: RuntimeContext,
}

impl<T: UserEvent> EventLoopProxy<T> for EventProxy {
  fn send_event(&self, event: T) -> Result<()> {
    let message = Message::UserEvent(Box::new(event));
    if self.context.is_running.load(Ordering::Relaxed) {
      self
        .context
        .run_tx
        .send(message)
        .map_err(|_| Error::EventLoopClosed)
    } else {
      // queue the event so it is delivered once the event loop starts running
      self
        .context
        .run_tx
        .try_send(message)
        .map_err(|_| Error::EventLoopClosed)
    }
  }
}

#[derive(Debug)]
pub struct MockRuntime {
  is_running: Arc<AtomicBool>,
  pub context: RuntimeContext,
  run_rx: Receiver<Message>,
}

impl MockRuntime {
  fn init() -> Self {
    let is_running = Arc::new(AtomicBool::new(false));
    let (tx, rx) = sync_channel(256);
    let context = RuntimeContext {
      is_running: is_running.clone(),
      windows: Default::default(),
      app_theme: Default::default(),
      cursor_position: Default::default(),
      device_event_filter: Default::default(),
      run_tx: tx,
      main_thread_id: std::thread::current().id(),
      next_window_id: Default::default(),
      next_webview_id: Default::default(),
      next_window_event_id: Default::default(),
      next_webview_event_id: Default::default(),
    };
    Self {
      is_running,
      context,
      run_rx: rx,
    }
  }
}

impl<T: UserEvent> Runtime<T> for MockRuntime {
  type WindowDispatcher = MockWindowDispatcher;
  type WebviewDispatcher = MockWebviewDispatcher;
  type Handle = MockRuntimeHandle;
  type EventLoopProxy = EventProxy;

  fn new(_args: RuntimeInitArgs) -> Result<Self> {
    Ok(Self::init())
  }

  #[cfg(any(
    windows,
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn new_any_thread(_args: RuntimeInitArgs) -> Result<Self> {
    Ok(Self::init())
  }

  fn create_proxy(&self) -> EventProxy {
    EventProxy {
      context: self.context.clone(),
    }
  }

  fn handle(&self) -> Self::Handle {
    MockRuntimeHandle {
      context: self.context.clone(),
    }
  }

  fn create_window<F: Fn(RawWindow<'_>) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self>,
    _after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self>> {
    self.context.create_window_impl(pending)
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self>,
  ) -> Result<DetachedWebview<T, Self>> {
    self.context.create_webview_impl(window_id, pending)
  }

  fn primary_monitor(&self) -> Option<Monitor> {
    Some(mock_monitor())
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Option<Monitor> {
    monitor_from_point(x, y)
  }

  fn available_monitors(&self) -> Vec<Monitor> {
    vec![mock_monitor()]
  }

  fn set_theme(&self, theme: Option<Theme>) {
    self.context.set_app_theme(theme);
  }

  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn set_activation_policy(&mut self, activation_policy: tauri_runtime::ActivationPolicy) {}

  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn set_dock_visibility(&mut self, visible: bool) {}

  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn show(&self) {}

  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn hide(&self) {}

  fn set_device_event_filter(&mut self, filter: DeviceEventFilter) {
    *self.context.device_event_filter.lock().unwrap() = filter;
  }

  #[cfg(any(
    target_os = "macos",
    windows,
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn run_iteration<F: FnMut(RunEvent<T>) + 'static>(&mut self, mut callback: F) {
    while let Ok(message) = self.run_rx.try_recv() {
      if process_message(&self.context, message, &mut callback).is_some() {
        break;
      }
    }
    callback(RunEvent::MainEventsCleared);
  }

  fn run_return<F: FnMut(RunEvent<T>) + 'static>(self, mut callback: F) -> i32 {
    self.is_running.store(true, Ordering::Relaxed);
    callback(RunEvent::Ready);

    let exit_code = loop {
      match self.run_rx.recv_timeout(Duration::from_millis(16)) {
        Ok(message) => {
          if let Some(code) = process_message(&self.context, message, &mut callback) {
            break code;
          }
        }
        Err(RecvTimeoutError::Timeout) => {}
        // cannot happen while the context holds a sender, but break instead of panicking
        Err(RecvTimeoutError::Disconnected) => break 0,
      }

      callback(RunEvent::MainEventsCleared);
    };

    self.is_running.store(false, Ordering::Relaxed);

    callback(RunEvent::Exit);

    exit_code
  }

  fn run<F: FnMut(RunEvent<T>) + 'static>(self, callback: F) {
    self.run_return(callback);
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    Ok(*self.context.cursor_position.lock().unwrap())
  }
}
