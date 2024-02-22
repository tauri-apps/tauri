// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/tauri/raw/dev/.github/splash.png)](https://tauri.app)
//!
//! The [`wry`] Tauri [`Runtime`].

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use raw_window_handle::{DisplayHandle, HasDisplayHandle, HasWindowHandle};

use tauri_runtime::{
  monitor::Monitor,
  webview::{DetachedWebview, DownloadEvent, PendingWebview, WebviewIpcHandler},
  window::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size},
    CursorIcon, DetachedWindow, FileDropEvent, PendingWindow, RawWindow, WebviewEvent,
    WindowBuilder, WindowBuilderBase, WindowEvent, WindowId,
  },
  DeviceEventFilter, Error, EventLoopProxy, ExitRequestedEventAction, Icon, ProgressBarState,
  ProgressBarStatus, Result, RunEvent, Runtime, RuntimeHandle, RuntimeInitArgs, UserAttentionType,
  UserEvent, WebviewDispatch, WebviewEventId, WindowDispatch, WindowEventId,
};

#[cfg(target_os = "macos")]
use tao::platform::macos::{EventLoopWindowTargetExtMacOS, WindowBuilderExtMacOS};
#[cfg(target_os = "linux")]
use tao::platform::unix::{WindowBuilderExtUnix, WindowExtUnix};
#[cfg(windows)]
use tao::platform::windows::{WindowBuilderExtWindows, WindowExtWindows};
#[cfg(windows)]
use webview2_com::FocusChangedEventHandler;
#[cfg(windows)]
use windows::Win32::{Foundation::HWND, System::WinRT::EventRegistrationToken};
#[cfg(windows)]
use wry::WebViewBuilderExtWindows;

use tao::{
  dpi::{
    LogicalPosition as TaoLogicalPosition, LogicalSize as TaoLogicalSize,
    PhysicalPosition as TaoPhysicalPosition, PhysicalSize as TaoPhysicalSize,
    Position as TaoPosition, Size as TaoSize,
  },
  event::{Event, StartCause, WindowEvent as TaoWindowEvent},
  event_loop::{
    ControlFlow, DeviceEventFilter as TaoDeviceEventFilter, EventLoop, EventLoopBuilder,
    EventLoopProxy as TaoEventLoopProxy, EventLoopWindowTarget,
  },
  monitor::MonitorHandle,
  window::{
    CursorIcon as TaoCursorIcon, Fullscreen, Icon as TaoWindowIcon,
    ProgressBarState as TaoProgressBarState, ProgressState as TaoProgressState, Theme as TaoTheme,
    UserAttentionType as TaoUserAttentionType,
  },
};
#[cfg(target_os = "macos")]
use tauri_utils::TitleBarStyle;
use tauri_utils::{config::WindowConfig, debug_eprintln, Theme};
use url::Url;
use wry::{
  FileDropEvent as WryFileDropEvent, ProxyConfig, ProxyEndpoint, WebContext, WebView,
  WebViewBuilder,
};

pub use tao;
pub use tao::window::{Window, WindowBuilder as TaoWindowBuilder, WindowId as TaoWindowId};
pub use wry;
pub use wry::webview_version;

#[cfg(windows)]
use wry::WebViewExtWindows;
#[cfg(target_os = "android")]
use wry::{
  prelude::{dispatch, find_class},
  WebViewBuilderExtAndroid, WebViewExtAndroid,
};
#[cfg(not(any(
  target_os = "windows",
  target_os = "macos",
  target_os = "ios",
  target_os = "android"
)))]
use wry::{WebViewBuilderExtUnix, WebViewExtUnix};

#[cfg(target_os = "macos")]
pub use tao::platform::macos::{
  ActivationPolicy as TaoActivationPolicy, EventLoopExtMacOS, WindowExtMacOS,
};
#[cfg(target_os = "macos")]
use tauri_runtime::ActivationPolicy;

use std::{
  cell::RefCell,
  collections::{
    hash_map::Entry::{Occupied, Vacant},
    BTreeMap, HashMap,
  },
  fmt,
  ops::Deref,
  path::PathBuf,
  rc::Rc,
  sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    mpsc::{channel, Sender},
    Arc, Mutex, Weak,
  },
  thread::{current as current_thread, ThreadId},
};

pub type WebviewId = u32;
type IpcHandler = dyn Fn(String) + 'static;

#[cfg(any(
  windows,
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
mod undecorated_resizing;

mod webview;
pub use webview::Webview;

pub type WebContextStore = Arc<Mutex<HashMap<Option<PathBuf>, WebContext>>>;
// window
pub type WindowEventHandler = Box<dyn Fn(&WindowEvent) + Send>;
pub type WindowEventListeners = Arc<Mutex<HashMap<WindowEventId, WindowEventHandler>>>;
pub type WebviewEventHandler = Box<dyn Fn(&WebviewEvent) + Send>;
pub type WebviewEventListeners = Arc<Mutex<HashMap<WebviewEventId, WebviewEventHandler>>>;

#[derive(Debug, Clone, Default)]
pub struct WindowIdStore(Arc<Mutex<HashMap<TaoWindowId, WindowId>>>);

impl WindowIdStore {
  pub fn insert(&self, w: TaoWindowId, id: WindowId) {
    self.0.lock().unwrap().insert(w, id);
  }

  fn get(&self, w: &TaoWindowId) -> Option<WindowId> {
    self.0.lock().unwrap().get(w).copied()
  }
}

#[macro_export]
macro_rules! getter {
  ($self: ident, $rx: expr, $message: expr) => {{
    $crate::send_user_message(&$self.context, $message)?;
    $rx
      .recv()
      .map_err(|_| $crate::Error::FailedToReceiveMessage)
  }};
}

macro_rules! window_getter {
  ($self: ident, $message: expr) => {{
    let (tx, rx) = channel();
    getter!($self, rx, Message::Window($self.window_id, $message(tx)))
  }};
}

macro_rules! webview_getter {
  ($self: ident, $message: expr) => {{
    let (tx, rx) = channel();
    getter!(
      $self,
      rx,
      Message::Webview($self.window_id, $self.webview_id, $message(tx))
    )
  }};
}

pub(crate) fn send_user_message<T: UserEvent>(
  context: &Context<T>,
  message: Message<T>,
) -> Result<()> {
  if current_thread().id() == context.main_thread_id {
    handle_user_message(
      &context.main_thread.window_target,
      message,
      UserMessageContext {
        window_id_map: context.window_id_map.clone(),
        windows: context.main_thread.windows.clone(),
      },
    );
    Ok(())
  } else {
    context
      .proxy
      .send_event(message)
      .map_err(|_| Error::FailedToSendMessage)
  }
}

#[derive(Clone)]
pub struct Context<T: UserEvent> {
  pub window_id_map: WindowIdStore,
  main_thread_id: ThreadId,
  pub proxy: TaoEventLoopProxy<Message<T>>,
  main_thread: DispatcherMainThreadContext<T>,
  plugins: Arc<Mutex<Vec<Box<dyn Plugin<T> + Send>>>>,
  next_window_id: Arc<AtomicU32>,
  next_webview_id: Arc<AtomicU32>,
  next_window_event_id: Arc<AtomicU32>,
  next_webview_event_id: Arc<AtomicU32>,
  next_webcontext_id: Arc<AtomicU32>,
}

impl<T: UserEvent> Context<T> {
  pub fn run_threaded<R, F>(&self, f: F) -> R
  where
    F: FnOnce(Option<&DispatcherMainThreadContext<T>>) -> R,
  {
    f(if current_thread().id() == self.main_thread_id {
      Some(&self.main_thread)
    } else {
      None
    })
  }

  fn next_window_id(&self) -> WindowId {
    self.next_window_id.fetch_add(1, Ordering::Relaxed).into()
  }

  fn next_webview_id(&self) -> WebviewId {
    self.next_webview_id.fetch_add(1, Ordering::Relaxed)
  }

  fn next_window_event_id(&self) -> u32 {
    self.next_window_event_id.fetch_add(1, Ordering::Relaxed)
  }

  fn next_webview_event_id(&self) -> u32 {
    self.next_webview_event_id.fetch_add(1, Ordering::Relaxed)
  }

  fn next_webcontext_id(&self) -> u32 {
    self.next_webcontext_id.fetch_add(1, Ordering::Relaxed)
  }
}

impl<T: UserEvent> Context<T> {
  fn create_window<F: Fn(RawWindow) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Wry<T>>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Wry<T>>> {
    let label = pending.label.clone();
    let context = self.clone();
    let window_id = self.next_window_id();
    let webview_id = pending.webview.as_ref().map(|_| context.next_webview_id());

    send_user_message(
      self,
      Message::CreateWindow(
        window_id,
        Box::new(move |event_loop| {
          create_window(
            window_id,
            webview_id.unwrap_or_default(),
            event_loop,
            &context,
            pending,
            after_window_creation,
          )
        }),
      ),
    )?;

    let dispatcher = WryWindowDispatcher {
      window_id,
      context: self.clone(),
    };

    let detached_webview = webview_id.map(|id| DetachedWebview {
      label: label.clone(),
      dispatcher: WryWebviewDispatcher {
        window_id,
        webview_id: id,
        context: self.clone(),
      },
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
    pending: PendingWebview<T, Wry<T>>,
  ) -> Result<DetachedWebview<T, Wry<T>>> {
    let label = pending.label.clone();
    let context = self.clone();

    let webview_id = self.next_webview_id();

    send_user_message(
      self,
      Message::CreateWebview(
        window_id,
        Box::new(move |window| {
          create_webview(
            WebviewKind::WindowChild,
            window,
            window_id,
            webview_id,
            &context,
            pending,
          )
        }),
      ),
    )?;

    let dispatcher = WryWebviewDispatcher {
      window_id,
      webview_id,
      context: self.clone(),
    };

    Ok(DetachedWebview { label, dispatcher })
  }
}

#[cfg(feature = "tracing")]
#[derive(Debug, Clone, Default)]
pub struct ActiveTraceSpanStore(Rc<RefCell<Vec<ActiveTracingSpan>>>);

#[cfg(feature = "tracing")]
impl ActiveTraceSpanStore {
  pub fn remove_window_draw(&self, window_id: TaoWindowId) {
    let mut store = self.0.borrow_mut();
    if let Some(index) = store
      .iter()
      .position(|t| matches!(t, ActiveTracingSpan::WindowDraw { id, span: _ } if id == &window_id))
    {
      store.remove(index);
    }
  }
}

#[cfg(feature = "tracing")]
#[derive(Debug)]
pub enum ActiveTracingSpan {
  WindowDraw {
    id: TaoWindowId,
    span: tracing::span::EnteredSpan,
  },
}

#[derive(Debug)]
pub struct WindowsStore(RefCell<BTreeMap<WindowId, WindowWrapper>>);

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for WindowsStore {}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for WindowsStore {}

#[derive(Debug, Clone)]
pub struct DispatcherMainThreadContext<T: UserEvent> {
  pub window_target: EventLoopWindowTarget<Message<T>>,
  pub web_context: WebContextStore,
  // changing this to an Rc will cause frequent app crashes.
  pub windows: Arc<WindowsStore>,
  #[cfg(feature = "tracing")]
  pub active_tracing_spans: ActiveTraceSpanStore,
}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Send for DispatcherMainThreadContext<T> {}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for DispatcherMainThreadContext<T> {}

impl<T: UserEvent> fmt::Debug for Context<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Context")
      .field("main_thread_id", &self.main_thread_id)
      .field("proxy", &self.proxy)
      .field("main_thread", &self.main_thread)
      .finish()
  }
}

pub struct DeviceEventFilterWrapper(pub TaoDeviceEventFilter);

impl From<DeviceEventFilter> for DeviceEventFilterWrapper {
  fn from(item: DeviceEventFilter) -> Self {
    match item {
      DeviceEventFilter::Always => Self(TaoDeviceEventFilter::Always),
      DeviceEventFilter::Never => Self(TaoDeviceEventFilter::Never),
      DeviceEventFilter::Unfocused => Self(TaoDeviceEventFilter::Unfocused),
    }
  }
}

/// Wrapper around a [`tao::window::Icon`] that can be created from an [`Icon`].
pub struct TaoIcon(pub TaoWindowIcon);

fn icon_err<E: std::error::Error + Send + Sync + 'static>(e: E) -> Error {
  Error::InvalidIcon(Box::new(e))
}

impl TryFrom<Icon> for TaoIcon {
  type Error = Error;
  fn try_from(icon: Icon) -> std::result::Result<Self, Self::Error> {
    TaoWindowIcon::from_rgba(icon.rgba, icon.width, icon.height)
      .map(Self)
      .map_err(icon_err)
  }
}

pub struct WindowEventWrapper(pub Option<WindowEvent>);

impl WindowEventWrapper {
  fn parse(window: &WindowWrapper, event: &TaoWindowEvent<'_>) -> Self {
    match event {
      // resized event from tao doesn't include a reliable size on macOS
      // because wry replaces the NSView
      TaoWindowEvent::Resized(_) => {
        if let Some(w) = &window.inner {
          Self(Some(WindowEvent::Resized(
            PhysicalSizeWrapper(inner_size(
              w,
              &window.webviews,
              window.has_children.load(Ordering::Relaxed),
            ))
            .into(),
          )))
        } else {
          Self(None)
        }
      }
      e => e.into(),
    }
  }
}

