// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::too_many_arguments)]

use std::{
  collections::HashMap,
  fmt,
  fs::create_dir_all,
  path::PathBuf,
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering},
    mpsc::{self, Receiver, Sender},
  },
  time::Duration,
};

use cef::*;
use raw_window_handle::{DisplayHandle, HasDisplayHandle};
use tauri_runtime::{
  DeviceEventFilter, Error, EventLoopProxy, ExitRequestedEventAction, Result, RunEvent, Runtime,
  RuntimeHandle, RuntimeInitArgs, UserEvent,
  dpi::PhysicalPosition,
  monitor::Monitor,
  webview::{DetachedWebview, PendingWebview},
  window::{
    DetachedWindow, DragDropEvent, PendingWindow, RawWindow, WebviewEvent, WindowEvent, WindowId,
  },
};
use tauri_utils::Theme;
use winit::{
  application::ApplicationHandler,
  event::{StartCause, WindowEvent as WinitWindowEvent},
  event_loop::{
    ActiveEventLoop, EventLoop, EventLoopBuilder, EventLoopProxy as WinitEventLoopProxy,
  },
  window::WindowId as WinitWindowId,
};

use crate::external_message_pump::CefExternalPump;
use crate::platform::EventLoopExt;
use crate::{
  cef_impl::{client as browser_client, ipc, request_handler},
  webview::{
    self, AppWebview, CefWebviewDispatcher, Webview, WebviewAtribute, WebviewMessage,
    create_webview_detached,
  },
  window::{
    AppWindow, CefWindowDispatcher, WindowMessage, create_window_detached,
    winit_monitor_to_tauri_monitor, winit_theme_to_tauri_theme,
  },
  window_handle::SendRawDisplayHandle,
};
#[cfg(target_os = "macos")]
use winit::platform::macos::EventLoopBuilderExtMacOS;
#[cfg(windows)]
use winit::platform::windows::EventLoopBuilderExtWindows;
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
use winit::platform::x11::EventLoopBuilderExtX11;

/// The `cef` crate used by this runtime, re-exported for convenience.
///
/// # Stability
///
/// The cef crate follows the Chromium Embedded Framework interface and there is
/// no API stability guarantees. The crate will be updated frequently, usually
/// in minor releases when a known breaking change is discovered.
pub use cef;

/// Platform-specific runtime init attributes.
#[derive(Clone, Debug)]
pub enum RuntimeInitAttribute {
  /// Command line arguments passed to CEF.
  CommandLineArgs { args: Vec<(String, Option<String>)> },
  /// Deep link schemes.
  DeepLinkSchemes { schemes: Vec<String> },
  /// Directory used for CEF disk cache (`Settings::cache_path`).
  ///
  /// If unspecified, defaults to `{user cache}/{app identifier}/cef`.
  CachePath { path: PathBuf },
}

impl tauri_runtime::InitAttribute for RuntimeInitAttribute {
  fn new(config: &tauri_utils::config::Config) -> Result<Vec<Self>> {
    let mut attrs = Vec::new();
    if let Some(plugin_config) = config
      .plugins
      .0
      .get("deep-link")
      .and_then(|config| config.get("desktop").cloned())
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
        DesktopDeepLinks::One(protocol) => protocol.schemes,
        DesktopDeepLinks::List(protocols) => protocols
          .into_iter()
          .flat_map(|protocol| protocol.schemes)
          .collect(),
      };

      attrs.push(RuntimeInitAttribute::DeepLinkSchemes { schemes });
    }
    Ok(attrs)
  }
}

#[derive(Debug)]
pub struct NewWindowOpener {}

#[derive(Clone, Debug)]
pub struct EventProxy<T: UserEvent> {
  context: RuntimeContext<T>,
}

impl<T: UserEvent> EventLoopProxy<T> for EventProxy<T> {
  fn send_event(&self, event: T) -> Result<()> {
    self.context.send_message(Message::UserEvent(event))
  }
}

#[derive(Clone)]
pub(crate) struct RuntimeContext<T: UserEvent> {
  pub(crate) sender: Sender<Message<T>>,
  pub(crate) proxy: WinitEventLoopProxy,
  main_thread_id: std::thread::ThreadId,
  next_window_id: Arc<AtomicU32>,
  next_webview_id: Arc<AtomicU32>,
  next_window_event_id: Arc<AtomicU32>,
  next_webview_event_id: Arc<AtomicU32>,
  current_dispatch: Arc<MainThreadDispatchSlot<T>>,
  pub(crate) app_wide_theme: Arc<Mutex<Option<Theme>>>,
  pub(crate) cef_pump: CefExternalPump,
  /// Root cache path passed to [`cef::Settings::cache_path`] during
  /// [`cef::initialize`]. Per-webview `data_directory` profiles must resolve
  /// under this root for CEF request contexts to be accepted.
  pub(crate) cache_path: Arc<PathBuf>,
}

/// Scoped access to the current winit callback state.
///
/// `ActiveEventLoop` is only borrowed during `ApplicationHandler` callbacks, but
/// setup-time runtime messages may synchronously need it. While a callback is
/// active, this slot lets main-thread `send_message` handle work immediately;
/// other threads still queue and wake the loop. The slot stores an atomic
/// pointer to guard-owned state so lookup is lock-free. The guard restores the
/// previous pointer before dropping that state, so the raw pointers are never
/// treated as valid beyond their callback.
#[derive(Clone, Copy)]
struct MainThreadDispatch<T: UserEvent> {
  app: *mut WinitCefApp<T>,
  event_loop: *const dyn ActiveEventLoop,
}

struct MainThreadDispatchSlot<T: UserEvent> {
  current: AtomicPtr<MainThreadDispatch<T>>,
}

impl<T: UserEvent> MainThreadDispatchSlot<T> {
  fn install(&self, dispatch: &mut MainThreadDispatch<T>) -> *mut MainThreadDispatch<T> {
    self.current.swap(dispatch, Ordering::AcqRel)
  }

