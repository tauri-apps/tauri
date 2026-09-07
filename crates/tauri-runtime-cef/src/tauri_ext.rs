// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Extension traits exposing CEF-specific APIs on [`tauri`] types.
//!
//! The traits are implemented for the statically typed [`CefRuntime`](crate::CefRuntime)
//! and for the type-erased [`tauri::DynRuntime`]. With the latter, the methods fail with
//! [`tauri_runtime::Error::RuntimeTypeMismatch`] when the application is not running on CEF.

use std::sync::Arc;

use tauri::{EventLoopMessage, Manager, Runtime, Webview, WebviewWindow};
use tauri_runtime::dynamic::{DynWebviewAttributes, DynWebviewDispatcher, DynWindowOpener};

use crate::{
  CefWebviewAttributes, CefWebviewDispatcher, ConsoleMessage, DevToolsProtocol, FrameEvent,
  NewWindowOpener, RuntimeStyle,
};

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

/// Window openers that may expose the CEF [`NewWindowOpener`].
///
/// Lets a new window handler read the CEF popup source regardless of the runtime generic in use:
///
/// ```rust,no_run
/// use tauri::{WebviewUrl, WebviewWindowBuilder, webview::NewWindowResponse};
/// use tauri_runtime_cef::AsCefWindowOpener;
///
/// tauri::Builder::default()
///   .runtime(tauri_runtime_cef::Cef::default())
///   .setup(|app| {
///     WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
///       .on_new_window(|url, features| {
///         if let Some(opener) = features.opener().as_cef_window_opener() {
///           println!("{url} was opened by {:?}", opener.source_url());
///         }
///         NewWindowResponse::Allow
///       })
///       .build()?;
///     Ok(())
///   });
/// ```
pub trait AsCefWindowOpener {
  /// Returns the CEF window opener, `None` when the opener belongs to another runtime.
  fn as_cef_window_opener(&self) -> Option<&NewWindowOpener>;
}

impl AsCefWindowOpener for NewWindowOpener {
  fn as_cef_window_opener(&self) -> Option<&NewWindowOpener> {
    Some(self)
  }
}

impl AsCefWindowOpener for DynWindowOpener {
  fn as_cef_window_opener(&self) -> Option<&NewWindowOpener> {
    self.downcast_ref()
  }
}

/// Runtime webview attributes that may expose the [`CefWebviewAttributes`].
pub trait AsCefWebviewAttributes {
  /// Returns the CEF attributes, `None` when the attributes belong to another runtime.
  fn as_cef_webview_attributes_mut(&mut self) -> Option<&mut CefWebviewAttributes>;
}

impl AsCefWebviewAttributes for CefWebviewAttributes {
  fn as_cef_webview_attributes_mut(&mut self) -> Option<&mut CefWebviewAttributes> {
    Some(self)
  }
}

impl AsCefWebviewAttributes for DynWebviewAttributes {
  fn as_cef_webview_attributes_mut(&mut self) -> Option<&mut CefWebviewAttributes> {
    self.get_or_default()
  }
}

/// Modifies the CEF attributes of a webview builder, if the builder's attributes are not of another runtime.
fn with_cef_webview_attributes<A: AsCefWebviewAttributes>(
  attributes: &mut A,
  f: impl FnOnce(&mut CefWebviewAttributes),
) {
  match attributes.as_cef_webview_attributes_mut() {
    Some(attributes) => f(attributes),
    None => log::warn!(
      "ignoring the CEF webview attributes: attributes of another runtime were already set on the webview builder"
    ),
  }
}

/// CEF-specific APIs of [`tauri::Webview`] and [`tauri::WebviewWindow`].
pub trait WebviewCefExt {
  /// Send a message to the DevTools agent. The message should be a UTF-8 encoded JSON
  /// string following the Chrome DevTools Protocol format.
  ///
  /// Callers share one native request identifier space on this browser, so the
  /// message's `id` must come from
  /// [`allocate_devtools_message_id`](crate::allocate_devtools_message_id).
  /// A hardcoded or self-incremented `id` can collide with a request another
  /// caller already sent, which consumes that producer's
  /// [`DevToolsProtocol::MethodResult`]. The runtime's own requests are issued
  /// from a reserved range the public allocator never returns, so they cannot
  /// be consumed this way.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::Manager;
  /// use tauri_runtime_cef::{WebviewCefExt, allocate_devtools_message_id};
  ///
  /// tauri::Builder::default()
  ///   .runtime(tauri_runtime_cef::Cef::default())
  ///   .setup(|app| {
  ///     let webview = app.get_webview_window("main").unwrap();
  ///     // Enable Page domain to receive page lifecycle events
  ///     let message_id = allocate_devtools_message_id()?;
  ///     let msg = format!(r#"{{"id":{message_id},"method":"Page.enable","params":{{}}}}"#);
  ///     webview.send_dev_tools_message(msg.as_bytes())?;
  ///     Ok(())
  ///   });
  /// ```
  fn send_dev_tools_message(&self, message: &[u8]) -> Result<()>;

