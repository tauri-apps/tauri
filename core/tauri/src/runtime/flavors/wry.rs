// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The [`wry`] Tauri [`Runtime`].

use crate::{
  api::config::WindowConfig,
  runtime::{
    webview::{
      CustomMenuItem, FileDropEvent, FileDropHandler, Menu, MenuItem, MenuItemId, RpcRequest,
      TrayMenuItem, WebviewRpcHandler, WindowBuilder, WindowBuilderBase,
    },
    window::{
      dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size},
      DetachedWindow, MenuEvent, PendingWindow, WindowEvent,
    },
    Dispatch, Monitor, Params, Runtime,
  },
  Icon,
};

use image::{GenericImageView, Pixel};
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
    menu::{
      CustomMenu as WryCustomMenu, Menu as WryMenu, MenuId as WryMenuId, MenuItem as WryMenuItem,
      MenuType,
    },
    monitor::MonitorHandle,
    platform::system_tray::SystemTrayBuilder,
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
  sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
  },
};

type CreateWebviewHandler =
  Box<dyn FnOnce(&EventLoopWindowTarget<Message>) -> crate::Result<WebView> + Send>;
type MainThreadTask = Box<dyn FnOnce() + Send>;
type WindowEventHandler = Box<dyn Fn(&WindowEvent) + Send>;
type WindowEventListeners = Arc<Mutex<HashMap<Uuid, WindowEventHandler>>>;
type MenuEventHandler = Box<dyn Fn(&MenuEvent) + Send>;
type MenuEventListeners = Arc<Mutex<HashMap<Uuid, MenuEventHandler>>>;

#[repr(C)]
#[derive(Debug)]
struct PixelValue {
  r: u8,
  g: u8,
  b: u8,
  a: u8,
}

const PIXEL_SIZE: usize = std::mem::size_of::<PixelValue>();

/// Wrapper around a [`wry::application::window::Icon`] that can be created from an [`Icon`].
pub struct WryIcon(WindowIcon);

impl TryFrom<Icon> for WryIcon {
  type Error = crate::Error;
  fn try_from(icon: Icon) -> Result<Self, Self::Error> {
    let image = match icon {
      Icon::File(path) => image::open(path).map_err(|e| crate::Error::InvalidIcon(Box::new(e)))?,
      Icon::Raw(raw) => {
        image::load_from_memory(&raw).map_err(|e| crate::Error::InvalidIcon(Box::new(e)))?
      }
    };
    let (width, height) = image.dimensions();
    let mut rgba = Vec::with_capacity((width * height) as usize * PIXEL_SIZE);
    for (_, _, pixel) in image.pixels() {
      rgba.extend_from_slice(&pixel.to_rgba().0);
    }
    let icon = WindowIcon::from_rgba(rgba, width, height)
      .map_err(|e| crate::Error::InvalidIcon(Box::new(e)))?;
    Ok(Self(icon))
  }
}

struct WindowEventWrapper(Option<WindowEvent>);

impl<'a> From<&WryWindowEvent<'a>> for WindowEventWrapper {
  fn from(event: &WryWindowEvent<'a>) -> Self {
    let event = match event {
      WryWindowEvent::Resized(size) => WindowEvent::Resized((*size).into()),
      WryWindowEvent::Moved(position) => WindowEvent::Moved((*position).into()),
      WryWindowEvent::CloseRequested => WindowEvent::CloseRequested,
      WryWindowEvent::Destroyed => WindowEvent::Destroyed,
      WryWindowEvent::Focused(focused) => WindowEvent::Focused(*focused),
      WryWindowEvent::ScaleFactorChanged {
        scale_factor,
        new_inner_size,
      } => WindowEvent::ScaleFactorChanged {
        scale_factor: *scale_factor,
        new_inner_size: (**new_inner_size).into(),
      },
      _ => return Self(None),
    };
    Self(Some(event))
  }
}

impl From<MonitorHandle> for Monitor {
  fn from(monitor: MonitorHandle) -> Monitor {
    Self {
      name: monitor.name(),
      position: monitor.position().into(),
      size: monitor.size().into(),
      scale_factor: monitor.scale_factor(),
    }
  }
}