pub fn map_theme(theme: &TaoTheme) -> Theme {
  match theme {
    TaoTheme::Light => Theme::Light,
    TaoTheme::Dark => Theme::Dark,
    _ => Theme::Light,
  }
}

#[cfg(target_os = "macos")]
fn tao_activation_policy(activation_policy: ActivationPolicy) -> TaoActivationPolicy {
  match activation_policy {
    ActivationPolicy::Regular => TaoActivationPolicy::Regular,
    ActivationPolicy::Accessory => TaoActivationPolicy::Accessory,
    ActivationPolicy::Prohibited => TaoActivationPolicy::Prohibited,
    _ => unimplemented!(),
  }
}

impl<'a> From<&TaoWindowEvent<'a>> for WindowEventWrapper {
  fn from(event: &TaoWindowEvent<'a>) -> Self {
    let event = match event {
      TaoWindowEvent::Resized(size) => WindowEvent::Resized(PhysicalSizeWrapper(*size).into()),
      TaoWindowEvent::Moved(position) => {
        WindowEvent::Moved(PhysicalPositionWrapper(*position).into())
      }
      TaoWindowEvent::Destroyed => WindowEvent::Destroyed,
      TaoWindowEvent::ScaleFactorChanged {
        scale_factor,
        new_inner_size,
      } => WindowEvent::ScaleFactorChanged {
        scale_factor: *scale_factor,
        new_inner_size: PhysicalSizeWrapper(**new_inner_size).into(),
      },
      #[cfg(any(target_os = "linux", target_os = "macos"))]
      TaoWindowEvent::Focused(focused) => WindowEvent::Focused(*focused),
      TaoWindowEvent::ThemeChanged(theme) => WindowEvent::ThemeChanged(map_theme(theme)),
      _ => return Self(None),
    };
    Self(Some(event))
  }
}

pub struct MonitorHandleWrapper(pub MonitorHandle);

impl From<MonitorHandleWrapper> for Monitor {
  fn from(monitor: MonitorHandleWrapper) -> Monitor {
    Self {
      name: monitor.0.name(),
      position: PhysicalPositionWrapper(monitor.0.position()).into(),
      size: PhysicalSizeWrapper(monitor.0.size()).into(),
      scale_factor: monitor.0.scale_factor(),
    }
  }
}

pub struct PhysicalPositionWrapper<T>(pub TaoPhysicalPosition<T>);

impl<T> From<PhysicalPositionWrapper<T>> for PhysicalPosition<T> {
  fn from(position: PhysicalPositionWrapper<T>) -> Self {
    Self {
      x: position.0.x,
      y: position.0.y,
    }
  }
}

impl<T> From<PhysicalPosition<T>> for PhysicalPositionWrapper<T> {
  fn from(position: PhysicalPosition<T>) -> Self {
    Self(TaoPhysicalPosition {
      x: position.x,
      y: position.y,
    })
  }
}

struct LogicalPositionWrapper<T>(TaoLogicalPosition<T>);

impl<T> From<LogicalPosition<T>> for LogicalPositionWrapper<T> {
  fn from(position: LogicalPosition<T>) -> Self {
    Self(TaoLogicalPosition {
      x: position.x,
      y: position.y,
    })
  }
}

pub struct PhysicalSizeWrapper<T>(pub TaoPhysicalSize<T>);

impl<T> From<PhysicalSizeWrapper<T>> for PhysicalSize<T> {
  fn from(size: PhysicalSizeWrapper<T>) -> Self {
    Self {
      width: size.0.width,
      height: size.0.height,
    }
  }
}

impl<T> From<PhysicalSize<T>> for PhysicalSizeWrapper<T> {
  fn from(size: PhysicalSize<T>) -> Self {
    Self(TaoPhysicalSize {
      width: size.width,
      height: size.height,
    })
  }
}

struct LogicalSizeWrapper<T>(TaoLogicalSize<T>);

impl<T> From<LogicalSize<T>> for LogicalSizeWrapper<T> {
  fn from(size: LogicalSize<T>) -> Self {
    Self(TaoLogicalSize {
      width: size.width,
      height: size.height,
    })
  }
}

pub struct SizeWrapper(pub TaoSize);

impl From<Size> for SizeWrapper {
  fn from(size: Size) -> Self {
    match size {
      Size::Logical(s) => Self(TaoSize::Logical(LogicalSizeWrapper::from(s).0)),
      Size::Physical(s) => Self(TaoSize::Physical(PhysicalSizeWrapper::from(s).0)),
    }
  }
}

pub struct PositionWrapper(pub TaoPosition);

impl From<Position> for PositionWrapper {
  fn from(position: Position) -> Self {
    match position {
      Position::Logical(s) => Self(TaoPosition::Logical(LogicalPositionWrapper::from(s).0)),
      Position::Physical(s) => Self(TaoPosition::Physical(PhysicalPositionWrapper::from(s).0)),
    }
  }
}

#[derive(Debug, Clone)]
pub struct UserAttentionTypeWrapper(pub TaoUserAttentionType);

impl From<UserAttentionType> for UserAttentionTypeWrapper {
  fn from(request_type: UserAttentionType) -> Self {
    let o = match request_type {
      UserAttentionType::Critical => TaoUserAttentionType::Critical,
      UserAttentionType::Informational => TaoUserAttentionType::Informational,
    };
    Self(o)
  }
}

#[derive(Debug)]
pub struct CursorIconWrapper(pub TaoCursorIcon);

impl From<CursorIcon> for CursorIconWrapper {
  fn from(icon: CursorIcon) -> Self {
    use CursorIcon::*;
    let i = match icon {
      Default => TaoCursorIcon::Default,
      Crosshair => TaoCursorIcon::Crosshair,
      Hand => TaoCursorIcon::Hand,
      Arrow => TaoCursorIcon::Arrow,
      Move => TaoCursorIcon::Move,
      Text => TaoCursorIcon::Text,
      Wait => TaoCursorIcon::Wait,
      Help => TaoCursorIcon::Help,
      Progress => TaoCursorIcon::Progress,
      NotAllowed => TaoCursorIcon::NotAllowed,
      ContextMenu => TaoCursorIcon::ContextMenu,
      Cell => TaoCursorIcon::Cell,
      VerticalText => TaoCursorIcon::VerticalText,
      Alias => TaoCursorIcon::Alias,
      Copy => TaoCursorIcon::Copy,
      NoDrop => TaoCursorIcon::NoDrop,
      Grab => TaoCursorIcon::Grab,
      Grabbing => TaoCursorIcon::Grabbing,
      AllScroll => TaoCursorIcon::AllScroll,
      ZoomIn => TaoCursorIcon::ZoomIn,
      ZoomOut => TaoCursorIcon::ZoomOut,
      EResize => TaoCursorIcon::EResize,
      NResize => TaoCursorIcon::NResize,
      NeResize => TaoCursorIcon::NeResize,
      NwResize => TaoCursorIcon::NwResize,
      SResize => TaoCursorIcon::SResize,
      SeResize => TaoCursorIcon::SeResize,
      SwResize => TaoCursorIcon::SwResize,
      WResize => TaoCursorIcon::WResize,
      EwResize => TaoCursorIcon::EwResize,
      NsResize => TaoCursorIcon::NsResize,
      NeswResize => TaoCursorIcon::NeswResize,
      NwseResize => TaoCursorIcon::NwseResize,
      ColResize => TaoCursorIcon::ColResize,
      RowResize => TaoCursorIcon::RowResize,
      _ => TaoCursorIcon::Default,
    };
    Self(i)
  }
}

pub struct ProgressStateWrapper(pub TaoProgressState);

impl From<ProgressBarStatus> for ProgressStateWrapper {
  fn from(status: ProgressBarStatus) -> Self {
    let state = match status {
      ProgressBarStatus::None => TaoProgressState::None,
      ProgressBarStatus::Normal => TaoProgressState::Normal,
      ProgressBarStatus::Indeterminate => TaoProgressState::Indeterminate,
      ProgressBarStatus::Paused => TaoProgressState::Paused,
      ProgressBarStatus::Error => TaoProgressState::Error,
    };
    Self(state)
  }
}

pub struct ProgressBarStateWrapper(pub TaoProgressBarState);

impl From<ProgressBarState> for ProgressBarStateWrapper {
  fn from(progress_state: ProgressBarState) -> Self {
    Self(TaoProgressBarState {
      progress: progress_state.progress,
      state: progress_state
        .status
        .map(|state| ProgressStateWrapper::from(state).0),
      desktop_filename: progress_state.desktop_filename,
    })
  }
}

#[derive(Clone, Default)]
pub struct WindowBuilderWrapper {
  inner: TaoWindowBuilder,
  center: bool,
  #[cfg(target_os = "macos")]
  tabbing_identifier: Option<String>,
}

impl std::fmt::Debug for WindowBuilderWrapper {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut s = f.debug_struct("WindowBuilderWrapper");
    s.field("inner", &self.inner).field("center", &self.center);
    #[cfg(target_os = "macos")]
    {
      s.field("tabbing_identifier", &self.tabbing_identifier);
    }
    s.finish()
  }
}

// SAFETY: this type is `Send` since `menu_items` are read only here
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for WindowBuilderWrapper {}

impl WindowBuilderBase for WindowBuilderWrapper {}
impl WindowBuilder for WindowBuilderWrapper {
  fn new() -> Self {
    Self::default().focused(true)
  }

