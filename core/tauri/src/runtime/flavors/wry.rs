// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The [`wry`] Tauri [`Runtime`].

use crate::{
  api::config::WindowConfig,
  runtime::{
    webview::{
      FileDropEvent, FileDropHandler, RpcRequest, WebviewRpcHandler, WindowBuilder,
      WindowBuilderBase,
    },
    window::{DetachedWindow, PendingWindow},
    Dispatch, Params, Runtime,
  },
  Icon,
};

use image::{GenericImageView, Pixel};
use wry::{
  application::{
    dpi::{LogicalPosition, LogicalSize, Size},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
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
    mpsc::{channel, Sender},
    Arc, Mutex,
  },
};

type CreateWebviewHandler =
  Box<dyn FnOnce(&EventLoopWindowTarget<Message>) -> crate::Result<WebView> + Send>;

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
      Icon::File(path) => {
        image::open(path).map_err(|e| crate::Error::InvalidIcon(e.to_string()))?
      }
      Icon::Raw(raw) => {
        image::load_from_memory(&raw).map_err(|e| crate::Error::InvalidIcon(e.to_string()))?
      }
    };
    let (width, height) = image.dimensions();
    let mut rgba = Vec::with_capacity((width * height) as usize * PIXEL_SIZE);
    for (_, _, pixel) in image.pixels() {
      rgba.extend_from_slice(&pixel.to_rgba().0);
    }
    let icon = WindowIcon::from_rgba(rgba, width, height)
      .map_err(|e| crate::Error::InvalidIcon(e.to_string()))?;
    Ok(Self(icon))
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
    if let Some(x) = config.x {
      window = window.x(x);
    }
    if let Some(y) = config.y {
      window = window.y(y);
    }

    window
  }

  fn x(self, _x: f64) -> Self {
    // TODO self.with_x(Some(x))
    self
  }

  fn y(self, _y: f64) -> Self {
    // TODO self.with_y(Some(y))
    self
  }

  fn inner_size(self, width: f64, height: f64) -> Self {
    self.with_inner_size(Size::new(LogicalSize::new(width, height)))
  }

  fn min_inner_size(self, min_width: f64, min_height: f64) -> Self {
    self.with_min_inner_size(Size::new(LogicalSize::new(min_width, min_height)))
  }

  fn max_inner_size(self, max_width: f64, max_height: f64) -> Self {
    self.with_max_inner_size(Size::new(LogicalSize::new(max_width, max_height)))
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
  SetWidth(f64),
  SetHeight(f64),
  Resize { width: f64, height: f64 },
  SetMinSize { min_width: f64, min_height: f64 },
  SetMaxSize { max_width: f64, max_height: f64 },
  SetX(f64),
  SetY(f64),
  SetPosition { x: f64, y: f64 },
  SetFullscreen(bool),
  SetIcon(WindowIcon),
}

#[derive(Debug, Clone)]
enum WebviewMessage {
  EvaluateScript(String),
}

#[derive(Clone)]
enum Message {
  Window(WindowId, WindowMessage),
  Webview(WindowId, WebviewMessage),
  CreateWebview(Arc<Mutex<Option<CreateWebviewHandler>>>, Sender<WindowId>),
}

/// The Tauri [`Dispatch`] for [`Wry`].
#[derive(Clone)]
pub struct WryDispatcher {
  window_id: WindowId,
  proxy: EventLoopProxy<Message>,
}

impl Dispatch for WryDispatcher {
  type Runtime = Wry;
  type WindowBuilder = WryWindowBuilder;

  fn create_window<M: Params<Runtime = Self::Runtime>>(
    &mut self,
    pending: PendingWindow<M>,
  ) -> crate::Result<DetachedWindow<M>> {
    let (tx, rx) = channel();
    let label = pending.label.clone();
    let proxy = self.proxy.clone();
    self
      .proxy
      .send_event(Message::CreateWebview(
        Arc::new(Mutex::new(Some(Box::new(move |event_loop| {
          create_webview(event_loop, proxy, pending)
        })))),
        tx,
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)?;
    let window_id = rx.recv().unwrap();
    let dispatcher = WryDispatcher {
      window_id,
      proxy: self.proxy.clone(),
    };
    Ok(DetachedWindow { label, dispatcher })
  }

  fn set_resizable(&self, resizable: bool) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetResizable(resizable),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_title<S: Into<String>>(&self, title: S) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetTitle(title.into()),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn maximize(&self) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Maximize))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn unmaximize(&self) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Unmaximize))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn minimize(&self) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Minimize))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn unminimize(&self) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Unminimize))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn show(&self) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Show))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn hide(&self) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Hide))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn close(&self) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Close))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_decorations(&self, decorations: bool) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetDecorations(decorations),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_always_on_top(&self, always_on_top: bool) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetAlwaysOnTop(always_on_top),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_width(&self, width: f64) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetWidth(width),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_height(&self, height: f64) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetHeight(height),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn resize(&self, width: f64, height: f64) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::Resize { width, height },
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_min_size(&self, min_width: f64, min_height: f64) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetMinSize {
          min_width,
          min_height,
        },
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_max_size(&self, max_width: f64, max_height: f64) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetMaxSize {
          max_width,
          max_height,
        },
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_x(&self, x: f64) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::SetX(x)))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_y(&self, y: f64) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::SetY(y)))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_position(&self, x: f64, y: f64) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetPosition { x, y },
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetFullscreen(fullscreen),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_icon(&self, icon: Icon) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Window(
        self.window_id,
        WindowMessage::SetIcon(WryIcon::try_from(icon)?.0),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn eval_script<S: Into<String>>(&self, script: S) -> crate::Result<()> {
    self
      .proxy
      .send_event(Message::Webview(
        self.window_id,
        WebviewMessage::EvaluateScript(script.into()),
      ))
      .map_err(|_| crate::Error::FailedToSendMessage)
  }
}