impl<T> From<WryPhysicalPosition<T>> for PhysicalPosition<T> {
  fn from(position: WryPhysicalPosition<T>) -> Self {
    Self {
      x: position.x,
      y: position.y,
    }
  }
}

impl<T> From<PhysicalPosition<T>> for WryPhysicalPosition<T> {
  fn from(position: PhysicalPosition<T>) -> Self {
    Self {
      x: position.x,
      y: position.y,
    }
  }
}

impl<T> From<LogicalPosition<T>> for WryLogicalPosition<T> {
  fn from(position: LogicalPosition<T>) -> Self {
    Self {
      x: position.x,
      y: position.y,
    }
  }
}

impl<T> From<WryPhysicalSize<T>> for PhysicalSize<T> {
  fn from(size: WryPhysicalSize<T>) -> Self {
    Self {
      width: size.width,
      height: size.height,
    }
  }
}

impl<T> From<PhysicalSize<T>> for WryPhysicalSize<T> {
  fn from(size: PhysicalSize<T>) -> Self {
    Self {
      width: size.width,
      height: size.height,
    }
  }
}

impl<T> From<LogicalSize<T>> for WryLogicalSize<T> {
  fn from(size: LogicalSize<T>) -> Self {
    Self {
      width: size.width,
      height: size.height,
    }
  }
}

impl From<Size> for WrySize {
  fn from(size: Size) -> Self {
    match size {
      Size::Logical(s) => Self::Logical(s.into()),
      Size::Physical(s) => Self::Physical(s.into()),
    }
  }
}

impl From<Position> for WryPosition {
  fn from(position: Position) -> Self {
    match position {
      Position::Logical(s) => Self::Logical(s.into()),
      Position::Physical(s) => Self::Physical(s.into()),
    }
  }
}

impl From<CustomMenuItem> for WryCustomMenu {
  fn from(item: CustomMenuItem) -> Self {
    Self {
      id: WryMenuId(item.id.0),
      name: item.name,
      keyboard_accelerators: None,
    }
  }
}

impl From<MenuItem> for WryMenuItem {
  fn from(item: MenuItem) -> Self {
    match item {
      MenuItem::Custom(custom) => Self::Custom(custom.into()),
      MenuItem::About(v) => Self::About(v),
      MenuItem::Hide => Self::Hide,
      MenuItem::Services => Self::Services,
      MenuItem::HideOthers => Self::HideOthers,
      MenuItem::ShowAll => Self::ShowAll,
      MenuItem::CloseWindow => Self::CloseWindow,
      MenuItem::Quit => Self::Quit,
      MenuItem::Copy => Self::Copy,
      MenuItem::Cut => Self::Cut,
      MenuItem::Undo => Self::Undo,
      MenuItem::Redo => Self::Redo,
      MenuItem::SelectAll => Self::SelectAll,
      MenuItem::Paste => Self::Paste,
      MenuItem::EnterFullScreen => Self::EnterFullScreen,
      MenuItem::Minimize => Self::Minimize,
      MenuItem::Zoom => Self::Zoom,
      MenuItem::Separator => Self::Separator,
    }
  }
}

impl From<Menu> for WryMenu {
  fn from(menu: Menu) -> Self {
    Self {
      title: menu.title,
      items: menu.items.into_iter().map(Into::into).collect(),
    }
  }
}

impl From<TrayMenuItem> for WryMenuItem {
  fn from(item: TrayMenuItem) -> Self {
    match item {
      TrayMenuItem::Custom(custom) => Self::Custom(custom.into()),
      TrayMenuItem::Separator => Self::Separator,
    }
  }
}

impl WindowBuilderBase for WryWindowBuilder {}
impl WindowBuilder for WryWindowBuilder {
  fn new() -> Self {
    Default::default()
  }

  fn with_config(config: WindowConfig) -> Self {
    let mut window = WryWindowBuilder::new()
      .title(config.title.to_string())
      .inner_size(config.width, config.height)
      .visible(config.visible)
      .resizable(config.resizable)
      .decorations(config.decorations)
      .maximized(config.maximized)
      .fullscreen(config.fullscreen)
      .transparent(config.transparent)
      .always_on_top(config.always_on_top);

    if let (Some(min_width), Some(min_height)) = (config.min_width, config.min_height) {
      window = window.min_inner_size(min_width, min_height);
    }
    if let (Some(max_width), Some(max_height)) = (config.max_width, config.max_height) {
      window = window.max_inner_size(max_width, max_height);
    }
    if let (Some(x), Some(y)) = (config.x, config.y) {
      window = window.position(x, y);
    }

    window
  }