  fn with_config(config: &WindowConfig) -> Self {
    let mut window = WindowBuilderWrapper::new();

    #[cfg(target_os = "macos")]
    {
      window = window
        .hidden_title(config.hidden_title)
        .title_bar_style(config.title_bar_style);
      if let Some(identifier) = &config.tabbing_identifier {
        window = window.tabbing_identifier(identifier);
      }
    }

    #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
    {
      window = window.transparent(config.transparent);
    }
    #[cfg(all(
      target_os = "macos",
      not(feature = "macos-private-api"),
      debug_assertions
    ))]
    if config.transparent {
      eprintln!(
        "The window is set to be transparent but the `macos-private-api` is not enabled.
        This can be enabled via the `tauri.macOSPrivateApi` configuration property <https://tauri.app/docs/api/config#tauri.macOSPrivateApi>
      ");
    }

    #[cfg(target_os = "linux")]
    {
      // Mouse event is disabled on Linux since sudden event bursts could block event loop.
      window.inner = window.inner.with_cursor_moved_event(false);
    }

    #[cfg(desktop)]
    {
      window = window
        .title(config.title.to_string())
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
        .theme(config.theme)
        .shadow(config.shadow);

      if let (Some(min_width), Some(min_height)) = (config.min_width, config.min_height) {
        window = window.min_inner_size(min_width, min_height);
      }
      if let (Some(max_width), Some(max_height)) = (config.max_width, config.max_height) {
        window = window.max_inner_size(max_width, max_height);
      }
      if let (Some(x), Some(y)) = (config.x, config.y) {
        window = window.position(x, y);
      }

      if config.center {
        window = window.center();
      }
    }

    window
  }

  fn center(mut self) -> Self {
    self.center = true;
    self
  }

  fn position(mut self, x: f64, y: f64) -> Self {
    self.inner = self.inner.with_position(TaoLogicalPosition::new(x, y));
    self
  }

  fn inner_size(mut self, width: f64, height: f64) -> Self {
    self.inner = self
      .inner
      .with_inner_size(TaoLogicalSize::new(width, height));
    self
  }

  fn min_inner_size(mut self, min_width: f64, min_height: f64) -> Self {
    self.inner = self
      .inner
      .with_min_inner_size(TaoLogicalSize::new(min_width, min_height));
    self
  }

  fn max_inner_size(mut self, max_width: f64, max_height: f64) -> Self {
    self.inner = self
      .inner
      .with_max_inner_size(TaoLogicalSize::new(max_width, max_height));
    self
  }

  fn resizable(mut self, resizable: bool) -> Self {
    self.inner = self.inner.with_resizable(resizable);
    self
  }

  fn maximizable(mut self, maximizable: bool) -> Self {
    self.inner = self.inner.with_maximizable(maximizable);
    self
  }

  fn minimizable(mut self, minimizable: bool) -> Self {
    self.inner = self.inner.with_minimizable(minimizable);
    self
  }

  fn closable(mut self, closable: bool) -> Self {
    self.inner = self.inner.with_closable(closable);
    self
  }

  fn title<S: Into<String>>(mut self, title: S) -> Self {
    self.inner = self.inner.with_title(title.into());
    self
  }

  fn fullscreen(mut self, fullscreen: bool) -> Self {
    self.inner = if fullscreen {
      self
        .inner
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
    } else {
      self.inner.with_fullscreen(None)
    };
    self
  }

  fn focused(mut self, focused: bool) -> Self {
    self.inner = self.inner.with_focused(focused);
    self
  }

  fn maximized(mut self, maximized: bool) -> Self {
    self.inner = self.inner.with_maximized(maximized);
    self
  }

  fn visible(mut self, visible: bool) -> Self {
    self.inner = self.inner.with_visible(visible);
    self
  }

  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  fn transparent(mut self, transparent: bool) -> Self {
    self.inner = self.inner.with_transparent(transparent);
    self
  }

  fn decorations(mut self, decorations: bool) -> Self {
    self.inner = self.inner.with_decorations(decorations);
    self
  }

  fn always_on_bottom(mut self, always_on_bottom: bool) -> Self {
    self.inner = self.inner.with_always_on_bottom(always_on_bottom);
    self
  }

  fn always_on_top(mut self, always_on_top: bool) -> Self {
    self.inner = self.inner.with_always_on_top(always_on_top);
    self
  }

  fn visible_on_all_workspaces(mut self, visible_on_all_workspaces: bool) -> Self {
    self.inner = self
      .inner
      .with_visible_on_all_workspaces(visible_on_all_workspaces);
    self
  }

  fn content_protected(mut self, protected: bool) -> Self {
    self.inner = self.inner.with_content_protection(protected);
    self
  }

  fn shadow(#[allow(unused_mut)] mut self, _enable: bool) -> Self {
    #[cfg(windows)]
    {
      self.inner = self.inner.with_undecorated_shadow(_enable);
    }
    #[cfg(target_os = "macos")]
    {
      self.inner = self.inner.with_has_shadow(_enable);
    }
    self
  }

  #[cfg(windows)]
  fn owner(mut self, owner: HWND) -> Self {
    self.inner = self.inner.with_owner_window(owner.0);
    self
  }

  #[cfg(windows)]
  fn parent(mut self, parent: HWND) -> Self {
    self.inner = self.inner.with_parent_window(parent.0);
    self
  }

  #[cfg(target_os = "macos")]
  fn parent(mut self, parent: *mut std::ffi::c_void) -> Self {
    self.inner = self.inner.with_parent_window(parent);
    self
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn transient_for(mut self, parent: &impl gtk::glib::IsA<gtk::Window>) -> Self {
    self.inner = self.inner.with_transient_for(parent);
    self
  }

  #[cfg(windows)]
  fn drag_and_drop(mut self, enabled: bool) -> Self {
    self.inner = self.inner.with_drag_and_drop(enabled);
    self
  }

  #[cfg(target_os = "macos")]
  fn title_bar_style(mut self, style: TitleBarStyle) -> Self {
    match style {
      TitleBarStyle::Visible => {
        self.inner = self.inner.with_titlebar_transparent(false);
        // Fixes rendering issue when resizing window with devtools open (https://github.com/tauri-apps/tauri/issues/3914)
        self.inner = self.inner.with_fullsize_content_view(true);
      }
      TitleBarStyle::Transparent => {
        self.inner = self.inner.with_titlebar_transparent(true);
        self.inner = self.inner.with_fullsize_content_view(false);
      }
      TitleBarStyle::Overlay => {
        self.inner = self.inner.with_titlebar_transparent(true);
        self.inner = self.inner.with_fullsize_content_view(true);
      }
    }
    self
  }

  #[cfg(target_os = "macos")]
  fn hidden_title(mut self, hidden: bool) -> Self {
    self.inner = self.inner.with_title_hidden(hidden);
    self
  }

  #[cfg(target_os = "macos")]
  fn tabbing_identifier(mut self, identifier: &str) -> Self {
    self.inner = self.inner.with_tabbing_identifier(identifier);
    self.tabbing_identifier.replace(identifier.into());
    self
  }

  fn icon(mut self, icon: Icon) -> Result<Self> {
    self.inner = self
      .inner
      .with_window_icon(Some(TaoIcon::try_from(icon)?.0));
    Ok(self)
  }

  #[cfg(any(windows, target_os = "linux"))]
  fn skip_taskbar(mut self, skip: bool) -> Self {
    self.inner = self.inner.with_skip_taskbar(skip);
    self
  }

  #[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
  fn skip_taskbar(self, _skip: bool) -> Self {
    self
  }

  #[allow(unused_variables, unused_mut)]
  fn theme(mut self, theme: Option<Theme>) -> Self {
    self.inner = self.inner.with_theme(if let Some(t) = theme {
      match t {
        Theme::Dark => Some(TaoTheme::Dark),
        _ => Some(TaoTheme::Light),
      }
    } else {
      None
    });

    self
  }

  fn has_icon(&self) -> bool {
    self.inner.window.window_icon.is_some()
  }
}

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
pub struct GtkWindow(pub gtk::ApplicationWindow);
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for GtkWindow {}

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
pub struct GtkBox(pub gtk::Box);
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for GtkBox {}

pub struct SendRawWindowHandle(pub raw_window_handle::RawWindowHandle);
unsafe impl Send for SendRawWindowHandle {}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub enum ApplicationMessage {
  Show,
  Hide,
}

pub enum WindowMessage {
  AddEventListener(WindowEventId, Box<dyn Fn(&WindowEvent) + Send>),
  // Getters
  ScaleFactor(Sender<f64>),
  InnerPosition(Sender<Result<PhysicalPosition<i32>>>),
  OuterPosition(Sender<Result<PhysicalPosition<i32>>>),
  InnerSize(Sender<PhysicalSize<u32>>),
  OuterSize(Sender<PhysicalSize<u32>>),
  IsFullscreen(Sender<bool>),
  IsMinimized(Sender<bool>),
  IsMaximized(Sender<bool>),
  IsFocused(Sender<bool>),
  IsDecorated(Sender<bool>),
  IsResizable(Sender<bool>),
  IsMaximizable(Sender<bool>),
  IsMinimizable(Sender<bool>),
  IsClosable(Sender<bool>),
  IsVisible(Sender<bool>),
  Title(Sender<String>),
  CurrentMonitor(Sender<Option<MonitorHandle>>),
  PrimaryMonitor(Sender<Option<MonitorHandle>>),
  AvailableMonitors(Sender<Vec<MonitorHandle>>),
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  GtkWindow(Sender<GtkWindow>),
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  GtkBox(Sender<GtkBox>),
  RawWindowHandle(Sender<std::result::Result<SendRawWindowHandle, raw_window_handle::HandleError>>),
  Theme(Sender<Theme>),
  // Setters
  Center,
  RequestUserAttention(Option<UserAttentionTypeWrapper>),
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
  Close,
  Destroy,
  SetDecorations(bool),
  SetShadow(bool),
  SetAlwaysOnBottom(bool),
  SetAlwaysOnTop(bool),
  SetVisibleOnAllWorkspaces(bool),
  SetContentProtected(bool),
  SetSize(Size),
  SetMinSize(Option<Size>),
  SetMaxSize(Option<Size>),
  SetPosition(Position),
  SetFullscreen(bool),
  SetFocus,
  SetIcon(TaoWindowIcon),
  SetSkipTaskbar(bool),
  SetCursorGrab(bool),
  SetCursorVisible(bool),
  SetCursorIcon(CursorIcon),
  SetCursorPosition(Position),
  SetIgnoreCursorEvents(bool),
  SetProgressBar(ProgressBarState),
  DragWindow,
  ResizeDragWindow(tauri_runtime::ResizeDirection),
  RequestRedraw,
}

#[derive(Debug, Clone)]
pub enum SynthesizedWindowEvent {
  Focused(bool),
  FileDrop(FileDropEvent),
}

impl From<SynthesizedWindowEvent> for WindowEventWrapper {
  fn from(event: SynthesizedWindowEvent) -> Self {
    let event = match event {
      SynthesizedWindowEvent::Focused(focused) => WindowEvent::Focused(focused),
      SynthesizedWindowEvent::FileDrop(event) => WindowEvent::FileDrop(event),
    };
    Self(Some(event))
  }
}

pub enum WebviewMessage {
  AddEventListener(WebviewEventId, Box<dyn Fn(&WebviewEvent) + Send>),
  #[cfg(not(all(feature = "tracing", not(target_os = "android"))))]
  EvaluateScript(String),
  #[cfg(all(feature = "tracing", not(target_os = "android")))]
  EvaluateScript(String, Sender<()>, tracing::Span),
  WebviewEvent(WebviewEvent),
  SynthesizedWindowEvent(SynthesizedWindowEvent),
  Navigate(Url),
  Print,
  Close,
  SetPosition(Position),
  SetSize(Size),
  SetFocus,
  // Getters
  Url(Sender<Url>),
  Position(Sender<PhysicalPosition<i32>>),
  Size(Sender<PhysicalSize<u32>>),
  WithWebview(Box<dyn FnOnce(Webview) + Send>),
  // Devtools
  #[cfg(any(debug_assertions, feature = "devtools"))]
  OpenDevTools,
  #[cfg(any(debug_assertions, feature = "devtools"))]
  CloseDevTools,
  #[cfg(any(debug_assertions, feature = "devtools"))]
  IsDevToolsOpen(Sender<bool>),
}

pub type CreateWindowClosure<T> =
  Box<dyn FnOnce(&EventLoopWindowTarget<Message<T>>) -> Result<WindowWrapper> + Send>;

pub type CreateWebviewClosure = Box<dyn FnOnce(&Window) -> Result<WebviewWrapper> + Send>;

pub enum Message<T: 'static> {
  Task(Box<dyn FnOnce() + Send>),
  #[cfg(target_os = "macos")]
  SetActivationPolicy(ActivationPolicy),
  RequestExit(i32),
  #[cfg(target_os = "macos")]
  Application(ApplicationMessage),
  Window(WindowId, WindowMessage),
  Webview(WindowId, WebviewId, WebviewMessage),
  CreateWebview(WindowId, CreateWebviewClosure),
  CreateWindow(WindowId, CreateWindowClosure<T>),
  CreateRawWindow(
    WindowId,
    Box<dyn FnOnce() -> (String, TaoWindowBuilder) + Send>,
    Sender<Result<Weak<Window>>>,
  ),
  UserEvent(T),
}

impl<T: UserEvent> Clone for Message<T> {
  fn clone(&self) -> Self {
    match self {
      Self::UserEvent(t) => Self::UserEvent(t.clone()),
      _ => unimplemented!(),
    }
  }
}

/// The Tauri [`WebviewDispatch`] for [`Wry`].
#[derive(Debug, Clone)]
pub struct WryWebviewDispatcher<T: UserEvent> {
  window_id: WindowId,
  webview_id: WebviewId,
  context: Context<T>,
}

impl<T: UserEvent> WebviewDispatch<T> for WryWebviewDispatcher<T> {
  type Runtime = Wry<T>;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    send_user_message(&self.context, Message::Task(Box::new(f)))
  }

  fn on_webview_event<F: Fn(&WebviewEvent) + Send + 'static>(&self, f: F) -> WindowEventId {
    let id = self.context.next_webview_event_id();
    let _ = self.context.proxy.send_event(Message::Webview(
      self.window_id,
      self.webview_id,
      WebviewMessage::AddEventListener(id, Box::new(f)),
    ));
    id
  }

  fn with_webview<F: FnOnce(Box<dyn std::any::Any>) + Send + 'static>(&self, f: F) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Webview(
        self.window_id,
        self.webview_id,
        WebviewMessage::WithWebview(Box::new(move |webview| f(Box::new(webview)))),
      ),
    )
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn open_devtools(&self) {
    let _ = send_user_message(
      &self.context,
      Message::Webview(
        self.window_id,
        self.webview_id,
        WebviewMessage::OpenDevTools,
      ),
    );
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn close_devtools(&self) {
    let _ = send_user_message(
      &self.context,
      Message::Webview(
        self.window_id,
        self.webview_id,
        WebviewMessage::CloseDevTools,
      ),
    );
  }

  /// Gets the devtools window's current open state.
  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn is_devtools_open(&self) -> Result<bool> {
    webview_getter!(self, WebviewMessage::IsDevToolsOpen)
  }

  // Getters

  fn url(&self) -> Result<Url> {
    webview_getter!(self, WebviewMessage::Url)
  }

  fn position(&self) -> Result<PhysicalPosition<i32>> {
    webview_getter!(self, WebviewMessage::Position)
  }

  fn size(&self) -> Result<PhysicalSize<u32>> {
    webview_getter!(self, WebviewMessage::Size)
  }

  // Setters

  fn navigate(&self, url: Url) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Webview(
        self.window_id,
        self.webview_id,
        WebviewMessage::Navigate(url),
      ),
    )
  }

  fn print(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Webview(self.window_id, self.webview_id, WebviewMessage::Print),
    )
  }

  fn close(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Webview(self.window_id, self.webview_id, WebviewMessage::Close),
    )
  }

  fn set_size(&self, size: Size) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Webview(
        self.window_id,
        self.webview_id,
        WebviewMessage::SetSize(size),
      ),
    )
  }

  fn set_position(&self, position: Position) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Webview(
        self.window_id,
        self.webview_id,
        WebviewMessage::SetPosition(position),
      ),
    )
  }

  fn set_focus(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Webview(self.window_id, self.webview_id, WebviewMessage::SetFocus),
    )
  }

  #[cfg(all(feature = "tracing", not(target_os = "android")))]
  fn eval_script<S: Into<String>>(&self, script: S) -> Result<()> {
    // use a channel so the EvaluateScript task uses the current span as parent
    let (tx, rx) = channel();
    getter!(
      self,
      rx,
      Message::Webview(
        self.window_id,
        self.webview_id,
        WebviewMessage::EvaluateScript(script.into(), tx, tracing::Span::current()),
      )
    )
  }

  #[cfg(not(all(feature = "tracing", not(target_os = "android"))))]
  fn eval_script<S: Into<String>>(&self, script: S) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Webview(
        self.window_id,
        self.webview_id,
        WebviewMessage::EvaluateScript(script.into()),
      ),
    )
  }
}

