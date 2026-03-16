// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::too_many_arguments)]

use cef::{CefString, ImplCommandLine, ImplTaskRunner};
use tauri_runtime::{
  Cookie, DeviceEventFilter, EventLoopProxy, Icon, InitAttribute, ProgressBarState, Result,
  RunEvent, Runtime, RuntimeHandle, RuntimeInitArgs, UserAttentionType, UserEvent, WebviewDispatch,
  WebviewEventId, WindowDispatch, WindowEventId,
  dpi::{PhysicalPosition, PhysicalSize, Position, Rect, Size},
  monitor::Monitor,
  webview::{DetachedWebview, PendingWebview, WebviewAttributes},
  window::{
    CursorIcon, DetachedWindow, DetachedWindowWebview, PendingWindow, RawWindow, WebviewEvent,
    WindowBuilder, WindowBuilderBase, WindowEvent, WindowId,
  },
};

#[cfg(target_os = "macos")]
use tauri_utils::TitleBarStyle;
use tauri_utils::{
  Theme,
  config::{Color, WindowConfig},
};
use url::Url;

#[cfg(windows)]
use windows::Win32::Foundation::HWND;

use dioxus_debug_cell::RefCell;
use std::{
  collections::HashMap,
  fmt,
  fs::create_dir_all,
  sync::{
    Arc, Mutex,
    atomic::AtomicBool,
    mpsc::{Sender, channel},
  },
  thread::{self, ThreadId},
};

#[cfg(target_os = "macos")]
use crate::application::AppDelegateEvent;
use crate::cef_webview::CefWebview;

mod cef_impl;
mod cef_webview;
mod utils;

type DevToolsProtocolHandler = dyn Fn(DevToolsProtocol) + Send + Sync;

pub fn webview_version() -> Result<String> {
  Ok(format!(
    "{}.{}.{}.{}",
    cef_dll_sys::CHROME_VERSION_MAJOR,
    cef_dll_sys::CHROME_VERSION_MINOR,
    cef_dll_sys::CHROME_VERSION_PATCH,
    cef_dll_sys::CHROME_VERSION_BUILD
  ))
}

#[macro_export]
macro_rules! getter {
  ($self: ident, $rx: expr, $message: expr) => {{
    $crate::send_user_message(&$self.context, $message)?;
    $rx
      .recv()
      .map_err(|_| tauri_runtime::Error::FailedToReceiveMessage)
  }};
}

macro_rules! webview_getter {
  ($self: ident, $message_variant: path) => {{
    let (tx, rx) = channel();
    getter!(
      $self,
      rx,
      Message::Webview {
        window_id: *$self.window_id.lock().unwrap(),
        webview_id: $self.webview_id,
        message: $message_variant(tx)
      }
    )
  }};
}

macro_rules! window_getter {
  ($self: ident, $message_variant: path) => {{
    let (tx, rx) = channel();
    getter!(
      $self,
      rx,
      Message::Window {
        window_id: $self.window_id,
        message: $message_variant(tx)
      }
    )
  }};
}

type AfterWindowCreation = Box<dyn Fn(RawWindow) + Send + 'static>;

enum Message<T: UserEvent + 'static> {
  Task(Box<dyn FnOnce() + Send>),
  CreateWindow {
    window_id: WindowId,
    webview_id: u32,
    pending: Box<PendingWindow<T, CefRuntime<T>>>,
    after_window_creation: Option<AfterWindowCreation>,
  },
  CreateWebview {
    window_id: WindowId,
    webview_id: u32,
    pending: Box<PendingWebview<T, CefRuntime<T>>>,
  },
  Window {
    window_id: WindowId,
    message: WindowMessage,
  },
  Webview {
    window_id: WindowId,
    webview_id: u32,
    message: WebviewMessage,
  },
  RequestExit(i32),
  UserEvent(T),
  Noop,
}

