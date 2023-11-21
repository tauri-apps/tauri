// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(dead_code)]
#![allow(missing_docs)]

use tauri_runtime::{
  monitor::Monitor,
  webview::{DetachedWebview, PendingWebview},
  window::{
    dpi::{PhysicalPosition, PhysicalSize, Position, Size},
    CursorIcon, DetachedWindow, PendingWindow, RawWindow, WindowEvent, WindowId,
  },
  window::{WindowBuilder, WindowBuilderBase},
  DeviceEventFilter, Error, EventLoopProxy, ExitRequestedEventAction, Icon, Result, RunEvent,
  Runtime, RuntimeHandle, RuntimeInitArgs, UserAttentionType, UserEvent, WebviewDispatch,
  WindowDispatch, WindowEventId,
};

#[cfg(target_os = "macos")]
use tauri_utils::TitleBarStyle;
use tauri_utils::{config::WindowConfig, ProgressBarState, Theme};
use url::Url;

#[cfg(windows)]
use windows::Win32::Foundation::HWND;

use std::{
  cell::RefCell,
  collections::HashMap,
  fmt,
  sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    mpsc::{channel, sync_channel, Receiver, SyncSender},
    Arc, Mutex,
  },
};

type ShortcutMap = HashMap<String, Box<dyn Fn() + Send + 'static>>;

enum Message {
  Task(Box<dyn FnOnce() + Send>),
  CloseWindow(WindowId),
}

struct Webview;

struct Window {
  webviews: Vec<Webview>,
}

#[derive(Clone)]
pub struct RuntimeContext {
  is_running: Arc<AtomicBool>,
  windows: Arc<RefCell<HashMap<WindowId, Window>>>,
  shortcuts: Arc<Mutex<ShortcutMap>>,
  run_tx: SyncSender<Message>,
  next_window_id: Arc<AtomicU32>,
  next_webview_id: Arc<AtomicU32>,
  next_window_event_id: Arc<AtomicU32>,
}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for RuntimeContext {}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for RuntimeContext {}

impl RuntimeContext {
  fn send_message(&self, message: Message) -> Result<()> {
    if self.is_running.load(Ordering::Relaxed) {
      self
        .run_tx
        .send(message)
        .map_err(|_| Error::FailedToSendMessage)
    } else {
      match message {
        Message::Task(task) => task(),
        Message::CloseWindow(id) => {
          self.windows.borrow_mut().remove(&id);
        }
      }
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
    EventProxy {}
  }

  /// Create a new webview window.
  fn create_window<F: Fn(RawWindow<'_>) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self::Runtime>,
    _before_webview_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    let id = self.context.next_window_id();

    let (webview_id, webviews) = if let Some(w) = &pending.webview {
      (Some(self.context.next_webview_id()), vec![Webview])
    } else {
      (None, Vec::new())
    };

    self
      .context
      .windows
      .borrow_mut()
      .insert(id, Window { webviews });

    let webview = webview_id.map(|id| DetachedWebview {
      label: pending.label.clone(),
      dispatcher: MockWebviewDispatcher {
        id,
        context: self.context.clone(),
        url: Arc::new(Mutex::new(pending.webview.unwrap().url)),
        last_evaluated_script: Default::default(),
      },
    });

    Ok(DetachedWindow {
      id,
      label: pending.label,
      dispatcher: MockWindowDispatcher {
        id,
        context: self.context.clone(),
      },
      webview,
    })
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>> {
    let id = self.context.next_webview_id();
    let webview = Webview;
    self
      .context
      .windows
      .borrow_mut()
      .get_mut(&window_id)
      .map(|w| {
        w.webviews.push(webview);
      });

    Ok(DetachedWebview {
      label: pending.label,
      dispatcher: MockWebviewDispatcher {
        id,
        context: self.context.clone(),
        last_evaluated_script: Default::default(),
        url: Arc::new(Mutex::new(pending.url)),
      },
    })
  }

  /// Run a task on the main thread.
  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.context.send_message(Message::Task(Box::new(f)))
  }

  fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
    #[cfg(target_os = "linux")]
    return raw_window_handle::RawDisplayHandle::Xlib(raw_window_handle::XlibDisplayHandle::empty());
    #[cfg(target_os = "macos")]
    return raw_window_handle::RawDisplayHandle::AppKit(
      raw_window_handle::AppKitDisplayHandle::empty(),
    );
    #[cfg(windows)]
    return raw_window_handle::RawDisplayHandle::Windows(
      raw_window_handle::WindowsDisplayHandle::empty(),
    );
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    return unimplemented!();
  }

