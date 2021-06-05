// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The [`wry`] Tauri [`Runtime`].

use tauri_runtime::{
  monitor::Monitor,
  webview::{
    FileDropEvent, FileDropHandler, RpcRequest, WebviewRpcHandler, WindowBuilder, WindowBuilderBase,
  },
  window::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size},
    DetachedWindow, PendingWindow, WindowEvent,
  },
  Dispatch, Error, Icon, Params, Result, RunIteration, Runtime, RuntimeHandle,
};

#[cfg(feature = "menu")]
use tauri_runtime::window::MenuEvent;
#[cfg(feature = "system-tray")]
use tauri_runtime::{SystemTray, SystemTrayEvent};
#[cfg(windows)]
use winapi::shared::windef::HWND;
#[cfg(windows)]
use wry::application::platform::windows::WindowBuilderExtWindows;
#[cfg(feature = "system-tray")]
use wry::application::system_tray::{SystemTray as WrySystemTray, SystemTrayBuilder};

use tauri_utils::config::WindowConfig;
use uuid::Uuid;
use wry::{
  application::{
    dpi::{
      LogicalPosition as WryLogicalPosition, LogicalSize as WryLogicalSize,
      PhysicalPosition as WryPhysicalPosition, PhysicalSize as WryPhysicalSize,
      Position as WryPosition, Size as WrySize,
    },
    event::{Event, WindowEvent as WryWindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
    monitor::MonitorHandle,
    window::{Fullscreen, Icon as WindowIcon, Window, WindowBuilder as WryWindowBuilder, WindowId},
  },
  webview::{
    FileDropEvent as WryFileDropEvent, RpcRequest as WryRpcRequest, RpcResponse, WebView,
    WebViewBuilder,
  },
};

use std::{
  collections::HashMap,
  convert::TryFrom,
  fs::read,
  sync::{
    mpsc::{channel, Sender},
    Arc, Mutex, MutexGuard,
  },
};

#[cfg(any(feature = "menu", feature = "system-tray"))]
mod menu;
#[cfg(any(feature = "menu", feature = "system-tray"))]
use menu::*;

type MainTask = Arc<Mutex<Option<Box<dyn FnOnce() + Send>>>>;
type CreateWebviewHandler =
  Box<dyn FnOnce(&EventLoopWindowTarget<Message>) -> Result<WebviewWrapper> + Send>;
type WindowEventHandler = Box<dyn Fn(&WindowEvent) + Send>;
type WindowEventListeners = Arc<Mutex<HashMap<Uuid, WindowEventHandler>>>;

/// Wrapper around a [`wry::application::window::Icon`] that can be created from an [`Icon`].
pub struct WryIcon(WindowIcon);

fn icon_err<E: std::error::Error + Send + 'static>(e: E) -> Error {
  Error::InvalidIcon(Box::new(e))
}

impl TryFrom<Icon> for WryIcon {
  type Error = Error;
  fn try_from(icon: Icon) -> std::result::Result<Self, Self::Error> {
    let image_bytes = match icon {
      Icon::File(path) => read(path).map_err(icon_err)?,
      Icon::Raw(raw) => raw,
      _ => unimplemented!(),
    };
    let extension = infer::get(&image_bytes)
      .expect("could not determine icon extension")
      .extension();
    match extension {
      #[cfg(windows)]
      "ico" => {
        let icon_dir = ico::IconDir::read(std::io::Cursor::new(image_bytes)).map_err(icon_err)?;
        let entry = &icon_dir.entries()[0];
        let icon = WindowIcon::from_rgba(
          entry.decode().map_err(icon_err)?.rgba_data().to_vec(),
          entry.width(),
          entry.height(),
        )
        .map_err(icon_err)?;
        Ok(Self(icon))
      }
      #[cfg(target_os = "linux")]
      "png" => {
        let decoder = png::Decoder::new(std::io::Cursor::new(image_bytes));
        let (info, mut reader) = decoder.read_info().map_err(icon_err)?;
        let mut buffer = Vec::new();
        while let Ok(Some(row)) = reader.next_row() {
          buffer.extend(row);
        }
        let icon = WindowIcon::from_rgba(buffer, info.width, info.height).map_err(icon_err)?;
        Ok(Self(icon))
      }
      _ => panic!(
        "image `{}` extension not supported; please file a Tauri feature request",
        extension
      ),
    }
  }
}

struct WindowEventWrapper(Option<WindowEvent>);

impl<'a> From<&WryWindowEvent<'a>> for WindowEventWrapper {
  fn from(event: &WryWindowEvent<'a>) -> Self {
    let event = match event {
      WryWindowEvent::Resized(size) => WindowEvent::Resized(PhysicalSizeWrapper(*size).into()),
      WryWindowEvent::Moved(position) => {
        WindowEvent::Moved(PhysicalPositionWrapper(*position).into())
      }
      WryWindowEvent::CloseRequested => WindowEvent::CloseRequested,
      WryWindowEvent::Destroyed => WindowEvent::Destroyed,
      WryWindowEvent::Focused(focused) => WindowEvent::Focused(*focused),
      WryWindowEvent::ScaleFactorChanged {
        scale_factor,
        new_inner_size,
      } => WindowEvent::ScaleFactorChanged {
        scale_factor: *scale_factor,
        new_inner_size: PhysicalSizeWrapper(**new_inner_size).into(),
      },
      _ => return Self(None),
    };
    Self(Some(event))
  }
}