enum WindowMessage {
  Close,
  Destroy,
  AddEventListener(WindowEventId, Box<dyn Fn(&WindowEvent) + Send>),
  // Getters
  ScaleFactor(Sender<Result<f64>>),
  InnerPosition(Sender<Result<PhysicalPosition<i32>>>),
  OuterPosition(Sender<Result<PhysicalPosition<i32>>>),
  InnerSize(Sender<Result<PhysicalSize<u32>>>),
  OuterSize(Sender<Result<PhysicalSize<u32>>>),
  IsFullscreen(Sender<Result<bool>>),
  IsMinimized(Sender<Result<bool>>),
  IsMaximized(Sender<Result<bool>>),
  IsFocused(Sender<Result<bool>>),
  IsDecorated(Sender<Result<bool>>),
  IsResizable(Sender<Result<bool>>),
  IsMaximizable(Sender<Result<bool>>),
  IsMinimizable(Sender<Result<bool>>),
  IsClosable(Sender<Result<bool>>),
  IsVisible(Sender<Result<bool>>),
  Title(Sender<Result<String>>),
  CurrentMonitor(Sender<Result<Option<Monitor>>>),
  PrimaryMonitor(Sender<Result<Option<Monitor>>>),
  MonitorFromPoint(Sender<Result<Option<Monitor>>>, f64, f64),
  AvailableMonitors(Sender<Result<Vec<Monitor>>>),
  Theme(Sender<Result<Theme>>),
  IsEnabled(Sender<Result<bool>>),
  IsAlwaysOnTop(Sender<Result<bool>>),
  RawWindowHandle(
    Sender<
      std::result::Result<raw_window_handle::WindowHandle<'static>, raw_window_handle::HandleError>,
    >,
  ),
  // Setters
  Center,
  RequestUserAttention(Option<UserAttentionType>),
  SetEnabled(bool),
  SetResizable(bool),
  SetMaximizable(bool),
  SetMinimizable(bool),
  SetClosable(bool),
  SetTitle(String),
  Maximize,
  Unmaximize,
  Minimize,
  Unminimize,
  Show,
  Hide,
  SetDecorations(bool),
  SetShadow(bool),
  SetAlwaysOnBottom(bool),
  SetAlwaysOnTop(bool),
  SetVisibleOnAllWorkspaces(bool),
  SetContentProtected(bool),
  SetSize(Size),
  SetMinSize(Option<Size>),
  SetMaxSize(Option<Size>),
  SetSizeConstraints(tauri_runtime::window::WindowSizeConstraints),
  SetPosition(Position),
  SetFullscreen(bool),
  #[cfg(target_os = "macos")]
  SetSimpleFullscreen(bool),
  SetFocus,
  SetFocusable(bool),
  SetIcon(Icon<'static>),
  SetSkipTaskbar(bool),
  SetCursorGrab(bool),
  SetCursorVisible(bool),
  SetCursorIcon(CursorIcon),
  SetCursorPosition(Position),
  SetIgnoreCursorEvents(bool),
  SetProgressBar(ProgressBarState),
  SetBadgeCount(Option<i64>, Option<String>),
  SetBadgeLabel(Option<String>),
  SetOverlayIcon(Option<Icon<'static>>),
  SetTitleBarStyle(tauri_utils::TitleBarStyle),
  SetTrafficLightPosition(Position),
  SetTheme(Option<Theme>),
  SetBackgroundColor(Option<Color>),
  StartDragging,
  StartResizeDragging(tauri_runtime::ResizeDirection),
}

pub enum WebviewMessage {
  AddEventListener(WebviewEventId, Box<dyn Fn(&WebviewEvent) + Send>),
  EvaluateScript(String),
  CookiesForUrl(Url, Sender<Result<Vec<Cookie<'static>>>>),
  Cookies(Sender<Result<Vec<Cookie<'static>>>>),
  SetCookie(Cookie<'static>),
  DeleteCookie(Cookie<'static>),
  Navigate(Url),
  Reload,
  GoBack,
  CanGoBack(Sender<Result<bool>>),
  GoForward,
  CanGoForward(Sender<Result<bool>>),
  Print,
  Close,
  Show,
  Hide,
  SetPosition(Position),
  SetSize(Size),
  SetBounds(Rect),
  SetFocus,
  Reparent(WindowId, Sender<Result<()>>),
  SetAutoResize(bool),
  SetZoom(f64),
  SetBackgroundColor(Option<Color>),
  ClearAllBrowsingData,
  // Getters
  Url(Sender<Result<String>>),
  Bounds(Sender<Result<Rect>>),
  Position(Sender<Result<PhysicalPosition<i32>>>),
  Size(Sender<Result<PhysicalSize<u32>>>),
  WithWebview(Box<dyn FnOnce(Box<dyn std::any::Any>) + Send>),
  // Devtools
  #[cfg(any(debug_assertions, feature = "devtools"))]
  OpenDevTools,
  #[cfg(any(debug_assertions, feature = "devtools"))]
  CloseDevTools,
  #[cfg(any(debug_assertions, feature = "devtools"))]
  IsDevToolsOpen(Sender<bool>),
  SendDevToolsMessage(Vec<u8>, Sender<Result<()>>),
  OnDevToolsProtocol(Arc<DevToolsProtocolHandler>, Sender<Result<()>>),
}

/// A DevTools protocol message delivered to [`on_dev_tools_protocol`](CefWebviewDispatcher::on_dev_tools_protocol) callbacks.
#[derive(Debug, Clone)]
pub enum DevToolsProtocol {
  /// Raw UTF-8 encoded JSON message (method result or event).
  Message(Vec<u8>),
  /// DevTools protocol event with method name and params.
  Event { method: String, params: Vec<u8> },
  /// Result of a DevTools method call.
  MethodResult {
    message_id: i32,
    success: bool,
    result: Vec<u8>,
  },
}

impl<T: UserEvent> Clone for Message<T> {
  fn clone(&self) -> Self {
    match self {
      Self::UserEvent(t) => Self::UserEvent(t.clone()),
      _ => unimplemented!(),
    }
  }
}

#[derive(Clone)]
pub(crate) struct AppWebview {
  pub webview_id: u32,
  #[allow(dead_code)]
  pub label: String,
  pub inner: CefWebview,
  // browser_view.browser is null on the scheme handler factory,
  // so we need to use the browser_id to identify the browser
  pub browser_id: Arc<RefCell<i32>>,
  pub bounds: Arc<Mutex<Option<WebviewBounds>>>,
  #[allow(unused)]
  pub devtools_enabled: bool,
  pub uri_scheme_protocols:
    Arc<HashMap<String, Arc<Box<tauri_runtime::webview::UriSchemeProtocolHandler>>>>,
  #[allow(dead_code)]
  pub initialization_scripts: Arc<Vec<cef_impl::CefInitScript>>,
  pub devtools_protocol_handlers: Arc<Mutex<Vec<Arc<DevToolsProtocolHandler>>>>,
  /// Keeps the DevTools message observer registered. Dropping this unregisters the observer.
  #[allow(dead_code)]
  pub devtools_observer_registration: Arc<Mutex<Option<cef::Registration>>>,
  pub webview_attributes: Arc<RefCell<WebviewAttributes>>,
}

#[derive(Debug, Clone)]
pub struct WebviewBounds {
  pub x_rate: f32,
  pub y_rate: f32,
  pub width_rate: f32,
  pub height_rate: f32,
}

pub type WindowEventHandler = Box<dyn Fn(&WindowEvent) + Send>;
pub type WindowEventListeners = Arc<Mutex<HashMap<WindowEventId, WindowEventHandler>>>;
pub type WebviewEventHandler = Box<dyn Fn(&tauri_runtime::window::WebviewEvent) + Send>;
pub type WebviewEventListeners =
  Arc<Mutex<HashMap<u32, Arc<Mutex<HashMap<tauri_runtime::WebviewEventId, WebviewEventHandler>>>>>>;

pub(crate) enum AppWindowKind {
  Window(cef::Window),
  BrowserWindow,
}

pub(crate) struct AppWindow {
  pub label: String,
  pub window: AppWindowKind,
  pub force_close: Arc<AtomicBool>,
  pub attributes: Arc<RefCell<CefWindowBuilder>>,
  pub webviews: Vec<AppWebview>,
  pub window_event_listeners: WindowEventListeners,
  pub webview_event_listeners: WebviewEventListeners,
}

impl AppWindow {
  fn window(&self) -> Option<cef::Window> {
    match &self.window {
      AppWindowKind::Window(window) => Some(window.clone()),
      AppWindowKind::BrowserWindow => None,
    }
  }
}

#[derive(Clone)]
pub struct RuntimeContext<T: UserEvent> {
  main_thread_task_runner: cef::TaskRunner,
  main_thread_id: ThreadId,
  cef_context: cef_impl::Context<T>,
}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Send for RuntimeContext<T> {}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for RuntimeContext<T> {}

impl<T: UserEvent> RuntimeContext<T> {
  fn post_message(&self, message: Message<T>) -> Result<()> {
    if thread::current().id() == self.main_thread_id {
      // Already on main thread, execute directly
      cef_impl::handle_message(&self.cef_context, message);
      Ok(())
    } else {
      // Post to main thread via TaskRunner
      self
        .main_thread_task_runner
        .post_task(Some(&mut cef_impl::SendMessageTask::new(
          self.cef_context.clone(),
          Arc::new(RefCell::new(message)),
        )));
      Ok(())
    }
  }

  fn create_window<F: Fn(RawWindow) + Send + 'static>(
    &self,
    pending: PendingWindow<T, CefRuntime<T>>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, CefRuntime<T>>> {
    let label = pending.label.clone();
    let context = self.clone();
    let window_id = self.cef_context.next_window_id();
    let (webview_id, use_https_scheme) = pending
      .webview
      .as_ref()
      .map(|w| {
        (
          Some(context.cef_context.next_webview_id()),
          w.webview_attributes.use_https_scheme,
        )
      })
      .unwrap_or((None, false));

    self.post_message(Message::CreateWindow {
      window_id,
      webview_id: webview_id.unwrap_or_default(),
      pending: Box::new(pending),
      after_window_creation: after_window_creation.map(|f| Box::new(f) as AfterWindowCreation),
    })?;

    let dispatcher = CefWindowDispatcher {
      window_id,
      context: self.clone(),
    };

    let detached_webview = webview_id.map(|id| {
      let webview = DetachedWebview {
        label: label.clone(),
        dispatcher: CefWebviewDispatcher {
          window_id: Arc::new(Mutex::new(window_id)),
          webview_id: id,
          context: self.clone(),
        },
      };
      DetachedWindowWebview {
        webview,
        use_https_scheme,
      }
    });

    Ok(DetachedWindow {
      id: window_id,
      label,
      dispatcher,
      webview: detached_webview,
    })
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, CefRuntime<T>>,
  ) -> Result<DetachedWebview<T, CefRuntime<T>>> {
    let label = pending.label.clone();
    let webview_id = self.cef_context.next_webview_id();

    self.post_message(Message::CreateWebview {
      window_id,
      webview_id,
      pending: Box::new(pending),
    })?;

    let dispatcher = CefWebviewDispatcher {
      window_id: Arc::new(Mutex::new(window_id)),
      webview_id,
      context: self.clone(),
    };

    Ok(DetachedWebview { label, dispatcher })
  }
}

// Mirrors tauri-runtime-wry's send_user_message behavior: if we're already on the main
// thread, handle the message immediately; otherwise, post it to the main thread.
pub(crate) fn send_user_message<T: UserEvent>(
  context: &RuntimeContext<T>,
  message: Message<T>,
) -> Result<()> {
  if thread::current().id() == context.main_thread_id {
    cef_impl::handle_message(&context.cef_context, message);
  } else {
    context
      .main_thread_task_runner
      .post_task(Some(&mut cef_impl::SendMessageTask::new(
        context.cef_context.clone(),
        Arc::new(RefCell::new(message)),
      )));
  }
  Ok(())
}

impl<T: UserEvent> fmt::Debug for RuntimeContext<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("RuntimeContext").finish()
  }
}

#[derive(Debug, Clone)]
pub struct CefRuntimeHandle<T: UserEvent> {
  context: RuntimeContext<T>,
}

impl<T: UserEvent> RuntimeHandle<T> for CefRuntimeHandle<T> {
  type Runtime = CefRuntime<T>;

  fn create_proxy(&self) -> <Self::Runtime as Runtime<T>>::EventLoopProxy {
    EventProxy {
      context: self.context.clone(),
    }
  }

  #[cfg(target_os = "macos")]
  fn set_activation_policy(
    &self,
    _activation_policy: tauri_runtime::ActivationPolicy,
  ) -> Result<()> {
    Ok(())
  }

  #[cfg(target_os = "macos")]
  fn set_dock_visibility(&self, _visible: bool) -> Result<()> {
    Ok(())
  }

  fn request_exit(&self, code: i32) -> Result<()> {
    // Request exit by posting a task to quit the message loop
    self.context.post_message(Message::RequestExit(code))
  }

  /// Create a new webview window.
  fn create_window<F: Fn(RawWindow<'_>) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self::Runtime>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    self.context.create_window(pending, after_window_creation)
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>> {
    self.context.create_webview(window_id, pending)
  }

  /// Run a task on the main thread.
  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.context.post_message(Message::Task(Box::new(f)))
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
    crate::cef_impl::get_primary_monitor()
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Option<Monitor> {
    crate::cef_impl::get_monitor_from_point(x, y)
  }

  fn available_monitors(&self) -> Vec<Monitor> {
    crate::cef_impl::get_available_monitors()
  }

  fn set_theme(&self, _theme: Option<Theme>) {}

  /// Shows the application, but does not automatically focus it.
  #[cfg(target_os = "macos")]
  fn show(&self) -> Result<()> {
    self.context.post_message(Message::Task(Box::new(|| {
      cef_impl::set_application_visibility(true);
    })))
  }

  /// Hides the application.
  #[cfg(target_os = "macos")]
  fn hide(&self) -> Result<()> {
    self.context.post_message(Message::Task(Box::new(|| {
      cef_impl::set_application_visibility(false);
    })))
  }

  fn set_device_event_filter(&self, _filter: DeviceEventFilter) {}

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
    F: FnOnce(&mut jni::JNIEnv, &jni::objects::JObject, &jni::objects::JObject) + Send + 'static,
  {
    todo!()
  }

  #[cfg(any(target_os = "macos", target_os = "ios"))]
  fn fetch_data_store_identifiers<F: FnOnce(Vec<[u8; 16]>) + Send + 'static>(
    &self,
    _cb: F,
  ) -> Result<()> {
    todo!()
  }

  #[cfg(any(target_os = "macos", target_os = "ios"))]
  fn remove_data_store<F: FnOnce(Result<()>) + Send + 'static>(
    &self,
    _uuid: [u8; 16],
    _cb: F,
  ) -> Result<()> {
    todo!()
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    Ok(PhysicalPosition::new(0.0, 0.0))
  }
}

#[derive(Debug, Clone)]
pub struct CefWebviewDispatcher<T: UserEvent> {
  window_id: Arc<Mutex<WindowId>>,
  webview_id: u32,
  context: RuntimeContext<T>,
}

#[derive(Debug, Clone)]
pub struct CefWindowDispatcher<T: UserEvent> {
  window_id: WindowId,
  context: RuntimeContext<T>,
}

#[derive(Debug, Clone)]
pub struct CefWindowBuilder {
  title: Option<String>,
  position: Option<Position>,
  inner_size: Option<Size>,
  min_inner_size: Option<Size>,
  max_inner_size: Option<Size>,
  inner_size_constraints: Option<tauri_runtime::window::WindowSizeConstraints>,
  center: bool,
  prevent_overflow: Option<Size>,
  resizable: Option<bool>,
  maximizable: Option<bool>,
  minimizable: Option<bool>,
  closable: Option<bool>,
  fullscreen: Option<bool>,
  focused: Option<bool>,
  focusable: Option<bool>,
  maximized: Option<bool>,
  visible: Option<bool>,
  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  transparent: Option<bool>,
  decorations: Option<bool>,
  always_on_bottom: Option<bool>,
  always_on_top: Option<bool>,
  visible_on_all_workspaces: Option<bool>,
  content_protected: Option<bool>,
  skip_taskbar: Option<bool>,
  shadow: Option<bool>,
  theme: Option<Theme>,
  background_color: Option<tauri_utils::config::Color>,
  #[cfg(target_os = "macos")]
  title_bar_style: Option<TitleBarStyle>,
  #[cfg(target_os = "macos")]
  traffic_light_position: Option<Position>,
  #[cfg(target_os = "macos")]
  hidden_title: Option<bool>,
  #[cfg(target_os = "macos")]
  tabbing_identifier: Option<String>,
  #[cfg(windows)]
  window_classname: Option<String>,
  #[cfg(windows)]
  owner: Option<HWND>,
  #[cfg(windows)]
  parent: Option<HWND>,
  #[cfg(windows)]
  drag_and_drop: Option<bool>,
  has_icon: bool,
  icon: Option<Icon<'static>>,
  browser_window: bool,
}

impl Default for CefWindowBuilder {
  fn default() -> Self {
    Self {
      title: None,
      position: None,
      inner_size: None,
      min_inner_size: None,
      max_inner_size: None,
      inner_size_constraints: None,
      center: false,
      prevent_overflow: None,
      resizable: None,
      maximizable: None,
      minimizable: None,
      closable: None,
      fullscreen: None,
      focused: Some(true),
      focusable: None,
      maximized: None,
      visible: Some(true),
      #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
      transparent: None,
      decorations: Some(true),
      always_on_bottom: None,
      always_on_top: None,
      visible_on_all_workspaces: None,
      content_protected: None,
      skip_taskbar: None,
      shadow: None,
      theme: None,
      background_color: None,
      #[cfg(target_os = "macos")]
      title_bar_style: None,
      #[cfg(target_os = "macos")]
      traffic_light_position: None,
      #[cfg(target_os = "macos")]
      hidden_title: None,
      #[cfg(target_os = "macos")]
      tabbing_identifier: None,
      #[cfg(windows)]
      window_classname: None,
      #[cfg(windows)]
      owner: None,
      #[cfg(windows)]
      parent: None,
      #[cfg(windows)]
      drag_and_drop: None,
      has_icon: false,
      icon: None,
      browser_window: false,
    }
  }
}

impl CefWindowBuilder {
  pub fn browser_window(mut self) -> Self {
    self.browser_window = true;
    self
  }
}

impl WindowBuilderBase for CefWindowBuilder {}

impl WindowBuilder for CefWindowBuilder {
  fn new() -> Self {
    Self::default().title("Tauri App")
  }

  fn with_config(config: &WindowConfig) -> Self {
    let mut builder = Self::default();

    builder = builder
      .title(config.title.to_string())
      .inner_size(config.width, config.height)
      .focused(config.focus)
      .focusable(config.focusable)
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
      .theme(config.theme)
      .closable(config.closable)
      .maximizable(config.maximizable)
      .minimizable(config.minimizable)
      .shadow(config.shadow);

    let mut constraints = tauri_runtime::window::WindowSizeConstraints::default();
    if let Some(min_width) = config.min_width {
      constraints.min_width = Some(tauri_runtime::dpi::LogicalUnit::new(min_width).into());
    }
    if let Some(min_height) = config.min_height {
      constraints.min_height = Some(tauri_runtime::dpi::LogicalUnit::new(min_height).into());
    }
    if let Some(max_width) = config.max_width {
      constraints.max_width = Some(tauri_runtime::dpi::LogicalUnit::new(max_width).into());
    }
    if let Some(max_height) = config.max_height {
      constraints.max_height = Some(tauri_runtime::dpi::LogicalUnit::new(max_height).into());
    }
    builder = builder.inner_size_constraints(constraints);

    if let Some(color) = config.background_color {
      builder = builder.background_color(color);
    }

    if let (Some(x), Some(y)) = (config.x, config.y) {
      builder = builder.position(x, y);
    }

    if config.center {
      builder = builder.center();
    }

    #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
    {
      builder = builder.transparent(config.transparent);
    }

    #[cfg(target_os = "macos")]
    {
      builder = builder
        .hidden_title(config.hidden_title)
        .title_bar_style(config.title_bar_style);
      if let Some(identifier) = &config.tabbing_identifier {
        builder = builder.tabbing_identifier(identifier);
      }
      if let Some(position) = &config.traffic_light_position {
        builder = builder.traffic_light_position(tauri_runtime::dpi::LogicalPosition::new(
          position.x, position.y,
        ));
      }
    }

    #[cfg(windows)]
    {
      if let Some(window_classname) = &config.window_classname {
        builder = builder.window_classname(window_classname);
      }
    }

    builder
  }

  fn center(mut self) -> Self {
    self.center = true;
    self
  }

  fn position(mut self, x: f64, y: f64) -> Self {
    self.position = Some(Position::Logical(tauri_runtime::dpi::LogicalPosition::new(
      x, y,
    )));
    self
  }

  fn inner_size(mut self, width: f64, height: f64) -> Self {
    self.inner_size = Some(Size::Logical(tauri_runtime::dpi::LogicalSize::new(
      width, height,
    )));
    self
  }

  fn min_inner_size(mut self, min_width: f64, min_height: f64) -> Self {
    self.min_inner_size = Some(Size::Logical(tauri_runtime::dpi::LogicalSize::new(
      min_width, min_height,
    )));
    self
  }

  fn max_inner_size(mut self, max_width: f64, max_height: f64) -> Self {
    self.max_inner_size = Some(Size::Logical(tauri_runtime::dpi::LogicalSize::new(
      max_width, max_height,
    )));
    self
  }

  fn inner_size_constraints(
    mut self,
    constraints: tauri_runtime::window::WindowSizeConstraints,
  ) -> Self {
    self.inner_size_constraints = Some(constraints);
    self
  }

  fn prevent_overflow(mut self) -> Self {
    self.prevent_overflow = Some(Size::Physical(PhysicalSize::new(0, 0)));
    self
  }

  fn prevent_overflow_with_margin(mut self, margin: Size) -> Self {
    self.prevent_overflow = Some(margin);
    self
  }

  fn resizable(mut self, resizable: bool) -> Self {
    self.resizable = Some(resizable);
    self
  }

  fn maximizable(mut self, maximizable: bool) -> Self {
    self.maximizable = Some(maximizable);
    self
  }

  fn minimizable(mut self, minimizable: bool) -> Self {
    self.minimizable = Some(minimizable);
    self
  }

  fn closable(mut self, closable: bool) -> Self {
    self.closable = Some(closable);
    self
  }

  fn title<S: Into<String>>(mut self, title: S) -> Self {
    self.title = Some(title.into());
    self
  }

  fn fullscreen(mut self, fullscreen: bool) -> Self {
    self.fullscreen = Some(fullscreen);
    self
  }

  fn focused(mut self, focused: bool) -> Self {
    self.focused = Some(focused);
    self
  }

  fn focusable(mut self, focusable: bool) -> Self {
    self.focusable = Some(focusable);
    self
  }

  fn maximized(mut self, maximized: bool) -> Self {
    self.maximized = Some(maximized);
    self
  }

  fn visible(mut self, visible: bool) -> Self {
    self.visible = Some(visible);
    self
  }

  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  fn transparent(self, transparent: bool) -> Self {
    let mut s = self;
    s.transparent = Some(transparent);
    s
  }

  fn decorations(mut self, decorations: bool) -> Self {
    self.decorations = Some(decorations);
    self
  }

  fn always_on_bottom(mut self, always_on_bottom: bool) -> Self {
    self.always_on_bottom = Some(always_on_bottom);
    self
  }

  fn always_on_top(mut self, always_on_top: bool) -> Self {
    self.always_on_top = Some(always_on_top);
    self
  }

  fn visible_on_all_workspaces(mut self, visible_on_all_workspaces: bool) -> Self {
    self.visible_on_all_workspaces = Some(visible_on_all_workspaces);
    self
  }

  fn content_protected(mut self, protected: bool) -> Self {
    self.content_protected = Some(protected);
    self
  }

  fn icon(mut self, icon: Icon<'_>) -> Result<Self> {
    self.has_icon = true;
    self.icon.replace(icon.into_owned());
    Ok(self)
  }

  fn skip_taskbar(mut self, skip: bool) -> Self {
    self.skip_taskbar = Some(skip);
    self
  }

  fn window_classname<S: Into<String>>(self, classname: S) -> Self {
    #[cfg(windows)]
    {
      let mut s = self;
      s.window_classname = Some(classname.into());
      s
    }
    #[cfg(not(windows))]
    {
      let _classname = classname;
      self
    }
  }

  fn shadow(mut self, enable: bool) -> Self {
    self.shadow = Some(enable);
    self
  }

  #[cfg(windows)]
  fn owner(mut self, owner: HWND) -> Self {
    self.owner = Some(owner);
    self
  }

  #[cfg(windows)]
  fn parent(mut self, parent: HWND) -> Self {
    self.parent = Some(parent);
    self
  }

  #[cfg(target_os = "macos")]
  fn parent(self, _parent: *mut std::ffi::c_void) -> Self {
    self
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn transient_for(self, _parent: &impl gtk::glib::IsA<gtk::Window>) -> Self {
    self
  }

  #[cfg(windows)]
  fn drag_and_drop(mut self, enabled: bool) -> Self {
    self.drag_and_drop = Some(enabled);
    self
  }

  #[cfg(target_os = "macos")]
  fn title_bar_style(mut self, style: TitleBarStyle) -> Self {
    self.title_bar_style = Some(style);
    self
  }

  #[cfg(target_os = "macos")]
  fn traffic_light_position<P: Into<Position>>(mut self, position: P) -> Self {
    self.traffic_light_position = Some(position.into());
    self
  }

  #[cfg(target_os = "macos")]
  fn hidden_title(mut self, hidden: bool) -> Self {
    self.hidden_title = Some(hidden);
    self
  }

  #[cfg(target_os = "macos")]
  fn tabbing_identifier(mut self, identifier: &str) -> Self {
    self.tabbing_identifier = Some(identifier.into());
    self
  }

  fn theme(mut self, theme: Option<Theme>) -> Self {
    self.theme = theme;
    self
  }

  fn has_icon(&self) -> bool {
    self.has_icon
  }

  fn get_theme(&self) -> Option<Theme> {
    self.theme
  }

  fn background_color(self, color: tauri_utils::config::Color) -> Self {
    let mut s = self;
    s.background_color = Some(color);
    s
  }
}

/// CEF-specific webview APIs.
impl<T: UserEvent> CefWebviewDispatcher<T> {
  /// Send a message to the DevTools agent. The message should be a UTF-8 encoded JSON
  /// string following the Chrome DevTools Protocol format.
  pub fn send_dev_tools_message(&self, message: &[u8]) -> Result<()> {
    let (tx, rx) = channel();
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SendDevToolsMessage(message.to_vec(), tx),
    })?;
    rx.recv()
      .map_err(|_| tauri_runtime::Error::FailedToReceiveMessage)?
  }

  /// Register a callback to receive DevTools protocol messages. Messages include
  /// both method results and events from the DevTools agent.
  pub fn on_dev_tools_protocol<F: Fn(DevToolsProtocol) + Send + Sync + 'static>(
    &self,
    f: F,
  ) -> Result<()> {
    let (tx, rx) = channel();
    let handler =
      Arc::new(move |protocol: DevToolsProtocol| f(protocol)) as Arc<DevToolsProtocolHandler>;
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::OnDevToolsProtocol(handler, tx),
    })?;
    rx.recv()
      .map_err(|_| tauri_runtime::Error::FailedToReceiveMessage)?
  }
}

