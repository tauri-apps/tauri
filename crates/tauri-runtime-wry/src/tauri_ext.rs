// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Extension traits exposing wry-specific APIs on [`tauri`] types.
//!
//! The traits are implemented for the statically typed [`WryRuntime`](crate::WryRuntime)
//! and for the type-erased [`tauri::DynRuntime`]. With the latter, the methods fail with
//! [`tauri_runtime::Error::RuntimeTypeMismatch`] when the application is not running on wry.

use std::sync::Weak;

use tao::window::Window;
use tauri::{App, AppHandle, EventLoopMessage, Manager, Runtime, Webview, WebviewWindow};
use tauri_runtime::dynamic::{DynRuntimeHandle, DynWebviewAttribute};

use crate::{
  Message, PluginBuilder, TaoWindowBuilder, TaoWindowId, WebviewAttribute, WindowMessage, WryHandle,
};

type Result<T> = std::result::Result<T, tauri::Error>;

fn not_wry() -> tauri::Error {
  tauri_runtime::Error::RuntimeTypeMismatch(
    "the application is not running on the wry runtime".into(),
  )
  .into()
}

/// Runtime handles that may expose the underlying [`WryHandle`].
pub trait AsWryHandle {
  /// Returns the wry handle, if the runtime is wry.
  fn as_wry_handle(&self) -> Option<&WryHandle<EventLoopMessage>>;
}

impl AsWryHandle for WryHandle<EventLoopMessage> {
  fn as_wry_handle(&self) -> Option<&WryHandle<EventLoopMessage>> {
    Some(self)
  }
}

impl AsWryHandle for DynRuntimeHandle<EventLoopMessage> {
  fn as_wry_handle(&self) -> Option<&WryHandle<EventLoopMessage>> {
    self.downcast_ref()
  }
}

/// Webview attribute types that can carry a wry [`WebviewAttribute`].
pub trait FromWryWebviewAttribute {
  /// Wraps the wry attribute.
  fn from_wry(attribute: WebviewAttribute) -> Self;
}

impl FromWryWebviewAttribute for WebviewAttribute {
  fn from_wry(attribute: WebviewAttribute) -> Self {
    attribute
  }
}

impl FromWryWebviewAttribute for DynWebviewAttribute {
  fn from_wry(attribute: WebviewAttribute) -> Self {
    DynWebviewAttribute::new(attribute)
  }
}

/// wry-specific APIs of [`tauri::AppHandle`].
pub trait AppHandleWryExt {
  /// Create a new tao window using a callback. The event loop must be running at this point.
  fn create_tao_window<F: FnOnce() -> (String, TaoWindowBuilder) + Send + 'static>(
    &self,
    f: F,
  ) -> Result<Weak<Window>>;

  /// Sends a window message to the event loop.
  fn send_tao_window_event(&self, window_id: TaoWindowId, message: WindowMessage) -> Result<()>;

  /// Adds a [`Plugin`](crate::Plugin) using its [`PluginBuilder`].
  ///
  /// # Stability
  ///
  /// This API is unstable.
  fn wry_plugin<P: PluginBuilder<EventLoopMessage> + 'static>(&self, plugin: P) -> Result<()>
  where
    <P as PluginBuilder<EventLoopMessage>>::Plugin: Send;
}

impl<R: Runtime> AppHandleWryExt for AppHandle<R>
where
  R::Handle: AsWryHandle,
{
  fn create_tao_window<F: FnOnce() -> (String, TaoWindowBuilder) + Send + 'static>(
    &self,
    f: F,
  ) -> Result<Weak<Window>> {
    self
      .runtime_handle()
      .as_wry_handle()
      .ok_or_else(not_wry)?
      .create_tao_window(f)
      .map_err(Into::into)
  }

  fn send_tao_window_event(&self, window_id: TaoWindowId, message: WindowMessage) -> Result<()> {
    let handle = self.runtime_handle().as_wry_handle().ok_or_else(not_wry)?;
    handle
      .send_event(Message::Window(handle.window_id(window_id), message))
      .map_err(Into::into)
  }

  fn wry_plugin<P: PluginBuilder<EventLoopMessage> + 'static>(&self, plugin: P) -> Result<()>
  where
    <P as PluginBuilder<EventLoopMessage>>::Plugin: Send,
  {
    self
      .runtime_handle()
      .as_wry_handle()
      .ok_or_else(not_wry)?
      .plugin(plugin);
    Ok(())
  }
}

/// wry-specific APIs of [`tauri::App`].
pub trait AppWryExt {
  /// Adds a [`Plugin`](crate::Plugin) using its [`PluginBuilder`].
  ///
  /// # Stability
  ///
  /// This API is unstable.
  fn wry_plugin<P: PluginBuilder<EventLoopMessage> + 'static>(&self, plugin: P) -> Result<()>
  where
    <P as PluginBuilder<EventLoopMessage>>::Plugin: Send;
}