pub struct MonitorHandleWrapper(MonitorHandle);

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

struct PhysicalPositionWrapper<T>(WryPhysicalPosition<T>);

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
    Self(WryPhysicalPosition {
      x: position.x,
      y: position.y,
    })
  }
}

struct LogicalPositionWrapper<T>(WryLogicalPosition<T>);

impl<T> From<LogicalPosition<T>> for LogicalPositionWrapper<T> {
  fn from(position: LogicalPosition<T>) -> Self {
    Self(WryLogicalPosition {
      x: position.x,
      y: position.y,
    })
  }
}

struct PhysicalSizeWrapper<T>(WryPhysicalSize<T>);

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
    Self(WryPhysicalSize {
      width: size.width,
      height: size.height,
    })
  }
}

struct LogicalSizeWrapper<T>(WryLogicalSize<T>);

impl<T> From<LogicalSize<T>> for LogicalSizeWrapper<T> {
  fn from(size: LogicalSize<T>) -> Self {
    Self(WryLogicalSize {
      width: size.width,
      height: size.height,
    })
  }
}

struct SizeWrapper(WrySize);

impl From<Size> for SizeWrapper {
  fn from(size: Size) -> Self {
    match size {
      Size::Logical(s) => Self(WrySize::Logical(LogicalSizeWrapper::from(s).0)),
      Size::Physical(s) => Self(WrySize::Physical(PhysicalSizeWrapper::from(s).0)),
    }
  }
}

struct PositionWrapper(WryPosition);

