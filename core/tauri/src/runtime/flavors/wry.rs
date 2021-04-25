// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The [`wry`] Tauri [`Runtime`].

use crate::{
  api::config::WindowConfig,
  runtime::{
    webview::{
      Attributes, AttributesBase, FileDropEvent, FileDropHandler, RpcRequest, WebviewRpcHandler,
    },
    window::{DetachedWindow, PendingWindow},
    Dispatch, Params, Runtime,
  },
  Icon,
};
use std::{
  collections::HashMap,
  convert::TryFrom,
  path::PathBuf,
  sync::{Arc, Mutex},
};

/// Wrapper around a [`wry::Icon`] that can be created from an [`Icon`].
pub struct WryIcon(wry::Icon);

impl TryFrom<Icon> for WryIcon {
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
    Ok(Self(icon))
  }
}

/// Wry attributes.
#[derive(Default, Clone)]
pub struct WryAttributes {
  attributes: wry::Attributes,
  webview_protocols: Arc<Mutex<HashMap<String, wry::CustomProtocol>>>,
}

impl AttributesBase for WryAttributes {}
impl Attributes for WryAttributes {
  type Icon = WryIcon;

  fn new() -> Self {
    Default::default()
  }

  fn with_config(config: WindowConfig) -> Self {
    let mut webview = WryAttributes::default()
      .title(config.title.to_string())
      .width(config.width)
      .height(config.height)
      .visible(config.visible)
      .resizable(config.resizable)
      .decorations(config.decorations)
      .maximized(config.maximized)
      .fullscreen(config.fullscreen)
      .transparent(config.transparent)
      .always_on_top(config.always_on_top);

    if let Some(min_width) = config.min_width {
      webview = webview.min_width(min_width);
    }
    if let Some(min_height) = config.min_height {
      webview = webview.min_height(min_height);
    }
    if let Some(max_width) = config.max_width {
      webview = webview.max_width(max_width);
    }
    if let Some(max_height) = config.max_height {
      webview = webview.max_height(max_height);
    }
    if let Some(x) = config.x {
      webview = webview.x(x);
    }
    if let Some(y) = config.y {
      webview = webview.y(y);
    }

    webview
  }

  fn initialization_script(mut self, init: &str) -> Self {
    self
      .attributes
      .initialization_scripts
      .push(init.to_string());
    self
  }

  fn x(mut self, x: f64) -> Self {
    self.attributes.x = Some(x);
    self
  }

  fn y(mut self, y: f64) -> Self {
    self.attributes.y = Some(y);
    self
  }

  fn width(mut self, width: f64) -> Self {
    self.attributes.width = width;
    self
  }

  fn height(mut self, height: f64) -> Self {
    self.attributes.height = height;
    self
  }

  fn min_width(mut self, min_width: f64) -> Self {
    self.attributes.min_width = Some(min_width);
    self
  }

  fn min_height(mut self, min_height: f64) -> Self {
    self.attributes.min_height = Some(min_height);
    self
  }

  fn max_width(mut self, max_width: f64) -> Self {
    self.attributes.max_width = Some(max_width);
    self
  }

  fn max_height(mut self, max_height: f64) -> Self {
    self.attributes.max_height = Some(max_height);
    self
  }

  fn resizable(mut self, resizable: bool) -> Self {
    self.attributes.resizable = resizable;
    self
  }

  fn title<S: Into<String>>(mut self, title: S) -> Self {
    self.attributes.title = title.into();
    self
  }

  fn fullscreen(mut self, fullscreen: bool) -> Self {
    self.attributes.fullscreen = fullscreen;
    self
  }

  fn maximized(mut self, maximized: bool) -> Self {
    self.attributes.maximized = maximized;
    self
  }

  fn visible(mut self, visible: bool) -> Self {
    self.attributes.visible = visible;
    self
  }

  fn transparent(mut self, transparent: bool) -> Self {
    self.attributes.transparent = transparent;
    self
  }

  fn decorations(mut self, decorations: bool) -> Self {
    self.attributes.decorations = decorations;
    self
  }

  fn always_on_top(mut self, always_on_top: bool) -> Self {
    self.attributes.always_on_top = always_on_top;
    self
  }