impl<T: UserEvent> WebviewDispatch<T> for CefWebviewDispatcher<T> {
  type Runtime = CefRuntime<T>;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.context.post_message(Message::Task(Box::new(f)))
  }

  fn on_webview_event<F: Fn(&tauri_runtime::window::WebviewEvent) + Send + 'static>(
    &self,
    f: F,
  ) -> tauri_runtime::WebviewEventId {
    let id = self.context.cef_context.next_webview_event_id();
    let _ = self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::AddEventListener(id, Box::new(f)),
    });
    id
  }

  fn with_webview<F: FnOnce(Box<dyn std::any::Any>) + Send + 'static>(&self, f: F) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::WithWebview(Box::new(f)),
    })
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn open_devtools(&self) {
    let _ = self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::OpenDevTools,
    });
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn close_devtools(&self) {
    let _ = self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::CloseDevTools,
    });
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn is_devtools_open(&self) -> Result<bool> {
    webview_getter!(self, WebviewMessage::IsDevToolsOpen)
  }

  fn set_zoom(&self, scale_factor: f64) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetZoom(scale_factor),
    })
  }

  fn eval_script<S: Into<String>>(&self, script: S) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::EvaluateScript(script.into()),
    })
  }

  fn url(&self) -> Result<String> {
    webview_getter!(self, WebviewMessage::Url)?
  }

  fn bounds(&self) -> Result<Rect> {
    webview_getter!(self, WebviewMessage::Bounds)?
  }

  fn position(&self) -> Result<PhysicalPosition<i32>> {
    webview_getter!(self, WebviewMessage::Position)?
  }

  fn size(&self) -> Result<PhysicalSize<u32>> {
    webview_getter!(self, WebviewMessage::Size)?
  }

  fn navigate(&self, url: Url) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::Navigate(url),
    })
  }

  fn reload(&self) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::Reload,
    })
  }

  fn go_back(&self) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::GoBack,
    })
  }

  fn can_go_back(&self) -> Result<bool> {
    webview_getter!(self, WebviewMessage::CanGoBack)?
  }

  fn go_forward(&self) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::GoForward,
    })
  }

  fn can_go_forward(&self) -> Result<bool> {
    webview_getter!(self, WebviewMessage::CanGoForward)?
  }

  fn print(&self) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::Print,
    })
  }

  fn close(&self) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::Close,
    })
  }

  fn set_bounds(&self, bounds: Rect) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetBounds(bounds),
    })
  }

  fn set_size(&self, size: Size) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetSize(size),
    })
  }

  fn set_position(&self, position: Position) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetPosition(position),
    })
  }

  fn set_focus(&self) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetFocus,
    })
  }

  fn reparent(&self, window_id: WindowId) -> Result<()> {
    let mut current_window_id = self.window_id.lock().unwrap();
    let (tx, rx) = channel();
    self.context.post_message(Message::Webview {
      window_id: *current_window_id,
      webview_id: self.webview_id,
      message: WebviewMessage::Reparent(window_id, tx),
    })?;

    rx.recv().unwrap()?;

    *current_window_id = window_id;
    Ok(())
  }

  fn cookies_for_url(&self, url: Url) -> Result<Vec<Cookie<'static>>> {
    let current_window_id = self.window_id.lock().unwrap();
    let (tx, rx) = channel();
    self.context.post_message(Message::Webview {
      window_id: *current_window_id,
      webview_id: self.webview_id,
      message: WebviewMessage::CookiesForUrl(url, tx),
    })?;

    rx.recv().unwrap()
  }

  fn cookies(&self) -> Result<Vec<Cookie<'static>>> {
    webview_getter!(self, WebviewMessage::Cookies)?
  }

  fn set_cookie(&self, cookie: Cookie<'_>) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetCookie(cookie.into_owned()),
    })
  }

  fn delete_cookie(&self, cookie: Cookie<'_>) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::DeleteCookie(cookie.into_owned()),
    })
  }

  fn set_auto_resize(&self, auto_resize: bool) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetAutoResize(auto_resize),
    })
  }

  fn clear_all_browsing_data(&self) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::ClearAllBrowsingData,
    })
  }

  fn hide(&self) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::Hide,
    })
  }

  fn show(&self) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::Show,
    })
  }

  fn set_background_color(&self, color: Option<tauri_utils::config::Color>) -> Result<()> {
    self.context.post_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetBackgroundColor(color),
    })
  }
}