impl From<Position> for PositionWrapper {
  fn from(position: Position) -> Self {
    match position {
      Position::Logical(s) => Self(WryPosition::Logical(LogicalPositionWrapper::from(s).0)),
      Position::Physical(s) => Self(WryPosition::Physical(PhysicalPositionWrapper::from(s).0)),
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct WindowBuilderWrapper {
  inner: WryWindowBuilder,
  center: bool,
  #[cfg(feature = "menu")]
  menu_items: HashMap<u32, WryCustomMenuItem>,
}

// safe since `menu_items` are read only here
unsafe impl Send for WindowBuilderWrapper {}

impl WindowBuilderBase for WindowBuilderWrapper {}
impl WindowBuilder for WindowBuilderWrapper {
  fn new() -> Self {
    Default::default()
  }

  fn with_config(config: WindowConfig) -> Self {
    let mut window = WindowBuilderWrapper::new()
      .title(config.title.to_string())
      .inner_size(config.width, config.height)
      .visible(config.visible)
      .resizable(config.resizable)
      .decorations(config.decorations)
      .maximized(config.maximized)
      .fullscreen(config.fullscreen)
      .transparent(config.transparent)
      .always_on_top(config.always_on_top)
      .skip_taskbar(config.skip_taskbar);

    if let (Some(min_width), Some(min_height)) = (config.min_width, config.min_height) {
      window = window.min_inner_size(min_width, min_height);
    }
    if let (Some(max_width), Some(max_height)) = (config.max_width, config.max_height) {
      window = window.max_inner_size(max_width, max_height);
    }
    if let (Some(x), Some(y)) = (config.x, config.y) {
      window = window.position(x, y);
    }

    if config.focus {
      window = window.focus();
    }

    window
  }

  #[cfg(feature = "menu")]
  fn menu<I: MenuId>(mut self, menu: Menu<I>) -> Self {
    let mut items = HashMap::new();
    let window_menu = to_wry_menu(&mut items, menu);
    self.menu_items = items;
    self.inner = self.inner.with_menu(window_menu);
    self
  }

  fn center(mut self) -> Self {
    self.center = true;
    self
  }

  fn position(mut self, x: f64, y: f64) -> Self {
    self.inner = self.inner.with_position(WryLogicalPosition::new(x, y));
    self
  }

  fn inner_size(mut self, width: f64, height: f64) -> Self {
    self.inner = self
      .inner
      .with_inner_size(WryLogicalSize::new(width, height));
    self
  }

  fn min_inner_size(mut self, min_width: f64, min_height: f64) -> Self {
    self.inner = self
      .inner
      .with_min_inner_size(WryLogicalSize::new(min_width, min_height));
    self
  }

  fn max_inner_size(mut self, max_width: f64, max_height: f64) -> Self {
    self.inner = self
      .inner
      .with_max_inner_size(WryLogicalSize::new(max_width, max_height));
    self
  }

  fn resizable(mut self, resizable: bool) -> Self {
    self.inner = self.inner.with_resizable(resizable);
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

  fn focus(mut self) -> Self {
    self.inner = self.inner.with_focus();
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

  fn transparent(mut self, transparent: bool) -> Self {
    self.inner = self.inner.with_transparent(transparent);
    self
  }

  fn decorations(mut self, decorations: bool) -> Self {
    self.inner = self.inner.with_decorations(decorations);
    self
  }

  fn always_on_top(mut self, always_on_top: bool) -> Self {
    self.inner = self.inner.with_always_on_top(always_on_top);
    self
  }

  #[cfg(windows)]
  fn parent_window(mut self, parent: HWND) -> Self {
    self.inner = self.inner.with_parent_window(parent);
    self
  }

  #[cfg(windows)]
  fn owner_window(mut self, owner: HWND) -> Self {
    self.inner = self.inner.with_owner_window(owner);
    self
  }

  fn icon(mut self, icon: Icon) -> Result<Self> {
    self.inner = self
      .inner
      .with_window_icon(Some(WryIcon::try_from(icon)?.0));
    Ok(self)
  }

  fn skip_taskbar(mut self, skip: bool) -> Self {
    self.inner = self.inner.with_skip_taskbar(skip);
    self
  }

  fn has_icon(&self) -> bool {
    self.inner.window.window_icon.is_some()
  }

  #[cfg(feature = "menu")]
  fn has_menu(&self) -> bool {
    self.inner.window.window_menu.is_some()
  }
}

pub struct RpcRequestWrapper(WryRpcRequest);

impl From<RpcRequestWrapper> for RpcRequest {
  fn from(request: RpcRequestWrapper) -> Self {
    Self {
      command: request.0.method,
      params: request.0.params,
    }
  }
}

pub struct FileDropEventWrapper(WryFileDropEvent);

impl From<FileDropEventWrapper> for FileDropEvent {
  fn from(event: FileDropEventWrapper) -> Self {
    match event.0 {
      WryFileDropEvent::Hovered(paths) => FileDropEvent::Hovered(paths),
      WryFileDropEvent::Dropped(paths) => FileDropEvent::Dropped(paths),
      WryFileDropEvent::Cancelled => FileDropEvent::Cancelled,
    }
  }
}

#[cfg(windows)]
struct Hwnd(*mut std::ffi::c_void);
#[cfg(windows)]
unsafe impl Send for Hwnd {}

#[derive(Debug, Clone)]
enum WindowMessage {
  // Getters
  ScaleFactor(Sender<f64>),
  InnerPosition(Sender<Result<PhysicalPosition<i32>>>),
  OuterPosition(Sender<Result<PhysicalPosition<i32>>>),
  InnerSize(Sender<PhysicalSize<u32>>),
  OuterSize(Sender<PhysicalSize<u32>>),
  IsFullscreen(Sender<bool>),
  IsMaximized(Sender<bool>),
  IsDecorated(Sender<bool>),
  IsResizable(Sender<bool>),
  IsVisible(Sender<bool>),
  CurrentMonitor(Sender<Option<MonitorHandle>>),
  PrimaryMonitor(Sender<Option<MonitorHandle>>),
  AvailableMonitors(Sender<Vec<MonitorHandle>>),
  #[cfg(windows)]
  Hwnd(Sender<Hwnd>),
  // Setters
  Center(Sender<Result<()>>),
  SetResizable(bool),
  SetTitle(String),
  Maximize,
  Unmaximize,
  Minimize,
  Unminimize,
  Show,
  Hide,
  Close,
  SetDecorations(bool),
  SetAlwaysOnTop(bool),
  SetSize(Size),
  SetMinSize(Option<Size>),
  SetMaxSize(Option<Size>),
  SetPosition(Position),
  SetFullscreen(bool),
  SetFocus,
  SetIcon(WindowIcon),
  SetSkipTaskbar(bool),
  DragWindow,
  #[cfg(feature = "menu")]
  UpdateMenuItem(u32, menu::MenuUpdate),
}

#[derive(Debug, Clone)]
enum WebviewMessage {
  EvaluateScript(String),
  Print,
}

#[cfg(feature = "system-tray")]
#[derive(Clone)]
pub(crate) enum TrayMessage {
  UpdateItem(u32, menu::MenuUpdate),
  UpdateIcon(Icon),
  #[cfg(windows)]
  Remove,
}

#[derive(Clone)]
pub(crate) enum Message {
  Task(MainTask),
  Window(WindowId, WindowMessage),
  Webview(WindowId, WebviewMessage),
  #[cfg(feature = "system-tray")]
  Tray(TrayMessage),
  CreateWebview(Arc<Mutex<Option<CreateWebviewHandler>>>, Sender<WindowId>),
}

#[derive(Clone)]
struct DispatcherContext {
  proxy: EventLoopProxy<Message>,
  window_event_listeners: WindowEventListeners,
  #[cfg(feature = "menu")]
  menu_event_listeners: MenuEventListeners,
}

/// The Tauri [`Dispatch`] for [`Wry`].
#[derive(Clone)]
pub struct WryDispatcher {
  window_id: WindowId,
  context: DispatcherContext,
}

macro_rules! dispatcher_getter {
  ($self: ident, $message: expr) => {{
    let (tx, rx) = channel();
    $self
      .context
      .proxy
      .send_event(Message::Window($self.window_id, $message(tx)))
      .map_err(|_| Error::FailedToSendMessage)?;
    rx.recv().unwrap()
  }};
}

impl Dispatch for WryDispatcher {
  type Runtime = Wry;
  type WindowBuilder = WindowBuilderWrapper;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Task(Arc::new(Mutex::new(Some(Box::new(f))))))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) -> Uuid {
    let id = Uuid::new_v4();
    self
      .context
      .window_event_listeners
      .lock()
      .unwrap()
      .insert(id, Box::new(f));
    id
  }

  #[cfg(feature = "menu")]
  fn on_menu_event<F: Fn(&MenuEvent) + Send + 'static>(&self, f: F) -> Uuid {
    let id = Uuid::new_v4();
    self
      .context
      .menu_event_listeners
      .lock()
      .unwrap()
      .insert(id, Box::new(f));
    id
  }

  // Getters

  fn scale_factor(&self) -> Result<f64> {
    Ok(dispatcher_getter!(self, WindowMessage::ScaleFactor))
  }

  fn inner_position(&self) -> Result<PhysicalPosition<i32>> {
    dispatcher_getter!(self, WindowMessage::InnerPosition)
  }

  fn outer_position(&self) -> Result<PhysicalPosition<i32>> {
    dispatcher_getter!(self, WindowMessage::OuterPosition)
  }

  fn inner_size(&self) -> Result<PhysicalSize<u32>> {
    Ok(dispatcher_getter!(self, WindowMessage::InnerSize))
  }

  fn outer_size(&self) -> Result<PhysicalSize<u32>> {
    Ok(dispatcher_getter!(self, WindowMessage::OuterSize))
  }

  fn is_fullscreen(&self) -> Result<bool> {
    Ok(dispatcher_getter!(self, WindowMessage::IsFullscreen))
  }

  fn is_maximized(&self) -> Result<bool> {
    Ok(dispatcher_getter!(self, WindowMessage::IsMaximized))
  }

  /// Gets the window’s current decoration state.
  fn is_decorated(&self) -> Result<bool> {
    Ok(dispatcher_getter!(self, WindowMessage::IsDecorated))
  }

  /// Gets the window’s current resizable state.
  fn is_resizable(&self) -> Result<bool> {
    Ok(dispatcher_getter!(self, WindowMessage::IsResizable))
  }

  fn is_visible(&self) -> Result<bool> {
    Ok(dispatcher_getter!(self, WindowMessage::IsVisible))
  }

  fn current_monitor(&self) -> Result<Option<Monitor>> {
    Ok(
      dispatcher_getter!(self, WindowMessage::CurrentMonitor)
        .map(|m| MonitorHandleWrapper(m).into()),
    )
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    Ok(
      dispatcher_getter!(self, WindowMessage::PrimaryMonitor)
        .map(|m| MonitorHandleWrapper(m).into()),
    )
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    Ok(
      dispatcher_getter!(self, WindowMessage::AvailableMonitors)
        .into_iter()
        .map(|m| MonitorHandleWrapper(m).into())
        .collect(),
    )
  }

  #[cfg(windows)]
  fn hwnd(&self) -> Result<*mut std::ffi::c_void> {
    Ok(dispatcher_getter!(self, WindowMessage::Hwnd).0)
  }

  // Setters

  fn center(&self) -> Result<()> {
    dispatcher_getter!(self, WindowMessage::Center)
  }

  fn print(&self) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Webview(self.window_id, WebviewMessage::Print))
      .map_err(|_| Error::FailedToSendMessage)
  }