/// The Tauri [`WindowDispatch`] for [`Wry`].
#[derive(Debug, Clone)]
pub struct WryWindowDispatcher<T: UserEvent> {
  window_id: WindowId,
  context: Context<T>,
}

// SAFETY: this is safe since the `Context` usage is guarded on `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for WryWindowDispatcher<T> {}

fn get_raw_window_handle<T: UserEvent>(
  dispatcher: &WryWindowDispatcher<T>,
) -> Result<std::result::Result<SendRawWindowHandle, raw_window_handle::HandleError>> {
  window_getter!(dispatcher, WindowMessage::RawWindowHandle)
}

impl<T: UserEvent> WindowDispatch<T> for WryWindowDispatcher<T> {
  type Runtime = Wry<T>;
  type WindowBuilder = WindowBuilderWrapper;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    send_user_message(&self.context, Message::Task(Box::new(f)))
  }

  fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) -> WindowEventId {
    let id = self.context.next_window_event_id();
    let _ = self.context.proxy.send_event(Message::Window(
      self.window_id,
      WindowMessage::AddEventListener(id, Box::new(f)),
    ));
    id
  }

  // Getters

  fn scale_factor(&self) -> Result<f64> {
    window_getter!(self, WindowMessage::ScaleFactor)
  }

  fn inner_position(&self) -> Result<PhysicalPosition<i32>> {
    window_getter!(self, WindowMessage::InnerPosition)?
  }

  fn outer_position(&self) -> Result<PhysicalPosition<i32>> {
    window_getter!(self, WindowMessage::OuterPosition)?
  }

  fn inner_size(&self) -> Result<PhysicalSize<u32>> {
    window_getter!(self, WindowMessage::InnerSize)
  }

  fn outer_size(&self) -> Result<PhysicalSize<u32>> {
    window_getter!(self, WindowMessage::OuterSize)
  }

  fn is_fullscreen(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsFullscreen)
  }

  fn is_minimized(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsMinimized)
  }

  fn is_maximized(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsMaximized)
  }

  fn is_focused(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsFocused)
  }

  /// Gets the window’s current decoration state.
  fn is_decorated(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsDecorated)
  }

  /// Gets the window’s current resizable state.
  fn is_resizable(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsResizable)
  }

  /// Gets the current native window's maximize button state
  fn is_maximizable(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsMaximizable)
  }

  /// Gets the current native window's minimize button state
  fn is_minimizable(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsMinimizable)
  }

  /// Gets the current native window's close button state
  fn is_closable(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsClosable)
  }

  fn is_visible(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsVisible)
  }

  fn title(&self) -> Result<String> {
    window_getter!(self, WindowMessage::Title)
  }

  fn current_monitor(&self) -> Result<Option<Monitor>> {
    Ok(window_getter!(self, WindowMessage::CurrentMonitor)?.map(|m| MonitorHandleWrapper(m).into()))
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    Ok(window_getter!(self, WindowMessage::PrimaryMonitor)?.map(|m| MonitorHandleWrapper(m).into()))
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    Ok(
      window_getter!(self, WindowMessage::AvailableMonitors)?
        .into_iter()
        .map(|m| MonitorHandleWrapper(m).into())
        .collect(),
    )
  }

  fn theme(&self) -> Result<Theme> {
    window_getter!(self, WindowMessage::Theme)
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn gtk_window(&self) -> Result<gtk::ApplicationWindow> {
    window_getter!(self, WindowMessage::GtkWindow).map(|w| w.0)
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn default_vbox(&self) -> Result<gtk::Box> {
    window_getter!(self, WindowMessage::GtkBox).map(|w| w.0)
  }

  fn window_handle(
    &self,
  ) -> std::result::Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
    get_raw_window_handle(self)
      .map_err(|_| raw_window_handle::HandleError::Unavailable)
      .and_then(|r| r.map(|h| unsafe { raw_window_handle::WindowHandle::borrow_raw(h.0) }))
  }

  // Setters

  fn center(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::Center),
    )
  }

  fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.window_id,
        WindowMessage::RequestUserAttention(request_type.map(Into::into)),
      ),
    )
  }

  // Creates a window by dispatching a message to the event loop.
  // Note that this must be called from a separate thread, otherwise the channel will introduce a deadlock.
  fn create_window<F: Fn(RawWindow) + Send + 'static>(
    &mut self,
    pending: PendingWindow<T, Self::Runtime>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    self.context.create_window(pending, after_window_creation)
  }

  // Creates a webview by dispatching a message to the event loop.
  // Note that this must be called from a separate thread, otherwise the channel will introduce a deadlock.
  fn create_webview(
    &mut self,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>> {
    self.context.create_webview(self.window_id, pending)
  }

  fn set_resizable(&self, resizable: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetResizable(resizable)),
    )
  }

  fn set_maximizable(&self, maximizable: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetMaximizable(maximizable)),
    )
  }

  fn set_minimizable(&self, minimizable: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetMinimizable(minimizable)),
    )
  }

  fn set_closable(&self, closable: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetClosable(closable)),
    )
  }

  fn set_title<S: Into<String>>(&self, title: S) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetTitle(title.into())),
    )
  }

  fn maximize(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::Maximize),
    )
  }

  fn unmaximize(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::Unmaximize),
    )
  }

  fn minimize(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::Minimize),
    )
  }

  fn unminimize(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::Unminimize),
    )
  }

  fn show(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::Show),
    )
  }

  fn hide(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::Hide),
    )
  }

  fn close(&self) -> Result<()> {
    // NOTE: close cannot use the `send_user_message` function because it accesses the event loop callback
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Close))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn destroy(&self) -> Result<()> {
    // NOTE: destroy cannot use the `send_user_message` function because it accesses the event loop callback
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Destroy))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_decorations(&self, decorations: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetDecorations(decorations)),
    )
  }

  fn set_shadow(&self, enable: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetShadow(enable)),
    )
  }

  fn set_always_on_bottom(&self, always_on_bottom: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.window_id,
        WindowMessage::SetAlwaysOnBottom(always_on_bottom),
      ),
    )
  }

  fn set_always_on_top(&self, always_on_top: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetAlwaysOnTop(always_on_top)),
    )
  }

  fn set_visible_on_all_workspaces(&self, visible_on_all_workspaces: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.window_id,
        WindowMessage::SetVisibleOnAllWorkspaces(visible_on_all_workspaces),
      ),
    )
  }

  fn set_content_protected(&self, protected: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.window_id,
        WindowMessage::SetContentProtected(protected),
      ),
    )
  }

  fn set_size(&self, size: Size) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetSize(size)),
    )
  }

  fn set_min_size(&self, size: Option<Size>) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetMinSize(size)),
    )
  }

  fn set_max_size(&self, size: Option<Size>) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetMaxSize(size)),
    )
  }

  fn set_position(&self, position: Position) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetPosition(position)),
    )
  }

  fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetFullscreen(fullscreen)),
    )
  }

  fn set_focus(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetFocus),
    )
  }

  fn set_icon(&self, icon: Icon) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.window_id,
        WindowMessage::SetIcon(TaoIcon::try_from(icon)?.0),
      ),
    )
  }

  fn set_skip_taskbar(&self, skip: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetSkipTaskbar(skip)),
    )
  }

  fn set_cursor_grab(&self, grab: bool) -> crate::Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetCursorGrab(grab)),
    )
  }

  fn set_cursor_visible(&self, visible: bool) -> crate::Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetCursorVisible(visible)),
    )
  }

  fn set_cursor_icon(&self, icon: CursorIcon) -> crate::Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetCursorIcon(icon)),
    )
  }

  fn set_cursor_position<Pos: Into<Position>>(&self, position: Pos) -> crate::Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.window_id,
        WindowMessage::SetCursorPosition(position.into()),
      ),
    )
  }

  fn set_ignore_cursor_events(&self, ignore: bool) -> crate::Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetIgnoreCursorEvents(ignore)),
    )
  }

  fn start_dragging(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::DragWindow),
    )
  }

  fn start_resize_dragging(&self, direction: tauri_runtime::ResizeDirection) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::ResizeDragWindow(direction)),
    )
  }

  fn set_progress_bar(&self, progress_state: ProgressBarState) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.window_id,
        WindowMessage::SetProgressBar(progress_state),
      ),
    )
  }
}

#[derive(Clone)]
pub struct WebviewWrapper {
  label: String,
  id: WebviewId,
  inner: Rc<WebView>,
  context_store: WebContextStore,
  webview_event_listeners: WebviewEventListeners,
  // the key of the WebContext if it's not shared
  context_key: Option<PathBuf>,
  bounds: Option<Arc<Mutex<WebviewBounds>>>,
}

impl Deref for WebviewWrapper {
  type Target = WebView;

  #[inline(always)]
  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl Drop for WebviewWrapper {
  fn drop(&mut self) {
    if Rc::get_mut(&mut self.inner).is_some() {
      self.context_store.lock().unwrap().remove(&self.context_key);
    }
  }
}

pub struct WindowWrapper {
  label: String,
  inner: Option<Arc<Window>>,
  // whether this window has child webviews
  // or it's just a container for a single webview
  has_children: AtomicBool,
  webviews: Vec<WebviewWrapper>,
  window_event_listeners: WindowEventListeners,
  #[cfg(windows)]
  is_window_fullscreen: bool,
  #[cfg(windows)]
  is_window_transparent: bool,
  #[cfg(windows)]
  surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
}

impl fmt::Debug for WindowWrapper {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("WindowWrapper")
      .field("label", &self.label)
      .field("inner", &self.inner)
      .finish()
  }
}

#[derive(Debug, Clone)]
pub struct EventProxy<T: UserEvent>(TaoEventLoopProxy<Message<T>>);

#[cfg(target_os = "ios")]
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for EventProxy<T> {}

impl<T: UserEvent> EventLoopProxy<T> for EventProxy<T> {
  fn send_event(&self, event: T) -> Result<()> {
    self
      .0
      .send_event(Message::UserEvent(event))
      .map_err(|_| Error::EventLoopClosed)
  }
}

pub trait PluginBuilder<T: UserEvent> {
  type Plugin: Plugin<T>;
  fn build(self, context: Context<T>) -> Self::Plugin;
}

pub trait Plugin<T: UserEvent> {
  fn on_event(
    &mut self,
    event: &Event<Message<T>>,
    event_loop: &EventLoopWindowTarget<Message<T>>,
    proxy: &TaoEventLoopProxy<Message<T>>,
    control_flow: &mut ControlFlow,
    context: EventLoopIterationContext<'_, T>,
    web_context: &WebContextStore,
  ) -> bool;
}

/// A Tauri [`Runtime`] wrapper around wry.
pub struct Wry<T: UserEvent> {
  context: Context<T>,
  event_loop: EventLoop<Message<T>>,
}

impl<T: UserEvent> fmt::Debug for Wry<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Wry")
      .field("main_thread_id", &self.context.main_thread_id)
      .field("event_loop", &self.event_loop)
      .field("windows", &self.context.main_thread.windows)
      .field("web_context", &self.context.main_thread.web_context)
      .finish()
  }
}

/// A handle to the Wry runtime.
#[derive(Debug, Clone)]
pub struct WryHandle<T: UserEvent> {
  context: Context<T>,
}

// SAFETY: this is safe since the `Context` usage is guarded on `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for WryHandle<T> {}

impl<T: UserEvent> WryHandle<T> {
  /// Creates a new tao window using a callback, and returns its window id.
  pub fn create_tao_window<F: FnOnce() -> (String, TaoWindowBuilder) + Send + 'static>(
    &self,
    f: F,
  ) -> Result<Weak<Window>> {
    let id = self.context.next_window_id();
    let (tx, rx) = channel();
    send_user_message(&self.context, Message::CreateRawWindow(id, Box::new(f), tx))?;
    rx.recv().unwrap()
  }

  /// Gets the [`WebviewId'] associated with the given [`WindowId`].
  pub fn window_id(&self, window_id: TaoWindowId) -> WindowId {
    *self
      .context
      .window_id_map
      .0
      .lock()
      .unwrap()
      .get(&window_id)
      .unwrap()
  }

  /// Send a message to the event loop.
  pub fn send_event(&self, message: Message<T>) -> Result<()> {
    self
      .context
      .proxy
      .send_event(message)
      .map_err(|_| Error::FailedToSendMessage)?;
    Ok(())
  }

  pub fn plugin<P: PluginBuilder<T> + 'static>(&mut self, plugin: P)
  where
    <P as PluginBuilder<T>>::Plugin: Send,
  {
    self
      .context
      .plugins
      .lock()
      .unwrap()
      .push(Box::new(plugin.build(self.context.clone())));
  }
}

impl<T: UserEvent> RuntimeHandle<T> for WryHandle<T> {
  type Runtime = Wry<T>;

  fn create_proxy(&self) -> EventProxy<T> {
    EventProxy(self.context.proxy.clone())
  }