impl<T: UserEvent> WindowDispatch<T> for CefWindowDispatcher<T> {
  type Runtime = CefRuntime<T>;

  type WindowBuilder = CefWindowBuilder;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.context.post_message(Message::Task(Box::new(f)))
  }

  fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) -> WindowEventId {
    let context = self.context.clone();
    let window_id = self.window_id;
    let event_id = context.cef_context.next_window_event_id();
    let handler = Box::new(f);

    // Register the listener on the main thread
    let _ = context.post_message(Message::Window {
      window_id,
      message: WindowMessage::AddEventListener(event_id, handler),
    });

    event_id
  }

  fn scale_factor(&self) -> Result<f64> {
    window_getter!(self, WindowMessage::ScaleFactor)?
  }

  fn inner_position(&self) -> Result<PhysicalPosition<i32>> {
    window_getter!(self, WindowMessage::InnerPosition)?
  }

  fn outer_position(&self) -> Result<PhysicalPosition<i32>> {
    window_getter!(self, WindowMessage::OuterPosition)?
  }

  fn inner_size(&self) -> Result<PhysicalSize<u32>> {
    window_getter!(self, WindowMessage::InnerSize)?
  }

  fn outer_size(&self) -> Result<PhysicalSize<u32>> {
    window_getter!(self, WindowMessage::OuterSize)?
  }

  fn is_fullscreen(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsFullscreen)?
  }

  fn is_minimized(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsMinimized)?
  }

  fn is_maximized(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsMaximized)?
  }

  fn is_focused(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsFocused)?
  }

  fn is_decorated(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsDecorated)?
  }

  fn is_resizable(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsResizable)?
  }

  fn is_maximizable(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsMaximizable)?
  }

  fn is_minimizable(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsMinimizable)?
  }

  fn is_closable(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsClosable)?
  }

  fn is_visible(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsVisible)?
  }

  fn title(&self) -> Result<String> {
    window_getter!(self, WindowMessage::Title)?
  }

  fn current_monitor(&self) -> Result<Option<Monitor>> {
    window_getter!(self, WindowMessage::CurrentMonitor)?
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    window_getter!(self, WindowMessage::PrimaryMonitor)?
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Result<Option<Monitor>> {
    let (tx, rx) = channel();
    getter!(
      self,
      rx,
      Message::Window {
        window_id: self.window_id,
        message: WindowMessage::MonitorFromPoint(tx, x, y)
      }
    )?
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    window_getter!(self, WindowMessage::AvailableMonitors)?
  }

  fn theme(&self) -> Result<Theme> {
    window_getter!(self, WindowMessage::Theme)?
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

  fn window_handle(
    &self,
  ) -> std::result::Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
    let (tx, rx) = channel();
    self
      .context
      .post_message(Message::Window {
        window_id: self.window_id,
        message: WindowMessage::RawWindowHandle(tx),
      })
      .map_err(|_| raw_window_handle::HandleError::Unavailable)?;
    rx.recv()
      .map_err(|_| raw_window_handle::HandleError::Unavailable)?
  }

  fn center(&self) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Center,
    })
  }

  fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::RequestUserAttention(request_type),
    })
  }

  fn create_window<F: Fn(RawWindow<'_>) + Send + 'static>(
    &mut self,
    pending: PendingWindow<T, Self::Runtime>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    self.context.create_window(pending, after_window_creation)
  }

  fn create_webview(
    &mut self,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>> {
    self.context.create_webview(self.window_id, pending)
  }

  fn set_resizable(&self, resizable: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetResizable(resizable),
    })
  }

  fn set_maximizable(&self, maximizable: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetMaximizable(maximizable),
    })
  }

  fn set_minimizable(&self, minimizable: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetMinimizable(minimizable),
    })
  }

  fn set_closable(&self, closable: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetClosable(closable),
    })
  }

  fn set_title<S: Into<String>>(&self, title: S) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetTitle(title.into()),
    })
  }

  fn maximize(&self) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Maximize,
    })
  }

  fn unmaximize(&self) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Unmaximize,
    })
  }

  fn minimize(&self) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Minimize,
    })
  }

  fn unminimize(&self) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Unminimize,
    })
  }

  fn show(&self) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Show,
    })
  }

  fn hide(&self) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Hide,
    })
  }

  fn close(&self) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Close,
    })
  }

  fn destroy(&self) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Destroy,
    })
  }

  fn set_decorations(&self, decorations: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetDecorations(decorations),
    })
  }

  fn set_shadow(&self, shadow: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetShadow(shadow),
    })
  }

  fn set_always_on_bottom(&self, always_on_bottom: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetAlwaysOnBottom(always_on_bottom),
    })
  }

  fn set_always_on_top(&self, always_on_top: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetAlwaysOnTop(always_on_top),
    })
  }

  fn set_visible_on_all_workspaces(&self, visible_on_all_workspaces: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetVisibleOnAllWorkspaces(visible_on_all_workspaces),
    })
  }

  fn set_content_protected(&self, protected: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetContentProtected(protected),
    })
  }

  fn set_size(&self, size: Size) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetSize(size),
    })
  }

  fn set_min_size(&self, size: Option<Size>) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetMinSize(size),
    })
  }

  fn set_max_size(&self, size: Option<Size>) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetMaxSize(size),
    })
  }

  fn set_position(&self, position: Position) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetPosition(position),
    })
  }

  fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetFullscreen(fullscreen),
    })
  }

  #[cfg(target_os = "macos")]
  fn set_simple_fullscreen(&self, fullscreen: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetSimpleFullscreen(fullscreen),
    })
  }

  fn set_focus(&self) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetFocus,
    })
  }

  fn set_focusable(&self, focusable: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetFocusable(focusable),
    })
  }

  fn set_icon(&self, icon: Icon<'_>) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetIcon(icon.into_owned()),
    })
  }

  fn set_skip_taskbar(&self, skip: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetSkipTaskbar(skip),
    })
  }

  fn set_cursor_grab(&self, grab: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetCursorGrab(grab),
    })
  }

  fn set_cursor_visible(&self, visible: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetCursorVisible(visible),
    })
  }

  fn set_cursor_icon(&self, icon: CursorIcon) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetCursorIcon(icon),
    })
  }

  fn set_cursor_position<Pos: Into<Position>>(&self, position: Pos) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetCursorPosition(position.into()),
    })
  }

  fn set_ignore_cursor_events(&self, ignore: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetIgnoreCursorEvents(ignore),
    })
  }

  fn start_dragging(&self) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::StartDragging,
    })
  }

  fn start_resize_dragging(&self, direction: tauri_runtime::ResizeDirection) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::StartResizeDragging(direction),
    })
  }

  fn set_progress_bar(&self, progress_state: ProgressBarState) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetProgressBar(progress_state),
    })
  }

  fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetBadgeCount(count, desktop_filename),
    })
  }

  fn set_badge_label(&self, label: Option<String>) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetBadgeLabel(label),
    })
  }

  fn set_overlay_icon(&self, icon: Option<Icon<'_>>) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetOverlayIcon(icon.map(|i| i.into_owned())),
    })
  }

  fn set_title_bar_style(&self, style: tauri_utils::TitleBarStyle) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetTitleBarStyle(style),
    })
  }

  fn set_traffic_light_position(&self, position: Position) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetTrafficLightPosition(position),
    })
  }

  fn set_size_constraints(
    &self,
    constraints: tauri_runtime::window::WindowSizeConstraints,
  ) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetSizeConstraints(constraints),
    })
  }

  fn set_theme(&self, theme: Option<Theme>) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetTheme(theme),
    })
  }

  fn set_enabled(&self, enabled: bool) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetEnabled(enabled),
    })
  }

  fn is_enabled(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsEnabled)?
  }

  fn is_always_on_top(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsAlwaysOnTop)?
  }

  fn set_background_color(&self, color: Option<tauri_utils::config::Color>) -> Result<()> {
    self.context.post_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetBackgroundColor(color),
    })
  }
}