  fn restore(&self, current: *mut MainThreadDispatch<T>, previous: *mut MainThreadDispatch<T>) {
    let installed = self.current.swap(previous, Ordering::AcqRel);
    debug_assert_eq!(installed, current);
  }

  fn current(&self) -> Option<&MainThreadDispatch<T>> {
    let current = self.current.load(Ordering::Acquire);
    if current.is_null() {
      None
    } else {
      // SAFETY: the pointer targets the boxed dispatch state owned by
      // `MainThreadDispatchGuard`, whose allocation remains stable while the
      // guard is moved. The slot is restored before that state is dropped, and
      // it is only read by `send_message` after verifying that it is running on
      // the runtime main thread.
      Some(unsafe { &*current })
    }
  }
}

impl<T: UserEvent> Default for MainThreadDispatchSlot<T> {
  fn default() -> Self {
    Self {
      current: AtomicPtr::new(std::ptr::null_mut()),
    }
  }
}

struct MainThreadDispatchGuard<T: UserEvent> {
  context: RuntimeContext<T>,
  dispatch: Box<MainThreadDispatch<T>>,
  previous: *mut MainThreadDispatch<T>,
}

impl<T: UserEvent> Drop for MainThreadDispatchGuard<T> {
  fn drop(&mut self) {
    self
      .context
      .current_dispatch
      .restore(self.dispatch.as_mut(), self.previous);
  }
}

#[allow(clippy::result_large_err)]
fn handle_main_thread_message<T: UserEvent>(
  context: &RuntimeContext<T>,
  message: Message<T>,
) -> std::result::Result<(), Message<T>> {
  let Some(dispatch) = context.current_dispatch.current() else {
    return Err(message);
  };

  // SAFETY: `WinitCefApp::install_current_dispatch` stores pointers to the currently
  // executing winit application handler and event-loop callback. This function
  // is only called on the runtime main thread while that callback is active.
  let app = unsafe { &mut *dispatch.app };
  let event_loop = unsafe { &*dispatch.event_loop };

  app.handle_message(event_loop, message);

  Ok(())
}

impl<T: UserEvent> fmt::Debug for RuntimeContext<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("RuntimeContext").finish()
  }
}

impl<T: UserEvent> RuntimeContext<T> {
  pub(crate) fn send_message(&self, message: Message<T>) -> Result<()> {
    let message = if self.is_main_thread() {
      match handle_main_thread_message(self, message) {
        Ok(()) => return Ok(()),
        Err(message) => message,
      }
    } else {
      message
    };

    self
      .sender
      .send(message)
      .map_err(|_| Error::FailedToSendMessage)?;
    self.proxy.wake_up();
    Ok(())
  }

  pub(crate) fn is_main_thread(&self) -> bool {
    std::thread::current().id() == self.main_thread_id
  }

  /// Run `f` on the main (event-loop) thread.
  ///
  /// When called from the main thread we execute `f` inline instead of posting
  /// it to the channel. Tauri implements several blocking getters as
  /// `run_on_main_thread(|| { .. tx.send(..) }); rx.recv()` (e.g.
  /// `Window::add_child`). Those run during `setup`, which the runtime drives
  /// from winit's `can_create_surfaces`; posting the closure instead of running
  /// it inline would deadlock because the loop cannot drain the task while the
  /// main thread blocks on `rx.recv()`.
  pub(crate) fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    if self.is_main_thread() {
      f();
      Ok(())
    } else {
      self.send_message(Message::Task(Box::new(f)))
    }
  }

  pub(crate) fn next_window_id(&self) -> WindowId {
    self.next_window_id.fetch_add(1, Ordering::Relaxed).into()
  }

  pub(crate) fn next_webview_id(&self) -> u32 {
    self.next_webview_id.fetch_add(1, Ordering::Relaxed)
  }

  pub(crate) fn next_window_event_id(&self) -> u32 {
    self.next_window_event_id.fetch_add(1, Ordering::Relaxed)
  }

  pub(crate) fn next_webview_event_id(&self) -> u32 {
    self.next_webview_event_id.fetch_add(1, Ordering::Relaxed)
  }
}

pub(crate) type AfterWindowCreationCallback = Box<dyn for<'a> Fn(RawWindow<'a>) + Send>;