  fn primary_monitor(&self) -> Option<Monitor> {
    unimplemented!()
  }

  fn available_monitors(&self) -> Vec<Monitor> {
    unimplemented!()
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
}

#[derive(Debug, Clone)]
pub struct MockWebviewDispatcher {
  id: u32,
  context: RuntimeContext,
  url: Arc<Mutex<String>>,
  last_evaluated_script: Arc<Mutex<Option<String>>>,
}

impl MockWebviewDispatcher {
  pub fn last_evaluated_script(&self) -> Option<String> {
    self.last_evaluated_script.lock().unwrap().clone()
  }
}

#[derive(Debug, Clone)]
pub struct MockWindowDispatcher {
  id: WindowId,
  context: RuntimeContext,
}

#[derive(Debug, Clone)]
pub struct MockWindowBuilder {}

impl WindowBuilderBase for MockWindowBuilder {}

impl WindowBuilder for MockWindowBuilder {
  fn new() -> Self {
    Self {}
  }

  fn with_config(config: WindowConfig) -> Self {
    Self {}
  }

  fn center(self) -> Self {
    self
  }

  fn position(self, x: f64, y: f64) -> Self {
    self
  }

  fn inner_size(self, min_width: f64, min_height: f64) -> Self {
    self
  }

  fn min_inner_size(self, min_width: f64, min_height: f64) -> Self {
    self
  }

  fn max_inner_size(self, max_width: f64, max_height: f64) -> Self {
    self
  }

  fn resizable(self, resizable: bool) -> Self {
    self
  }

  fn maximizable(self, resizable: bool) -> Self {
    self
  }

  fn minimizable(self, resizable: bool) -> Self {
    self
  }

  fn closable(self, resizable: bool) -> Self {
    self
  }

  fn title<S: Into<String>>(self, title: S) -> Self {
    self
  }

  fn fullscreen(self, fullscreen: bool) -> Self {
    self
  }

  fn focused(self, focused: bool) -> Self {
    self
  }

  fn maximized(self, maximized: bool) -> Self {
    self
  }

  fn visible(self, visible: bool) -> Self {
    self
  }

  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  #[cfg_attr(
    docsrs,
    doc(cfg(any(not(target_os = "macos"), feature = "macos-private-api")))
  )]
  fn transparent(self, transparent: bool) -> Self {
    self
  }

  fn decorations(self, decorations: bool) -> Self {
    self
  }

  fn always_on_bottom(self, always_on_bottom: bool) -> Self {
    self
  }

  fn always_on_top(self, always_on_top: bool) -> Self {
    self
  }

  fn visible_on_all_workspaces(self, visible_on_all_workspaces: bool) -> Self {
    self
  }

  fn content_protected(self, protected: bool) -> Self {
    self
  }

  fn icon(self, icon: Icon) -> Result<Self> {
    Ok(self)
  }

  fn skip_taskbar(self, skip: bool) -> Self {
    self
  }

  fn shadow(self, enable: bool) -> Self {
    self
  }

  #[cfg(windows)]
  fn parent_window(self, parent: HWND) -> Self {
    self
  }

  #[cfg(target_os = "macos")]
  fn parent_window(self, parent: *mut std::ffi::c_void) -> Self {
    self
  }

  #[cfg(windows)]
  fn owner_window(self, owner: HWND) -> Self {
    self
  }

  #[cfg(target_os = "macos")]
  fn title_bar_style(self, style: TitleBarStyle) -> Self {
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

  fn theme(self, theme: Option<Theme>) -> Self {
    self
  }

  fn has_icon(&self) -> bool {
    false
  }
}