#[derive(Clone)]
pub struct EventProxy<T: UserEvent> {
  context: RuntimeContext<T>,
}

// SAFETY: we ensure the context is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Send for EventProxy<T> {}

// SAFETY: we ensure the context is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for EventProxy<T> {}

impl<T: UserEvent> fmt::Debug for EventProxy<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("EventProxy").finish()
  }
}

impl<T: UserEvent> EventLoopProxy<T> for EventProxy<T> {
  fn send_event(&self, event: T) -> Result<()> {
    self.context.post_message(Message::UserEvent(event))
  }
}

#[derive(Debug)]
pub struct CefRuntime<T: UserEvent> {
  pub context: RuntimeContext<T>,
  event_tx: std::sync::mpsc::Sender<RunEvent<T>>,
  event_rx: std::sync::mpsc::Receiver<RunEvent<T>>,
}

#[cfg(target_os = "macos")]
fn is_cef_helper_process() -> bool {
  const HELPER_SUFFIXES: &[&str] = &[
    " Helper (GPU)",
    " Helper (Renderer)",
    " Helper (Plugin)",
    " Helper (Alerts)",
    " Helper",
  ];
  std::env::current_exe()
    .ok()
    .and_then(|p| {
      p.file_name()
        .and_then(|s| s.to_str())
        .map(|name| HELPER_SUFFIXES.iter().any(|s| name.ends_with(s)))
    })
    .unwrap_or_default()
}