pub(crate) enum Message<T: UserEvent> {
  EventLoop(EventLoopMessage),
  BrowserClosed(WindowId, u32),
  Opened(Vec<url::Url>),
  #[cfg(target_os = "macos")]
  Reopen {
    has_visible_windows: bool,
  },
  #[cfg(target_os = "macos")]
  AccessibilityChanged {
    enabled: bool,
  },
  CreateWindow {
    window_id: WindowId,
    webview_id: Option<u32>,
    pending: Box<PendingWindow<T, CefRuntime<T>>>,
    after_window_creation: Option<AfterWindowCreationCallback>,
    result_tx: Sender<Result<()>>,
  },
  CreateWebview {
    window_id: WindowId,
    webview_id: u32,
    pending: Box<PendingWebview<T, CefRuntime<T>>>,
    result_tx: Sender<Result<()>>,
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
  NavigateFirstWebview {
    window_id: WindowId,
    url: String,
  },
  DragDropScriptEvent {
    window_id: WindowId,
    webview_id: u32,
    target: browser_client::DragDropEventTarget,
    drag_drop_state: Arc<Mutex<browser_client::DragDropState>>,
    event: browser_client::DragDropScriptEvent,
  },
  Task(Box<dyn FnOnce() + Send>),
  RequestExit(i32),
  UserEvent(T),
}

fn device_event_filter_to_winit(filter: DeviceEventFilter) -> winit::event_loop::DeviceEvents {
  match filter {
    DeviceEventFilter::Always => winit::event_loop::DeviceEvents::Never,
    DeviceEventFilter::Unfocused => winit::event_loop::DeviceEvents::WhenFocused,
    DeviceEventFilter::Never => winit::event_loop::DeviceEvents::Always,
  }
}

pub(crate) enum EventLoopMessage {
  SetTheme(Option<Theme>),
  SetDeviceEventFilter(DeviceEventFilter),
  PrimaryMonitor(Sender<Option<Monitor>>),
  MonitorFromPoint(Sender<Option<Monitor>>, f64, f64),
  AvailableMonitors(Sender<Vec<Monitor>>),
  CursorPosition(Sender<Result<PhysicalPosition<f64>>>),
  DisplayHandle(Sender<std::result::Result<SendRawDisplayHandle, raw_window_handle::HandleError>>),
  #[cfg(target_os = "macos")]
  SetActivationPolicy(tauri_runtime::ActivationPolicy),
  #[cfg(target_os = "macos")]
  SetDockVisibility(bool),
  #[cfg(target_os = "macos")]
  ShowApplication,
  #[cfg(target_os = "macos")]
  HideApplication,
}

macro_rules! event_loop_getter {
  ($self:ident, $variant:ident) => {{
    let (tx, rx) = mpsc::channel();
    match $self
      .context
      .send_message(Message::EventLoop(EventLoopMessage::$variant(tx)))
    {
      Ok(()) => rx.recv().map_err(|_| Error::FailedToReceiveMessage),
      Err(error) => Err(error),
    }
  }};
}

fn find_monitor_from_point(
  monitors: impl Iterator<Item = winit::monitor::MonitorHandle>,
  x: f64,
  y: f64,
) -> Option<winit::monitor::MonitorHandle> {
  monitors.into_iter().find(|monitor| {
    let pos = monitor.position().unwrap_or_default();
    let size = monitor
      .current_video_mode()
      .map(|mode| mode.size())
      .unwrap_or_default();
    x >= pos.x as f64
      && x <= pos.x as f64 + size.width as f64
      && y >= pos.y as f64
      && y <= pos.y as f64 + size.height as f64
  })
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
    .and_then(|path| {
      path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| HELPER_SUFFIXES.iter().any(|suffix| name.ends_with(suffix)))
    })
    .unwrap_or_default()
}

pub(crate) struct AppState<T: UserEvent> {
  pub(crate) windows: HashMap<WindowId, AppWindow>,
  pub(crate) winid_id_to_window_id_map: HashMap<WinitWindowId, WindowId>,
  pub(crate) callback: Box<dyn FnMut(RunEvent<T>)>,
  pub(crate) live_browsers: usize,
  pub(crate) exiting: bool,
}

pub(crate) struct WinitCefApp<T: UserEvent> {
  pub(crate) context: RuntimeContext<T>,
  receiver: Receiver<Message<T>>,
  pub(crate) state: AppState<T>,
  pub(crate) scheme_registry: request_handler::SchemeRegistry,
}

impl<T: UserEvent> WinitCefApp<T> {
  fn new(
    context: RuntimeContext<T>,
    receiver: Receiver<Message<T>>,
    callback: Box<dyn FnMut(RunEvent<T>)>,
    scheme_registry: request_handler::SchemeRegistry,
  ) -> Self {
    Self {
      context,
      receiver,
      state: AppState {
        windows: HashMap::new(),
        winid_id_to_window_id_map: HashMap::new(),
        callback,
        live_browsers: 0,
        exiting: false,
      },
      scheme_registry,
    }
  }

  fn run_callback(&mut self, event: RunEvent<T>) {
    (self.state.callback)(event);
  }

  fn install_current_dispatch(
    &mut self,
    event_loop: &dyn ActiveEventLoop,
  ) -> MainThreadDispatchGuard<T> {
    let mut dispatch = Box::new(MainThreadDispatch {
      app: self as *mut _,
      event_loop: event_loop as *const _,
    });

    let previous = self.context.current_dispatch.install(dispatch.as_mut());

    MainThreadDispatchGuard {
      context: self.context.clone(),
      dispatch,
      previous,
    }
  }

  fn drain_messages(&mut self, event_loop: &dyn ActiveEventLoop) {
    while let Ok(message) = self.receiver.try_recv() {
      self.handle_message(event_loop, message);
    }
  }

  fn handle_message(&mut self, event_loop: &dyn ActiveEventLoop, message: Message<T>) {
    match message {
      Message::EventLoop(message) => self.handle_event_loop_message(event_loop, message),
      Message::BrowserClosed(_window_id, webview_id) => {
        // Standalone webview.close() keeps the child in state until this
        // callback, so cleanup happens here. Window/app teardown removes child
        // bookkeeping before asking CEF to close; then this message is only the
        // lifecycle acknowledgement that lets live_browsers drain.
        //
        // The window_id baked into the browser's handlers can be stale after a
        // reparent, so locate the webview by its process-unique id across every
        // window rather than trusting the message's window_id — otherwise a
        // reparented webview's scheme-handler entries would leak and its
        // AppWebview would linger in the target window forever.
        let child = self.state.windows.values_mut().find_map(|appwindow| {
          appwindow
            .children
            .iter()
            .position(|child| child.webview_id == webview_id)
            .map(|index| appwindow.children.remove(index))
        });
        if let Some(child) = child {
          self.remove_scheme_handler_entries(&child);
        }

        self.state.live_browsers = self.state.live_browsers.saturating_sub(1);
        self.exit_if_done(event_loop);
      }
      Message::CreateWindow {
        window_id,
        webview_id,
        pending,
        after_window_creation,
        result_tx,
      } => {
        let result = self.create_window(
          event_loop,
          window_id,
          webview_id,
          pending,
          after_window_creation,
        );
        let _ = result_tx.send(result);
      }
      Message::CreateWebview {
        window_id,
        webview_id,
        pending,
        result_tx,
      } => {
        let _ = result_tx.send(self.create_webview(window_id, webview_id, *pending));
      }
      Message::Window { window_id, message } => {
        self.handle_window_message(event_loop, window_id, message)
      }
      Message::Webview {
        window_id,
        webview_id,
        message,
      } => self.handle_webview_message(window_id, webview_id, message),
      Message::NavigateFirstWebview { window_id, url } => {
        self.navigate_first_webview(window_id, &url)
      }
      Message::DragDropScriptEvent {
        window_id,
        webview_id,
        target,
        drag_drop_state,
        event,
      } => {
        if let Some(event) = browser_client::event_from_script_event(&drag_drop_state, event) {
          self.emit_drag_drop_event(window_id, webview_id, target, event);
        }
      }
      Message::Task(task) => task(),
      Message::RequestExit(code) => {
        if self.request_exit(Some(code)) {
          self.close_all_browsers();
          self.exit_if_done(event_loop);
        }
      }
      Message::Opened(urls) => self.run_callback(RunEvent::Opened { urls }),
      #[cfg(target_os = "macos")]
      Message::Reopen {
        has_visible_windows,
      } => self.run_callback(RunEvent::Reopen {
        has_visible_windows,
      }),
      #[cfg(target_os = "macos")]
      Message::AccessibilityChanged { enabled } => self.set_browsers_accessibility_state(enabled),
      Message::UserEvent(event) => self.run_callback(RunEvent::UserEvent(event)),
    }
  }

