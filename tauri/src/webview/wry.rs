use super::{
  ApplicationDispatcherExt, ApplicationExt, Callback, Event, WebviewBuilderExt, WindowBuilderExt,
};

use wry::{ApplicationDispatcher, ApplicationExt as _, WindowExt};

use once_cell::sync::Lazy;

use crate::plugin::PluginStore;

use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

impl WindowBuilderExt for wry::AppWindowAttributes {
  type Window = Self;

  fn new() -> Self {
    Default::default()
  }

  fn resizable(mut self, resizable: bool) -> Self {
    self.resizable = resizable;
    self
  }

  fn title(mut self, title: String) -> Self {
    self.title = title;
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

  /// Whether the window should always be on top of other windows.
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
  windows: Arc<Mutex<HashMap<String, wry::WindowId>>>,
}

impl ApplicationDispatcherExt for WryDispatcher {
  fn eval(&mut self, js: &str) {
    #[cfg(target_os = "linux")]
    let window_id = self.current_window;
    #[cfg(not(target_os = "linux"))]
    let window_id = self.current_window.clone();

    self
      .inner
      .lock()
      .unwrap()
      .dispatch_message(wry::Message::Script(window_id, js.to_string()))
      .unwrap();
  }

  fn eval_on_window(&mut self, window_id: &str, js: &str) {
    if let Some(window_id) = self.windows.lock().unwrap().get(window_id) {
      #[cfg(target_os = "linux")]
      let window_id = *window_id;
      #[cfg(not(target_os = "linux"))]
      let window_id = window_id.clone();
      self
        .inner
        .lock()
        .unwrap()
        .dispatch_message(wry::Message::Script(window_id, js.to_string()))
        .unwrap();
    }
  }

  fn send_event(&self, event: Event) {
    self
      .inner
      .lock()
      .unwrap()
      .dispatch_message(wry::Message::Custom(event))
      .unwrap();
  }
}

/// A wrapper around the wry Application interface.
pub struct WryApplication {
  inner: wry::Application<Event>,
  windows: Arc<Mutex<HashMap<String, wry::WindowId>>>,
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
    let app = wry::Application::new()?;
    let dispatcher = app.dispatcher();
    let windows = Arc::new(Mutex::new(HashMap::new()));

    Ok(Self {
      inner: app,
      windows,
      dispatcher_handle: Arc::new(Mutex::new(dispatcher)),
    })
  }

  fn dispatcher(&self, window: &Self::Window) -> Self::Dispatcher {
    WryDispatcher {
      inner: self.dispatcher_handle.clone(),
      windows: self.windows.clone(),
      current_window: window.id(),
    }
  }

  fn create_window(&self, window_builder: Self::WindowBuilder) -> crate::Result<Self::Window> {
    let window = self.inner.create_window(window_builder.finish()?)?;
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
      let windows = self.windows.clone();
      let window_id = window.id();

      let callback = wry::Callback {
        name: callback.name.to_string(),
        function: Box::new(move |_, seq, req| {
          (callback.function)(
            &WryDispatcher {
              inner: dispatcher_handle.clone(),
              windows: windows.clone(),
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
      .create_webview(window, webview_builder.finish()?, Some(wry_callbacks))?;
    Ok(())
  }

  fn run(self) {
    wry::Application::run(self.inner)
  }
}