impl<T: UserEvent> CefRuntime<T> {
  fn init(runtime_args: RuntimeInitArgs<RuntimeInitAttribute>) -> Self {
    let args = cef::args::Args::new();

    let (event_tx, event_rx) = channel();

    #[cfg(target_os = "macos")]
    let (_sandbox, _loader) = {
      let is_helper = is_cef_helper_process();

      #[cfg(feature = "sandbox")]
      let sandbox = if is_helper {
        let mut sandbox = cef::sandbox::Sandbox::new();
        sandbox.initialize(args.as_main_args());
        Some(sandbox)
      } else {
        None
      };
      #[cfg(not(feature = "sandbox"))]
      let sandbox = ();

      let loader =
        cef::library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), is_helper);
      assert!(loader.load());

      if !is_helper {
        let event_tx_ = event_tx.clone();
        init_ns_app(Box::new(move |event| match event {
          AppDelegateEvent::ShouldTerminate { tx } => {
            tx.send(objc2_app_kit::NSApplicationTerminateReply::TerminateCancel)
              .unwrap();
            event_tx_.send(RunEvent::Exit).unwrap();
          }
          AppDelegateEvent::OpenURLs { urls } => {
            event_tx_.send(RunEvent::Opened { urls }).unwrap();
          }
        }));
      }

