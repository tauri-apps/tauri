use super::{
  ApplicationDispatcherExt, ApplicationExt, Callback, CustomProtocol, Icon, WebviewBuilderExt,
  WebviewBuilderExtPrivate, WindowConfig,
};

use once_cell::sync::Lazy;

use crate::plugin::PluginStore;

use std::{
  convert::{TryFrom, TryInto},
  sync::{Arc, Mutex},
};

impl TryFrom<Icon> for wry::Icon {
  type Error = crate::Error;
  fn try_from(icon: Icon) -> Result<Self, Self::Error> {
    let icon = match icon {
      Icon::File(path) => {
        wry::Icon::from_file(path).map_err(|e| crate::Error::InvalidIcon(e.to_string()))?
      }
      Icon::Raw(raw) => {
        wry::Icon::from_bytes(raw).map_err(|e| crate::Error::InvalidIcon(e.to_string()))?
      }
    };
    Ok(icon)
  }
}

impl WebviewBuilderExtPrivate for wry::Attributes {
  fn url(mut self, url: String) -> Self {
    self.url.replace(url);
    self
  }
}

impl From<WindowConfig> for wry::Attributes {
  fn from(window_config: WindowConfig) -> Self {
    let mut webview = wry::Attributes::default()
      .title(window_config.0.title.to_string())
      .width(window_config.0.width)
      .height(window_config.0.height)
      .visible(window_config.0.visible)
      .resizable(window_config.0.resizable)
      .decorations(window_config.0.decorations)
      .maximized(window_config.0.maximized)
      .fullscreen(window_config.0.fullscreen)
      .transparent(window_config.0.transparent)
      .always_on_top(window_config.0.always_on_top);
    if let Some(min_width) = window_config.0.min_width {
      webview = webview.min_width(min_width);
    }
    if let Some(min_height) = window_config.0.min_height {
      webview = webview.min_height(min_height);
    }
    if let Some(max_width) = window_config.0.max_width {
      webview = webview.max_width(max_width);
    }
    if let Some(max_height) = window_config.0.max_height {
      webview = webview.max_height(max_height);
    }
    if let Some(x) = window_config.0.x {
      webview = webview.x(x);
    }
    if let Some(y) = window_config.0.y {
      webview = webview.y(y);
    }
    webview
  }
}

/// The webview builder.
impl WebviewBuilderExt for wry::Attributes {
  /// The webview object that this builder creates.
  type Webview = Self;

  fn new() -> Self {
    Default::default()
  }