  fn menu(self, menu: Vec<Menu>) -> Self {
    self.with_menu(menu.into_iter().map(Into::into).collect::<Vec<WryMenu>>())
  }

  fn position(self, x: f64, y: f64) -> Self {
    self.with_position(WryLogicalPosition::new(x, y))
  }

  fn inner_size(self, width: f64, height: f64) -> Self {
    self.with_inner_size(WryLogicalSize::new(width, height))
  }

  fn min_inner_size(self, min_width: f64, min_height: f64) -> Self {
    self.with_min_inner_size(WryLogicalSize::new(min_width, min_height))
  }

  fn max_inner_size(self, max_width: f64, max_height: f64) -> Self {
    self.with_max_inner_size(WryLogicalSize::new(max_width, max_height))
  }

  fn resizable(self, resizable: bool) -> Self {
    self.with_resizable(resizable)
  }

  fn title<S: Into<String>>(self, title: S) -> Self {
    self.with_title(title.into())
  }

  fn fullscreen(self, fullscreen: bool) -> Self {
    if fullscreen {
      self.with_fullscreen(Some(Fullscreen::Borderless(None)))
    } else {
      self.with_fullscreen(None)
    }
  }

  fn maximized(self, maximized: bool) -> Self {
    self.with_maximized(maximized)
  }

  fn visible(self, visible: bool) -> Self {
    self.with_visible(visible)
  }

  fn transparent(self, transparent: bool) -> Self {
    self.with_transparent(transparent)
  }

  fn decorations(self, decorations: bool) -> Self {
    self.with_decorations(decorations)
  }

  fn always_on_top(self, always_on_top: bool) -> Self {
    self.with_always_on_top(always_on_top)
  }

  fn icon(self, icon: Icon) -> crate::Result<Self> {
    Ok(self.with_window_icon(Some(WryIcon::try_from(icon)?.0)))
  }

  fn has_icon(&self) -> bool {
    self.window.window_icon.is_some()
  }

  fn has_menu(&self) -> bool {
    self.window.window_menu.is_some()
  }
}

impl From<WryRpcRequest> for RpcRequest {
  fn from(request: WryRpcRequest) -> Self {
    Self {
      command: request.method,
      params: request.params,
    }
  }
}

impl From<WryFileDropEvent> for FileDropEvent {
  fn from(event: WryFileDropEvent) -> Self {
    match event {
      WryFileDropEvent::Hovered(paths) => FileDropEvent::Hovered(paths),
      WryFileDropEvent::Dropped(paths) => FileDropEvent::Dropped(paths),
      WryFileDropEvent::Cancelled => FileDropEvent::Cancelled,
    }
  }
}

#[derive(Debug, Clone)]
enum WindowMessage {
  // Getters
  ScaleFactor(Sender<f64>),
  InnerPosition(Sender<crate::Result<PhysicalPosition<i32>>>),
  OuterPosition(Sender<crate::Result<PhysicalPosition<i32>>>),
  InnerSize(Sender<PhysicalSize<u32>>),
  OuterSize(Sender<PhysicalSize<u32>>),
  IsFullscreen(Sender<bool>),
  IsMaximized(Sender<bool>),
  CurrentMonitor(Sender<Option<MonitorHandle>>),
  PrimaryMonitor(Sender<Option<MonitorHandle>>),
  AvailableMonitors(Sender<Vec<MonitorHandle>>),
  // Setters
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
  SetIcon(WindowIcon),
  DragWindow,
}

#[derive(Debug, Clone)]
enum WebviewMessage {
  EvaluateScript(String),
  Print,
}

#[derive(Clone)]
enum Message {
  Window(WindowId, WindowMessage),
  Webview(WindowId, WebviewMessage),
  CreateWebview(Arc<Mutex<Option<CreateWebviewHandler>>>, Sender<WindowId>),
}