  /// Register a callback to receive DevTools protocol messages. Messages include
  /// both method results and events from the DevTools agent.
  ///
  /// The callback observes the whole browser, including requests the runtime and
  /// other callers sent. Match [`DevToolsProtocol::MethodResult`] against an
  /// identifier obtained from
  /// [`allocate_devtools_message_id`](crate::allocate_devtools_message_id)
  /// instead of assuming every result belongs to this observer.
  ///
  /// It is scoped to this webview's own native browser, so a CEF-owned popup is
  /// a separate browser whose protocol traffic — its page content, its network
  /// activity and its dialog messages — is never reported here; observe popups
  /// through [`Webview::popups`](crate::Webview::popups).
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::Manager;
  /// use tauri_runtime_cef::{DevToolsProtocol, WebviewCefExt, allocate_devtools_message_id};
  ///
  /// tauri::Builder::default()
  ///   .runtime(tauri_runtime_cef::Cef::default())
  ///   .setup(|app| {
  ///     let webview = app.get_webview_window("main").unwrap();
  ///     let message_id = allocate_devtools_message_id()?;
  ///     webview.on_dev_tools_protocol(move |protocol| {
  ///       match protocol {
  ///         DevToolsProtocol::Message(msg) => {
  ///           if let Ok(s) = std::str::from_utf8(&msg) {
  ///             println!("DevTools message: {}", s);
  ///           }
  ///         }
  ///         DevToolsProtocol::Event { method, params } => {
  ///           println!("DevTools event: {} {:?}", method, params);
  ///         }
  ///         // Only this result answers the request sent below.
  ///         DevToolsProtocol::MethodResult { message_id: id, success, .. } if id == message_id => {
  ///           println!("Page.enable success={}", success);
  ///         }
  ///         DevToolsProtocol::MethodResult { .. } => {}
  ///       }
  ///     })?;
  ///     let msg = format!(r#"{{"id":{message_id},"method":"Page.enable","params":{{}}}}"#);
  ///     webview.send_dev_tools_message(msg.as_bytes())?;
  ///     Ok(())
  ///   });
  /// ```
  fn on_dev_tools_protocol<F: Fn(DevToolsProtocol) + Send + Sync + 'static>(
    &self,
    f: F,
  ) -> Result<()>;

  /// Executes a closure with the CEF platform webview handle, on the CEF UI thread.
  ///
  /// See [`crate::Webview`] for the native state it exposes, which is sampled
  /// immediately before the closure runs and is not refreshed afterwards.
  fn with_cef_webview<F: FnOnce(&crate::Webview) + Send + 'static>(&self, f: F) -> Result<()>;
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

  fn with_cef_webview<F: FnOnce(&crate::Webview) + Send + 'static>(&self, f: F) -> Result<()> {
    if self.dispatcher().as_cef_webview_dispatcher().is_none() {
      return Err(not_cef());
    }
    self.with_webview(move |webview| {
      if let Some(webview) = webview.downcast_ref::<crate::Webview>() {
        f(webview)
      }
    })
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

  fn with_cef_webview<F: FnOnce(&crate::Webview) + Send + 'static>(&self, f: F) -> Result<()> {
    self.as_ref().with_cef_webview(f)
  }
}

/// CEF-specific APIs of [`tauri::WebviewWindowBuilder`].
pub trait WebviewWindowBuilderCefExt {
  /// Sets the browser runtime style.
  ///
  /// See [`RuntimeStyle`] for more information.
  #[must_use]
  fn browser_runtime_style(self, style: RuntimeStyle) -> Self;

  /// Observes native CEF lifecycle events for main and child frames.
  ///
  /// The callback runs synchronously on CEF's UI thread. It must return
  /// promptly and must not wait for an event-loop operation. This observer
  /// does not replace the navigation policy configured by `on_navigation`.
  /// It is scoped to this webview's own native browser, so a CEF-owned popup
  /// is a separate browser that is never reported here — observe popups
  /// through [`Webview::popups`](crate::Webview::popups).
  #[must_use]
  fn on_frame_event<F: Fn(FrameEvent) + Send + Sync + 'static>(self, handler: F) -> Self;