  // Creates a window by dispatching a message to the event loop.
  // Note that this must be called from a separate thread, otherwise the channel will introduce a deadlock.
  fn create_window<P: Params<Runtime = Self::Runtime>>(
    &mut self,
    pending: PendingWindow<P>,
  ) -> Result<DetachedWindow<P>> {
    let (tx, rx) = channel();
    let label = pending.label.clone();
    let context = self.context.clone();
    self
      .context
      .proxy
      .send_event(Message::CreateWebview(
        Arc::new(Mutex::new(Some(Box::new(move |event_loop| {
          create_webview(event_loop, context, pending)
        })))),
        tx,
      ))
      .map_err(|_| Error::FailedToSendMessage)?;
    let window_id = rx.recv().unwrap();
    let dispatcher = WryDispatcher {
      window_id,
      context: self.context.clone(),
    };
    Ok(DetachedWindow { label, dispatcher })
  }

  fn set_resizable(&self, resizable: bool) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetResizable(resizable),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_title<S: Into<String>>(&self, title: S) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetTitle(title.into()),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn maximize(&self) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Maximize))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn unmaximize(&self) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Unmaximize))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn minimize(&self) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Minimize))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn unminimize(&self) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Unminimize))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn show(&self) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Show))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn hide(&self) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Hide))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn close(&self) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Close))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_decorations(&self, decorations: bool) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetDecorations(decorations),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_always_on_top(&self, always_on_top: bool) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetAlwaysOnTop(always_on_top),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_size(&self, size: Size) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetSize(size),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_min_size(&self, size: Option<Size>) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetMinSize(size),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_max_size(&self, size: Option<Size>) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetMaxSize(size),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_position(&self, position: Position) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetPosition(position),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetFullscreen(fullscreen),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_focus(&self) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::SetFocus))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_icon(&self, icon: Icon) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetIcon(WryIcon::try_from(icon)?.0),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_skip_taskbar(&self, skip: bool) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetSkipTaskbar(skip),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn start_dragging(&self) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::DragWindow))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn eval_script<S: Into<String>>(&self, script: S) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Webview(
        self.window_id,
        WebviewMessage::EvaluateScript(script.into()),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  #[cfg(feature = "menu")]
  fn update_menu_item(&self, id: u32, update: menu::MenuUpdate) -> Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::UpdateMenuItem(id, update),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }
}