  #[cfg(target_os = "macos")]
  fn set_activation_policy(&self, activation_policy: ActivationPolicy) -> Result<()> {
    send_user_message(
      &self.context,
      Message::SetActivationPolicy(activation_policy),
    )
  }

  fn request_exit(&self, code: i32) -> Result<()> {
    // NOTE: request_exit cannot use the `send_user_message` function because it accesses the event loop callback
    self
      .context
      .proxy
      .send_event(Message::RequestExit(code))
      .map_err(|_| Error::FailedToSendMessage)
  }

  // Creates a window by dispatching a message to the event loop.
  // Note that this must be called from a separate thread, otherwise the channel will introduce a deadlock.
  fn create_window<F: Fn(RawWindow) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self::Runtime>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    self.context.create_window(pending, after_window_creation)
  }

  // Creates a webview by dispatching a message to the event loop.
  // Note that this must be called from a separate thread, otherwise the channel will introduce a deadlock.
  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>> {
    self.context.create_webview(window_id, pending)
  }

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    send_user_message(&self.context, Message::Task(Box::new(f)))
  }

  fn display_handle(&self) -> std::result::Result<DisplayHandle, raw_window_handle::HandleError> {
    self.context.main_thread.window_target.display_handle()
  }

  fn primary_monitor(&self) -> Option<Monitor> {
    self
      .context
      .main_thread
      .window_target
      .primary_monitor()
      .map(|m| MonitorHandleWrapper(m).into())
  }

  fn available_monitors(&self) -> Vec<Monitor> {
    self
      .context
      .main_thread
      .window_target
      .available_monitors()
      .map(|m| MonitorHandleWrapper(m).into())
      .collect()
  }

  #[cfg(target_os = "macos")]
  fn show(&self) -> tauri_runtime::Result<()> {
    send_user_message(
      &self.context,
      Message::Application(ApplicationMessage::Show),
    )
  }

  #[cfg(target_os = "macos")]
  fn hide(&self) -> tauri_runtime::Result<()> {
    send_user_message(
      &self.context,
      Message::Application(ApplicationMessage::Hide),
    )
  }

  #[cfg(target_os = "android")]
  fn find_class<'a>(
    &self,
    env: &mut jni::JNIEnv<'a>,
    activity: &jni::objects::JObject<'_>,
    name: impl Into<String>,
  ) -> std::result::Result<jni::objects::JClass<'a>, jni::errors::Error> {
    find_class(env, activity, name.into())
  }

  #[cfg(target_os = "android")]
  fn run_on_android_context<F>(&self, f: F)
  where
    F: FnOnce(&mut jni::JNIEnv, &jni::objects::JObject, &jni::objects::JObject) + Send + 'static,
  {
    dispatch(f)
  }
}

impl<T: UserEvent> Wry<T> {
  fn init_with_builder(
    mut event_loop_builder: EventLoopBuilder<Message<T>>,
    #[allow(unused_variables)] args: RuntimeInitArgs,
  ) -> Result<Self> {
    #[cfg(windows)]
    if let Some(hook) = args.msg_hook {
      use tao::platform::windows::EventLoopBuilderExtWindows;
      event_loop_builder.with_msg_hook(hook);
    }
    Self::init(event_loop_builder.build())
  }

  fn init(event_loop: EventLoop<Message<T>>) -> Result<Self> {
    let main_thread_id = current_thread().id();
    let web_context = WebContextStore::default();

    let windows = Arc::new(WindowsStore(RefCell::new(BTreeMap::default())));
    let window_id_map = WindowIdStore::default();

    let context = Context {
      window_id_map,
      main_thread_id,
      proxy: event_loop.create_proxy(),
      main_thread: DispatcherMainThreadContext {
        window_target: event_loop.deref().clone(),
        web_context,
        windows,
        #[cfg(feature = "tracing")]
        active_tracing_spans: Default::default(),
      },
      plugins: Default::default(),
      next_window_id: Default::default(),
      next_webview_id: Default::default(),
      next_window_event_id: Default::default(),
      next_webview_event_id: Default::default(),
      next_webcontext_id: Default::default(),
    };

    Ok(Self {
      context,
      event_loop,
    })
  }
}

impl<T: UserEvent> Runtime<T> for Wry<T> {
  type WindowDispatcher = WryWindowDispatcher<T>;
  type WebviewDispatcher = WryWebviewDispatcher<T>;
  type Handle = WryHandle<T>;

  type EventLoopProxy = EventProxy<T>;

  fn new(args: RuntimeInitArgs) -> Result<Self> {
    Self::init_with_builder(EventLoopBuilder::<Message<T>>::with_user_event(), args)
  }
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn new_any_thread(args: RuntimeInitArgs) -> Result<Self> {
    use tao::platform::unix::EventLoopBuilderExtUnix;
    let mut event_loop_builder = EventLoopBuilder::<Message<T>>::with_user_event();
    event_loop_builder.with_any_thread(true);
    Self::init_with_builder(event_loop_builder, args)
  }

  #[cfg(windows)]
  fn new_any_thread(args: RuntimeInitArgs) -> Result<Self> {
    use tao::platform::windows::EventLoopBuilderExtWindows;
    let mut event_loop_builder = EventLoopBuilder::<Message<T>>::with_user_event();
    event_loop_builder.with_any_thread(true);
    Self::init_with_builder(event_loop_builder, args)
  }

  fn create_proxy(&self) -> EventProxy<T> {
    EventProxy(self.event_loop.create_proxy())
  }

  fn handle(&self) -> Self::Handle {
    WryHandle {
      context: self.context.clone(),
    }
  }

  fn create_window<F: Fn(RawWindow) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self>> {
    let label = pending.label.clone();
    let window_id = self.context.next_window_id();
    let webview_id = pending
      .webview
      .as_ref()
      .map(|_| self.context.next_webview_id());

    let window = create_window(
      window_id,
      webview_id.unwrap_or_default(),
      &self.event_loop,
      &self.context,
      pending,
      after_window_creation,
    )?;

    let dispatcher = WryWindowDispatcher {
      window_id,
      context: self.context.clone(),
    };

    self
      .context
      .main_thread
      .windows
      .0
      .borrow_mut()
      .insert(window_id, window);

    let detached_webview = webview_id.map(|id| DetachedWebview {
      label: label.clone(),
      dispatcher: WryWebviewDispatcher {
        window_id,
        webview_id: id,
        context: self.context.clone(),
      },
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

    let window = self
      .context
      .main_thread
      .windows
      .0
      .borrow()
      .get(&window_id)
      .and_then(|w| w.inner.clone());
    if let Some(window) = window {
      let webview_id = self.context.next_webview_id();

      let webview = create_webview(
        WebviewKind::WindowChild,
        &window,
        window_id,
        webview_id,
        &self.context,
        pending,
      )?;

      self
        .context
        .main_thread
        .windows
        .0
        .borrow_mut()
        .get_mut(&window_id)
        .map(|w| {
          w.webviews.push(webview);
          w.has_children.store(true, Ordering::Relaxed);
          w
        });

      let dispatcher = WryWebviewDispatcher {
        window_id,
        webview_id,
        context: self.context.clone(),
      };

      Ok(DetachedWebview { label, dispatcher })
    } else {
      Err(Error::WindowNotFound)
    }
  }

  fn primary_monitor(&self) -> Option<Monitor> {
    self
      .context
      .main_thread
      .window_target
      .primary_monitor()
      .map(|m| MonitorHandleWrapper(m).into())
  }

  fn available_monitors(&self) -> Vec<Monitor> {
    self
      .context
      .main_thread
      .window_target
      .available_monitors()
      .map(|m| MonitorHandleWrapper(m).into())
      .collect()
  }

  #[cfg(target_os = "macos")]
  fn set_activation_policy(&mut self, activation_policy: ActivationPolicy) {
    self
      .event_loop
      .set_activation_policy(tao_activation_policy(activation_policy));
  }

  #[cfg(target_os = "macos")]
  fn show(&self) {
    self.event_loop.show_application();
  }

  #[cfg(target_os = "macos")]
  fn hide(&self) {
    self.event_loop.hide_application();
  }

  fn set_device_event_filter(&mut self, filter: DeviceEventFilter) {
    self
      .event_loop
      .set_device_event_filter(DeviceEventFilterWrapper::from(filter).0);
  }

  #[cfg(desktop)]
  fn run_iteration<F: FnMut(RunEvent<T>) + 'static>(&mut self, mut callback: F) {
    use tao::platform::run_return::EventLoopExtRunReturn;
    let windows = self.context.main_thread.windows.clone();
    let window_id_map = self.context.window_id_map.clone();
    let web_context = &self.context.main_thread.web_context;
    let plugins = self.context.plugins.clone();

    #[cfg(feature = "tracing")]
    let active_tracing_spans = self.context.main_thread.active_tracing_spans.clone();

    let proxy = self.event_loop.create_proxy();

    self
      .event_loop
      .run_return(|event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::MainEventsCleared = &event {
          *control_flow = ControlFlow::Exit;
        }

        for p in plugins.lock().unwrap().iter_mut() {
          let prevent_default = p.on_event(
            &event,
            event_loop,
            &proxy,
            control_flow,
            EventLoopIterationContext {
              callback: &mut callback,
              window_id_map: window_id_map.clone(),
              windows: windows.clone(),
              #[cfg(feature = "tracing")]
              active_tracing_spans: active_tracing_spans.clone(),
            },
            web_context,
          );
          if prevent_default {
            return;
          }
        }

        handle_event_loop(
          event,
          event_loop,
          control_flow,
          EventLoopIterationContext {
            callback: &mut callback,
            windows: windows.clone(),
            window_id_map: window_id_map.clone(),
            #[cfg(feature = "tracing")]
            active_tracing_spans: active_tracing_spans.clone(),
          },
        );
      });
  }

  fn run<F: FnMut(RunEvent<T>) + 'static>(self, mut callback: F) {
    let windows = self.context.main_thread.windows.clone();
    let window_id_map = self.context.window_id_map.clone();
    let web_context = self.context.main_thread.web_context;
    let plugins = self.context.plugins.clone();

    #[cfg(feature = "tracing")]
    let active_tracing_spans = self.context.main_thread.active_tracing_spans.clone();
    let proxy = self.event_loop.create_proxy();

    self.event_loop.run(move |event, event_loop, control_flow| {
      for p in plugins.lock().unwrap().iter_mut() {
        let prevent_default = p.on_event(
          &event,
          event_loop,
          &proxy,
          control_flow,
          EventLoopIterationContext {
            callback: &mut callback,
            window_id_map: window_id_map.clone(),
            windows: windows.clone(),
            #[cfg(feature = "tracing")]
            active_tracing_spans: active_tracing_spans.clone(),
          },
          &web_context,
        );
        if prevent_default {
          return;
        }
      }
      handle_event_loop(
        event,
        event_loop,
        control_flow,
        EventLoopIterationContext {
          callback: &mut callback,
          window_id_map: window_id_map.clone(),
          windows: windows.clone(),
          #[cfg(feature = "tracing")]
          active_tracing_spans: active_tracing_spans.clone(),
        },
      );
    })
  }
}

pub struct EventLoopIterationContext<'a, T: UserEvent> {
  pub callback: &'a mut (dyn FnMut(RunEvent<T>) + 'static),
  pub window_id_map: WindowIdStore,
  pub windows: Arc<WindowsStore>,
  #[cfg(feature = "tracing")]
  pub active_tracing_spans: ActiveTraceSpanStore,
}

struct UserMessageContext {
  windows: Arc<WindowsStore>,
  window_id_map: WindowIdStore,
}