  fn initialization_script(mut self, init: &str) -> Self {
    self.initialization_scripts.push(init.to_string());
    self
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

  fn title<S: Into<String>>(mut self, title: S) -> Self {
    self.title = title.into();
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

  fn icon(mut self, icon: Icon) -> crate::Result<Self> {
    self.icon = Some(icon.try_into()?);
    Ok(self)
  }

  fn has_icon(&self) -> bool {
    self.icon.is_some()
  }

  fn finish(self) -> crate::Result<Self::Webview> {
    Ok(self)
  }
}

#[derive(Clone)]
pub struct WryDispatcher(
  Arc<Mutex<wry::WindowProxy>>,
  Arc<Mutex<wry::ApplicationProxy>>,
);

impl ApplicationDispatcherExt for WryDispatcher {
  type WebviewBuilder = wry::Attributes;

  fn create_webview(
    &self,
    attributes: Self::WebviewBuilder,
    callbacks: Vec<Callback<Self>>,
    custom_protocol: Option<CustomProtocol>,
  ) -> crate::Result<Self> {
    let mut wry_callbacks = Vec::new();
    let app_dispatcher = self.1.clone();
    for mut callback in callbacks {
      let app_dispatcher = app_dispatcher.clone();
      let callback = wry::Callback {
        name: callback.name.to_string(),
        function: Box::new(move |dispatcher, _, req| {
          (callback.function)(
            Self(Arc::new(Mutex::new(dispatcher)), app_dispatcher.clone()),
            req,
          );
          Ok(())
        }),
      };
      wry_callbacks.push(callback);
    }

    let window_dispatcher = self
      .1
      .lock()
      .unwrap()
      .add_window_with_configs(
        attributes,
        Some(wry_callbacks),
        custom_protocol.map(|p| wry::CustomProtocol {
          name: p.name.clone(),
          handler: Box::new(move |a| (*p.handler)(a).map_err(|_| wry::Error::InitScriptError)),
        }),
      )
      .map_err(|_| crate::Error::FailedToSendMessage)?;
    Ok(Self(
      Arc::new(Mutex::new(window_dispatcher)),
      self.1.clone(),
    ))
  }

  fn set_resizable(&self, resizable: bool) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .set_resizable(resizable)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_title<S: Into<String>>(&self, title: S) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .set_title(title)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn maximize(&self) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .maximize()
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn unmaximize(&self) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .unmaximize()
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn minimize(&self) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .minimize()
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn unminimize(&self) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .unminimize()
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn show(&self) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .show()
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn hide(&self) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .hide()
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn close(&self) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .close()
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_decorations(&self, decorations: bool) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .set_decorations(decorations)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_always_on_top(&self, always_on_top: bool) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .set_always_on_top(always_on_top)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_width(&self, width: f64) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .set_width(width)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_height(&self, height: f64) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .set_height(height)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn resize(&self, width: f64, height: f64) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .resize(width, height)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_min_size(&self, min_width: f64, min_height: f64) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .set_min_size(min_width, min_height)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_max_size(&self, max_width: f64, max_height: f64) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .set_max_size(max_width, max_height)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_x(&self, x: f64) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .set_x(x)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_y(&self, y: f64) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .set_y(y)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_position(&self, x: f64, y: f64) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .set_position(x, y)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .set_fullscreen(fullscreen)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_icon(&self, icon: Icon) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .set_icon(icon.try_into()?)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn eval_script<S: Into<String>>(&self, script: S) -> crate::Result<()> {
    self
      .0
      .lock()
      .unwrap()
      .evaluate_script(script)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }
}

/// A wrapper around the wry Application interface.
pub struct WryApplication {
  inner: wry::Application,
}

impl ApplicationExt for WryApplication {
  type WebviewBuilder = wry::Attributes;
  type Dispatcher = WryDispatcher;

  fn plugin_store() -> &'static PluginStore<Self> {
    static PLUGINS: Lazy<PluginStore<WryApplication>> = Lazy::new(Default::default);
    &PLUGINS
  }

  fn new() -> crate::Result<Self> {
    let app = wry::Application::new().map_err(|_| crate::Error::CreateWebview)?;
    Ok(Self { inner: app })
  }

  fn create_webview(
    &mut self,
    webview_builder: Self::WebviewBuilder,
    callbacks: Vec<Callback<Self::Dispatcher>>,
    custom_protocol: Option<CustomProtocol>,
  ) -> crate::Result<Self::Dispatcher> {
    let mut wry_callbacks = Vec::new();
    let app_dispatcher = Arc::new(Mutex::new(self.inner.application_proxy()));
    for mut callback in callbacks {
      let app_dispatcher = app_dispatcher.clone();
      let callback = wry::Callback {
        name: callback.name.to_string(),
        function: Box::new(move |dispatcher, _, req| {
          (callback.function)(
            WryDispatcher(Arc::new(Mutex::new(dispatcher)), app_dispatcher.clone()),
            req,
          );
          Ok(())
        }),
      };
      wry_callbacks.push(callback);
    }

    let dispatcher = self
      .inner
      .add_window_with_configs(
        webview_builder.finish()?,
        Some(wry_callbacks),
        custom_protocol.map(|p| wry::CustomProtocol {
          name: p.name.clone(),
          handler: Box::new(move |a| (*p.handler)(a).map_err(|_| wry::Error::InitScriptError)),
        }),
      )
      .map_err(|_| crate::Error::CreateWebview)?;
    Ok(WryDispatcher(
      Arc::new(Mutex::new(dispatcher)),
      app_dispatcher,
    ))
  }

  fn run(self) {
    wry::Application::run(self.inner)
  }
}