#[derive(Clone)]
struct DispatcherContext {
  proxy: EventLoopProxy<Message>,
  task_tx: Sender<MainThreadTask>,
  window_event_listeners: WindowEventListeners,
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
      .map_err(|_| crate::Error::FailedToSendMessage)?;
    rx.recv().unwrap()
  }};
}

macro_rules! window_result_getter {
  ($window: ident, $tx: ident, $call: ident) => {
    $tx
      .send(
        $window
          .$call()
          .map(Into::into)
          .map_err(|_| crate::Error::FailedToSendMessage),
      )
      .unwrap()
  };
}

macro_rules! window_getter {
  ($window: ident, $tx: ident, $call: ident) => {
    $tx.send($window.$call().into()).unwrap()
  };
}

impl Dispatch for WryDispatcher {
  type Runtime = Wry;
  type WindowBuilder = WryWindowBuilder;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> crate::Result<()> {
    self
      .context
      .task_tx
      .send(Box::new(f))
      .map_err(|_| crate::Error::FailedToSendMessage)
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

  fn scale_factor(&self) -> crate::Result<f64> {
    Ok(dispatcher_getter!(self, WindowMessage::ScaleFactor))
  }

  fn inner_position(&self) -> crate::Result<PhysicalPosition<i32>> {
    dispatcher_getter!(self, WindowMessage::InnerPosition)
  }

  fn outer_position(&self) -> crate::Result<PhysicalPosition<i32>> {
    dispatcher_getter!(self, WindowMessage::OuterPosition)
  }

  fn inner_size(&self) -> crate::Result<PhysicalSize<u32>> {
    Ok(dispatcher_getter!(self, WindowMessage::InnerSize))
  }

  fn outer_size(&self) -> crate::Result<PhysicalSize<u32>> {
    Ok(dispatcher_getter!(self, WindowMessage::OuterSize))
  }

  fn is_fullscreen(&self) -> crate::Result<bool> {
    Ok(dispatcher_getter!(self, WindowMessage::IsFullscreen))
  }

  fn is_maximized(&self) -> crate::Result<bool> {
    Ok(dispatcher_getter!(self, WindowMessage::IsMaximized))
  }

  fn current_monitor(&self) -> crate::Result<Option<Monitor>> {
    Ok(dispatcher_getter!(self, WindowMessage::CurrentMonitor).map(Into::into))
  }

  fn primary_monitor(&self) -> crate::Result<Option<Monitor>> {
    Ok(dispatcher_getter!(self, WindowMessage::PrimaryMonitor).map(Into::into))
  }

  fn available_monitors(&self) -> crate::Result<Vec<Monitor>> {
    Ok(
      dispatcher_getter!(self, WindowMessage::AvailableMonitors)
        .into_iter()
        .map(Into::into)
        .collect(),
    )
  }

  // Setters

  fn print(&self) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Webview(self.window_id, WebviewMessage::Print))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn create_window<M: Params<Runtime = Self::Runtime>>(
    &mut self,
    pending: PendingWindow<M>,
  ) -> crate::Result<DetachedWindow<M>> {
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
      .map_err(|_| crate::Error::FailedToSendMessage)?;
    let window_id = rx.recv().unwrap();
    let dispatcher = WryDispatcher {
      window_id,
      context: self.context.clone(),
    };
    Ok(DetachedWindow { label, dispatcher })
  }