  fn handle_event_loop_message(
    &mut self,
    event_loop: &dyn ActiveEventLoop,
    message: EventLoopMessage,
  ) {
    match message {
      EventLoopMessage::SetTheme(theme) => {
        *self.context.app_wide_theme.lock().unwrap() = theme;
        for appwindow in self.state.windows.values_mut() {
          appwindow.set_theme(theme);
        }
      }
      EventLoopMessage::PrimaryMonitor(tx) => {
        let monitor = event_loop
          .primary_monitor()
          .map(|monitor| winit_monitor_to_tauri_monitor(&monitor));
        let _ = tx.send(monitor);
      }
      EventLoopMessage::MonitorFromPoint(tx, x, y) => {
        let monitor = find_monitor_from_point(event_loop.available_monitors(), x, y)
          .map(|monitor| winit_monitor_to_tauri_monitor(&monitor));
        let _ = tx.send(monitor);
      }
      EventLoopMessage::AvailableMonitors(tx) => {
        let monitors = event_loop
          .available_monitors()
          .map(|monitor| winit_monitor_to_tauri_monitor(&monitor))
          .collect();
        let _ = tx.send(monitors);
      }
      EventLoopMessage::SetDeviceEventFilter(filter) => {
        event_loop.listen_device_events(device_event_filter_to_winit(filter));
      }
      EventLoopMessage::CursorPosition(tx) => {
        let _ = tx.send(event_loop.cursor_position());
      }
      EventLoopMessage::DisplayHandle(tx) => {
        let handle = event_loop
          .display_handle()
          .map(|handle| SendRawDisplayHandle(handle.as_raw()));
        let _ = tx.send(handle);
      }
      #[cfg(target_os = "macos")]
      EventLoopMessage::SetActivationPolicy(activation_policy) => {
        event_loop.set_activation_policy(activation_policy)
      }
      #[cfg(target_os = "macos")]
      EventLoopMessage::SetDockVisibility(visible) => event_loop.set_dock_visibility(visible),
      #[cfg(target_os = "macos")]
      EventLoopMessage::ShowApplication => event_loop.show_application(),
      #[cfg(target_os = "macos")]
      EventLoopMessage::HideApplication => event_loop.hide_application(),
    }
  }

  /// Removes the webview's `(browser_id, scheme)` entries from the scheme registry.
  fn remove_scheme_handler_entries(&self, child: &AppWebview) {
    let mut registry = self.scheme_registry.lock().unwrap();
    for scheme in child.uri_scheme_protocols.keys() {
      registry.remove(&(child.browser_id, scheme.clone()));
    }
  }

  fn emit_drag_drop_event(
    &mut self,
    window_id: WindowId,
    webview_id: u32,
    target: browser_client::DragDropEventTarget,
    event: DragDropEvent,
  ) {
    match target {
      browser_client::DragDropEventTarget::Window => {
        self.emit_window_event(window_id, WindowEvent::DragDrop(event));
      }
      browser_client::DragDropEventTarget::Webview => {
        self.emit_webview_event(window_id, webview_id, WebviewEvent::DragDrop(event));
      }
    }
  }

  fn emit_window_event(&mut self, window_id: WindowId, event: WindowEvent) {
    let Some(appwindow) = self.state.windows.get(&window_id) else {
      return;
    };
    let label = appwindow.label.clone();
    let listeners = appwindow.listeners.clone();

    self.run_callback(RunEvent::WindowEvent {
      label,
      event: event.clone(),
    });

    {
      let listeners = listeners.lock().unwrap();
      for handler in listeners.values() {
        handler(&event);
      }
    }
  }

  fn emit_webview_event(&mut self, window_id: WindowId, webview_id: u32, event: WebviewEvent) {
    let Some(appwindow) = self.state.windows.get(&window_id) else {
      return;
    };
    let Some(child) = appwindow
      .children
      .iter()
      .find(|child| child.webview_id == webview_id)
    else {
      return;
    };
    let label = child.label.clone();
    let listeners = child.listeners.clone();

    self.run_callback(RunEvent::WebviewEvent {
      label,
      event: event.clone(),
    });

    {
      let listeners = listeners.lock().unwrap();
      for handler in listeners.values() {
        handler(&event);
      }
    }
  }

  fn request_exit(&mut self, code: Option<i32>) -> bool {
    // if we already exiting, don't request exit again
    if self.state.exiting {
      return false;
    }

    let (tx, rx) = mpsc::channel();
    self.run_callback(RunEvent::ExitRequested { code, tx });

    if matches!(rx.try_recv(), Ok(ExitRequestedEventAction::Prevent)) {
      false
    } else {
      self.state.exiting = true;
      true
    }
  }

