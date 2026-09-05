// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Extension traits exposing CEF-specific APIs on [`tauri`] types.
//!
//! The traits are implemented for the statically typed [`CefRuntime`](crate::CefRuntime)
//! and for the type-erased [`tauri::DynRuntime`]. With the latter, the methods fail with
//! [`tauri_runtime::Error::RuntimeTypeMismatch`] when the application is not running on CEF.

use tauri::{EventLoopMessage, Manager, Runtime, Webview, WebviewWindow};
use tauri_runtime::dynamic::{DynWebviewAttribute, DynWebviewDispatcher};

use crate::{CefWebviewDispatcher, DevToolsProtocol, RuntimeStyle, WebviewAtribute};

type Result<T> = std::result::Result<T, tauri::Error>;

fn not_cef() -> tauri::Error {
  tauri_runtime::Error::RuntimeTypeMismatch(
    "the application is not running on the CEF runtime".into(),
  )
  .into()
}

/// Webview dispatchers that may expose the underlying [`CefWebviewDispatcher`].
pub trait AsCefWebviewDispatcher {
  /// Returns the CEF webview dispatcher, if the runtime is CEF.
  fn as_cef_webview_dispatcher(&self) -> Option<&CefWebviewDispatcher<EventLoopMessage>>;
}

impl AsCefWebviewDispatcher for CefWebviewDispatcher<EventLoopMessage> {
  fn as_cef_webview_dispatcher(&self) -> Option<&CefWebviewDispatcher<EventLoopMessage>> {
    Some(self)
  }
}

impl AsCefWebviewDispatcher for DynWebviewDispatcher<EventLoopMessage> {
  fn as_cef_webview_dispatcher(&self) -> Option<&CefWebviewDispatcher<EventLoopMessage>> {
    self.downcast_ref()
  }
}

/// Webview attribute types that can carry a CEF [`WebviewAtribute`].
pub trait FromCefWebviewAttribute {
  /// Wraps the CEF attribute.
  fn from_cef(attribute: WebviewAtribute) -> Self;
}

impl FromCefWebviewAttribute for WebviewAtribute {
  fn from_cef(attribute: WebviewAtribute) -> Self {
    attribute
  }
}

impl FromCefWebviewAttribute for DynWebviewAttribute {
  fn from_cef(attribute: WebviewAtribute) -> Self {
    DynWebviewAttribute::new(attribute)
  }
}

/// CEF-specific APIs of [`tauri::Webview`] and [`tauri::WebviewWindow`].
pub trait WebviewCefExt {
  /// Send a message to the DevTools agent. The message should be a UTF-8 encoded JSON
  /// string following the Chrome DevTools Protocol format.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::Manager;
  /// use tauri_runtime_cef::WebviewCefExt;
  ///
  /// tauri::Builder::default()
  ///   .runtime(tauri_runtime_cef::Cef::default())
  ///   .setup(|app| {
  ///     let webview = app.get_webview_window("main").unwrap();
  ///     // Enable Page domain to receive page lifecycle events
  ///     let msg = br#"{"id":1,"method":"Page.enable","params":{}}"#;
  ///     webview.send_dev_tools_message(msg)?;
  ///     Ok(())
  ///   });
  /// ```
  fn send_dev_tools_message(&self, message: &[u8]) -> Result<()>;

  /// Register a callback to receive DevTools protocol messages. Messages include
  /// both method results and events from the DevTools agent.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::Manager;
  /// use tauri_runtime_cef::{DevToolsProtocol, WebviewCefExt};
  ///
  /// tauri::Builder::default()
  ///   .runtime(tauri_runtime_cef::Cef::default())
  ///   .setup(|app| {
  ///     let webview = app.get_webview_window("main").unwrap();
  ///     webview.on_dev_tools_protocol(|protocol| {
  ///       match protocol {
  ///         DevToolsProtocol::Message(msg) => {
  ///           if let Ok(s) = std::str::from_utf8(&msg) {
  ///             println!("DevTools message: {}", s);
  ///           }
  ///         }
  ///         DevToolsProtocol::Event { method, params } => {
  ///           println!("DevTools event: {} {:?}", method, params);
  ///         }
  ///         DevToolsProtocol::MethodResult { message_id, success, result } => {
  ///           println!("DevTools result: id={} success={}", message_id, success);
  ///         }
  ///       }
  ///     })?;
  ///     Ok(())
  ///   });
  /// ```
  fn on_dev_tools_protocol<F: Fn(DevToolsProtocol) + Send + Sync + 'static>(
    &self,
    f: F,
  ) -> Result<()>;
}

impl<R: Runtime> WebviewCefExt for Webview<R>
where
  R::WebviewDispatcher: AsCefWebviewDispatcher,
{
  fn send_dev_tools_message(&self, message: &[u8]) -> Result<()> {
    self
      .dispatcher()
      .as_cef_webview_dispatcher()
      .ok_or_else(not_cef)?
      .send_dev_tools_message(message)
      .map_err(Into::into)
  }

  fn on_dev_tools_protocol<F: Fn(DevToolsProtocol) + Send + Sync + 'static>(
    &self,
    f: F,
  ) -> Result<()> {
    self
      .dispatcher()
      .as_cef_webview_dispatcher()
      .ok_or_else(not_cef)?
      .on_dev_tools_protocol(f)
      .map_err(Into::into)
  }
}

impl<R: Runtime> WebviewCefExt for WebviewWindow<R>
where
  R::WebviewDispatcher: AsCefWebviewDispatcher,
{
  fn send_dev_tools_message(&self, message: &[u8]) -> Result<()> {
    self.as_ref().send_dev_tools_message(message)
  }

  fn on_dev_tools_protocol<F: Fn(DevToolsProtocol) + Send + Sync + 'static>(
    &self,
    f: F,
  ) -> Result<()> {
    self.as_ref().on_dev_tools_protocol(f)
  }
}

/// CEF-specific APIs of [`tauri::WebviewWindowBuilder`].
pub trait WebviewWindowBuilderCefExt {
  /// Sets the browser runtime style.
  ///
  /// See [`RuntimeStyle`] for more information.
  #[must_use]
  fn browser_runtime_style(self, style: RuntimeStyle) -> Self;
}

impl<'a, R: Runtime, M: Manager<R>> WebviewWindowBuilderCefExt
  for tauri::WebviewWindowBuilder<'a, R, M>
where
  R::PlatformSpecificWebviewAttribute: FromCefWebviewAttribute,
{
  fn browser_runtime_style(mut self, style: RuntimeStyle) -> Self {
    self.platform_specific_attribute(FromCefWebviewAttribute::from_cef(
      WebviewAtribute::RuntimeStyle { style },
    ));
    self
  }
}

/// CEF-specific APIs of [`tauri::webview::WebviewBuilder`].
#[cfg(feature = "unstable")]
pub trait WebviewBuilderCefExt {
  /// Sets the browser runtime style.
  ///
  /// See [`RuntimeStyle`] for more information.
  #[must_use]
  fn browser_runtime_style(self, style: RuntimeStyle) -> Self;
}

#[cfg(feature = "unstable")]
impl<R: Runtime> WebviewBuilderCefExt for tauri::webview::WebviewBuilder<R>
where
  R::PlatformSpecificWebviewAttribute: FromCefWebviewAttribute,
{
  fn browser_runtime_style(mut self, style: RuntimeStyle) -> Self {
    self.platform_specific_attribute(FromCefWebviewAttribute::from_cef(
      WebviewAtribute::RuntimeStyle { style },
    ));
    self
  }
}