fn handle_user_message<T: UserEvent>(
  event_loop: &EventLoopWindowTarget<Message<T>>,
  message: Message<T>,
  context: UserMessageContext,
) {
  let UserMessageContext {
    window_id_map,
    windows,
  } = context;
  match message {
    Message::Task(task) => task(),
    #[cfg(target_os = "macos")]
    Message::SetActivationPolicy(activation_policy) => {
      event_loop.set_activation_policy_at_runtime(tao_activation_policy(activation_policy))
    }
    Message::RequestExit(_code) => panic!("cannot handle RequestExit on the main thread"),
    #[cfg(target_os = "macos")]
    Message::Application(application_message) => match application_message {
      ApplicationMessage::Show => {
        event_loop.show_application();
      }
      ApplicationMessage::Hide => {
        event_loop.hide_application();
      }
    },
    Message::Window(id, window_message) => {
      let w = windows.0.borrow().get(&id).map(|w| {
        (
          w.inner.clone(),
          w.webviews.clone(),
          w.has_children.load(Ordering::Relaxed),
          w.window_event_listeners.clone(),
        )
      });
      if let Some((Some(window), webviews, has_children, window_event_listeners)) = w {
        match window_message {
          WindowMessage::AddEventListener(id, listener) => {
            window_event_listeners.lock().unwrap().insert(id, listener);
          }

          // Getters
          WindowMessage::ScaleFactor(tx) => tx.send(window.scale_factor()).unwrap(),
          WindowMessage::InnerPosition(tx) => tx
            .send(
              window
                .inner_position()
                .map(|p| PhysicalPositionWrapper(p).into())
                .map_err(|_| Error::FailedToSendMessage),
            )
            .unwrap(),
          WindowMessage::OuterPosition(tx) => tx
            .send(
              window
                .outer_position()
                .map(|p| PhysicalPositionWrapper(p).into())
                .map_err(|_| Error::FailedToSendMessage),
            )
            .unwrap(),
          WindowMessage::InnerSize(tx) => tx
            .send(PhysicalSizeWrapper(inner_size(&window, &webviews, has_children)).into())
            .unwrap(),
          WindowMessage::OuterSize(tx) => tx
            .send(PhysicalSizeWrapper(window.outer_size()).into())
            .unwrap(),
          WindowMessage::IsFullscreen(tx) => tx.send(window.fullscreen().is_some()).unwrap(),
          WindowMessage::IsMinimized(tx) => tx.send(window.is_minimized()).unwrap(),
          WindowMessage::IsMaximized(tx) => tx.send(window.is_maximized()).unwrap(),
          WindowMessage::IsFocused(tx) => tx.send(window.is_focused()).unwrap(),
          WindowMessage::IsDecorated(tx) => tx.send(window.is_decorated()).unwrap(),
          WindowMessage::IsResizable(tx) => tx.send(window.is_resizable()).unwrap(),
          WindowMessage::IsMaximizable(tx) => tx.send(window.is_maximizable()).unwrap(),
          WindowMessage::IsMinimizable(tx) => tx.send(window.is_minimizable()).unwrap(),
          WindowMessage::IsClosable(tx) => tx.send(window.is_closable()).unwrap(),
          WindowMessage::IsVisible(tx) => tx.send(window.is_visible()).unwrap(),
          WindowMessage::Title(tx) => tx.send(window.title()).unwrap(),
          WindowMessage::CurrentMonitor(tx) => tx.send(window.current_monitor()).unwrap(),
          WindowMessage::PrimaryMonitor(tx) => tx.send(window.primary_monitor()).unwrap(),
          WindowMessage::AvailableMonitors(tx) => {
            tx.send(window.available_monitors().collect()).unwrap()
          }
          #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
          ))]
          WindowMessage::GtkWindow(tx) => tx.send(GtkWindow(window.gtk_window().clone())).unwrap(),
          #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
          ))]
          WindowMessage::GtkBox(tx) => tx
            .send(GtkBox(window.default_vbox().unwrap().clone()))
            .unwrap(),
          WindowMessage::RawWindowHandle(tx) => tx
            .send(
              window
                .window_handle()
                .map(|h| SendRawWindowHandle(h.as_raw())),
            )
            .unwrap(),
          WindowMessage::Theme(tx) => {
            tx.send(map_theme(&window.theme())).unwrap();
          }
          // Setters
          WindowMessage::Center => {
            if let Some(monitor) = window.current_monitor() {
              let window_size = inner_size(&window, &webviews, has_children);
              let screen_size = monitor.size();
              let monitor_pos = monitor.position();
              let x = (screen_size.width as i32 - window_size.width as i32) / 2 + monitor_pos.x;
              let y = (screen_size.height as i32 - window_size.height as i32) / 2 + monitor_pos.y;
              window.set_outer_position(TaoPhysicalPosition::new(x, y));
            }
          }
          WindowMessage::RequestUserAttention(request_type) => {
            window.request_user_attention(request_type.map(|r| r.0));
          }
          WindowMessage::SetResizable(resizable) => window.set_resizable(resizable),
          WindowMessage::SetMaximizable(maximizable) => window.set_maximizable(maximizable),
          WindowMessage::SetMinimizable(minimizable) => window.set_minimizable(minimizable),
          WindowMessage::SetClosable(closable) => window.set_closable(closable),
          WindowMessage::SetTitle(title) => window.set_title(&title),
          WindowMessage::Maximize => window.set_maximized(true),
          WindowMessage::Unmaximize => window.set_maximized(false),
          WindowMessage::Minimize => window.set_minimized(true),
          WindowMessage::Unminimize => window.set_minimized(false),
          WindowMessage::Show => window.set_visible(true),
          WindowMessage::Hide => window.set_visible(false),
          WindowMessage::Close => {
            panic!("cannot handle `WindowMessage::Close` on the main thread")
          }
          WindowMessage::Destroy => {
            panic!("cannot handle `WindowMessage::Destroy` on the main thread")
          }
          WindowMessage::SetDecorations(decorations) => window.set_decorations(decorations),
          WindowMessage::SetShadow(_enable) => {
            #[cfg(windows)]
            window.set_undecorated_shadow(_enable);
            #[cfg(target_os = "macos")]
            window.set_has_shadow(_enable);
          }
          WindowMessage::SetAlwaysOnBottom(always_on_bottom) => {
            window.set_always_on_bottom(always_on_bottom)
          }
          WindowMessage::SetAlwaysOnTop(always_on_top) => window.set_always_on_top(always_on_top),
          WindowMessage::SetVisibleOnAllWorkspaces(visible_on_all_workspaces) => {
            window.set_visible_on_all_workspaces(visible_on_all_workspaces)
          }
          WindowMessage::SetContentProtected(protected) => window.set_content_protection(protected),
          WindowMessage::SetSize(size) => {
            window.set_inner_size(SizeWrapper::from(size).0);
          }
          WindowMessage::SetMinSize(size) => {
            window.set_min_inner_size(size.map(|s| SizeWrapper::from(s).0));
          }
          WindowMessage::SetMaxSize(size) => {
            window.set_max_inner_size(size.map(|s| SizeWrapper::from(s).0));
          }
          WindowMessage::SetPosition(position) => {
            window.set_outer_position(PositionWrapper::from(position).0)
          }
          WindowMessage::SetFullscreen(fullscreen) => {
            if fullscreen {
              window.set_fullscreen(Some(Fullscreen::Borderless(None)))
            } else {
              window.set_fullscreen(None)
            }
            #[cfg(windows)]
            if let Some(w) = windows.0.borrow_mut().get_mut(&id) {
              w.is_window_fullscreen = fullscreen;
            }
          }
          WindowMessage::SetFocus => {
            window.set_focus();
          }
          WindowMessage::SetIcon(icon) => {
            window.set_window_icon(Some(icon));
          }
          #[allow(unused_variables)]
          WindowMessage::SetSkipTaskbar(skip) => {
            #[cfg(any(windows, target_os = "linux"))]
            window.set_skip_taskbar(skip);
          }
          WindowMessage::SetCursorGrab(grab) => {
            let _ = window.set_cursor_grab(grab);
          }
          WindowMessage::SetCursorVisible(visible) => {
            window.set_cursor_visible(visible);
          }
          WindowMessage::SetCursorIcon(icon) => {
            window.set_cursor_icon(CursorIconWrapper::from(icon).0);
          }
          WindowMessage::SetCursorPosition(position) => {
            let _ = window.set_cursor_position(PositionWrapper::from(position).0);
          }
          WindowMessage::SetIgnoreCursorEvents(ignore) => {
            let _ = window.set_ignore_cursor_events(ignore);
          }
          WindowMessage::DragWindow => {
            let _ = window.drag_window();
          }
          WindowMessage::ResizeDragWindow(direction) => {
            let _ = window.drag_resize_window(match direction {
              tauri_runtime::ResizeDirection::East => tao::window::ResizeDirection::East,
              tauri_runtime::ResizeDirection::North => tao::window::ResizeDirection::North,
              tauri_runtime::ResizeDirection::NorthEast => tao::window::ResizeDirection::NorthEast,
              tauri_runtime::ResizeDirection::NorthWest => tao::window::ResizeDirection::NorthWest,
              tauri_runtime::ResizeDirection::South => tao::window::ResizeDirection::South,
              tauri_runtime::ResizeDirection::SouthEast => tao::window::ResizeDirection::SouthEast,
              tauri_runtime::ResizeDirection::SouthWest => tao::window::ResizeDirection::SouthWest,
              tauri_runtime::ResizeDirection::West => tao::window::ResizeDirection::West,
            });
          }
          WindowMessage::RequestRedraw => {
            window.request_redraw();
          }
          WindowMessage::SetProgressBar(progress_state) => {
            window.set_progress_bar(ProgressBarStateWrapper::from(progress_state).0);
          }
        }
      }
    }

    Message::Webview(window_id, webview_id, webview_message) => {
      let webview_handle = windows.0.borrow().get(&window_id).map(|w| {
        (
          w.inner.clone(),
          w.webviews.iter().find(|w| w.id == webview_id).cloned(),
        )
      });
      if let Some((Some(window), Some(webview))) = webview_handle {
        match webview_message {
          WebviewMessage::WebviewEvent(_) => { /* already handled */ }
          WebviewMessage::SynthesizedWindowEvent(_) => { /* already handled */ }

          WebviewMessage::AddEventListener(id, listener) => {
            webview
              .webview_event_listeners
              .lock()
              .unwrap()
              .insert(id, listener);
          }

          #[cfg(all(feature = "tracing", not(target_os = "android")))]
          WebviewMessage::EvaluateScript(script, tx, span) => {
            let _span = span.entered();
            if let Err(e) = webview.evaluate_script(&script) {
              debug_eprintln!("{}", e);
            }
            tx.send(()).unwrap();
          }
          #[cfg(not(all(feature = "tracing", not(target_os = "android"))))]
          WebviewMessage::EvaluateScript(script) => {
            if let Err(e) = webview.evaluate_script(&script) {
              debug_eprintln!("{}", e);
            }
          }
          WebviewMessage::Navigate(url) => webview.load_url(url.as_str()),
          WebviewMessage::Print => {
            let _ = webview.print();
          }
          WebviewMessage::Close => {
            windows.0.borrow_mut().get_mut(&window_id).map(|window| {
              if let Some(i) = window.webviews.iter().position(|w| w.id == webview.id) {
                window.webviews.remove(i);
              }
              window
            });
          }
          WebviewMessage::SetSize(size) => {
            let mut bounds = webview.bounds();
            let size = size.to_logical(window.scale_factor());
            bounds.width = size.width;
            bounds.height = size.height;

            if let Some(b) = &webview.bounds {
              let window_size = window.inner_size();
              let mut bounds = b.lock().unwrap();
              bounds.width_rate = size.width as f32 / window_size.width as f32;
              bounds.height_rate = size.height as f32 / window_size.height as f32;
            }

            webview.set_bounds(bounds);
          }
          WebviewMessage::SetPosition(position) => {
            let mut bounds = webview.bounds();
            let position = position.to_logical(window.scale_factor());
            bounds.x = position.x;
            bounds.y = position.y;

            if let Some(b) = &webview.bounds {
              let window_size = window.inner_size();
              let mut bounds = b.lock().unwrap();
              bounds.width_rate = position.x as f32 / window_size.width as f32;
              bounds.height_rate = position.y as f32 / window_size.height as f32;
            }

            webview.set_bounds(bounds);
          }
          WebviewMessage::SetFocus => {
            webview.focus();
          }
          WebviewMessage::WithWebview(f) => {
            #[cfg(any(
              target_os = "linux",
              target_os = "dragonfly",
              target_os = "freebsd",
              target_os = "netbsd",
              target_os = "openbsd"
            ))]
            {
              f(webview.webview());
            }
            #[cfg(target_os = "macos")]
            {
              use wry::WebViewExtMacOS;
              f(Webview {
                webview: webview.webview(),
                manager: webview.manager(),
                ns_window: webview.ns_window(),
              });
            }
            #[cfg(target_os = "ios")]
            {
              use tao::platform::ios::WindowExtIOS;
              use wry::WebViewExtIOS;

              f(Webview {
                webview: webview.inner.webview(),
                manager: webview.inner.manager(),
                view_controller: window.ui_view_controller() as cocoa::base::id,
              });
            }
            #[cfg(windows)]
            {
              f(Webview {
                controller: webview.controller(),
              });
            }
            #[cfg(target_os = "android")]
            {
              f(webview.handle())
            }
          }

          #[cfg(any(debug_assertions, feature = "devtools"))]
          WebviewMessage::OpenDevTools => {
            webview.open_devtools();
          }
          #[cfg(any(debug_assertions, feature = "devtools"))]
          WebviewMessage::CloseDevTools => {
            webview.close_devtools();
          }
          #[cfg(any(debug_assertions, feature = "devtools"))]
          WebviewMessage::IsDevToolsOpen(tx) => {
            tx.send(webview.is_devtools_open()).unwrap();
          }

          // Getters
          WebviewMessage::Url(tx) => {
            tx.send(webview.url().parse().unwrap()).unwrap();
          }
          WebviewMessage::Position(tx) => {
            let bounds = webview.bounds();
            let position =
              LogicalPosition::new(bounds.x, bounds.y).to_physical(window.scale_factor());
            tx.send(position).unwrap();
          }
          WebviewMessage::Size(tx) => {
            let bounds = webview.bounds();
            let size =
              LogicalSize::new(bounds.width, bounds.height).to_physical(window.scale_factor());
            tx.send(size).unwrap();
          }
        }
      }
    }
    Message::CreateWebview(window_id, handler) => {
      let window = windows
        .0
        .borrow()
        .get(&window_id)
        .and_then(|w| w.inner.clone());
      if let Some(window) = window {
        match handler(&window) {
          Ok(webview) => {
            windows.0.borrow_mut().get_mut(&window_id).map(|w| {
              w.webviews.push(webview);
              w
            });
          }
          Err(e) => {
            debug_eprintln!("{}", e);
          }
        }
      }
    }
    Message::CreateWindow(window_id, handler) => match handler(event_loop) {
      Ok(webview) => {
        windows.0.borrow_mut().insert(window_id, webview);
      }
      Err(e) => {
        debug_eprintln!("{}", e);
      }
    },
    Message::CreateRawWindow(window_id, handler, sender) => {
      let (label, builder) = handler();

      #[cfg(windows)]
      let is_window_fullscreen = builder.window.fullscreen.is_some();
      #[cfg(windows)]
      let is_window_transparent = builder.window.transparent;

      if let Ok(window) = builder.build(event_loop) {
        window_id_map.insert(window.id(), window_id);

        let window = Arc::new(window);

        #[cfg(windows)]
        let surface = if is_window_transparent {
          if let Ok(context) = softbuffer::Context::new(window.clone()) {
            if let Ok(mut surface) = softbuffer::Surface::new(&context, window.clone()) {
              clear_window_surface(&window, &mut surface);
              Some(surface)
            } else {
              None
            }
          } else {
            None
          }
        } else {
          None
        };

        windows.0.borrow_mut().insert(
          window_id,
          WindowWrapper {
            label,
            has_children: AtomicBool::new(false),
            inner: Some(window.clone()),
            window_event_listeners: Default::default(),
            webviews: Vec::new(),
            #[cfg(windows)]
            is_window_fullscreen,
            #[cfg(windows)]
            is_window_transparent,
            #[cfg(windows)]
            surface,
          },
        );
        sender.send(Ok(Arc::downgrade(&window))).unwrap();
      } else {
        sender.send(Err(Error::CreateWindow)).unwrap();
      }
    }

    Message::UserEvent(_) => (),
  }
}