  pub(crate) fn close_window(&mut self, window_id: WindowId, event_loop: &dyn ActiveEventLoop) {
    let Some(appwindow) = self.state.windows.remove(&window_id) else {
      return;
    };
    self
      .state
      .winid_id_to_window_id_map
      .remove(&appwindow.window.id());
    // The window is gone from state, so BrowserClosed will not find these
    // children later. Clean registry entries while we still hold them; the CEF
    // shutdown drain is still enforced by live_browsers.
    for child in &appwindow.children {
      self.remove_scheme_handler_entries(child);
      child.host.close_browser(1);
    }
    self.exit_if_done(event_loop);
  }

  pub(crate) fn request_window_close(
    &mut self,
    window_id: WindowId,
    event_loop: &dyn ActiveEventLoop,
  ) {
    // Avoid requesting window close if we already exisitng
    if self.state.exiting {
      self.close_window(window_id, event_loop);
      return;
    }

    let (tx, rx) = mpsc::channel();
    let Some(appwindow) = self.state.windows.get(&window_id) else {
      return;
    };
    let label = appwindow.label.clone();
    let listeners = appwindow.listeners.clone();

    {
      let listeners = listeners.lock().unwrap();
      for handler in listeners.values() {
        handler(&WindowEvent::CloseRequested {
          signal_tx: tx.clone(),
        });
      }
    }

    self.run_callback(RunEvent::WindowEvent {
      label,
      event: WindowEvent::CloseRequested { signal_tx: tx },
    });

    if !matches!(rx.try_recv(), Ok(true)) {
      self.close_window(window_id, event_loop);
    }
  }

  fn navigate_first_webview(&self, window_id: WindowId, url: &str) {
    let Some(frame) = self
      .state
      .windows
      .get(&window_id)
      .and_then(|window| window.children.first())
      .and_then(|webview| webview.browser.main_frame())
    else {
      return;
    };

    frame.load_url(Some(&CefString::from(url)));
  }

  fn close_all_browsers(&mut self) {
    // App shutdown follows the same eager bookkeeping cleanup as window
    // teardown. live_browsers keeps the loop alive until CEF confirms every
    // browser close through BrowserClosed.
    for appwindow in self.state.windows.values() {
      for child in &appwindow.children {
        self.remove_scheme_handler_entries(child);
        child.host.close_browser(1);
      }
    }
    self.state.windows.clear();
    self.state.winid_id_to_window_id_map.clear();
  }

  #[cfg(target_os = "macos")]
  fn set_browsers_accessibility_state(&self, enabled: bool) {
    let state = if enabled {
      State::ENABLED
    } else {
      State::DISABLED
    };
    for appwindow in self.state.windows.values() {
      for child in &appwindow.children {
        child.host.set_accessibility_state(state);
      }
    }
  }

  fn exit_if_done(&mut self, event_loop: &dyn ActiveEventLoop) {
    if self.state.live_browsers != 0 {
      return;
    }

    if self.state.exiting || (self.state.windows.is_empty() && self.request_exit(None)) {
      self.run_callback(RunEvent::Exit);
      event_loop.exit();
    }
  }

  /// Service the default GLib main context so the external message pump's GLib
  /// source (and any GTK work CEF schedules) gets dispatched, then arm winit to
  /// wake when the next GLib pump deadline is due. Windows/macOS need no
  /// equivalent: their pump timers live on the native loop winit already runs.
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn service_glib(&self, event_loop: &dyn ActiveEventLoop) {
    let context = gtk::glib::MainContext::default();
    while context.pending() {
      context.iteration(false);
    }
    if let Some(deadline) = self.context.cef_pump.next_deadline() {
      event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(deadline));
    }
  }
}

