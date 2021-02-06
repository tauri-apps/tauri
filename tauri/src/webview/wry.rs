#[cfg(not(target_os = "linux"))]
use wry::platform::{
  event::{Event, StartCause, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::Window,
};

#[cfg(target_os = "linux")]
use wry::platform::{Window, WindowType};

use once_cell::sync::Lazy;

use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use super::{SizeHint, Webview, WebviewBuilder, WebviewDispatcher};
use crate::plugin::PluginStore;

/// The wry webview builder.
#[derive(Default)]
pub struct WryWebviewBuilder {
  bind: HashMap<String, Box<dyn FnMut(&WryDispatcher, i8, Vec<String>) -> i32 + Send>>,
  title: Option<String>,
  init: Option<String>,
  url: Option<String>,
}

impl WryWebviewBuilder {
  fn build_webview(self, window: Window) -> crate::Result<wry::WebView> {
    let mut webview_builder = wry::WebViewBuilder::new(window)?;

    let dispatcher = WryDispatcher {
      inner: Arc::new(Mutex::new(webview_builder.dispatch_sender())),
    };

    for (name, mut f) in self.bind {
      let dispatcher = dispatcher.clone();
      webview_builder = webview_builder.bind(&name, move |seq, req| f(&dispatcher, seq, req))?;
    }

    if let Some(init) = self.init {
      webview_builder = webview_builder.init(&init)?;
    }

    if let Some(url) = self.url {
      webview_builder = webview_builder.load_url(&url)?;
    }

    Ok(webview_builder.build()?)
  }
}

impl WebviewBuilder for WryWebviewBuilder {
  type WebviewObject = WryWebview;

  fn new() -> Self {
    WryWebviewBuilder::default()
  }

  fn bind<F>(mut self, name: &str, f: F) -> Self
  where
    F: FnMut(
        &<<Self as WebviewBuilder>::WebviewObject as Webview>::Dispatcher,
        i8,
        Vec<String>,
      ) -> i32
      + Send
      + 'static,
  {
    self.bind.insert(name.to_string(), Box::new(f));
    self
  }

  fn debug(self, _debug: bool) -> Self {
    self
  }

  fn title(mut self, title: &str) -> Self {
    self.title = Some(title.to_string());
    self
  }

  fn url(mut self, url: &str) -> Self {
    self.url = Some(url.to_string());
    self
  }

  fn init(mut self, init: &str) -> Self {
    self.init = Some(init.to_string());
    self
  }

  fn width(self, _width: usize) -> Self {
    self
  }

  fn height(self, _height: usize) -> Self {
    self
  }

  fn resizable(self, _hint: SizeHint) -> Self {
    self
  }

  fn finish(self) -> crate::Result<Self::WebviewObject> {
    #[cfg(target_os = "linux")]
    {
      gtk::init().unwrap();
      let window = Window::new(WindowType::Toplevel);
      let webview = self.build_webview(window)?;
      Ok(WryWebview { inner: webview })
    }

    #[cfg(not(target_os = "linux"))]
    {
      let events = EventLoop::new();
      let window = Window::new(&events)?;
      let webview = WebViewBuilder::new(window)?;
      let webview = self.build_webview(window)?;
      Ok(WryWebview { inner: webview })
    }
  }
}

/// The wry Webview dispatcher.
#[derive(Clone)]
pub struct WryDispatcher {
  inner: Arc<Mutex<wry::DispatchSender>>,
}

impl WebviewDispatcher for WryDispatcher {
  fn eval(&mut self, js: &str) {
    self.inner.lock().unwrap().send(js).unwrap();
  }
}

/// A wrapper around wry's webview.
pub struct WryWebview {
  inner: wry::WebView,
}

impl Webview for WryWebview {
  type Builder = WryWebviewBuilder;
  type Dispatcher = WryDispatcher;

  fn plugin_store() -> &'static PluginStore<Self::Dispatcher> {
    static PLUGINS: Lazy<PluginStore<WryDispatcher>> = Lazy::new(Default::default);
    &PLUGINS
  }

  fn set_title(&mut self, _title: &str) {}

  fn set_size(&mut self, _width: i32, _height: i32, _hint: SizeHint) {}

  fn terminate(&mut self) {}

  fn eval(&mut self, js: &str) {
    self.inner.dispatch(js).unwrap();
  }

  fn dispatcher(&mut self) -> Self::Dispatcher {
    WryDispatcher {
      inner: Arc::new(Mutex::new(self.inner.dispatch_sender())),
    }
  }

  fn run(&mut self) {
    #[cfg(target_os = "linux")]
    loop {
      self.inner.evaluate().unwrap();
      gtk::main_iteration();
    }
    #[cfg(not(target_os = "linux"))]
    {}
  }
}