#[cfg(feature = "system-tray")]
#[derive(Clone, Default)]
struct TrayContext {
  tray: Arc<Mutex<Option<Arc<Mutex<WrySystemTray>>>>>,
  listeners: SystemTrayEventListeners,
  items: SystemTrayItems,
}

struct WebviewWrapper {
  inner: WebView,
  #[cfg(feature = "menu")]
  menu_items: HashMap<u32, WryCustomMenuItem>,
}

/// A Tauri [`Runtime`] wrapper around wry.
pub struct Wry {
  event_loop: EventLoop<Message>,
  webviews: Arc<Mutex<HashMap<WindowId, WebviewWrapper>>>,
  window_event_listeners: WindowEventListeners,
  #[cfg(feature = "menu")]
  menu_event_listeners: MenuEventListeners,
  #[cfg(feature = "system-tray")]
  tray_context: TrayContext,
}

/// A handle to the Wry runtime.
#[derive(Clone)]
pub struct WryHandle {
  dispatcher_context: DispatcherContext,
}

impl RuntimeHandle for WryHandle {
  type Runtime = Wry;

  // Creates a window by dispatching a message to the event loop.
  // Note that this must be called from a separate thread, otherwise the channel will introduce a deadlock.
  fn create_window<P: Params<Runtime = Self::Runtime>>(
    &self,
    pending: PendingWindow<P>,
  ) -> Result<DetachedWindow<P>> {
    let (tx, rx) = channel();
    let label = pending.label.clone();
    let dispatcher_context = self.dispatcher_context.clone();
    self
      .dispatcher_context
      .proxy
      .send_event(Message::CreateWebview(
        Arc::new(Mutex::new(Some(Box::new(move |event_loop| {
          create_webview(event_loop, dispatcher_context, pending)
        })))),
        tx,
      ))
      .map_err(|_| Error::FailedToSendMessage)?;
    let window_id = rx.recv().unwrap();
    let dispatcher = WryDispatcher {
      window_id,
      context: self.dispatcher_context.clone(),
    };
    Ok(DetachedWindow { label, dispatcher })
  }

  #[cfg(all(windows, feature = "system-tray"))]
  fn remove_system_tray(&self) -> Result<()> {
    self
      .dispatcher_context
      .proxy
      .send_event(Message::Tray(TrayMessage::Remove))
      .map_err(|_| Error::FailedToSendMessage)
  }
}

impl Runtime for Wry {
  type Dispatcher = WryDispatcher;
  type Handle = WryHandle;
  #[cfg(feature = "system-tray")]
  type TrayHandler = SystemTrayHandle;

  fn new() -> Result<Self> {
    let event_loop = EventLoop::<Message>::with_user_event();
    Ok(Self {
      event_loop,
      webviews: Default::default(),
      window_event_listeners: Default::default(),
      #[cfg(feature = "menu")]
      menu_event_listeners: Default::default(),
      #[cfg(feature = "system-tray")]
      tray_context: Default::default(),
    })
  }

  fn handle(&self) -> Self::Handle {
    WryHandle {
      dispatcher_context: DispatcherContext {
        proxy: self.event_loop.create_proxy(),
        window_event_listeners: self.window_event_listeners.clone(),
        #[cfg(feature = "menu")]
        menu_event_listeners: self.menu_event_listeners.clone(),
      },
    }
  }

  fn create_window<P: Params<Runtime = Self>>(
    &self,
    pending: PendingWindow<P>,
  ) -> Result<DetachedWindow<P>> {
    let label = pending.label.clone();
    let proxy = self.event_loop.create_proxy();
    let webview = create_webview(
      &self.event_loop,
      DispatcherContext {
        proxy: proxy.clone(),
        window_event_listeners: self.window_event_listeners.clone(),
        #[cfg(feature = "menu")]
        menu_event_listeners: self.menu_event_listeners.clone(),
      },
      pending,
    )?;

    let dispatcher = WryDispatcher {
      window_id: webview.inner.window().id(),
      context: DispatcherContext {
        proxy,
        window_event_listeners: self.window_event_listeners.clone(),
        #[cfg(feature = "menu")]
        menu_event_listeners: self.menu_event_listeners.clone(),
      },
    };

    self
      .webviews
      .lock()
      .unwrap()
      .insert(webview.inner.window().id(), webview);

    Ok(DetachedWindow { label, dispatcher })
  }