  fn icon(mut self, icon: Self::Icon) -> Self {
    self.attributes.icon = Some(icon.0);
    self
  }

  fn has_icon(&self) -> bool {
    self.attributes.icon.is_some()
  }

  fn user_data_path(mut self, user_data_path: Option<PathBuf>) -> Self {
    self.attributes.user_data_path = user_data_path;
    self
  }

  fn url(mut self, url: String) -> Self {
    self.attributes.url.replace(url);
    self
  }

  fn has_webview_protocol(&self, name: &str) -> bool {
    self.webview_protocols.lock().unwrap().contains_key(name)
  }

  fn register_webview_protocol<
    N: Into<String>,
    H: Fn(&str) -> crate::Result<Vec<u8>> + Send + Sync + 'static,
  >(
    self,
    uri_scheme: N,
    handler: H,
  ) -> Self {
    let uri_scheme = uri_scheme.into();
    self.webview_protocols.lock().unwrap().insert(
      uri_scheme.clone(),
      wry::CustomProtocol {
        name: uri_scheme,
        handler: Box::new(move |data| (handler)(data).map_err(|_| wry::Error::InitScriptError)),
      },
    );
    self
  }

  fn build(self) -> Self {
    self
  }
}

impl From<wry::RpcRequest> for RpcRequest {
  fn from(request: wry::RpcRequest) -> Self {
    Self {
      command: request.method,
      params: request.params,
    }
  }
}

impl From<wry::FileDropEvent> for FileDropEvent {
  fn from(event: wry::FileDropEvent) -> Self {
    match event {
      wry::FileDropEvent::Hovered(paths) => FileDropEvent::Hovered(paths),
      wry::FileDropEvent::Dropped(paths) => FileDropEvent::Dropped(paths),
      wry::FileDropEvent::Cancelled => FileDropEvent::Cancelled,
    }
  }
}

/// The Tauri [`Dispatch`] for [`Wry`].
#[derive(Clone)]
pub struct WryDispatcher {
  window: wry::WindowProxy,
  application: wry::ApplicationProxy,
}

impl Dispatch for WryDispatcher {
  type Runtime = Wry;
  type Icon = WryIcon;
  type Attributes = WryAttributes;

  fn create_window<M: Params<Runtime = Self::Runtime>>(
    &mut self,
    pending: PendingWindow<M>,
  ) -> crate::Result<DetachedWindow<M>> {
    let PendingWindow {
      attributes,
      rpc_handler,
      file_drop_handler,
      label,
      ..
    } = pending;

    let proxy = self.application.clone();

    let rpc_handler =
      rpc_handler.map(|handler| create_rpc_handler(proxy.clone(), label.clone(), handler));

    let file_drop_handler = file_drop_handler
      .map(|handler| create_file_drop_handler(proxy.clone(), label.clone(), handler));

    let webview_protocols = {
      let mut lock = attributes
        .webview_protocols
        .lock()
        .expect("poisoned custom protocols");
      std::mem::take(&mut *lock)
    };

    let window = self
      .application
      .add_window_with_configs(
        attributes.attributes,
        rpc_handler,
        webview_protocols.into_iter().map(|(_, p)| p).collect(),
        file_drop_handler,
      )
      .map_err(|e| crate::Error::CreateWebview(e.to_string()))?;

    let dispatcher = WryDispatcher {
      window,
      application: proxy,
    };

    Ok(DetachedWindow { label, dispatcher })
  }