impl<R: Runtime> AppWryExt for App<R>
where
  R::Handle: AsWryHandle,
{
  fn wry_plugin<P: PluginBuilder<EventLoopMessage> + 'static>(&self, plugin: P) -> Result<()>
  where
    <P as PluginBuilder<EventLoopMessage>>::Plugin: Send,
  {
    self.handle().wry_plugin(plugin)
  }
}

/// wry-specific APIs of [`tauri::Webview`] and [`tauri::WebviewWindow`].
pub trait WebviewWryExt {
  /// Executes a closure with the wry platform webview handle, on the main thread.
  ///
  /// See [`crate::Webview`] for the platform-specific APIs it exposes.
  fn with_wry_webview<F: FnOnce(&crate::Webview) + Send + 'static>(&self, f: F) -> Result<()>;
}

impl<R: Runtime> WebviewWryExt for Webview<R>
where
  R::Handle: AsWryHandle,
{
  fn with_wry_webview<F: FnOnce(&crate::Webview) + Send + 'static>(&self, f: F) -> Result<()> {
    if self.app_handle().runtime_handle().as_wry_handle().is_none() {
      return Err(not_wry());
    }
    self.with_webview(move |webview| {
      if let Some(webview) = webview.downcast_ref::<crate::Webview>() {
        f(webview)
      }
    })
  }
}

impl<R: Runtime> WebviewWryExt for WebviewWindow<R>
where
  R::Handle: AsWryHandle,
{
  fn with_wry_webview<F: FnOnce(&crate::Webview) + Send + 'static>(&self, f: F) -> Result<()> {
    self.as_ref().with_wry_webview(f)
  }
}

macro_rules! webview_builder_ext {
  ($(#[$meta:meta])* $trait:ident) => {
    $(#[$meta])*
    pub trait $trait {
      /// Set the environment for the webview.
      /// Useful if you need to share the same environment, for instance when using `on_new_window`.
      #[cfg(windows)]
      #[must_use]
      fn with_environment(
        self,
        environment: webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment,
      ) -> Self;

      /// Creates a new webview sharing the same web process with the provided webview.
      /// Useful if you need to link a webview to another, for instance when using `on_new_window`.
      #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
      ))]
      #[must_use]
      fn with_related_view(self, related_view: webkit2gtk::WebView) -> Self;

      /// Set the webview configuration.
      /// Useful if you need to use a predefined webview configuration, for instance when using `on_new_window`.
      #[cfg(target_os = "macos")]
      #[must_use]
      fn with_webview_configuration(
        self,
        webview_configuration: objc2::rc::Retained<objc2_web_kit::WKWebViewConfiguration>,
      ) -> Self;
    }
  };
}

macro_rules! webview_builder_ext_impl {
  ($trait:ident, $($impl_header:tt)*) => {
    $($impl_header)* {
      #[cfg(windows)]
      fn with_environment(
        mut self,
        environment: webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment,
      ) -> Self {
        self.platform_specific_attribute(FromWryWebviewAttribute::from_wry(
          WebviewAttribute::Environment(environment),
        ));
        self
      }

      #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
      ))]
      fn with_related_view(mut self, related_view: webkit2gtk::WebView) -> Self {
        self.platform_specific_attribute(FromWryWebviewAttribute::from_wry(
          WebviewAttribute::RelatedView(related_view),
        ));
        self
      }

      #[cfg(target_os = "macos")]
      fn with_webview_configuration(
        mut self,
        webview_configuration: objc2::rc::Retained<objc2_web_kit::WKWebViewConfiguration>,
      ) -> Self {
        self.platform_specific_attribute(FromWryWebviewAttribute::from_wry(
          WebviewAttribute::WebviewConfiguration(webview_configuration),
        ));
        self
      }
    }
  };
}

webview_builder_ext!(
  /// wry-specific APIs of [`tauri::WebviewWindowBuilder`].
  WebviewWindowBuilderWryExt
);

webview_builder_ext_impl!(
  WebviewWindowBuilderWryExt,
  impl<'a, R: Runtime, M: Manager<R>> WebviewWindowBuilderWryExt for tauri::WebviewWindowBuilder<'a, R, M>
  where
    R::PlatformSpecificWebviewAttribute: FromWryWebviewAttribute,
);

webview_builder_ext!(
  /// wry-specific APIs of [`tauri::webview::WebviewBuilder`].
  #[cfg(feature = "unstable")]
  WebviewBuilderWryExt
);

#[cfg(feature = "unstable")]
webview_builder_ext_impl!(
  WebviewBuilderWryExt,
  impl<R: Runtime> WebviewBuilderWryExt for tauri::webview::WebviewBuilder<R>
  where
    R::PlatformSpecificWebviewAttribute: FromWryWebviewAttribute,
);