  #[cfg(feature = "system-tray")]
  fn system_tray<I: MenuId>(&self, system_tray: SystemTray<I>) -> Result<Self::TrayHandler> {
    let icon = system_tray
      .icon
      .expect("tray icon not set")
      .into_tray_icon();

    let mut items = HashMap::new();

    let tray = SystemTrayBuilder::new(
      icon,
      system_tray
        .menu
        .map(|menu| to_wry_context_menu(&mut items, menu)),
    )
    .build(&self.event_loop)
    .map_err(|e| Error::SystemTray(Box::new(e)))?;

    *self.tray_context.items.lock().unwrap() = items;
    *self.tray_context.tray.lock().unwrap() = Some(Arc::new(Mutex::new(tray)));

    Ok(SystemTrayHandle {
      proxy: self.event_loop.create_proxy(),
    })
  }

  #[cfg(feature = "system-tray")]
  fn on_system_tray_event<F: Fn(&SystemTrayEvent) + Send + 'static>(&mut self, f: F) -> Uuid {
    let id = Uuid::new_v4();
    self
      .tray_context
      .listeners
      .lock()
      .unwrap()
      .insert(id, Box::new(f));
    id
  }

  #[cfg(any(target_os = "windows", target_os = "macos"))]
  fn run_iteration(&mut self) -> RunIteration {
    use wry::application::platform::run_return::EventLoopExtRunReturn;
    let webviews = self.webviews.clone();
    let window_event_listeners = self.window_event_listeners.clone();
    #[cfg(feature = "menu")]
    let menu_event_listeners = self.menu_event_listeners.clone();
    #[cfg(feature = "system-tray")]
    let tray_context = self.tray_context.clone();

    let mut iteration = RunIteration::default();

    self
      .event_loop
      .run_return(|event, event_loop, control_flow| {
        if let Event::MainEventsCleared = &event {
          *control_flow = ControlFlow::Exit;
        }
        iteration = handle_event_loop(
          event,
          event_loop,
          control_flow,
          EventLoopIterationContext {
            callback: None,
            webviews: webviews.lock().expect("poisoned webview collection"),
            window_event_listeners: window_event_listeners.clone(),
            #[cfg(feature = "menu")]
            menu_event_listeners: menu_event_listeners.clone(),
            #[cfg(feature = "system-tray")]
            tray_context: tray_context.clone(),
          },
        );
      });

    iteration
  }

  fn run<F: Fn() + 'static>(self, callback: F) {
    let webviews = self.webviews.clone();
    let window_event_listeners = self.window_event_listeners.clone();
    #[cfg(feature = "menu")]
    let menu_event_listeners = self.menu_event_listeners.clone();
    #[cfg(feature = "system-tray")]
    let tray_context = self.tray_context;

    self.event_loop.run(move |event, event_loop, control_flow| {
      handle_event_loop(
        event,
        event_loop,
        control_flow,
        EventLoopIterationContext {
          callback: Some(&callback),
          webviews: webviews.lock().expect("poisoned webview collection"),
          window_event_listeners: window_event_listeners.clone(),
          #[cfg(feature = "menu")]
          menu_event_listeners: menu_event_listeners.clone(),
          #[cfg(feature = "system-tray")]
          tray_context: tray_context.clone(),
        },
      );
    })
  }
}

struct EventLoopIterationContext<'a> {
  callback: Option<&'a (dyn Fn() + 'static)>,
  webviews: MutexGuard<'a, HashMap<WindowId, WebviewWrapper>>,
  window_event_listeners: WindowEventListeners,
  #[cfg(feature = "menu")]
  menu_event_listeners: MenuEventListeners,
  #[cfg(feature = "system-tray")]
  tray_context: TrayContext,
}