impl<T: UserEvent> ApplicationHandler for WinitCefApp<T> {
  fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
    let _guard = self.install_current_dispatch(event_loop);
    self.drain_messages(event_loop);
  }

  fn new_events(&mut self, event_loop: &dyn ActiveEventLoop, cause: StartCause) {
    let _guard = self.install_current_dispatch(event_loop);
    match cause {
      StartCause::Init => {
        self.run_callback(RunEvent::Ready);
        self.context.cef_pump.do_work();
      }
      // Match wry/tao, which emit `Resumed` on each `Poll` start cause.
      StartCause::Poll => self.run_callback(RunEvent::Resumed),
      _ => {}
    }
  }

  fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
    let _guard = self.install_current_dispatch(event_loop);
    self.drain_messages(event_loop);
  }

  fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
    let _guard = self.install_current_dispatch(event_loop);
    // TODO: remove once migrated to winit-gtk4
    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "netbsd",
      target_os = "openbsd"
    ))]
    self.service_glib(event_loop);
    self.run_callback(RunEvent::MainEventsCleared);
  }

  fn window_event(
    &mut self,
    event_loop: &dyn ActiveEventLoop,
    winit_id: WinitWindowId,
    event: WinitWindowEvent,
  ) {
    let _guard = self.install_current_dispatch(event_loop);
    let Some(window_id) = self.state.winid_id_to_window_id_map.get(&winit_id).copied() else {
      return;
    };
    let Some(appwindow) = self.state.windows.get_mut(&window_id) else {
      return;
    };

    match event {
      WinitWindowEvent::CloseRequested => self.request_window_close(window_id, event_loop),

      WinitWindowEvent::Destroyed => {
        if !self.state.exiting {
          self.emit_window_event(window_id, WindowEvent::Destroyed);
        }
        self.close_window(window_id, event_loop);
      }
      WinitWindowEvent::SurfaceResized(size) => {
        webview::layout_app_window(appwindow);
        self.emit_window_event(window_id, WindowEvent::Resized(size));
      }
      WinitWindowEvent::ScaleFactorChanged {
        scale_factor,
        surface_size_writer,
      } => {
        let new_inner_size = surface_size_writer
          .surface_size()
          .unwrap_or_else(|_| appwindow.window.surface_size());
        webview::layout_app_window(appwindow);
        self.emit_window_event(
          window_id,
          WindowEvent::ScaleFactorChanged {
            scale_factor,
            new_inner_size,
          },
        );
      }
      WinitWindowEvent::Moved(pos) => {
        self.emit_window_event(
          window_id,
          WindowEvent::Moved(PhysicalPosition::new(pos.x, pos.y)),
        );
      }
      WinitWindowEvent::Focused(focused) => {
        self.emit_window_event(window_id, WindowEvent::Focused(focused));
      }
      WinitWindowEvent::ThemeChanged(theme) => {
        let system_theme = winit_theme_to_tauri_theme(theme);
        if let Some(explicit_theme) = appwindow.preferred_theme() {
          appwindow.set_theme(Some(explicit_theme));
        }
        self.emit_window_event(window_id, WindowEvent::ThemeChanged(system_theme));
      }
      WinitWindowEvent::DragEntered { paths, position } => {
        let event = DragDropEvent::Enter { paths, position };
        self.emit_window_event(window_id, WindowEvent::DragDrop(event));
      }
      WinitWindowEvent::DragMoved { position } => {
        let event = DragDropEvent::Over { position };
        self.emit_window_event(window_id, WindowEvent::DragDrop(event));
      }
      WinitWindowEvent::DragDropped { paths, position } => {
        let event = DragDropEvent::Drop { paths, position };
        self.emit_window_event(window_id, WindowEvent::DragDrop(event));
      }
      WinitWindowEvent::DragLeft { .. } => {
        self.emit_window_event(window_id, WindowEvent::DragDrop(DragDropEvent::Leave));
      }
      #[cfg(windows)]
      WinitWindowEvent::RedrawRequested => {
        appwindow.draw_background_surface();
      }
      _ => {}
    }
  }
}

wrap_app! {
  struct TauriCefApp<T: UserEvent> {
    context: RuntimeContext<T>,
    context_initialized: Arc<AtomicBool>,
    deep_link_schemes: Vec<String>,
    command_line_args: Vec<(String, Option<String>)>,
  }

  impl App {
    fn render_process_handler(&self) -> Option<RenderProcessHandler> {
      Some(ipc::TauriRenderProcessHandler::new())
    }

    fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
      Some(browser_client::TauriCefBrowserProcessHandler::new(
        self.context.clone(),
        self.context_initialized.clone(),
        self.deep_link_schemes.clone(),
      ))
    }

    fn on_before_command_line_processing(
      &self,
      _process_type: Option<&CefString>,
      command_line: Option<&mut CommandLine>,
    ) {
      if let Some(command_line) = command_line {
        for (arg, value) in &self.command_line_args {
          if let Some(value) = value {
            command_line.append_switch_with_value(
              Some(&CefString::from(arg.as_str())),
              Some(&CefString::from(value.as_str())),
            );
          } else if arg.starts_with("-") {
            command_line.append_switch(Some(&CefString::from(arg.as_str())));
          } else {
            command_line.append_argument(Some(&CefString::from(arg.as_str())));
          }
        }
      }
    }
  }
}

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

  let _ = cef::api_hash(sys::CEF_API_VERSION_LAST, 0);
  let mut app = TauriCefHelperApp::new();
  let _ = cef::execute_process(
    Some(args.as_main_args()),
    Some(&mut app),
    std::ptr::null_mut(),
  );
}

wrap_app! {
  struct TauriCefHelperApp;

  impl App {
    fn render_process_handler(&self) -> Option<RenderProcessHandler> {
      Some(ipc::TauriRenderProcessHandler::new())
    }
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
    activation_policy: tauri_runtime::ActivationPolicy,
  ) -> Result<()> {
    let message = Message::EventLoop(EventLoopMessage::SetActivationPolicy(activation_policy));
    self.context.send_message(message)
  }

  #[cfg(target_os = "macos")]
  fn set_dock_visibility(&self, visible: bool) -> Result<()> {
    let message = Message::EventLoop(EventLoopMessage::SetDockVisibility(visible));
    self.context.send_message(message)
  }

  fn request_exit(&self, code: i32) -> Result<()> {
    self.context.send_message(Message::RequestExit(code))
  }

  fn create_window<F: Fn(RawWindow<'_>) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self::Runtime>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    create_window_detached(&self.context, pending, after_window_creation)
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>> {
    create_webview_detached(&self.context, window_id, pending)
  }

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.context.run_on_main_thread(f)
  }

  fn display_handle(
    &self,
  ) -> std::result::Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
    let raw = event_loop_getter!(self, DisplayHandle)
      .map_err(|_| raw_window_handle::HandleError::Unavailable)??;
    // SAFETY: the descriptor was produced by the live event loop on its own
    // thread; the borrowed handle is valid for as long as the runtime is.
    Ok(unsafe { DisplayHandle::borrow_raw(raw.0) })
  }

  fn primary_monitor(&self) -> Option<Monitor> {
    event_loop_getter!(self, PrimaryMonitor).ok().flatten()
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Option<Monitor> {
    let (tx, rx) = mpsc::channel();
    self
      .context
      .send_message(Message::EventLoop(EventLoopMessage::MonitorFromPoint(
        tx, x, y,
      )))
      .and_then(|_| rx.recv().map_err(|_| Error::FailedToReceiveMessage))
      .ok()
      .flatten()
  }

  fn available_monitors(&self) -> Vec<Monitor> {
    event_loop_getter!(self, AvailableMonitors).unwrap_or_default()
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    event_loop_getter!(self, CursorPosition)?
  }

  fn set_theme(&self, theme: Option<Theme>) {
    let message = Message::EventLoop(EventLoopMessage::SetTheme(theme));
    let _ = self.context.send_message(message);
  }

  #[cfg(target_os = "macos")]
  fn show(&self) -> Result<()> {
    let message = Message::EventLoop(EventLoopMessage::ShowApplication);
    self.context.send_message(message)
  }

  #[cfg(target_os = "macos")]
  fn hide(&self) -> Result<()> {
    let message = Message::EventLoop(EventLoopMessage::HideApplication);
    self.context.send_message(message)
  }

  fn set_device_event_filter(&self, filter: DeviceEventFilter) {
    let message = Message::EventLoop(EventLoopMessage::SetDeviceEventFilter(filter));
    let _ = self.context.send_message(message);
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
    _uuid: [u8; 16],
    cb: F,
  ) -> Result<()> {
    cb(Ok(()));
    Ok(())
  }
}

