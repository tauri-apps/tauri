use super::{
  ApplicationDispatcherExt, ApplicationExt, Callback, Event, Icon, Message, WebviewBuilderExt,
  WindowBuilderExt,
};

use wry::{ApplicationDispatcher, ApplicationExt as _, WebviewMessage, WindowExt, WindowMessage};

use once_cell::sync::Lazy;

use crate::plugin::PluginStore;

use std::{
  convert::{TryFrom, TryInto},
  sync::{Arc, Mutex},
};

impl TryInto<wry::Icon> for Icon {
  type Error = crate::Error;
  fn try_into(self) -> Result<wry::Icon, Self::Error> {
    let icon = match self {
      Self::File(path) => {
        wry::Icon::from_file(path).map_err(|e| crate::Error::InvalidIcon(e.to_string()))?
      }
      Self::Raw(raw) => {
        wry::Icon::from_bytes(raw).map_err(|e| crate::Error::InvalidIcon(e.to_string()))?
      }
    };
    Ok(icon)
  }
}

impl WindowBuilderExt for wry::AppWindowAttributes {
  type Window = Self;

  fn new() -> Self {
    Default::default()
  }

  fn x(mut self, x: f64) -> Self {
    self.x = Some(x);
    self
  }

  fn y(mut self, y: f64) -> Self {
    self.y = Some(y);
    self
  }

  fn width(mut self, width: f64) -> Self {
    self.width = width;
    self
  }

  fn height(mut self, height: f64) -> Self {
    self.height = height;
    self
  }

  fn min_width(mut self, min_width: f64) -> Self {
    self.min_width = Some(min_width);
    self
  }

  fn min_height(mut self, min_height: f64) -> Self {
    self.min_height = Some(min_height);
    self
  }

  fn max_width(mut self, max_width: f64) -> Self {
    self.max_width = Some(max_width);
    self
  }

  fn max_height(mut self, max_height: f64) -> Self {
    self.max_height = Some(max_height);
    self
  }

  fn resizable(mut self, resizable: bool) -> Self {
    self.resizable = resizable;
    self
  }

  fn title(mut self, title: String) -> Self {
    self.title = title;
    self
  }

  fn fullscreen(mut self, fullscreen: bool) -> Self {
    self.fullscreen = fullscreen;
    self
  }

  fn maximized(mut self, maximized: bool) -> Self {
    self.maximized = maximized;
    self
  }

  fn visible(mut self, visible: bool) -> Self {
    self.visible = visible;
    self
  }

  fn transparent(mut self, transparent: bool) -> Self {
    self.transparent = transparent;
    self
  }

  fn decorations(mut self, decorations: bool) -> Self {
    self.decorations = decorations;
    self
  }

  fn always_on_top(mut self, always_on_top: bool) -> Self {
    self.always_on_top = always_on_top;
    self
  }

  /// build the window.
  fn finish(self) -> crate::Result<Self::Window> {
    Ok(self)
  }
}

/// The webview builder.
impl WebviewBuilderExt for wry::WebViewAttributes {
  /// The webview object that this builder creates.
  type Webview = Self;

  fn new() -> Self {
    Default::default()
  }

  fn url(mut self, url: String) -> Self {
    self.url.replace(url);
    self
  }

  fn initialization_script(mut self, init: &str) -> Self {
    self.initialization_script.push(init.to_string());
    self
  }

  fn finish(self) -> crate::Result<Self::Webview> {
    Ok(self)
  }
}

#[derive(Clone)]
pub struct WryDispatcher {
  inner: Arc<Mutex<wry::AppDispatcher<Event>>>,
  current_window: wry::WindowId,
}

struct WryMessage(wry::Message<wry::WindowId, Event>);