/// A Tauri [`Runtime`] wrapper around [`wry::Application`].
pub struct Wry {
  event_loop: EventLoop<Message>,
  webviews: HashMap<WindowId, WebView>,
}

impl Runtime for Wry {
  type Dispatcher = WryDispatcher;

  fn new() -> crate::Result<Self> {
    let event_loop = EventLoop::<Message>::with_user_event();
    Ok(Self {
      event_loop,
      webviews: Default::default(),
    })
  }

  fn create_window<M: Params<Runtime = Self>>(
    &mut self,
    pending: PendingWindow<M>,
  ) -> crate::Result<DetachedWindow<M>> {
    let label = pending.label.clone();
    let proxy = self.event_loop.create_proxy();
    let webview = create_webview(&self.event_loop, proxy.clone(), pending)?;

    let dispatcher = WryDispatcher {
      window_id: webview.window().id(),
      proxy,
    };

    self.webviews.insert(webview.window().id(), webview);

    Ok(DetachedWindow { label, dispatcher })
  }

  fn run(self) {
    let mut webviews = self.webviews;
    self.event_loop.run(move |event, event_loop, control_flow| {
      *control_flow = ControlFlow::Poll;

      for (_, w) in webviews.iter() {
        if let Err(e) = w.evaluate_script() {
          eprintln!("{}", e);
        }
      }

      match event {
        Event::WindowEvent { event, window_id } => match event {
          WindowEvent::CloseRequested => {
            webviews.remove(&window_id);
            println!("{:?}", webviews.len());
            if webviews.is_empty() {
              *control_flow = ControlFlow::Exit;
            }
          }
          WindowEvent::Resized(_) => {
            if let Err(e) = webviews[&window_id].resize() {
              eprintln!("{}", e);
            }
          }
          _ => {}
        },
        Event::UserEvent(message) => match message {
          Message::Window(id, window_message) => {
            if let Some(webview) = webviews.get_mut(&id) {
              let window = webview.window();
              match window_message {
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
                WindowMessage::SetWidth(width) => {
                  let mut size = window.inner_size().to_logical(window.scale_factor());
                  size.width = width;
                  window.set_inner_size(size);
                }
                WindowMessage::SetHeight(height) => {
                  let mut size = window.inner_size().to_logical(window.scale_factor());
                  size.height = height;
                  window.set_inner_size(size);
                }
                WindowMessage::Resize { width, height } => {
                  window.set_inner_size(LogicalSize::new(width, height));
                }
                WindowMessage::SetMinSize {
                  min_width,
                  min_height,
                } => {
                  window.set_min_inner_size(Some(LogicalSize::new(min_width, min_height)));
                }
                WindowMessage::SetMaxSize {
                  max_width,
                  max_height,
                } => {
                  window.set_max_inner_size(Some(LogicalSize::new(max_width, max_height)));
                }
                WindowMessage::SetX(x) => {
                  if let Ok(outer_position) = window.outer_position() {
                    let mut outer_position = outer_position.to_logical(window.scale_factor());
                    outer_position.x = x;
                    window.set_outer_position(outer_position);
                  }
                }
                WindowMessage::SetY(y) => {
                  if let Ok(outer_position) = window.outer_position() {
                    let mut outer_position = outer_position.to_logical(window.scale_factor());
                    outer_position.y = y;
                    window.set_outer_position(outer_position);
                  }
                }
                WindowMessage::SetPosition { x, y } => {
                  window.set_outer_position(LogicalPosition::new(x, y))
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
              }
            }
          }
          Message::Webview(id, webview_message) => {
            if let Some(webview) = webviews.get_mut(&id) {
              match webview_message {
                WebviewMessage::EvaluateScript(script) => {
                  let _ = webview.dispatch_script(&script);
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
  proxy: EventLoopProxy<Message>,
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
    .map_err(|e| crate::Error::CreateWebview(e.to_string()))?
    .with_url(&url)
    .unwrap(); // safe to unwrap because we validate the URL beforehand
  if let Some(handler) = rpc_handler {
    webview_builder =
      webview_builder.with_rpc_handler(create_rpc_handler(proxy.clone(), label.clone(), handler));
  }
  if let Some(handler) = file_drop_handler {
    webview_builder =
      webview_builder.with_file_drop_handler(create_file_drop_handler(proxy, label, handler));
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
    .map_err(|e| crate::Error::CreateWebview(e.to_string()))
}

/// Create a wry rpc handler from a tauri rpc handler.
fn create_rpc_handler<M: Params<Runtime = Wry>>(
  proxy: EventLoopProxy<Message>,
  label: M::Label,
  handler: WebviewRpcHandler<M>,
) -> Box<dyn Fn(&Window, WryRpcRequest) -> Option<RpcResponse> + 'static> {
  Box::new(move |window, request| {
    handler(
      DetachedWindow {
        dispatcher: WryDispatcher {
          window_id: window.id(),
          proxy: proxy.clone(),
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
  proxy: EventLoopProxy<Message>,
  label: M::Label,
  handler: FileDropHandler<M>,
) -> Box<dyn Fn(&Window, WryFileDropEvent) -> bool + 'static> {
  Box::new(move |window, event| {
    handler(
      event.into(),
      DetachedWindow {
        dispatcher: WryDispatcher {
          window_id: window.id(),
          proxy: proxy.clone(),
        },
        label: label.clone(),
      },
    )
  })
}