pub struct CefRuntime<T: UserEvent> {
  event_loop: EventLoop,
  receiver: Receiver<Message<T>>,
  context: RuntimeContext<T>,
  scheme_registry: request_handler::SchemeRegistry,
  #[cfg(target_os = "macos")]
  _app_delegate: Option<objc2::rc::Retained<crate::platform::macos::AppDelegate>>,
}

impl<T: UserEvent> fmt::Debug for CefRuntime<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("CefRuntime").finish()
  }
}

impl<T: UserEvent> CefRuntime<T> {
  fn init(
    mut event_loop_builder: EventLoopBuilder,
    runtime_args: RuntimeInitArgs<RuntimeInitAttribute>,
  ) -> Result<Self> {
    let args = cef::args::Args::new();

    #[cfg(target_os = "macos")]
    let is_helper = is_cef_helper_process();

    #[cfg(target_os = "macos")]
    let (_sandbox, _loader) = {
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

      (sandbox, loader)
    };

    #[cfg(target_os = "macos")]
    if !is_helper {
      crate::platform::macos::setup_application();
    }

    // The CEF API version table must be initialized before any other CEF call
    // (e.g. `args.as_cmd_line()` below), otherwise the process crashes with no
    // diagnostics.
    let _ = cef::api_hash(sys::CEF_API_VERSION_LAST, 0);

    // Handle CEF subprocesses (renderer/GPU/utility) before any browser-only
    // setup such as building the event loop, creating cache directories, or the
    // runtime context. The browser (main) process has no `type` switch;
    // subprocesses are launched with one (e.g. `--type=renderer`).
    let is_browser_process = args
      .as_cmd_line()
      .map(|cmd| cmd.has_switch(Some(&CefString::from("type"))) != 1)
      .unwrap_or(true);

    if !is_browser_process {
      let mut helper_app = TauriCefHelperApp::new();
      let ret = cef::execute_process(
        Some(args.as_main_args()),
        Some(&mut helper_app),
        std::ptr::null_mut(),
      );
      // A subprocess finished its work; exit with its exit code instead of
      // falling through to browser runtime initialization.
      std::process::exit(ret.max(0));
    }

    let mut command_line_args = Vec::new();
    let mut deep_link_schemes = Vec::new();
    let mut cache_path_override = None::<PathBuf>;
    for arg in runtime_args.platform_specific_attributes {
      match arg {
        RuntimeInitAttribute::CommandLineArgs { args } => command_line_args.extend(args),
        RuntimeInitAttribute::DeepLinkSchemes { schemes } => deep_link_schemes.extend(schemes),
        RuntimeInitAttribute::CachePath { path } => cache_path_override = Some(path),
      }
    }

    let cache_path = cache_path_override.unwrap_or_else(|| {
      let cache_base = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
      cache_base.join(&runtime_args.identifier).join("cef")
    });
    let _ = create_dir_all(&cache_path);

    // Force X11 usage on Linux
    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "netbsd",
      target_os = "openbsd"
    ))]
    {
      command_line_args.push(("ozone-platform".to_string(), Some("x11".to_string())));
      event_loop_builder.with_x11();
    }

    #[cfg(windows)]
    if let Some(hook) = runtime_args.msg_hook {
      use winit::platform::windows::EventLoopBuilderExtWindows;
      event_loop_builder.with_msg_hook(hook);
    }

    #[cfg(target_os = "macos")]
    event_loop_builder.with_default_menu(false);

    let event_loop = event_loop_builder
      .build()
      .map_err(|_| Error::CreateWindow)?;
    let proxy = event_loop.create_proxy();
    let (sender, receiver) = mpsc::channel();
    let context_initialized = Arc::new(AtomicBool::new(false));
    let cef_pump = CefExternalPump::new();
    let context = RuntimeContext {
      sender: sender.clone(),
      proxy: proxy.clone(),
      main_thread_id: std::thread::current().id(),
      next_window_id: Default::default(),
      next_webview_id: Default::default(),
      next_window_event_id: Default::default(),
      next_webview_event_id: Default::default(),
      current_dispatch: Default::default(),
      app_wide_theme: Default::default(),
      cef_pump,
      cache_path: Arc::new(cache_path.clone()),
    };

    command_line_args.push(("--enable-media-stream".to_string(), None));
    let mut app = TauriCefApp::new(
      context.clone(),
      context_initialized.clone(),
      deep_link_schemes,
      command_line_args,
    );

    // Subprocesses already exited above, so this must be the browser process;
    // `execute_process` returns -1 there to signal normal startup should follow.
    let ret = cef::execute_process(
      Some(args.as_main_args()),
      Some(&mut app),
      std::ptr::null_mut(),
    );
    assert_eq!(
      ret, -1,
      "CEF browser process unexpectedly returned from execute_process"
    );

    let settings = cef::Settings {
      no_sandbox: !cfg!(feature = "sandbox") as i32,
      cache_path: cache_path.to_string_lossy().to_string().as_str().into(),
      external_message_pump: 1,
      ..Default::default()
    };
    if cef::initialize(
      Some(args.as_main_args()),
      Some(&settings),
      Some(&mut app),
      std::ptr::null_mut(),
    ) != 1
    {
      return Err(Error::WebviewRuntimeNotInstalled);
    }

    #[cfg(target_os = "macos")]
    let app_delegate = if !is_helper {
      use crate::platform::macos::AppDelegateEvent;

      let context_ = context.clone();
      let handler = Box::new(move |event| match event {
        AppDelegateEvent::TryTerminate => {
          let _ = context_.send_message(Message::RequestExit(0));
        }
        AppDelegateEvent::Reopen {
          has_visible_windows,
        } => {
          let _ = context_.send_message(Message::Reopen {
            has_visible_windows,
          });
        }
        AppDelegateEvent::AccessibilityChanged { enabled } => {
          let _ = context_.send_message(Message::AccessibilityChanged { enabled });
        }
        AppDelegateEvent::OpenURLs { urls } => {
          let _ = context_.send_message(Message::Opened(urls));
        }
      });
      let app_delegate = crate::platform::macos::set_application_event_handler(handler);
      Some(app_delegate)
    } else {
      None
    };

    // Wait for the CEF context to initialize before returning, so that the runtime is ready to create browsers.
    while !context_initialized.load(Ordering::SeqCst) {
      context.cef_pump.do_work();
      std::thread::sleep(Duration::from_millis(1));
    }

    Ok(Self {
      event_loop,
      receiver,
      context,
      scheme_registry: Default::default(),
      #[cfg(target_os = "macos")]
      _app_delegate: app_delegate,
    })
  }
}