      (sandbox, loader)
    };

    let _ = cef::api_hash(cef::sys::CEF_API_VERSION_LAST, 0);

    let cache_base = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
    let cache_path = cache_base.join(&runtime_args.identifier).join("cef");

    // Ensure the cache directory exists
    let _ = create_dir_all(&cache_path);

    let event_tx_ = event_tx.clone();
    let cef_context = cef_impl::Context {
      windows: Default::default(),
      callback: Arc::new(RefCell::new(Box::new(move |event| {
        event_tx_.send(event).unwrap();
      }))),
      next_webview_event_id: Default::default(),
      next_webview_id: Default::default(),
      next_window_id: Default::default(),
      next_window_event_id: Default::default(),
      scheme_handler_registry: Default::default(),
    };

    let mut command_line_args = Vec::new();
    let mut deep_link_schemes = Vec::new();
    for arg in runtime_args.platform_specific_attributes {
      match arg {
        RuntimeInitAttribute::CommandLineArgs { args } => command_line_args.extend(args),
        RuntimeInitAttribute::DeepLinkSchemes { schemes } => deep_link_schemes.extend(schemes),
      }
    }
    command_line_args.push(("--enable-media-stream".to_string(), None));

    let mut app = cef_impl::TauriApp::new(
      cef_context.clone(),
      runtime_args.custom_schemes,
      deep_link_schemes,
      command_line_args,
    );

    let cmd = args.as_cmd_line().unwrap();
    let switch = CefString::from("type");
    let is_browser_process = cmd.has_switch(Some(&switch)) != 1;

    let ret = cef::execute_process(
      Some(args.as_main_args()),
      Some(&mut app),
      std::ptr::null_mut(),
    );

    if is_browser_process {
      assert!(ret == -1, "cannot execute browser process");
    } else {
      assert!(ret >= 0, "cannot execute non-browser process");
      // non-browser process does not initialize cef
      std::process::exit(0);
    }

    let settings = cef::Settings {
      no_sandbox: !cfg!(feature = "sandbox") as i32,
      cache_path: cache_path.to_string_lossy().to_string().as_str().into(),
      ..Default::default()
    };
    assert_eq!(
      cef::initialize(
        Some(args.as_main_args()),
        Some(&settings),
        Some(&mut app),
        std::ptr::null_mut()
      ),
      1
    );

    let main_thread_id = thread::current().id();
    let context = RuntimeContext {
      main_thread_task_runner: cef::task_runner_get_for_current_thread().expect("null task runner"),
      main_thread_id,
      cef_context,
    };
    Self {
      context,
      event_tx,
      event_rx,
    }
  }
}

/// Helper function for non-browser CEF processes (renderer, GPU, plugin, etc.).
/// This should be called from the entry point macro when the process is not the browser process.
pub fn run_cef_helper_process() {
  let args = cef::args::Args::new();

  #[cfg(all(target_os = "macos", feature = "sandbox"))]
  let _sandbox = {
    let mut sandbox = cef::sandbox::Sandbox::new();
    sandbox.initialize(args.as_main_args());
    sandbox
  };

  #[cfg(target_os = "macos")]
  let _loader = {
    let loader = cef::library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), true);
    assert!(loader.load());
    loader
  };

  cef::execute_process(
    Some(args.as_main_args()),
    None::<&mut cef::App>,
    std::ptr::null_mut(),
  );
}

/// Platform-specific runtime init attributes.
pub enum RuntimeInitAttribute {
  /// Command line arguments passed to CEF.
  CommandLineArgs { args: Vec<(String, Option<String>)> },
  /// Deep link schemes.
  DeepLinkSchemes { schemes: Vec<String> },
}

impl InitAttribute for RuntimeInitAttribute {
  fn new(config: &tauri_utils::config::Config) -> Result<Vec<Self>> {
    let mut attrs = Vec::new();
    if let Some(plugin_config) = config
      .plugins
      .0
      .get("deep-link")
      .and_then(|c| c.get("desktop").cloned())
    {
      #[derive(serde::Deserialize)]
      #[serde(untagged)]
      enum DesktopDeepLinks {
        One(tauri_utils::config::DeepLinkProtocol),
        List(Vec<tauri_utils::config::DeepLinkProtocol>),
      }

      let protocols: DesktopDeepLinks =
        serde_json::from_value(plugin_config).map_err(tauri_runtime::Error::Json)?;
      let schemes = match protocols {
        DesktopDeepLinks::One(p) => p.schemes,
        DesktopDeepLinks::List(p) => p.into_iter().flat_map(|p| p.schemes).collect(),
      };

      attrs.push(RuntimeInitAttribute::DeepLinkSchemes { schemes });
    }
    Ok(attrs)
  }
}

/// Webview attributes.
pub enum WebviewAtribute {
  /// Sets the browser runtime style.
  RuntimeStyle { style: RuntimeStyle },
}

/// The browser runtime style.
#[derive(Clone, Copy)]
pub enum RuntimeStyle {
  /// Alloy runtime.
  ///
  /// Used by default on multiwebview mode.
  Alloy,
  /// Chrome runtime.
  ///
  /// Used by default on webview window mode.
  ///
  /// Only a single browser view can use the Chrome runtime in a given window.
  Chrome,
}

#[derive(Debug)]
pub struct NewWindowOpener {}

impl<T: UserEvent> Runtime<T> for CefRuntime<T> {
  type WindowDispatcher = CefWindowDispatcher<T>;
  type WebviewDispatcher = CefWebviewDispatcher<T>;
  type Handle = CefRuntimeHandle<T>;
  type EventLoopProxy = EventProxy<T>;
  type PlatformSpecificWebviewAttribute = WebviewAtribute;
  type PlatformSpecificInitAttribute = RuntimeInitAttribute;
  type WindowOpener = NewWindowOpener;

  fn new(args: RuntimeInitArgs<RuntimeInitAttribute>) -> Result<Self> {
    Ok(Self::init(args))
  }

  #[cfg(any(windows, target_os = "linux"))]
  fn new_any_thread(args: RuntimeInitArgs<RuntimeInitAttribute>) -> Result<Self> {
    Ok(Self::init(args))
  }

  fn create_proxy(&self) -> Self::EventLoopProxy {
    EventProxy {
      context: self.context.clone(),
    }
  }

  fn handle(&self) -> Self::Handle {
    CefRuntimeHandle {
      context: self.context.clone(),
    }
  }