  fn set_resizable(&self, resizable: bool) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetResizable(resizable),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_title<S: Into<String>>(&self, title: S) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetTitle(title.into()),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn maximize(&self) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Maximize))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn unmaximize(&self) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Unmaximize))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn minimize(&self) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Minimize))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn unminimize(&self) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Unminimize))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn show(&self) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Show))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn hide(&self) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Hide))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn close(&self) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Close))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_decorations(&self, decorations: bool) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetDecorations(decorations),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_always_on_top(&self, always_on_top: bool) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetAlwaysOnTop(always_on_top),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_size(&self, size: Size) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetSize(size),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_min_size(&self, size: Option<Size>) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetMinSize(size),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_max_size(&self, size: Option<Size>) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetMaxSize(size),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_position(&self, position: Position) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetPosition(position),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetFullscreen(fullscreen),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_icon(&self, icon: Icon) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetIcon(WryIcon::try_from(icon)?.0),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn start_dragging(&self) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::DragWindow))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn eval_script<S: Into<String>>(&self, script: S) -> crate::Result<()> {
    self
      .context
      .proxy
      .send_event(Message::Webview(
        self.window_id,
        WebviewMessage::EvaluateScript(script.into()),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }
}

/// A Tauri [`Runtime`] wrapper around wry.
pub struct Wry {
  event_loop: EventLoop<Message>,
  webviews: HashMap<WindowId, WebView>,
  task_tx: Sender<MainThreadTask>,
  window_event_listeners: WindowEventListeners,
  menu_event_listeners: MenuEventListeners,
  task_rx: Receiver<MainThreadTask>,
}

impl Runtime for Wry {
  type Dispatcher = WryDispatcher;

  fn new() -> crate::Result<Self> {
    let event_loop = EventLoop::<Message>::with_user_event();
    let (task_tx, task_rx) = channel();
    Ok(Self {
      event_loop,
      webviews: Default::default(),
      task_tx,
      task_rx,
      window_event_listeners: Default::default(),
      menu_event_listeners: Default::default(),
    })
  }

  fn create_window<M: Params<Runtime = Self>>(
    &mut self,
    pending: PendingWindow<M>,
  ) -> crate::Result<DetachedWindow<M>> {
    let label = pending.label.clone();
    let proxy = self.event_loop.create_proxy();
    let webview = create_webview(
      &self.event_loop,
      DispatcherContext {
        proxy: proxy.clone(),
        task_tx: self.task_tx.clone(),
        window_event_listeners: self.window_event_listeners.clone(),
        menu_event_listeners: self.menu_event_listeners.clone(),
      },
      pending,
    )?;

    let dispatcher = WryDispatcher {
      window_id: webview.window().id(),
      context: DispatcherContext {
        proxy,
        task_tx: self.task_tx.clone(),
        window_event_listeners: self.window_event_listeners.clone(),
        menu_event_listeners: self.menu_event_listeners.clone(),
      },
    };

    self.webviews.insert(webview.window().id(), webview);

    Ok(DetachedWindow { label, dispatcher })
  }

  #[cfg(target_os = "linux")]
  fn tray(&self, icon: std::path::PathBuf, menu: Vec<TrayMenuItem>) -> crate::Result<()> {
    SystemTrayBuilder::new(icon, menu.into_iter().map(Into::into).collect())
      .build(&self.event_loop)
      .map_err(|e| crate::Error::Tray(Box::new(e)))?;
    Ok(())
  }

  #[cfg(not(target_os = "linux"))]
  fn tray(&self, icon: Vec<u8>, menu: Vec<TrayMenuItem>) -> crate::Result<()> {
    SystemTrayBuilder::new(icon, menu.into_iter().map(Into::into).collect())
      .build(&self.event_loop)
      .map_err(|e| crate::Error::Tray(Box::new(e)))?;
    Ok(())
  }

  fn run(self) {
    let mut webviews = self.webviews;
    let task_rx = self.task_rx;
    let window_event_listeners = self.window_event_listeners.clone();
    let menu_event_listeners = self.menu_event_listeners.clone();
    self.event_loop.run(move |event, event_loop, control_flow| {
      *control_flow = ControlFlow::Wait;

      for (_, w) in webviews.iter() {
        if let Err(e) = w.evaluate_script() {
          eprintln!("{}", e);
        }
      }

      while let Ok(task) = task_rx.try_recv() {
        task();
      }

      match event {
        Event::MenuEvent {
          menu_id,
          origin: MenuType::Menubar,
        } => {
          let event = MenuEvent {
            menu_item_id: MenuItemId(menu_id.0),
          };
          for handler in menu_event_listeners.lock().unwrap().values() {
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
              }
            }
            WryWindowEvent::Resized(_) => {
              if let Err(e) = webviews[&window_id].resize() {
                eprintln!("{}", e);
              }
            }
            _ => {}
          }
        }
        Event::UserEvent(message) => match message {
          Message::Window(id, window_message) => {
            if let Some(webview) = webviews.get_mut(&id) {
              let window = webview.window();
              match window_message {
                // Getters
                WindowMessage::ScaleFactor(tx) => window_getter!(window, tx, scale_factor),
                WindowMessage::InnerPosition(tx) => {
                  window_result_getter!(window, tx, inner_position)
                }
                WindowMessage::OuterPosition(tx) => {
                  window_result_getter!(window, tx, outer_position)
                }
                WindowMessage::InnerSize(tx) => window_getter!(window, tx, inner_size),
                WindowMessage::OuterSize(tx) => window_getter!(window, tx, outer_size),
                WindowMessage::IsFullscreen(tx) => tx.send(window.fullscreen().is_some()).unwrap(),
                WindowMessage::IsMaximized(tx) => window_getter!(window, tx, is_maximized),
                WindowMessage::CurrentMonitor(tx) => window_getter!(window, tx, current_monitor),
                WindowMessage::PrimaryMonitor(tx) => window_getter!(window, tx, primary_monitor),
                WindowMessage::AvailableMonitors(tx) => {
                  tx.send(window.available_monitors().collect()).unwrap()
                }
                // Setters
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
                WindowMessage::SetAlwaysOnTop(always_on_top) => {
                  window.set_always_on_top(always_on_top)
                }
                WindowMessage::SetSize(size) => {
                  window.set_inner_size(WrySize::from(size));
                }
                WindowMessage::SetMinSize(size) => {
                  window.set_min_inner_size(size.map(WrySize::from));
                }
                WindowMessage::SetMaxSize(size) => {
                  window.set_max_inner_size(size.map(WrySize::from));
                }
                WindowMessage::SetPosition(position) => {
                  window.set_outer_position(WryPosition::from(position))
                }
                WindowMessage::SetFullscreen(fullscreen) => {
                  if fullscreen {
                    window.set_fullscreen(Some(Fullscreen::Borderless(None)))
                  } else {
                    window.set_fullscreen(None)
                  }
                }
                WindowMessage::SetIcon(icon) => {
                  window.set_window_icon(Some(icon));
                }
                WindowMessage::DragWindow => {
                  let _ = window.drag_window();
                }
              }
            }
          }
          Message::Webview(id, webview_message) => {
            if let Some(webview) = webviews.get_mut(&id) {
              match webview_message {
                WebviewMessage::EvaluateScript(script) => {
                  let _ = webview.dispatch_script(&script);
                }
                WebviewMessage::Print => {
                  let _ = webview.print();
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
                let window_id = webview.window().id();
                webviews.insert(window_id, webview);
                sender.send(window_id).unwrap();
              }
              Err(e) => {
                eprintln!("{}", e);
              }
            }
          }
        },
        _ => (),
      }
    })
  }
}