fn handle_event_loop(
  event: Event<Message>,
  event_loop: &EventLoopWindowTarget<Message>,
  control_flow: &mut ControlFlow,
  context: EventLoopIterationContext<'_>,
) -> RunIteration {
  let EventLoopIterationContext {
    callback,
    mut webviews,
    window_event_listeners,
    #[cfg(feature = "menu")]
    menu_event_listeners,
    #[cfg(feature = "system-tray")]
    tray_context,
  } = context;
  *control_flow = ControlFlow::Wait;

  for (_, w) in webviews.iter() {
    if let Err(e) = w.inner.evaluate_script() {
      eprintln!("{}", e);
    }
  }

  match event {
    #[cfg(feature = "menu")]
    Event::MenuEvent {
      menu_id,
      origin: MenuType::MenuBar,
    } => {
      let event = MenuEvent {
        menu_item_id: menu_id.0,
      };
      for handler in menu_event_listeners.lock().unwrap().values() {
        handler(&event);
      }
    }
    #[cfg(feature = "system-tray")]
    Event::MenuEvent {
      menu_id,
      origin: MenuType::ContextMenu,
    } => {
      let event = SystemTrayEvent::MenuItemClick(menu_id.0);
      for handler in tray_context.listeners.lock().unwrap().values() {
        handler(&event);
      }
    }
    #[cfg(feature = "system-tray")]
    Event::TrayEvent {
      bounds,
      event,
      position: _cursor_position,
    } => {
      let (position, size) = (
        PhysicalPositionWrapper(bounds.position).into(),
        PhysicalSizeWrapper(bounds.size).into(),
      );
      let event = match event {
        TrayEvent::LeftClick => SystemTrayEvent::LeftClick { position, size },
        TrayEvent::RightClick => SystemTrayEvent::RightClick { position, size },
        TrayEvent::DoubleClick => SystemTrayEvent::DoubleClick { position, size },
      };
      for handler in tray_context.listeners.lock().unwrap().values() {
        handler(&event);
      }
    }
    Event::WindowEvent { event, window_id } => {
      if let Some(event) = WindowEventWrapper::from(&event).0 {
        for handler in window_event_listeners.lock().unwrap().values() {
          handler(&event);
        }
      }
      match event {
        WryWindowEvent::CloseRequested => {
          webviews.remove(&window_id);
          if webviews.is_empty() {
            *control_flow = ControlFlow::Exit;
            if let Some(callback) = callback {
              callback();
            }
          }
        }
        WryWindowEvent::Resized(_) => {
          if let Err(e) = webviews[&window_id].inner.resize() {
            eprintln!("{}", e);
          }
        }
        _ => {}
      }
    }
    Event::UserEvent(message) => match message {
      Message::Task(task) => {
        if let Some(task) = task.lock().unwrap().take() {
          task();
        }
      }
      Message::Window(id, window_message) => {
        if let Some(webview) = webviews.get_mut(&id) {
          let window = webview.inner.window();
          match window_message {
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
              .send(PhysicalSizeWrapper(window.inner_size()).into())
              .unwrap(),
            WindowMessage::OuterSize(tx) => tx
              .send(PhysicalSizeWrapper(window.outer_size()).into())
              .unwrap(),
            WindowMessage::IsFullscreen(tx) => tx.send(window.fullscreen().is_some()).unwrap(),
            WindowMessage::IsMaximized(tx) => tx.send(window.is_maximized()).unwrap(),
            WindowMessage::IsDecorated(tx) => tx.send(window.is_decorated()).unwrap(),
            WindowMessage::IsResizable(tx) => tx.send(window.is_resizable()).unwrap(),
            WindowMessage::IsVisible(tx) => tx.send(window.is_visible()).unwrap(),
            WindowMessage::CurrentMonitor(tx) => tx.send(window.current_monitor()).unwrap(),
            WindowMessage::PrimaryMonitor(tx) => tx.send(window.primary_monitor()).unwrap(),
            WindowMessage::AvailableMonitors(tx) => {
              tx.send(window.available_monitors().collect()).unwrap()
            }
            #[cfg(windows)]
            WindowMessage::Hwnd(tx) => {
              use wry::application::platform::windows::WindowExtWindows;
              tx.send(Hwnd(window.hwnd())).unwrap()
            }
            // Setters
            WindowMessage::Center(tx) => {
              tx.send(center_window(window)).unwrap();
            }
            WindowMessage::SetResizable(resizable) => window.set_resizable(resizable),
            WindowMessage::SetTitle(title) => window.set_title(&title),
            WindowMessage::Maximize => window.set_maximized(true),
            WindowMessage::Unmaximize => window.set_maximized(false),
            WindowMessage::Minimize => window.set_minimized(true),
            WindowMessage::Unminimize => window.set_minimized(false),
            WindowMessage::Show => window.set_visible(true),
            WindowMessage::Hide => window.set_visible(false),
            WindowMessage::Close => {
              webviews.remove(&id);
              if webviews.is_empty() {
                *control_flow = ControlFlow::Exit;
              }
            }
            WindowMessage::SetDecorations(decorations) => window.set_decorations(decorations),
            WindowMessage::SetAlwaysOnTop(always_on_top) => window.set_always_on_top(always_on_top),
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
            }
            WindowMessage::SetFocus => {
              window.set_focus();
            }
            WindowMessage::SetIcon(icon) => {
              window.set_window_icon(Some(icon));
            }
            WindowMessage::SetSkipTaskbar(skip) => {
              window.set_skip_taskbar(skip);
            }
            WindowMessage::DragWindow => {
              let _ = window.drag_window();
            }
            #[cfg(feature = "menu")]
            WindowMessage::UpdateMenuItem(id, update) => {
              let item = webview
                .menu_items
                .get_mut(&id)
                .expect("menu item not found");
              match update {
                MenuUpdate::SetEnabled(enabled) => item.set_enabled(enabled),
                MenuUpdate::SetTitle(title) => item.set_title(&title),
                MenuUpdate::SetSelected(selected) => item.set_selected(selected),
                #[cfg(target_os = "macos")]
                MenuUpdate::SetNativeImage(image) => {
                  item.set_native_image(NativeImageWrapper::from(image).0)
                }
              }
            }
          }
        }
      }
      Message::Webview(id, webview_message) => {
        if let Some(webview) = webviews.get_mut(&id) {
          match webview_message {
            WebviewMessage::EvaluateScript(script) => {
              let _ = webview.inner.dispatch_script(&script);
            }
            WebviewMessage::Print => {
              let _ = webview.inner.print();
            }
          }
        }
      }
      Message::CreateWebview(handler, sender) => {
        let handler = {
          let mut lock = handler.lock().expect("poisoned create webview handler");
          std::mem::take(&mut *lock).unwrap()
        };
        match handler(event_loop) {
          Ok(webview) => {
            let window_id = webview.inner.window().id();
            webviews.insert(window_id, webview);
            sender.send(window_id).unwrap();
          }
          Err(e) => {
            eprintln!("{}", e);
          }
        }
      }
      #[cfg(feature = "system-tray")]
      Message::Tray(tray_message) => match tray_message {
        TrayMessage::UpdateItem(menu_id, update) => {
          let mut tray = tray_context.items.as_ref().lock().unwrap();
          let item = tray.get_mut(&menu_id).expect("menu item not found");
          match update {
            MenuUpdate::SetEnabled(enabled) => item.set_enabled(enabled),
            MenuUpdate::SetTitle(title) => item.set_title(&title),
            MenuUpdate::SetSelected(selected) => item.set_selected(selected),
            #[cfg(target_os = "macos")]
            MenuUpdate::SetNativeImage(image) => {
              item.set_native_image(NativeImageWrapper::from(image).0)
            }
          }
        }
        TrayMessage::UpdateIcon(icon) => {
          if let Some(tray) = &*tray_context.tray.lock().unwrap() {
            tray.lock().unwrap().set_icon(icon.into_tray_icon());
          }
        }
        #[cfg(windows)]
        TrayMessage::Remove => {
          if let Some(tray) = tray_context.tray.lock().unwrap().as_ref() {
            use wry::application::platform::windows::SystemTrayExtWindows;
            tray.lock().unwrap().remove();
          }
        }
      },
    },
    _ => (),
  }

  RunIteration {
    webview_count: webviews.len(),
  }
}