  /// Observes the messages the renderer writes to the JavaScript console,
  /// without DevTools having to be open.
  ///
  /// The callback runs synchronously on CEF's UI thread. It must return
  /// promptly and must not wait for an event-loop operation. Observing a
  /// message does not suppress it: CEF logs it as it normally would.
  /// It is scoped to this webview's own native browser, so a CEF-owned popup
  /// is a separate browser that is never reported here, and neither is a
  /// DevTools window opened on this webview.
  #[must_use]
  fn on_console_message<F: Fn(ConsoleMessage) + Send + Sync + 'static>(self, handler: F) -> Self;
}

impl<'a, R: Runtime, M: Manager<R>> WebviewWindowBuilderCefExt
  for tauri::WebviewWindowBuilder<'a, R, M>
where
  R::RuntimeWebviewAttributes: AsCefWebviewAttributes,
{
  fn browser_runtime_style(mut self, style: RuntimeStyle) -> Self {
    with_cef_webview_attributes(self.runtime_specific_attributes_mut(), |attributes| {
      attributes.runtime_style = Some(style);
    });
    self
  }

  fn on_frame_event<F: Fn(FrameEvent) + Send + Sync + 'static>(mut self, handler: F) -> Self {
    let handler = Arc::new(handler);
    with_cef_webview_attributes(self.runtime_specific_attributes_mut(), |attributes| {
      attributes.frame_event_handler = Some(handler);
    });
    self
  }

  fn on_console_message<F: Fn(ConsoleMessage) + Send + Sync + 'static>(
    mut self,
    handler: F,
  ) -> Self {
    let handler = Arc::new(handler);
    with_cef_webview_attributes(self.runtime_specific_attributes_mut(), |attributes| {
      attributes.console_message_handler = Some(handler);
    });
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

  /// Observes native CEF lifecycle events for main and child frames.
  ///
  /// The callback runs synchronously on CEF's UI thread. It must return
  /// promptly and must not wait for an event-loop operation. This observer
  /// does not replace the navigation policy configured by `on_navigation`.
  /// It is scoped to this webview's own native browser, so a CEF-owned popup
  /// is a separate browser that is never reported here — observe popups
  /// through [`Webview::popups`](crate::Webview::popups).
  #[must_use]
  fn on_frame_event<F: Fn(FrameEvent) + Send + Sync + 'static>(self, handler: F) -> Self;

  /// Observes the messages the renderer writes to the JavaScript console,
  /// without DevTools having to be open.
  ///
  /// The callback runs synchronously on CEF's UI thread. It must return
  /// promptly and must not wait for an event-loop operation. Observing a
  /// message does not suppress it: CEF logs it as it normally would.
  /// It is scoped to this webview's own native browser, so a CEF-owned popup
  /// is a separate browser that is never reported here, and neither is a
  /// DevTools window opened on this webview.
  #[must_use]
  fn on_console_message<F: Fn(ConsoleMessage) + Send + Sync + 'static>(self, handler: F) -> Self;
}

#[cfg(feature = "unstable")]
impl<R: Runtime> WebviewBuilderCefExt for tauri::webview::WebviewBuilder<R>
where
  R::RuntimeWebviewAttributes: AsCefWebviewAttributes,
{
  fn browser_runtime_style(mut self, style: RuntimeStyle) -> Self {
    with_cef_webview_attributes(self.runtime_specific_attributes_mut(), |attributes| {
      attributes.runtime_style = Some(style);
    });
    self
  }

  fn on_frame_event<F: Fn(FrameEvent) + Send + Sync + 'static>(mut self, handler: F) -> Self {
    let handler = Arc::new(handler);
    with_cef_webview_attributes(self.runtime_specific_attributes_mut(), |attributes| {
      attributes.frame_event_handler = Some(handler);
    });
    self
  }

  fn on_console_message<F: Fn(ConsoleMessage) + Send + Sync + 'static>(
    mut self,
    handler: F,
  ) -> Self {
    let handler = Arc::new(handler);
    with_cef_webview_attributes(self.runtime_specific_attributes_mut(), |attributes| {
      attributes.console_message_handler = Some(handler);
    });
    self
  }
}