  fn set_resizable(&self, resizable: bool) -> crate::Result<()> {
    self
      .window
      .set_resizable(resizable)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_title<S: Into<String>>(&self, title: S) -> crate::Result<()> {
    self
      .window
      .set_title(title)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn maximize(&self) -> crate::Result<()> {
    self
      .window
      .maximize()
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn unmaximize(&self) -> crate::Result<()> {
    self
      .window
      .unmaximize()
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn minimize(&self) -> crate::Result<()> {
    self
      .window
      .minimize()
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn unminimize(&self) -> crate::Result<()> {
    self
      .window
      .unminimize()
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn show(&self) -> crate::Result<()> {
    self
      .window
      .show()
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn hide(&self) -> crate::Result<()> {
    self
      .window
      .hide()
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn close(&self) -> crate::Result<()> {
    self
      .window
      .close()
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_decorations(&self, decorations: bool) -> crate::Result<()> {
    self
      .window
      .set_decorations(decorations)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_always_on_top(&self, always_on_top: bool) -> crate::Result<()> {
    self
      .window
      .set_always_on_top(always_on_top)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_width(&self, width: f64) -> crate::Result<()> {
    self
      .window
      .set_width(width)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_height(&self, height: f64) -> crate::Result<()> {
    self
      .window
      .set_height(height)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn resize(&self, width: f64, height: f64) -> crate::Result<()> {
    self
      .window
      .resize(width, height)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_min_size(&self, min_width: f64, min_height: f64) -> crate::Result<()> {
    self
      .window
      .set_min_size(min_width, min_height)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_max_size(&self, max_width: f64, max_height: f64) -> crate::Result<()> {
    self
      .window
      .set_max_size(max_width, max_height)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_x(&self, x: f64) -> crate::Result<()> {
    self
      .window
      .set_x(x)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_y(&self, y: f64) -> crate::Result<()> {
    self
      .window
      .set_y(y)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_position(&self, x: f64, y: f64) -> crate::Result<()> {
    self
      .window
      .set_position(x, y)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()> {
    self
      .window
      .set_fullscreen(fullscreen)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn set_icon(&self, icon: Self::Icon) -> crate::Result<()> {
    self
      .window
      .set_icon(icon.0)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }

  fn eval_script<S: Into<String>>(&self, script: S) -> crate::Result<()> {
    self
      .window
      .evaluate_script(script)
      .map_err(|_| crate::Error::FailedToSendMessage)
  }
}

/// A Tauri [`Runtime`] wrapper around [`wry::Application`].
pub struct Wry {
  inner: wry::Application,
}

impl Runtime for Wry {
  type Dispatcher = WryDispatcher;

  fn new() -> crate::Result<Self> {
    let app = wry::Application::new().map_err(|e| crate::Error::CreateWebview(e.to_string()))?;
    Ok(Self { inner: app })
  }

  fn create_window<M: Params<Runtime = Self>>(
    &mut self,
    pending: PendingWindow<M>,
  ) -> crate::Result<DetachedWindow<M>> {
    let PendingWindow {
      attributes,
      rpc_handler,
      file_drop_handler,
      label,
      ..
    } = pending;

    let proxy = self.inner.application_proxy();

    let rpc_handler =
      rpc_handler.map(|handler| create_rpc_handler(proxy.clone(), label.clone(), handler));

    let file_drop_handler = file_drop_handler
      .map(|handler| create_file_drop_handler(proxy.clone(), label.clone(), handler));

    let webview_protocols = {
      let mut lock = attributes
        .webview_protocols
        .lock()
        .expect("poisoned custom protocols");
      std::mem::take(&mut *lock)
    };

    let window = self
      .inner
      .add_window_with_configs(
        attributes.attributes,
        rpc_handler,
        webview_protocols.into_iter().map(|(_, p)| p).collect(),
        file_drop_handler,
      )
      .map_err(|e| crate::Error::CreateWebview(e.to_string()))?;

    let dispatcher = WryDispatcher {
      window,
      application: proxy,
    };

    Ok(DetachedWindow { label, dispatcher })
  }

  fn run(self) {
    wry::Application::run(self.inner)
  }
}

/// Create a wry rpc handler from a tauri rpc handler.
fn create_rpc_handler<M: Params<Runtime = Wry>>(
  app_proxy: wry::ApplicationProxy,
  label: M::Label,
  handler: WebviewRpcHandler<M>,
) -> wry::WindowRpcHandler {
  Box::new(move |window, request| {
    handler(
      DetachedWindow {
        dispatcher: WryDispatcher {
          window,
          application: app_proxy.clone(),
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
  app_proxy: wry::ApplicationProxy,
  label: M::Label,
  handler: FileDropHandler<M>,
) -> wry::WindowFileDropHandler {
  Box::new(move |window, event| {
    handler(
      event.into(),
      DetachedWindow {
        dispatcher: WryDispatcher {
          window,
          application: app_proxy.clone(),
        },
        label: label.clone(),
      },
    )
  })
}