  fn create_window<F: Fn(RawWindow<'_>) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self>,
    _after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self>> {
    let label = pending.label.clone();
    let window_id = self.context.cef_context.next_window_id();
    let (webview_id, use_https_scheme) = pending
      .webview
      .as_ref()
      .map(|w| {
        (
          Some(self.context.cef_context.next_webview_id()),
          w.webview_attributes.use_https_scheme,
        )
      })
      .unwrap_or((None, false));

    cef_impl::create_window(
      &self.context.cef_context,
      window_id,
      webview_id.unwrap_or_default(),
      pending,
    );

    let dispatcher = CefWindowDispatcher {
      window_id,
      context: self.context.clone(),
    };

    let detached_webview = webview_id.map(|id| {
      let webview = DetachedWebview {
        label: label.clone(),
        dispatcher: CefWebviewDispatcher {
          window_id: Arc::new(Mutex::new(window_id)),
          webview_id: id,
          context: self.context.clone(),
        },
      };
      DetachedWindowWebview {
        webview,
        use_https_scheme,
      }
    });

    Ok(DetachedWindow {
      id: window_id,
      label,
      dispatcher,
      webview: detached_webview,
    })
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self>,
  ) -> Result<DetachedWebview<T, Self>> {
    let label = pending.label.clone();
    let webview_id = self.context.cef_context.next_webview_id();

    cef_impl::create_webview(
      cef_impl::WebviewKind::WindowChild,
      &self.context.cef_context,
      window_id,
      webview_id,
      pending,
    );

    let dispatcher = CefWebviewDispatcher {
      window_id: Arc::new(Mutex::new(window_id)),
      webview_id,
      context: self.context.clone(),
    };

    Ok(DetachedWebview { label, dispatcher })
  }

  fn primary_monitor(&self) -> Option<Monitor> {
    crate::cef_impl::get_primary_monitor()
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Option<Monitor> {
    crate::cef_impl::get_monitor_from_point(x, y)
  }

  fn available_monitors(&self) -> Vec<Monitor> {
    crate::cef_impl::get_available_monitors()
  }

  fn set_theme(&self, _theme: Option<Theme>) {}

  #[cfg(target_os = "macos")]
  fn set_activation_policy(&mut self, _activation_policy: tauri_runtime::ActivationPolicy) {}

  #[cfg(target_os = "macos")]
  fn set_dock_visibility(&mut self, _visible: bool) {}

  #[cfg(target_os = "macos")]
  fn show(&self) {
    cef_impl::set_application_visibility(true);
  }

  #[cfg(target_os = "macos")]
  fn hide(&self) {
    cef_impl::set_application_visibility(false);
  }

  fn set_device_event_filter(&mut self, _filter: DeviceEventFilter) {}

  fn custom_scheme_url(scheme: &str, https: bool) -> String {
    // CEF always uses http/https format regardless of platform
    format!(
      "{}://{scheme}.localhost",
      if https { "https" } else { "http" }
    )
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
  fn run_iteration<F: FnMut(RunEvent<T>)>(&mut self, _callback: F) {}

  fn run_return<F: FnMut(RunEvent<T>) + 'static>(self, _callback: F) -> i32 {
    0
  }

  fn run<F: FnMut(RunEvent<T>) + 'static>(self, callback: F) {
    let callback = Arc::new(RefCell::new(callback));
    let callback_ = callback.clone();
    let event_tx_ = self.event_tx.clone();
    let _ = std::mem::replace(
      &mut *self.context.cef_context.callback.borrow_mut(),
      Box::new(move |event| {
        if let RunEvent::Exit = event {
          // notify the event loop to exit
          let _ = event_tx_.send(RunEvent::Exit);
        } else {
          // Try to call callback directly, if busy queue to channel
          if let Ok(mut cb) = callback.try_borrow_mut() {
            cb(event);
          } else {
            let _ = event_tx_.send(event);
          }
        }
      }),
    );

    'main_loop: loop {
      while let Ok(event) = self.event_rx.try_recv() {
        if matches!(&event, RunEvent::Exit) {
          // Exit event is triggered when we break out of the loop
          break 'main_loop;
        }

        (self.context.cef_context.callback.borrow())(event);
      }

      // Do CEF message loop work
      // This processes one iteration of the message loop
      cef::do_message_loop_work();

      // Emit MainEventsCleared event
      (self.context.cef_context.callback.borrow())(RunEvent::MainEventsCleared);
    }

    cef::shutdown();

    // Final Exit event
    // use callback_ directly because cef_context.callback posts Exit events to the event loop rx
    (callback_.borrow_mut())(RunEvent::Exit);
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    Ok(PhysicalPosition::new(0.0, 0.0))
  }
}

#[cfg(target_os = "macos")]
fn init_ns_app(on_event: Box<dyn Fn(AppDelegateEvent)>) {
  use objc2::{ClassType, MainThreadMarker, msg_send, rc::Retained, runtime::NSObjectProtocol};
  use objc2_app_kit::{NSApp, NSApplication};

  use application::{AppDelegate, SimpleApplication};

  let mtm = MainThreadMarker::new().unwrap();

  unsafe {
    // Initialize the SimpleApplication instance.
    // SAFETY: mtm ensures that here is the main thread.

    use objc2::runtime::ProtocolObject;

    let app: Retained<NSApplication> = msg_send![SimpleApplication::class(), sharedApplication];
    let delegate = AppDelegate::new(mtm, on_event);
    let proto_delegate = ProtocolObject::from_ref(&*delegate);
    app.setDelegate(Some(proto_delegate));
  }

  // If there was an invocation to NSApp prior to here,
  // then the NSApp will not be a SimpleApplication.
  // The following assertion ensures that this doesn't happen.
  assert!(NSApp(mtm).isKindOfClass(SimpleApplication::class()));
}

#[cfg(target_os = "macos")]
mod application {
  use std::{cell::Cell, sync::mpsc::channel};

  use cef::application_mac::{CefAppProtocol, CrAppControlProtocol, CrAppProtocol};
  use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::Retained,
    runtime::{Bool, NSObject, NSObjectProtocol},
  };
  use objc2_app_kit::{NSApplication, NSApplicationDelegate, NSApplicationTerminateReply};
  use objc2_foundation::{NSArray, NSURL};

  pub enum AppDelegateEvent {
    ShouldTerminate {
      tx: std::sync::mpsc::Sender<NSApplicationTerminateReply>,
    },
    OpenURLs {
      urls: Vec<url::Url>,
    },
  }

  pub struct CefAppDelegateIvars {
    pub on_event: Box<dyn Fn(AppDelegateEvent)>,
  }

  define_class!(
    #[unsafe(super(NSObject))]
    #[name = "CefAppDelegate"]
    #[ivars = CefAppDelegateIvars]
    #[thread_kind = MainThreadOnly]
    pub struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSApplicationDelegate for AppDelegate {
      #[unsafe(method(application:openURLs:))]
      unsafe fn application_openURLs(&self, _application: &NSApplication, urls: &NSArray<NSURL>) {
        let converted_urls: Vec<url::Url> = urls
          .iter()
          .filter_map(|ns_url| unsafe {
            ns_url
              .absoluteString()
              .and_then(|url_string| url_string.to_string().parse().ok())
          })
          .collect();

        let handler = &self.ivars().on_event;
        handler(AppDelegateEvent::OpenURLs {
          urls: converted_urls,
        });
      }

      #[unsafe(method(applicationShouldTerminate:))]
      unsafe fn applicationShouldTerminate(
        &self,
        _sender: &NSApplication,
      ) -> NSApplicationTerminateReply {
        let (tx, rx) = channel();
        let handler = &self.ivars().on_event;
        handler(AppDelegateEvent::ShouldTerminate { tx });
        rx.try_recv()
          .unwrap_or(NSApplicationTerminateReply::TerminateNow)
      }
    }
  );

  impl AppDelegate {
    pub fn new(mtm: MainThreadMarker, on_event: Box<dyn Fn(AppDelegateEvent)>) -> Retained<Self> {
      let delegate = Self::alloc(mtm).set_ivars(CefAppDelegateIvars { on_event });
      let delegate: Retained<Self> = unsafe { msg_send![super(delegate), init] };
      delegate
    }
  }

  /// Instance variables of `SimpleApplication`.
  pub struct SimpleApplicationIvars {
    handling_send_event: Cell<Bool>,
  }

  define_class!(
    /// A `NSApplication` subclass that implements the required CEF protocols.
    ///
    /// This class provides the necessary `CefAppProtocol` conformance to
    /// ensure that events are handled correctly by the Chromium framework on macOS.
    #[unsafe(super(NSApplication))]
    #[ivars = SimpleApplicationIvars]
    pub struct SimpleApplication;

    unsafe impl CrAppControlProtocol for SimpleApplication {
      #[unsafe(method(setHandlingSendEvent:))]
      unsafe fn set_handling_send_event(&self, handling_send_event: Bool) {
        self.ivars().handling_send_event.set(handling_send_event);
      }
    }

    unsafe impl CrAppProtocol for SimpleApplication {
      #[unsafe(method(isHandlingSendEvent))]
      unsafe fn is_handling_send_event(&self) -> Bool {
        self.ivars().handling_send_event.get()
      }
    }

    unsafe impl CefAppProtocol for SimpleApplication {}
  );
}