fn handle_event_loop<T: UserEvent>(
  event: Event<'_, Message<T>>,
  event_loop: &EventLoopWindowTarget<Message<T>>,
  control_flow: &mut ControlFlow,
  context: EventLoopIterationContext<'_, T>,
) {
  let EventLoopIterationContext {
    callback,
    window_id_map,
    windows,
    #[cfg(feature = "tracing")]
    active_tracing_spans,
  } = context;
  if *control_flow != ControlFlow::Exit {
    *control_flow = ControlFlow::Wait;
  }

  match event {
    Event::NewEvents(StartCause::Init) => {
      callback(RunEvent::Ready);
    }

    Event::NewEvents(StartCause::Poll) => {
      callback(RunEvent::Resumed);
    }

    Event::MainEventsCleared => {
      callback(RunEvent::MainEventsCleared);
    }

    Event::LoopDestroyed => {
      callback(RunEvent::Exit);
    }

    #[cfg(any(feature = "tracing", windows))]
    Event::RedrawRequested(id) => {
      #[cfg(windows)]
      if let Some(window_id) = window_id_map.get(&id) {
        let mut windows_ref = windows.0.borrow_mut();
        if let Some(window) = windows_ref.get_mut(&window_id) {
          if window.is_window_transparent {
            if let Some(surface) = &mut window.surface {
              if let Some(window) = &window.inner {
                clear_window_surface(window, surface)
              }
            }
          }
        }
      }

      #[cfg(feature = "tracing")]
      active_tracing_spans.remove_window_draw(id);
    }

    Event::UserEvent(Message::Webview(
      window_id,
      webview_id,
      WebviewMessage::WebviewEvent(event),
    )) => {
      let windows_ref = windows.0.borrow();
      if let Some(window) = windows_ref.get(&window_id) {
        if let Some(webview) = window.webviews.iter().find(|w| w.id == webview_id) {
          let label = webview.label.clone();
          let webview_event_listeners = webview.webview_event_listeners.clone();

          drop(windows_ref);

          callback(RunEvent::WebviewEvent {
            label,
            event: event.clone(),
          });
          let listeners = webview_event_listeners.lock().unwrap();
          let handlers = listeners.values();
          for handler in handlers {
            handler(&event);
          }
        }
      }
    }

    Event::UserEvent(Message::Webview(
      window_id,
      _webview_id,
      WebviewMessage::SynthesizedWindowEvent(event),
    )) => {
      if let Some(event) = WindowEventWrapper::from(event).0 {
        let windows_ref = windows.0.borrow();
        let window = windows_ref.get(&window_id);
        if let Some(window) = window {
          let label = window.label.clone();
          let window_event_listeners = window.window_event_listeners.clone();

          drop(windows_ref);

          callback(RunEvent::WindowEvent {
            label,
            event: event.clone(),
          });

          let listeners = window_event_listeners.lock().unwrap();
          let handlers = listeners.values();
          for handler in handlers {
            handler(&event);
          }
        }
      }
    }

    Event::WindowEvent {
      event, window_id, ..
    } => {
      if let Some(window_id) = window_id_map.get(&window_id) {
        {
          let windows_ref = windows.0.borrow();
          if let Some(window) = windows_ref.get(&window_id) {
            if let Some(event) = WindowEventWrapper::parse(window, &event).0 {
              let label = window.label.clone();
              let window_event_listeners = window.window_event_listeners.clone();

              drop(windows_ref);

              callback(RunEvent::WindowEvent {
                label,
                event: event.clone(),
              });
              let listeners = window_event_listeners.lock().unwrap();
              let handlers = listeners.values();
              for handler in handlers {
                handler(&event);
              }
            }
          }
        }

        match event {
          #[cfg(windows)]
          TaoWindowEvent::ThemeChanged(theme) => {
            if let Some(window) = windows.0.borrow().get(&window_id) {
              for webview in &window.webviews {
                let theme = match theme {
                  TaoTheme::Dark => wry::Theme::Dark,
                  TaoTheme::Light => wry::Theme::Light,
                  _ => wry::Theme::Light,
                };
                webview.set_theme(theme);
              }
            }
          }
          TaoWindowEvent::CloseRequested => {
            on_close_requested(callback, window_id, windows.clone());
          }
          TaoWindowEvent::Destroyed => {
            let removed = windows.0.borrow_mut().remove(&window_id).is_some();
            if removed {
              let is_empty = windows.0.borrow().is_empty();
              if is_empty {
                let (tx, rx) = channel();
                callback(RunEvent::ExitRequested { code: None, tx });

                let recv = rx.try_recv();
                let should_prevent = matches!(recv, Ok(ExitRequestedEventAction::Prevent));

                if !should_prevent {
                  *control_flow = ControlFlow::Exit;
                }
              }
            }
          }
          TaoWindowEvent::Resized(size) => {
            if let Some(webviews) = windows
              .0
              .borrow()
              .get(&window_id)
              .map(|w| w.webviews.clone())
            {
              for webview in webviews {
                if let Some(bounds) = &webview.bounds {
                  let b = bounds.lock().unwrap();
                  webview.set_bounds(wry::Rect {
                    x: (size.width as f32 * b.x_rate) as i32,
                    y: (size.height as f32 * b.y_rate) as i32,
                    width: (size.width as f32 * b.width_rate) as u32,
                    height: (size.height as f32 * b.height_rate) as u32,
                  });
                }
              }
            }
          }
          _ => {}
        }
      }
    }
    Event::UserEvent(message) => match message {
      Message::RequestExit(code) => {
        let (tx, rx) = channel();
        callback(RunEvent::ExitRequested {
          code: Some(code),
          tx,
        });

        let recv = rx.try_recv();
        let should_prevent = matches!(recv, Ok(ExitRequestedEventAction::Prevent));

        if !should_prevent {
          *control_flow = ControlFlow::Exit;
        }
      }
      Message::Window(id, WindowMessage::Close) => {
        on_close_requested(callback, id, windows.clone());
      }
      Message::Window(id, WindowMessage::Destroy) => {
        on_window_close(id, windows.clone());
      }
      Message::UserEvent(t) => callback(RunEvent::UserEvent(t)),
      message => {
        handle_user_message(
          event_loop,
          message,
          UserMessageContext {
            window_id_map,
            windows,
          },
        );
      }
    },
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    Event::Opened { urls } => {
      callback(RunEvent::Opened { urls });
    }
    _ => (),
  }
}

fn on_close_requested<'a, T: UserEvent>(
  callback: &'a mut (dyn FnMut(RunEvent<T>) + 'static),
  window_id: WindowId,
  windows: Arc<WindowsStore>,
) {
  let (tx, rx) = channel();
  let windows_ref = windows.0.borrow();
  if let Some(w) = windows_ref.get(&window_id) {
    let label = w.label.clone();
    let window_event_listeners = w.window_event_listeners.clone();

    drop(windows_ref);

    let listeners = window_event_listeners.lock().unwrap();
    let handlers = listeners.values();
    for handler in handlers {
      handler(&WindowEvent::CloseRequested {
        signal_tx: tx.clone(),
      });
    }
    callback(RunEvent::WindowEvent {
      label,
      event: WindowEvent::CloseRequested { signal_tx: tx },
    });
    if let Ok(true) = rx.try_recv() {
    } else {
      on_window_close(window_id, windows);
    }
  }
}

fn on_window_close(window_id: WindowId, windows: Arc<WindowsStore>) {
  if let Some(window_wrapper) = windows.0.borrow_mut().get_mut(&window_id) {
    window_wrapper.inner = None;
    #[cfg(windows)]
    window_wrapper.surface.take();
  }
}

fn parse_proxy_url(url: &Url) -> Result<ProxyConfig> {
  let host = url.host().map(|h| h.to_string()).unwrap_or_default();
  let port = url.port().map(|p| p.to_string()).unwrap_or_default();

  if url.scheme() == "http" {
    let config = ProxyConfig::Http(ProxyEndpoint { host, port });

    Ok(config)
  } else if url.scheme() == "socks5" {
    let config = ProxyConfig::Socks5(ProxyEndpoint { host, port });

    Ok(config)
  } else {
    Err(Error::InvalidProxyUrl)
  }
}

