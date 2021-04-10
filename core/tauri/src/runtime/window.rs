// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  api::config::WindowUrl,
  event::{Event, EventHandler},
  hooks::{InvokeMessage, InvokePayload, PageLoadPayload},
  runtime::{
    sealed::ManagerPrivate,
    tag::ToJavascript,
    webview::{CustomProtocol, FileDropHandler, Icon, WebviewRpcHandler},
    Dispatch, Manager, Params, Runtime, RuntimeOrDispatch,
  },
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{
  convert::TryInto,
  hash::{Hash, Hasher},
};

/// A webview window that has yet to be built.
pub struct PendingWindow<M: Params> {
  /// The label that the window will be named.
  pub label: M::Label,

  /// The url the window will open with.
  pub url: WindowUrl,

  /// The [`Attributes`] that the webview window be created with.
  pub attributes: <<M::Runtime as Runtime>::Dispatcher as Dispatch>::Attributes,

  /// How to handle RPC calls on the webview window.
  pub rpc_handler: Option<WebviewRpcHandler<M>>,

  /// How to handle custom protocols for the webview window.
  pub custom_protocol: Option<CustomProtocol>,

  /// How to handle a file dropping onto the webview window.
  pub file_drop_handler: Option<FileDropHandler<M>>,
}

impl<M: Params> PendingWindow<M> {
  pub fn new(
    attributes: impl Into<<<M::Runtime as Runtime>::Dispatcher as Dispatch>::Attributes>,
    label: M::Label,
    url: WindowUrl,
  ) -> Self {
    Self {
      attributes: attributes.into(),
      label,
      url,
      rpc_handler: None,
      custom_protocol: None,
      file_drop_handler: None,
    }
  }
}

/// A webview window that is not yet managed by Tauri.
pub struct DetachedWindow<M: Params> {
  pub label: M::Label,
  pub dispatcher: <M::Runtime as Runtime>::Dispatcher,
}

impl<M: Params> Clone for DetachedWindow<M> {
  fn clone(&self) -> Self {
    Self {
      label: self.label.clone(),
      dispatcher: self.dispatcher.clone(),
    }
  }
}

impl<M: Params> Hash for DetachedWindow<M> {
  /// Only use the [`DetachedWindow`]'s label to represent its hash.
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.label.hash(state)
  }
}

impl<M: Params> Eq for DetachedWindow<M> {}
impl<M: Params> PartialEq for DetachedWindow<M> {
  /// Only use the [`DetachedWindow`]'s label to compare equality.
  fn eq(&self, other: &Self) -> bool {
    self.label.eq(&other.label)
  }
}

/// A webview window managed by Tarui.
///
/// TODO: expand these docs since this is a pretty important type
pub struct Window<M: Params> {
  /// The webview window created by the runtime.
  window: DetachedWindow<M>,

  /// The manager to associate this webview window with.
  manager: M,
}

impl<M: Params> Clone for Window<M> {
  fn clone(&self) -> Self {
    Self {
      window: self.window.clone(),
      manager: self.manager.clone(),
    }
  }
}

impl<M: Params> Hash for Window<M> {
  /// Only use the [`Window`]'s label to represent its hash.
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.window.label.hash(state)
  }
}

impl<M: Params> Eq for Window<M> {}
impl<M: Params> PartialEq for Window<M> {
  /// Only use the [`Window`]'s label to compare equality.
  fn eq(&self, other: &Self) -> bool {
    self.window.label.eq(&other.window.label)
  }
}

impl<M: Params> Manager<M> for Window<M> {}
impl<M: Params> ManagerPrivate<M> for Window<M> {
  fn manager(&self) -> &M {
    &self.manager
  }

  fn runtime(&mut self) -> RuntimeOrDispatch<'_, M> {
    RuntimeOrDispatch::Dispatch(self.dispatcher())
  }
}

impl<M: Params> Window<M> {
  /// Create a new window that is attached to the manager.
  pub(crate) fn new(manager: M, window: DetachedWindow<M>) -> Self {
    Self { manager, window }
  }

  /// The current window's dispatcher.
  pub(crate) fn dispatcher(&self) -> <M::Runtime as Runtime>::Dispatcher {
    self.window.dispatcher.clone()
  }

  /// How to handle this window receiving an [`InvokeMessage`].
  pub(crate) fn on_message(self, command: String, payload: InvokePayload) -> crate::Result<()> {
    let manager = self.manager.clone();
    if &command == "__initialized" {
      let payload: PageLoadPayload = serde_json::from_value(payload.inner)?;
      manager.run_on_page_load(self, payload);
    } else {
      let message = InvokeMessage::new(self, command.to_string(), payload);
      if let Some(module) = &message.payload.tauri_module {
        let module = module.to_string();
        crate::endpoints::handle(module, message, manager.config(), manager.package_info());
      } else if command.starts_with("plugin:") {
        manager.extend_api(command, message);
      } else {
        manager.run_invoke_handler(message);
      }
    }

    Ok(())
  }

  /// The label of this window.
  pub fn label(&self) -> &M::Label {
    &self.window.label
  }

  pub(crate) fn emit_internal<E: ToJavascript, S: Serialize>(
    &self,
    event: E,
    payload: Option<S>,
  ) -> crate::Result<()> {
    let js_payload = match payload {
      Some(payload_value) => serde_json::to_value(payload_value)?,
      None => JsonValue::Null,
    };

    self.eval(&format!(
      "window['{}']({{event: {}, payload: {}}}, '{}')",
      self.manager.event_emit_function_name(),
      event.to_javascript()?,
      js_payload,
      self.manager.generate_salt(),
    ))?;

    Ok(())
  }