fn center_window(window: &Window) -> Result<()> {
  if let Some(monitor) = window.current_monitor() {
    let screen_size = monitor.size();
    let window_size = window.inner_size();
    let x = (screen_size.width - window_size.width) / 2;
    let y = (screen_size.height - window_size.height) / 2;
    window.set_outer_position(WryPhysicalPosition::new(x, y));
    Ok(())
  } else {
    Err(Error::FailedToGetMonitor)
  }
}

fn create_webview<P: Params<Runtime = Wry>>(
  event_loop: &EventLoopWindowTarget<Message>,
  context: DispatcherContext,
  pending: PendingWindow<P>,
) -> Result<WebviewWrapper> {
  let PendingWindow {
    webview_attributes,
    window_builder,
    rpc_handler,
    file_drop_handler,
    label,
    url,
    ..
  } = pending;

  let is_window_transparent = window_builder.inner.window.transparent;
  #[cfg(feature = "menu")]
  let menu_items = window_builder.menu_items;
  let window = window_builder.inner.build(event_loop).unwrap();
  if window_builder.center {
    let _ = center_window(&window);
  }
  let mut webview_builder = WebViewBuilder::new(window)
    .map_err(|e| Error::CreateWebview(Box::new(e)))?
    .with_url(&url)
    .unwrap() // safe to unwrap because we validate the URL beforehand
    .with_transparent(is_window_transparent);
  if let Some(handler) = rpc_handler {
    webview_builder =
      webview_builder.with_rpc_handler(create_rpc_handler(context.clone(), label.clone(), handler));
  }
  if let Some(handler) = file_drop_handler {
    webview_builder =
      webview_builder.with_file_drop_handler(create_file_drop_handler(context, label, handler));
  }
  for (scheme, protocol) in webview_attributes.uri_scheme_protocols {
    webview_builder = webview_builder.with_custom_protocol(scheme, move |_window, url| {
      protocol(url).map_err(|_| wry::Error::InitScriptError)
    });
  }
  if let Some(data_directory) = webview_attributes.data_directory {
    webview_builder = webview_builder.with_data_directory(data_directory);
  }
  for script in webview_attributes.initialization_scripts {
    webview_builder = webview_builder.with_initialization_script(&script);
  }

  let webview = webview_builder
    .build()
    .map_err(|e| Error::CreateWebview(Box::new(e)))?;

  Ok(WebviewWrapper {
    inner: webview,
    #[cfg(feature = "menu")]
    menu_items,
  })
}

/// Create a wry rpc handler from a tauri rpc handler.
fn create_rpc_handler<P: Params<Runtime = Wry>>(
  context: DispatcherContext,
  label: P::Label,
  handler: WebviewRpcHandler<P>,
) -> Box<dyn Fn(&Window, WryRpcRequest) -> Option<RpcResponse> + 'static> {
  Box::new(move |window, request| {
    handler(
      DetachedWindow {
        dispatcher: WryDispatcher {
          window_id: window.id(),
          context: context.clone(),
        },
        label: label.clone(),
      },
      RpcRequestWrapper(request).into(),
    );
    None
  })
}

/// Create a wry file drop handler from a tauri file drop handler.
fn create_file_drop_handler<P: Params<Runtime = Wry>>(
  context: DispatcherContext,
  label: P::Label,
  handler: FileDropHandler<P>,
) -> Box<dyn Fn(&Window, WryFileDropEvent) -> bool + 'static> {
  Box::new(move |window, event| {
    handler(
      FileDropEventWrapper(event).into(),
      DetachedWindow {
        dispatcher: WryDispatcher {
          window_id: window.id(),
          context: context.clone(),
        },
        label: label.clone(),
      },
    )
  })
}