fn create_webview<M: Params<Runtime = Wry>>(
  event_loop: &EventLoopWindowTarget<Message>,
  context: DispatcherContext,
  pending: PendingWindow<M>,
) -> crate::Result<WebView> {
  let PendingWindow {
    webview_attributes,
    window_attributes,
    rpc_handler,
    file_drop_handler,
    label,
    url,
    ..
  } = pending;

  let window = window_attributes.build(event_loop).unwrap();
  let mut webview_builder = WebViewBuilder::new(window)
    .map_err(|e| crate::Error::CreateWebview(Box::new(e)))?
    .with_url(&url)
    .unwrap(); // safe to unwrap because we validate the URL beforehand
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

  webview_builder
    .build()
    .map_err(|e| crate::Error::CreateWebview(Box::new(e)))
}

/// Create a wry rpc handler from a tauri rpc handler.
fn create_rpc_handler<M: Params<Runtime = Wry>>(
  context: DispatcherContext,
  label: M::Label,
  handler: WebviewRpcHandler<M>,
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
      request.into(),
    );
    None
  })
}

/// Create a wry file drop handler from a tauri file drop handler.
fn create_file_drop_handler<M: Params<Runtime = Wry>>(
  context: DispatcherContext,
  label: M::Label,
  handler: FileDropHandler<M>,
) -> Box<dyn Fn(&Window, WryFileDropEvent) -> bool + 'static> {
  Box::new(move |window, event| {
    handler(
      event.into(),
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