  /// Emits an event to the current window.
  pub fn emit<S: Serialize>(&self, event: &M::Event, payload: Option<S>) -> crate::Result<()> {
    self.emit_internal(event.clone(), payload)
  }

  pub(crate) fn emit_others_internal<S: Serialize + Clone>(
    &self,
    event: String,
    payload: Option<S>,
  ) -> crate::Result<()> {
    self
      .manager
      .emit_filter_internal(event, payload, |w| w != self)
  }

  /// Emits an event on all windows except this one.
  pub fn emit_others<S: Serialize + Clone>(
    &self,
    event: M::Event,
    payload: Option<S>,
  ) -> crate::Result<()> {
    self.manager.emit_filter(event, payload, |w| w != self)
  }

  /// Listen to an event on this window.
  pub fn listen<F>(&self, event: M::Event, handler: F) -> EventHandler
  where
    F: Fn(Event) + Send + 'static,
  {
    let label = self.window.label.clone();
    self.manager.listen(event, Some(label), handler)
  }

  /// Listen to a an event on this window a single time.
  pub fn once<F>(&self, event: M::Event, handler: F)
  where
    F: Fn(Event) + Send + 'static,
  {
    let label = self.window.label.clone();
    self.manager.once(event, Some(label), handler)
  }

  /// Triggers an event on this window.
  pub(crate) fn trigger(&self, event: M::Event, data: Option<String>) {
    let label = self.window.label.clone();
    self.manager.trigger(event, Some(label), data)
  }

  /// Evaluates JavaScript on this window.
  pub fn eval(&self, js: &str) -> crate::Result<()> {
    self.window.dispatcher.eval_script(js)
  }

  /// Determines if this window should be resizable.
  pub fn set_resizable(&self, resizable: bool) -> crate::Result<()> {
    self.window.dispatcher.set_resizable(resizable)
  }

  /// Set this window's title.
  pub fn set_title(&self, title: &str) -> crate::Result<()> {
    self.window.dispatcher.set_title(title.to_string())
  }

  /// Maximizes this window.
  pub fn maximize(&self) -> crate::Result<()> {
    self.window.dispatcher.maximize()
  }

  /// Un-maximizes this window.
  pub fn unmaximize(&self) -> crate::Result<()> {
    self.window.dispatcher.unmaximize()
  }

  /// Minimizes this window.
  pub fn minimize(&self) -> crate::Result<()> {
    self.window.dispatcher.minimize()
  }

  /// Un-minimizes this window.
  pub fn unminimize(&self) -> crate::Result<()> {
    self.window.dispatcher.unminimize()
  }

  /// Show this window.
  pub fn show(&self) -> crate::Result<()> {
    self.window.dispatcher.show()
  }

  /// Hide this window.
  pub fn hide(&self) -> crate::Result<()> {
    self.window.dispatcher.hide()
  }

  /// Closes this window.
  pub fn close(&self) -> crate::Result<()> {
    self.window.dispatcher.close()
  }

  /// Determines if this window should be [decorated].
  ///
  /// [decorated]: https://en.wikipedia.org/wiki/Window_(computing)#Window_decoration
  pub fn set_decorations(&self, decorations: bool) -> crate::Result<()> {
    self.window.dispatcher.set_decorations(decorations)
  }

  /// Determines if this window should always be on top of other windows.
  pub fn set_always_on_top(&self, always_on_top: bool) -> crate::Result<()> {
    self.window.dispatcher.set_always_on_top(always_on_top)
  }

  /// Sets this window's width.
  pub fn set_width(&self, width: impl Into<f64>) -> crate::Result<()> {
    self.window.dispatcher.set_width(width.into())
  }

  /// Sets this window's height.
  pub fn set_height(&self, height: impl Into<f64>) -> crate::Result<()> {
    self.window.dispatcher.set_height(height.into())
  }

  /// Resizes this window.
  pub fn resize(&self, width: impl Into<f64>, height: impl Into<f64>) -> crate::Result<()> {
    self.window.dispatcher.resize(width.into(), height.into())
  }

  /// Sets this window's minimum size.
  pub fn set_min_size(
    &self,
    min_width: impl Into<f64>,
    min_height: impl Into<f64>,
  ) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_min_size(min_width.into(), min_height.into())
  }

  /// Sets this window's maximum size.
  pub fn set_max_size(
    &self,
    max_width: impl Into<f64>,
    max_height: impl Into<f64>,
  ) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_max_size(max_width.into(), max_height.into())
  }

  /// Sets this window's x position.
  pub fn set_x(&self, x: impl Into<f64>) -> crate::Result<()> {
    self.window.dispatcher.set_x(x.into())
  }

  /// Sets this window's y position.
  pub fn set_y(&self, y: impl Into<f64>) -> crate::Result<()> {
    self.window.dispatcher.set_y(y.into())
  }

  /// Sets this window's position.
  pub fn set_position(&self, x: impl Into<f64>, y: impl Into<f64>) -> crate::Result<()> {
    self.window.dispatcher.set_position(x.into(), y.into())
  }

  /// Determines if this window should be fullscreen.
  pub fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()> {
    self.window.dispatcher.set_fullscreen(fullscreen)
  }

  /// Sets this window' icon.
  pub fn set_icon(&self, icon: Icon) -> crate::Result<()> {
    self.window.dispatcher.set_icon(icon.try_into()?)
  }

  pub(crate) fn verify_salt(&self, salt: String) -> bool {
    self.manager.verify_salt(salt)
  }
}