impl<T: UserEvent> Runtime<T> for CefRuntime<T> {
  type WindowDispatcher = CefWindowDispatcher<T>;
  type WebviewDispatcher = CefWebviewDispatcher<T>;
  type Handle = CefRuntimeHandle<T>;
  type EventLoopProxy = EventProxy<T>;
  type PlatformSpecificWebviewAttribute = WebviewAtribute;
  type Webview = Webview;
  type PlatformSpecificInitAttribute = RuntimeInitAttribute;
  type WindowOpener = NewWindowOpener;

  fn new(args: RuntimeInitArgs<Self::PlatformSpecificInitAttribute>) -> Result<Self> {
    Self::init(EventLoopBuilder::default(), args)
  }

  #[cfg(any(
    windows,
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn new_any_thread(args: RuntimeInitArgs<Self::PlatformSpecificInitAttribute>) -> Result<Self> {
    let mut event_loop_builder = EventLoopBuilder::default();
    event_loop_builder.with_any_thread(true);
    Self::init(event_loop_builder, args)
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
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self>> {
    create_window_detached(&self.context, pending, after_window_creation)
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self>,
  ) -> Result<DetachedWebview<T, Self>> {
    create_webview_detached(&self.context, window_id, pending)
  }

  fn primary_monitor(&self) -> Option<Monitor> {
    event_loop_getter!(self, PrimaryMonitor).ok().flatten()
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Option<Monitor> {
    let (tx, rx) = mpsc::channel();
    self
      .context
      .send_message(Message::EventLoop(EventLoopMessage::MonitorFromPoint(
        tx, x, y,
      )))
      .and_then(|_| rx.recv().map_err(|_| Error::FailedToReceiveMessage))
      .ok()
      .flatten()
  }

  fn available_monitors(&self) -> Vec<Monitor> {
    event_loop_getter!(self, AvailableMonitors).unwrap_or_default()
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    event_loop_getter!(self, CursorPosition)?
  }

  fn set_theme(&self, theme: Option<Theme>) {
    let message = Message::EventLoop(EventLoopMessage::SetTheme(theme));
    let _ = self.context.send_message(message);
  }

  #[cfg(target_os = "macos")]
  fn set_activation_policy(&mut self, activation_policy: tauri_runtime::ActivationPolicy) {
    let message = Message::EventLoop(EventLoopMessage::SetActivationPolicy(activation_policy));
    let _ = self.context.send_message(message);
  }

  #[cfg(target_os = "macos")]
  fn set_dock_visibility(&mut self, visible: bool) {
    let message = Message::EventLoop(EventLoopMessage::SetDockVisibility(visible));
    let _ = self.context.send_message(message);
  }

  #[cfg(target_os = "macos")]
  fn show(&self) {
    let message = Message::EventLoop(EventLoopMessage::ShowApplication);
    let _ = self.context.send_message(message);
  }

  #[cfg(target_os = "macos")]
  fn hide(&self) {
    let message = Message::EventLoop(EventLoopMessage::HideApplication);
    let _ = self.context.send_message(message);
  }

  fn set_device_event_filter(&mut self, filter: DeviceEventFilter) {
    self
      .event_loop
      .listen_device_events(device_event_filter_to_winit(filter));
  }

  fn custom_scheme_url(scheme: &str, https: bool) -> String {
    format!(
      "{}://{scheme}.localhost",
      if https { "https" } else { "http" }
    )
  }

  fn run_iteration<F: FnMut(RunEvent<T>) + 'static>(&mut self, mut callback: F) {
    while let Ok(message) = self.receiver.try_recv() {
      if let Message::UserEvent(event) = message {
        callback(RunEvent::UserEvent(event));
      }
    }
    self.context.cef_pump.do_work();
    callback(RunEvent::MainEventsCleared);
  }

  fn run_return<F: FnMut(RunEvent<T>) + 'static>(self, callback: F) -> i32 {
    self.run(callback);
    // TODO: return the exit code from the runtime, if possible. For now, always return 0
    0
  }

  fn run<F: FnMut(RunEvent<T>) + 'static>(self, callback: F) {
    let app = WinitCefApp::new(
      self.context,
      self.receiver,
      Box::new(callback),
      self.scheme_registry,
    );
    let _ = self.event_loop.run_app(app);
    cef::shutdown();
  }
}