impl TryFrom<(wry::WindowId, Message)> for WryMessage {
  type Error = crate::Error;
  fn try_from((id, message): (wry::WindowId, Message)) -> crate::Result<Self> {
    let message = match message {
      Message::EvalScript(js) => wry::Message::Webview(id, WebviewMessage::EvalScript(js)),
      Message::Event(event) => wry::Message::Custom(event),
      Message::SetResizable(resizable) => {
        wry::Message::Window(id, WindowMessage::SetResizable(resizable))
      }
      Message::SetTitle(title) => wry::Message::Window(id, WindowMessage::SetTitle(title)),
      Message::Maximize => wry::Message::Window(id, WindowMessage::Maximize),
      Message::Unmaximize => wry::Message::Window(id, WindowMessage::Unmaximize),
      Message::Minimize => wry::Message::Window(id, WindowMessage::Minimize),
      Message::Unminimize => wry::Message::Window(id, WindowMessage::Unminimize),
      Message::Show => wry::Message::Window(id, WindowMessage::Show),
      Message::Hide => wry::Message::Window(id, WindowMessage::Hide),
      Message::SetTransparent(transparent) => {
        wry::Message::Window(id, WindowMessage::SetTransparent(transparent))
      }
      Message::SetDecorations(decorations) => {
        wry::Message::Window(id, WindowMessage::SetDecorations(decorations))
      }
      Message::SetAlwaysOnTop(always_on_top) => {
        wry::Message::Window(id, WindowMessage::SetAlwaysOnTop(always_on_top))
      }
      Message::SetWidth(width) => wry::Message::Window(id, WindowMessage::SetWidth(width)),
      Message::SetHeight(height) => wry::Message::Window(id, WindowMessage::SetHeight(height)),
      Message::Resize { width, height } => {
        wry::Message::Window(id, WindowMessage::Resize { width, height })
      }
      Message::SetMinSize {
        min_width,
        min_height,
      } => wry::Message::Window(
        id,
        WindowMessage::SetMinSize {
          min_width,
          min_height,
        },
      ),
      Message::SetMaxSize {
        max_width,
        max_height,
      } => wry::Message::Window(
        id,
        WindowMessage::SetMaxSize {
          max_width,
          max_height,
        },
      ),
      Message::SetX(x) => wry::Message::Window(id, WindowMessage::SetX(x)),
      Message::SetY(y) => wry::Message::Window(id, WindowMessage::SetY(y)),
      Message::SetPosition { x, y } => {
        wry::Message::Window(id, WindowMessage::SetPosition { x, y })
      }
      Message::SetFullscreen(fullscreen) => {
        wry::Message::Window(id, WindowMessage::SetFullscreen(fullscreen))
      }
      Message::SetIcon(icon) => wry::Message::Window(id, WindowMessage::SetIcon(icon.try_into()?)),
    };
    Ok(WryMessage(message))
  }
}

impl ApplicationDispatcherExt for WryDispatcher {
  fn send_message(&self, message: Message) {
    let message_res: crate::Result<WryMessage> = (self.current_window, message).try_into();
    // TODO error propagation
    if let Ok(message) = message_res {
      self
        .inner
        .lock()
        .unwrap()
        .dispatch_message(message.0)
        .unwrap();
    }
  }
}

/// A wrapper around the wry Application interface.
pub struct WryApplication {
  inner: wry::Application<Event>,
  dispatcher_handle: Arc<Mutex<wry::AppDispatcher<Event>>>,
}

impl ApplicationExt for WryApplication {
  type WebviewBuilder = wry::WebViewAttributes;
  type WindowBuilder = wry::AppWindowAttributes;
  type Window = wry::Window;
  type Dispatcher = WryDispatcher;

  fn plugin_store() -> &'static PluginStore<Self::Dispatcher> {
    static PLUGINS: Lazy<PluginStore<WryDispatcher>> = Lazy::new(Default::default);
    &PLUGINS
  }

  fn new() -> crate::Result<Self> {
    let app = wry::Application::new().map_err(|_| crate::Error::CreateWebview)?;
    let dispatcher = app.dispatcher();

    Ok(Self {
      inner: app,
      dispatcher_handle: Arc::new(Mutex::new(dispatcher)),
    })
  }

  fn dispatcher(&self, window: &Self::Window) -> Self::Dispatcher {
    WryDispatcher {
      inner: self.dispatcher_handle.clone(),
      current_window: window.id(),
    }
  }

  fn create_window(&self, window_builder: Self::WindowBuilder) -> crate::Result<Self::Window> {
    let window = self
      .inner
      .create_window(window_builder.finish()?)
      .map_err(|_| crate::Error::CreateWindow)?;
    Ok(window)
  }

  fn create_webview(
    &mut self,
    webview_builder: Self::WebviewBuilder,
    window: Self::Window,
    callbacks: Vec<Callback<Self::Dispatcher>>,
  ) -> crate::Result<()> {
    let mut wry_callbacks = Vec::new();
    for mut callback in callbacks {
      let dispatcher_handle = self.dispatcher_handle.clone();
      let window_id = window.id();

      let callback = wry::Callback {
        name: callback.name.to_string(),
        function: Box::new(move |_, seq, req| {
          (callback.function)(
            &WryDispatcher {
              inner: dispatcher_handle.clone(),
              current_window: window_id,
            },
            seq,
            req,
          )
        }),
      };
      wry_callbacks.push(callback);
    }

    self
      .inner
      .create_webview(window, webview_builder.finish()?, Some(wry_callbacks))
      .map_err(|_| crate::Error::CreateWebview)?;
    Ok(())
  }

  fn run(mut self) {
    self.inner.set_message_handler(|message| match message {
      Event::Run(task) => task(),
    });
    wry::Application::run(self.inner)
  }
}