fn create_window<T: UserEvent, F: Fn(RawWindow) + Send + 'static>(
  window_id: WindowId,
  webview_id: u32,
  event_loop: &EventLoopWindowTarget<Message<T>>,
  context: &Context<T>,
  pending: PendingWindow<T, Wry<T>>,
  after_window_creation: Option<F>,
) -> Result<WindowWrapper> {
  #[allow(unused_mut)]
  let PendingWindow {
    mut window_builder,
    label,
    webview,
  } = pending;

  #[cfg(feature = "tracing")]
  let _webview_create_span = tracing::debug_span!("wry::webview::create").entered();
  #[cfg(feature = "tracing")]
  let window_draw_span = tracing::debug_span!("wry::window::draw").entered();
  #[cfg(feature = "tracing")]
  let window_create_span =
    tracing::debug_span!(parent: &window_draw_span, "wry::window::create").entered();

  let window_event_listeners = WindowEventListeners::default();

  #[cfg(windows)]
  let is_window_transparent = window_builder.inner.window.transparent;
  #[cfg(windows)]
  let is_window_fullscreen = window_builder.inner.window.fullscreen.is_some();

  #[cfg(target_os = "macos")]
  {
    if window_builder.tabbing_identifier.is_none()
      || window_builder.inner.window.transparent
      || !window_builder.inner.window.decorations
    {
      window_builder.inner = window_builder.inner.with_automatic_window_tabbing(false);
    }
  }

  #[cfg(desktop)]
  if window_builder.center {
    let monitor = if let Some(window_position) = &window_builder.inner.window.position {
      event_loop.available_monitors().find(|m| {
        let monitor_pos = m.position();
        let monitor_size = m.size();

        let window_position = window_position.to_logical(m.scale_factor());

        monitor_pos.x <= window_position.x
          && window_position.x <= monitor_pos.x + monitor_size.width as i32
          && monitor_pos.y <= window_position.y
          && window_position.y <= monitor_pos.y + monitor_size.height as i32
      })
    } else {
      event_loop.primary_monitor()
    };

    if let Some(monitor) = monitor {
      let desired_size = window_builder
        .inner
        .window
        .inner_size
        .unwrap_or_else(|| TaoPhysicalSize::new(800, 600).into());
      let window_size = window_builder
        .inner
        .window
        .inner_size_constraints
        .clamp(desired_size, monitor.scale_factor())
        .to_logical::<i32>(monitor.scale_factor());
      let scale_factor = monitor.scale_factor();
      let screen_size = monitor.size().to_logical::<i32>(scale_factor);
      let monitor_pos = monitor.position().to_logical::<i32>(scale_factor);
      let x = (screen_size.width as i32 - window_size.width) / 2 + monitor_pos.x;
      let y = (screen_size.height as i32 - window_size.height) / 2 + monitor_pos.y;

      window_builder = window_builder.position(x as f64, y as f64);
    }
  }

  let window = window_builder.inner.build(event_loop).unwrap();

  #[cfg(feature = "tracing")]
  {
    drop(window_create_span);

    context
      .main_thread
      .active_tracing_spans
      .0
      .borrow_mut()
      .push(ActiveTracingSpan::WindowDraw {
        id: window.id(),
        span: window_draw_span,
      });
  }

  context.window_id_map.insert(window.id(), window_id);

  if let Some(handler) = after_window_creation {
    let raw = RawWindow {
      #[cfg(windows)]
      hwnd: window.hwnd(),
      #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
      ))]
      gtk_window: window.gtk_window(),
      #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
      ))]
      default_vbox: window.default_vbox(),
      _marker: &std::marker::PhantomData,
    };
    handler(raw);
  }

  let mut webviews = Vec::new();

  if let Some(webview) = webview {
    webviews.push(create_webview(
      WebviewKind::WindowContent,
      &window,
      window_id,
      webview_id,
      context,
      webview,
    )?);
  }

  let window = Arc::new(window);

  #[cfg(windows)]
  let surface = if is_window_transparent {
    if let Ok(context) = softbuffer::Context::new(window.clone()) {
      if let Ok(mut surface) = softbuffer::Surface::new(&context, window.clone()) {
        clear_window_surface(&window, &mut surface);
        Some(surface)
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  };

  Ok(WindowWrapper {
    label,
    has_children: AtomicBool::new(false),
    inner: Some(window),
    webviews,
    window_event_listeners,
    #[cfg(windows)]
    is_window_fullscreen,
    #[cfg(windows)]
    is_window_transparent,
    #[cfg(windows)]
    surface,
  })
}

/// the kind of the webview
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum WebviewKind {
  // webview is the entire window content
  WindowContent,
  // webview is a child of the window, which can contain other webviews too
  WindowChild,
}

#[derive(Clone)]
struct WebviewBounds {
  x_rate: f32,
  y_rate: f32,
  width_rate: f32,
  height_rate: f32,
}

fn create_webview<T: UserEvent>(
  kind: WebviewKind,
  window: &Window,
  window_id: WindowId,
  id: WebviewId,
  context: &Context<T>,
  pending: PendingWebview<T, Wry<T>>,
) -> Result<WebviewWrapper> {
  #[allow(unused_mut)]
  let PendingWebview {
    webview_attributes,
    uri_scheme_protocols,
    label,
    ipc_handler,
    url,
    ..
  } = pending;

  let builder = match kind {
    #[cfg(not(any(
      target_os = "windows",
      target_os = "macos",
      target_os = "ios",
      target_os = "android"
    )))]
    WebviewKind::WindowChild => {
      // only way to account for menu bar height, and also works for multiwebviews :)
      let vbox = window.default_vbox().unwrap();
      WebViewBuilder::new_gtk(vbox)
    }
    #[cfg(any(
      target_os = "windows",
      target_os = "macos",
      target_os = "ios",
      target_os = "android"
    ))]
    WebviewKind::WindowChild => WebViewBuilder::new_as_child(&window),
    WebviewKind::WindowContent => {
      #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
      ))]
      let builder = WebViewBuilder::new(&window);
      #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
      )))]
      let builder = {
        let vbox = window.default_vbox().unwrap();
        WebViewBuilder::new_gtk(vbox)
      };
      builder
    }
  };

  let mut webview_builder = builder
    .with_focused(window.is_focused())
    .with_url(&url)
    .with_transparent(webview_attributes.transparent)
    .with_accept_first_mouse(webview_attributes.accept_first_mouse);

  #[cfg(windows)]
  if kind == WebviewKind::WindowContent {
    webview_builder = webview_builder.with_initialization_script(undecorated_resizing::SCRIPT);
  }

  if webview_attributes.file_drop_handler_enabled {
    let proxy = context.proxy.clone();
    webview_builder = webview_builder.with_file_drop_handler(move |event| {
      let event = match event {
        WryFileDropEvent::Hovered {
          paths,
          position: (x, y),
        } => FileDropEvent::Hovered {
          paths,
          position: PhysicalPosition::new(x as _, y as _),
        },
        WryFileDropEvent::Dropped {
          paths,
          position: (x, y),
        } => FileDropEvent::Dropped {
          paths,
          position: PhysicalPosition::new(x as _, y as _),
        },
        WryFileDropEvent::Cancelled => FileDropEvent::Cancelled,
        _ => unimplemented!(),
      };

      let message = if kind == WebviewKind::WindowContent {
        WebviewMessage::SynthesizedWindowEvent(SynthesizedWindowEvent::FileDrop(event))
      } else {
        WebviewMessage::WebviewEvent(WebviewEvent::FileDrop(event))
      };

      let _ = proxy.send_event(Message::Webview(window_id, id, message));
      true
    });
  }

  if let Some(navigation_handler) = pending.navigation_handler {
    webview_builder = webview_builder.with_navigation_handler(move |url| {
      Url::parse(&url)
        .map(|url| navigation_handler(&url))
        .unwrap_or(true)
    });
  }

  let webview_bounds = if let Some((position, size)) = webview_attributes.bounds {
    let size = size.to_logical(window.scale_factor());
    let position = position.to_logical(window.scale_factor());
    webview_builder = webview_builder.with_bounds(wry::Rect {
      x: position.x,
      y: position.y,
      width: size.width,
      height: size.height,
    });

    let window_size = window.inner_size();

    if webview_attributes.auto_resize {
      Some(WebviewBounds {
        x_rate: (position.x as f32) / window_size.width as f32,
        y_rate: (position.y as f32) / window_size.height as f32,
        width_rate: (size.width as f32) / window_size.width as f32,
        height_rate: (size.height as f32) / window_size.height as f32,
      })
    } else {
      None
    }
  } else {
    None
  };

  if let Some(download_handler) = pending.download_handler {
    let download_handler_ = download_handler.clone();
    webview_builder = webview_builder.with_download_started_handler(move |url, path| {
      if let Ok(url) = url.parse() {
        download_handler_(DownloadEvent::Requested {
          url,
          destination: path,
        })
      } else {
        false
      }
    });
    webview_builder = webview_builder.with_download_completed_handler(move |url, path, success| {
      if let Ok(url) = url.parse() {
        download_handler(DownloadEvent::Finished { url, path, success });
      }
    });
  }

  if let Some(page_load_handler) = pending.on_page_load_handler {
    webview_builder = webview_builder.with_on_page_load_handler(move |event, url| {
      let _ = Url::parse(&url).map(|url| {
        page_load_handler(
          url,
          match event {
            wry::PageLoadEvent::Started => tauri_runtime::webview::PageLoadEvent::Started,
            wry::PageLoadEvent::Finished => tauri_runtime::webview::PageLoadEvent::Finished,
          },
        )
      });
    });
  }

  if let Some(user_agent) = webview_attributes.user_agent {
    webview_builder = webview_builder.with_user_agent(&user_agent);
  }

  if let Some(proxy_url) = webview_attributes.proxy_url {
    let config = parse_proxy_url(&proxy_url)?;

    webview_builder = webview_builder.with_proxy_config(config);
  }

  #[cfg(windows)]
  {
    if let Some(additional_browser_args) = webview_attributes.additional_browser_args {
      webview_builder = webview_builder.with_additional_browser_args(&additional_browser_args);
    }

    webview_builder = webview_builder.with_theme(match window.theme() {
      TaoTheme::Dark => wry::Theme::Dark,
      TaoTheme::Light => wry::Theme::Light,
      _ => wry::Theme::Light,
    });
  }

  #[cfg(windows)]
  {
    webview_builder = webview_builder.with_https_scheme(false);
  }

  webview_builder = webview_builder.with_ipc_handler(create_ipc_handler(
    kind,
    window_id,
    id,
    context.clone(),
    label.clone(),
    ipc_handler,
  ));

  for (scheme, protocol) in uri_scheme_protocols {
    webview_builder =
      webview_builder.with_asynchronous_custom_protocol(scheme, move |request, responder| {
        protocol(
          request,
          Box::new(move |response| responder.respond(response)),
        )
      });
  }

  for script in webview_attributes.initialization_scripts {
    webview_builder = webview_builder.with_initialization_script(&script);
  }

  let mut web_context = context
    .main_thread
    .web_context
    .lock()
    .expect("poisoned WebContext store");
  let is_first_context = web_context.is_empty();
  let automation_enabled = std::env::var("TAURI_WEBVIEW_AUTOMATION").as_deref() == Ok("true");
  let web_context_key = // force a unique WebContext when automation is false;
    // the context must be stored on the HashMap because it must outlive the WebView on macOS
    if automation_enabled {
      webview_attributes.data_directory.clone()
    } else {
      // unique key
      let key = context.next_webcontext_id().to_string().into();
      Some(key)
    };
  let entry = web_context.entry(web_context_key.clone());
  let web_context = match entry {
    Occupied(occupied) => occupied.into_mut(),
    Vacant(vacant) => {
      let mut web_context = WebContext::new(webview_attributes.data_directory);
      web_context.set_allows_automation(if automation_enabled {
        is_first_context
      } else {
        false
      });
      vacant.insert(web_context)
    }
  };

  if webview_attributes.clipboard {
    webview_builder.attrs.clipboard = true;
  }

  if webview_attributes.incognito {
    webview_builder.attrs.incognito = true;
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  {
    webview_builder = webview_builder.with_devtools(true);
  }

  #[cfg(target_os = "android")]
  {
    if let Some(on_webview_created) = pending.on_webview_created {
      webview_builder = webview_builder.on_webview_created(move |ctx| {
        on_webview_created(tauri_runtime::webview::CreationContext {
          env: ctx.env,
          activity: ctx.activity,
          webview: ctx.webview,
        })
      });
    }
  }

  let webview = webview_builder
    .with_web_context(web_context)
    .build()
    .map_err(|e| Error::CreateWebview(Box::new(e)))?;

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  if kind == WebviewKind::WindowContent {
    undecorated_resizing::attach_resize_handler(&webview);
  }

  #[cfg(windows)]
  if kind == WebviewKind::WindowContent {
    let controller = webview.controller();
    let proxy = context.proxy.clone();
    let proxy_ = proxy.clone();
    let mut token = EventRegistrationToken::default();
    unsafe {
      controller.add_GotFocus(
        &FocusChangedEventHandler::create(Box::new(move |_, _| {
          let _ = proxy_.send_event(Message::Webview(
            window_id,
            id,
            WebviewMessage::SynthesizedWindowEvent(SynthesizedWindowEvent::Focused(true)),
          ));
          Ok(())
        })),
        &mut token,
      )
    }
    .unwrap();
    unsafe {
      controller.add_LostFocus(
        &FocusChangedEventHandler::create(Box::new(move |_, _| {
          let _ = proxy.send_event(Message::Webview(
            window_id,
            id,
            WebviewMessage::SynthesizedWindowEvent(SynthesizedWindowEvent::Focused(false)),
          ));
          Ok(())
        })),
        &mut token,
      )
    }
    .unwrap();
  }

  Ok(WebviewWrapper {
    label,
    id,
    inner: Rc::new(webview),
    context_store: context.main_thread.web_context.clone(),
    webview_event_listeners: Default::default(),
    context_key: if automation_enabled {
      None
    } else {
      web_context_key
    },
    bounds: webview_bounds.map(|b| Arc::new(Mutex::new(b))),
  })
}

/// Create a wry ipc handler from a tauri ipc handler.
fn create_ipc_handler<T: UserEvent>(
  _kind: WebviewKind,
  window_id: WindowId,
  webview_id: WebviewId,
  context: Context<T>,
  label: String,
  ipc_handler: Option<WebviewIpcHandler<T, Wry<T>>>,
) -> Box<IpcHandler> {
  Box::new(move |request| {
    #[cfg(windows)]
    if _kind == WebviewKind::WindowContent
      && undecorated_resizing::handle_request(context.clone(), window_id, &request)
    {
      return;
    }

    if let Some(handler) = &ipc_handler {
      handler(
        DetachedWebview {
          label: label.clone(),
          dispatcher: WryWebviewDispatcher {
            window_id,
            webview_id,
            context: context.clone(),
          },
        },
        request,
      );
    }
  })
}

#[cfg(target_os = "macos")]
fn inner_size(
  window: &Window,
  webviews: &[WebviewWrapper],
  has_children: bool,
) -> TaoPhysicalSize<u32> {
  if has_children && webviews.len() == 1 {
    use wry::WebViewExtMacOS;
    let webview = webviews.first().unwrap();
    let view_frame = unsafe { cocoa::appkit::NSView::frame(webview.webview()) };
    let logical: TaoLogicalSize<f64> = (view_frame.size.width, view_frame.size.height).into();
    return logical.to_physical(window.scale_factor());
  }

  window.inner_size()
}

#[cfg(not(target_os = "macos"))]
#[allow(unused_variables)]
fn inner_size(
  window: &Window,
  webviews: &[WebviewWrapper],
  has_children: bool,
) -> TaoPhysicalSize<u32> {
  window.inner_size()
}

#[cfg(windows)]
fn clear_window_surface(
  window: &Window,
  surface: &mut softbuffer::Surface<Arc<Window>, Arc<Window>>,
) {
  let size = window.inner_size();
  if let (Some(width), Some(height)) = (
    std::num::NonZeroU32::new(size.width),
    std::num::NonZeroU32::new(size.height),
  ) {
    surface.resize(width, height).unwrap();
    let mut buffer = surface.buffer_mut().unwrap();
    buffer.fill(0);
    let _ = buffer.present();
  }
}
