// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A layer between raw [`Runtime`] webview windows and Tauri.

use crate::{
  api::config::WindowConfig,
  event::{Event, EventHandler},
  hooks::{InvokeMessage, InvokeResolver, PageLoadPayload},
  runtime::{
    tag::ToJsString,
    webview::{FileDropHandler, InvokePayload, WebviewAttributes, WebviewRpcHandler},
    Dispatch, Runtime,
  },
  sealed::{ManagerBase, RuntimeOrDispatch},
  Icon, Manager, Params, WindowBuilder,
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::hash::{Hash, Hasher};

/// A webview window that has yet to be built.
pub struct PendingWindow<M: Params> {
  /// The label that the window will be named.
  pub label: M::Label,

  /// The [`WindowBuilder`] that the window will be created with.
  pub window_attributes: <<M::Runtime as Runtime>::Dispatcher as Dispatch>::WindowBuilder,

  /// The [`WebviewAttributes`] that the webview will be created with.
  pub webview_attributes: WebviewAttributes,

  /// How to handle RPC calls on the webview window.
  pub rpc_handler: Option<WebviewRpcHandler<M>>,

  /// How to handle a file dropping onto the webview window.
  pub file_drop_handler: Option<FileDropHandler<M>>,

  /// The resolved URL to load on the webview.
  pub url: String,
}

impl<M: Params> PendingWindow<M> {
  /// Create a new [`PendingWindow`] with a label and starting url.
  pub fn new(
    window_attributes: <<M::Runtime as Runtime>::Dispatcher as Dispatch>::WindowBuilder,
    webview_attributes: WebviewAttributes,
    label: M::Label,
  ) -> Self {
    Self {
      window_attributes,
      webview_attributes,
      label,
      rpc_handler: None,
      file_drop_handler: None,
      url: "tauri://localhost".to_string(),
    }
  }

  /// Create a new [`PendingWindow`] from a [`WindowConfig`] with a label and starting url.
  pub fn with_config(
    window_config: WindowConfig,
    webview_attributes: WebviewAttributes,
    label: M::Label,
  ) -> Self {
    Self {
      window_attributes:
        <<<M::Runtime as Runtime>::Dispatcher as Dispatch>::WindowBuilder>::with_config(
          window_config,
        ),
      webview_attributes,
      label,
      rpc_handler: None,
      file_drop_handler: None,
      url: "tauri://localhost".to_string(),
    }
  }
}

/// A webview window that is not yet managed by Tauri.
pub struct DetachedWindow<M: Params> {
  /// Name of the window
  pub label: M::Label,

  /// The [`Dispatch`](crate::runtime::Dispatch) associated with the window.
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

/// We want to export the runtime related window at the crate root, but not look like a re-export.
pub(crate) mod export {
  use super::*;
  use crate::command::FromCommand;
  use crate::runtime::{manager::WindowManager, tag::TagRef};
  use std::borrow::Borrow;

  /// A webview window managed by Tauri.
  ///
  /// This type also implements [`Manager`] which allows you to manage other windows attached to
  /// the same application.
  ///
  /// TODO: expand these docs since this is a pretty important type
  pub struct Window<P: Params> {
    /// The webview window created by the runtime.
    window: DetachedWindow<P>,

    /// The manager to associate this webview window with.
    manager: WindowManager<P>,
  }

  impl<M: Params> Clone for Window<M> {
    fn clone(&self) -> Self {
      Self {
        window: self.window.clone(),
        manager: self.manager.clone(),
      }
    }
  }

  impl<P: Params> Hash for Window<P> {
    /// Only use the [`Window`]'s label to represent its hash.
    fn hash<H: Hasher>(&self, state: &mut H) {
      self.window.label.hash(state)
    }
  }

  impl<P: Params> Eq for Window<P> {}
  impl<P: Params> PartialEq for Window<P> {
    /// Only use the [`Window`]'s label to compare equality.
    fn eq(&self, other: &Self) -> bool {
      self.window.label.eq(&other.window.label)
    }
  }

  impl<P: Params> Manager<P> for Window<P> {}
  impl<P: Params> ManagerBase<P> for Window<P> {
    fn manager(&self) -> &WindowManager<P> {
      &self.manager
    }

    fn runtime(&mut self) -> RuntimeOrDispatch<'_, P> {
      RuntimeOrDispatch::Dispatch(self.dispatcher())
    }
  }

  impl<'de, P: Params> FromCommand<'de, P> for Window<P> {
    fn from_command(
      _: &'de str,
      _: &'de str,
      message: &'de InvokeMessage<P>,
    ) -> Result<Self, serde_json::Error> {
      Ok(message.window())
    }
  }

  impl<P: Params> Window<P> {
    /// Create a new window that is attached to the manager.
    pub(crate) fn new(manager: WindowManager<P>, window: DetachedWindow<P>) -> Self {
      Self { window, manager }
    }

    /// The current window's dispatcher.
    pub(crate) fn dispatcher(&self) -> <P::Runtime as Runtime>::Dispatcher {
      self.window.dispatcher.clone()
    }

    /// How to handle this window receiving an [`InvokeMessage`].
    pub(crate) fn on_message(self, command: String, payload: InvokePayload) -> crate::Result<()> {
      let manager = self.manager.clone();
      match command.as_str() {
        "__initialized" => {
          let payload: PageLoadPayload = serde_json::from_value(payload.inner)?;
          manager.run_on_page_load(self, payload);
        }
        _ => {
          let message = InvokeMessage::new(
            self.clone(),
            manager.state(),
            command.to_string(),
            payload.inner,
          );
          let resolver =
            InvokeResolver::new(self, payload.main_thread, payload.callback, payload.error);
          if let Some(module) = &payload.tauri_module {
            let module = module.to_string();
            crate::endpoints::handle(
              module,
              message,
              resolver,
              manager.config(),
              manager.package_info(),
            );
          } else if command.starts_with("plugin:") {
            manager.extend_api(message, resolver);
          } else {
            manager.run_invoke_handler(message, resolver);
          }
        }
      }

      Ok(())
    }

    /// The label of this window.
    pub fn label(&self) -> &P::Label {
      &self.window.label
    }

    pub(crate) fn emit_internal<E: ?Sized, S>(
      &self,
      event: &E,
      payload: Option<S>,
    ) -> crate::Result<()>
    where
      P::Event: Borrow<E>,
      E: TagRef<P::Event>,
      S: Serialize,
    {
      let js_payload = match payload {
        Some(payload_value) => serde_json::to_value(payload_value)?,
        None => JsonValue::Null,
      };

      self.eval(&format!(
        "window['{}']({{event: {}, payload: {}}}, '{}')",
        self.manager.event_emit_function_name(),
        event.to_js_string()?,
        js_payload,
        self.manager.generate_salt(),
      ))?;

      Ok(())
    }

    /// Emits an event to the current window.
    pub fn emit<E: ?Sized, S>(&self, event: &E, payload: Option<S>) -> crate::Result<()>
    where
      P::Event: Borrow<E>,
      E: TagRef<P::Event>,
      S: Serialize,
    {
      self.emit_internal(event, payload)
    }

    /// Emits an event on all windows except this one.
    pub fn emit_others<E: ?Sized, S>(&self, event: &E, payload: Option<S>) -> crate::Result<()>
    where
      P::Event: Borrow<E>,
      E: TagRef<P::Event>,
      S: Serialize + Clone,
    {
      self.manager.emit_filter(event, payload, |w| w != self)
    }

    /// Listen to an event on this window.
    pub fn listen<E: Into<P::Event>, F>(&self, event: E, handler: F) -> EventHandler
    where
      F: Fn(Event) + Send + 'static,
    {
      let label = self.window.label.clone();
      self.manager.listen(event.into(), Some(label), handler)
    }

    /// Listen to a an event on this window a single time.
    pub fn once<E: Into<P::Event>, F>(&self, event: E, handler: F) -> EventHandler
    where
      F: Fn(Event) + Send + 'static,
    {
      let label = self.window.label.clone();
      self.manager.once(event.into(), Some(label), handler)
    }

    /// Triggers an event on this window.
    pub fn trigger<E: ?Sized>(&self, event: &E, data: Option<String>)
    where
      P::Event: Borrow<E>,
      E: TagRef<P::Event>,
    {
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
      self.window.dispatcher.set_icon(icon)
    }

    /// Starts dragging the window.
    pub fn start_dragging(&self) -> crate::Result<()> {
      self.window.dispatcher.start_dragging()
    }

    pub(crate) fn verify_salt(&self, salt: String) -> bool {
      self.manager.verify_salt(salt)
    }
  }
}