impl<T: UserEvent> WebviewDispatch<T> for MockWebviewDispatcher {
  type Runtime = MockRuntime;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.context.send_message(Message::Task(Box::new(f)))
  }

  fn with_webview<F: FnOnce(Box<dyn std::any::Any>) + Send + 'static>(&self, f: F) -> Result<()> {
    Ok(())
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn open_devtools(&self) {}

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn close_devtools(&self) {}

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn is_devtools_open(&self) -> Result<bool> {
    Ok(false)
  }

  fn eval_script<S: Into<String>>(&self, script: S) -> Result<()> {
    self
      .last_evaluated_script
      .lock()
      .unwrap()
      .replace(script.into());
    Ok(())
  }

  fn url(&self) -> Result<url::Url> {
    self
      .url
      .lock()
      .unwrap()
      .parse()
      .map_err(|_| Error::FailedToReceiveMessage)
  }

  fn position(&self) -> Result<PhysicalPosition<i32>> {
    Ok(PhysicalPosition { x: 0, y: 0 })
  }

  fn size(&self) -> Result<PhysicalSize<u32>> {
    Ok(PhysicalSize {
      width: 0,
      height: 0,
    })
  }

  fn navigate(&self, url: Url) -> Result<()> {
    *self.url.lock().unwrap() = url.to_string();
    Ok(())
  }

  fn print(&self) -> Result<()> {
    Ok(())
  }

  fn close(&self) -> Result<()> {
    Ok(())
  }

  fn set_size(&self, _size: Size) -> Result<()> {
    Ok(())
  }

  fn set_position(&self, _position: Position) -> Result<()> {
    Ok(())
  }

  fn set_focus(&self) -> Result<()> {
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
    self.context.next_window_event_id()
  }

  fn scale_factor(&self) -> Result<f64> {
    Ok(1.0)
  }

  fn inner_position(&self) -> Result<PhysicalPosition<i32>> {
    Ok(PhysicalPosition { x: 0, y: 0 })
  }

  fn outer_position(&self) -> Result<PhysicalPosition<i32>> {
    Ok(PhysicalPosition { x: 0, y: 0 })
  }

  fn inner_size(&self) -> Result<PhysicalSize<u32>> {
    Ok(PhysicalSize {
      width: 0,
      height: 0,
    })
  }

  fn outer_size(&self) -> Result<PhysicalSize<u32>> {
    Ok(PhysicalSize {
      width: 0,
      height: 0,
    })
  }

  fn is_fullscreen(&self) -> Result<bool> {
    Ok(false)
  }

  fn is_minimized(&self) -> Result<bool> {
    Ok(false)
  }

  fn is_maximized(&self) -> Result<bool> {
    Ok(false)
  }

  fn is_focused(&self) -> Result<bool> {
    Ok(false)
  }

  fn is_decorated(&self) -> Result<bool> {
    Ok(false)
  }

  fn is_resizable(&self) -> Result<bool> {
    Ok(false)
  }

  fn is_maximizable(&self) -> Result<bool> {
    Ok(true)
  }

  fn is_minimizable(&self) -> Result<bool> {
    Ok(true)
  }

  fn is_closable(&self) -> Result<bool> {
    Ok(true)
  }

  fn is_visible(&self) -> Result<bool> {
    Ok(true)
  }

  fn title(&self) -> Result<String> {
    Ok(String::new())
  }

  fn current_monitor(&self) -> Result<Option<Monitor>> {
    Ok(None)
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    Ok(None)
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    Ok(Vec::new())
  }

  fn theme(&self) -> Result<Theme> {
    Ok(Theme::Light)
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

  fn raw_window_handle(&self) -> Result<raw_window_handle::RawWindowHandle> {
    #[cfg(target_os = "linux")]
    return Ok(raw_window_handle::RawWindowHandle::Xlib(
      raw_window_handle::XlibWindowHandle::empty(),
    ));
    #[cfg(target_os = "macos")]
    return Ok(raw_window_handle::RawWindowHandle::AppKit(
      raw_window_handle::AppKitWindowHandle::empty(),
    ));
    #[cfg(windows)]
    return Ok(raw_window_handle::RawWindowHandle::Win32(
      raw_window_handle::Win32WindowHandle::empty(),
    ));
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    return unimplemented!();
  }

  fn center(&self) -> Result<()> {
    Ok(())
  }

  fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> Result<()> {
    Ok(())
  }

  fn create_window<F: Fn(RawWindow<'_>) + Send + 'static>(
    &mut self,
    pending: PendingWindow<T, Self::Runtime>,
    _before_webview_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    let id = self.context.next_window_id();

    let (webview_id, webviews) = if let Some(w) = &pending.webview {
      (Some(self.context.next_webview_id()), vec![Webview])
    } else {
      (None, Vec::new())
    };

    self
      .context
      .windows
      .borrow_mut()
      .insert(id, Window { webviews });

    let webview = webview_id.map(|id| DetachedWebview {
      label: pending.label.clone(),
      dispatcher: MockWebviewDispatcher {
        id,
        context: self.context.clone(),
        url: Arc::new(Mutex::new(pending.webview.unwrap().url)),
        last_evaluated_script: Default::default(),
      },
    });

    Ok(DetachedWindow {
      id: id.into(),
      label: pending.label,
      dispatcher: MockWindowDispatcher {
        id,
        context: self.context.clone(),
      },
      webview,
    })
  }

  fn create_webview(
    &mut self,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>> {
    let id = self.context.next_webview_id();
    let webview = Webview;
    self
      .context
      .windows
      .borrow_mut()
      .get_mut(&self.id)
      .map(|w| {
        w.webviews.push(webview);
      });

    Ok(DetachedWebview {
      label: pending.label,
      dispatcher: MockWebviewDispatcher {
        id,
        context: self.context.clone(),
        last_evaluated_script: Default::default(),
        url: Arc::new(Mutex::new(pending.url)),
      },
    })
  }

  fn set_resizable(&self, resizable: bool) -> Result<()> {
    Ok(())
  }

  fn set_maximizable(&self, maximizable: bool) -> Result<()> {
    Ok(())
  }

  fn set_minimizable(&self, minimizable: bool) -> Result<()> {
    Ok(())
  }

  fn set_closable(&self, closable: bool) -> Result<()> {
    Ok(())
  }

  fn set_title<S: Into<String>>(&self, title: S) -> Result<()> {
    Ok(())
  }

  fn maximize(&self) -> Result<()> {
    Ok(())
  }

  fn unmaximize(&self) -> Result<()> {
    Ok(())
  }

  fn minimize(&self) -> Result<()> {
    Ok(())
  }

  fn unminimize(&self) -> Result<()> {
    Ok(())
  }

  fn show(&self) -> Result<()> {
    Ok(())
  }

  fn hide(&self) -> Result<()> {
    Ok(())
  }

  fn close(&self) -> Result<()> {
    self.context.send_message(Message::CloseWindow(self.id))?;
    Ok(())
  }

  fn set_decorations(&self, decorations: bool) -> Result<()> {
    Ok(())
  }

  fn set_shadow(&self, shadow: bool) -> Result<()> {
    Ok(())
  }

  fn set_always_on_bottom(&self, always_on_bottom: bool) -> Result<()> {
    Ok(())
  }

  fn set_always_on_top(&self, always_on_top: bool) -> Result<()> {
    Ok(())
  }

  fn set_visible_on_all_workspaces(&self, visible_on_all_workspaces: bool) -> Result<()> {
    Ok(())
  }

  fn set_content_protected(&self, protected: bool) -> Result<()> {
    Ok(())
  }

  fn set_size(&self, size: Size) -> Result<()> {
    Ok(())
  }

  fn set_min_size(&self, size: Option<Size>) -> Result<()> {
    Ok(())
  }

  fn set_max_size(&self, size: Option<Size>) -> Result<()> {
    Ok(())
  }

  fn set_position(&self, position: Position) -> Result<()> {
    Ok(())
  }

  fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
    Ok(())
  }

  fn set_focus(&self) -> Result<()> {
    Ok(())
  }

  fn set_icon(&self, icon: Icon) -> Result<()> {
    Ok(())
  }

  fn set_skip_taskbar(&self, skip: bool) -> Result<()> {
    Ok(())
  }

  fn set_cursor_grab(&self, grab: bool) -> Result<()> {
    Ok(())
  }

  fn set_cursor_visible(&self, visible: bool) -> Result<()> {
    Ok(())
  }

  fn set_cursor_icon(&self, icon: CursorIcon) -> Result<()> {
    Ok(())
  }

  fn set_cursor_position<Pos: Into<Position>>(&self, position: Pos) -> Result<()> {
    Ok(())
  }

  fn set_ignore_cursor_events(&self, ignore: bool) -> Result<()> {
    Ok(())
  }

  fn start_dragging(&self) -> Result<()> {
    Ok(())
  }

  fn set_progress_bar(&self, progress_state: ProgressBarState) -> Result<()> {
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct EventProxy {}

impl<T: UserEvent> EventLoopProxy<T> for EventProxy {
  fn send_event(&self, event: T) -> Result<()> {
    Ok(())
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
    let (tx, rx) = sync_channel(1);
    let context = RuntimeContext {
      is_running: is_running.clone(),
      windows: Default::default(),
      shortcuts: Default::default(),
      run_tx: tx,
      next_window_id: Default::default(),
      next_webview_id: Default::default(),
      next_window_event_id: Default::default(),
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

  #[cfg(any(windows, target_os = "linux"))]
  fn new_any_thread(_args: RuntimeInitArgs) -> Result<Self> {
    Ok(Self::init())
  }

  fn create_proxy(&self) -> EventProxy {
    EventProxy {}
  }

  fn handle(&self) -> Self::Handle {
    MockRuntimeHandle {
      context: self.context.clone(),
    }
  }

  fn create_window<F: Fn(RawWindow<'_>) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self>,
    _before_webview_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self>> {
    let id = self.context.next_window_id();

    let (webview_id, webviews) = if let Some(w) = &pending.webview {
      (Some(self.context.next_webview_id()), vec![Webview])
    } else {
      (None, Vec::new())
    };

    self
      .context
      .windows
      .borrow_mut()
      .insert(id, Window { webviews });

    let webview = webview_id.map(|id| DetachedWebview {
      label: pending.label.clone(),
      dispatcher: MockWebviewDispatcher {
        id,
        context: self.context.clone(),
        url: Arc::new(Mutex::new(pending.webview.unwrap().url)),
        last_evaluated_script: Default::default(),
      },
    });

    Ok(DetachedWindow {
      id: id.into(),
      label: pending.label,
      dispatcher: MockWindowDispatcher {
        id,
        context: self.context.clone(),
      },
      webview,
    })
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self>,
  ) -> Result<DetachedWebview<T, Self>> {
    let id = self.context.next_webview_id();
    let webview = Webview;
    self
      .context
      .windows
      .borrow_mut()
      .get_mut(&window_id)
      .map(|w| {
        w.webviews.push(webview);
      });

    Ok(DetachedWebview {
      label: pending.label,
      dispatcher: MockWebviewDispatcher {
        id,
        context: self.context.clone(),
        last_evaluated_script: Default::default(),
        url: Arc::new(Mutex::new(pending.url)),
      },
    })
  }

  fn primary_monitor(&self) -> Option<Monitor> {
    unimplemented!()
  }

  fn available_monitors(&self) -> Vec<Monitor> {
    unimplemented!()
  }

  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn set_activation_policy(&mut self, activation_policy: tauri_runtime::ActivationPolicy) {}

  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn show(&self) {}

  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn hide(&self) {}

  fn set_device_event_filter(&mut self, filter: DeviceEventFilter) {}

  #[cfg(any(
    target_os = "macos",
    windows,
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn run_iteration<F: Fn(RunEvent<T>) + 'static>(
    &mut self,
    callback: F,
  ) -> tauri_runtime::RunIteration {
    Default::default()
  }

  fn run<F: FnMut(RunEvent<T>) + 'static>(self, mut callback: F) {
    self.is_running.store(true, Ordering::Relaxed);
    callback(RunEvent::Ready);

    loop {
      if let Ok(m) = self.run_rx.try_recv() {
        match m {
          Message::Task(p) => p(),
          Message::CloseWindow(id) => {
            let removed = self.context.windows.borrow_mut().remove(&id).is_some();
            if removed {
              let is_empty = self.context.windows.borrow().is_empty();
              if is_empty {
                let (tx, rx) = channel();
                callback(RunEvent::ExitRequested { tx });

                let recv = rx.try_recv();
                let should_prevent = matches!(recv, Ok(ExitRequestedEventAction::Prevent));

                if !should_prevent {
                  break;
                }
              }
            }
          }
        }
      }

      callback(RunEvent::MainEventsCleared);

      std::thread::sleep(std::time::Duration::from_secs(1));
    }

    callback(RunEvent::Exit);
  }
}
