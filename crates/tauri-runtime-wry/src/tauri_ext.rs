//! Extension traits for [`tauri`] types when using the [`Wry`] runtime.

use crate::{Message, PluginBuilder, TaoWindowBuilder, WebviewAttribute, Wry};
use std::sync::Weak;
use tao::window::Window;
#[cfg(target_os = "ios")]
use tauri::Manager;
use tauri_runtime::Result;
#[cfg(target_os = "ios")]
use wry::WebViewExtIOS;

/// Extension trait for [`tauri::AppHandle`] when using the [`Wry`] runtime.
pub trait AppHandleWryExt {
  /// Creates a new tao window using a callback, and returns its window id.
  fn create_tao_window<F: FnOnce() -> (String, TaoWindowBuilder) + Send + 'static>(
    &self,
    f: F,
  ) -> Result<Weak<Window>>;

  /// Send a message to the event loop.
  fn send_tao_event(&self, message: Message<tauri::EventLoopMessage>) -> Result<()>;
}

impl AppHandleWryExt for tauri::AppHandle<Wry<tauri::EventLoopMessage>> {
  fn create_tao_window<F: FnOnce() -> (String, TaoWindowBuilder) + Send + 'static>(
    &self,
    f: F,
  ) -> Result<Weak<Window>> {
    self.runtime_handle().create_tao_window(f)
  }

  fn send_tao_event(&self, message: Message<tauri::EventLoopMessage>) -> Result<()> {
    self.runtime_handle().send_event(message)
  }
}

/// Extension trait for [`tauri::App`] when using the [`Wry`] runtime.
pub trait AppWryExt {
  /// Registers a wry plugin.
  fn wry_plugin<P: PluginBuilder<tauri::EventLoopMessage> + 'static>(&mut self, plugin: P)
  where
    <P as PluginBuilder<tauri::EventLoopMessage>>::Plugin: Send;
}

impl AppWryExt for tauri::App<Wry<tauri::EventLoopMessage>> {
  fn wry_plugin<P: PluginBuilder<tauri::EventLoopMessage> + 'static>(&mut self, plugin: P)
  where
    <P as PluginBuilder<tauri::EventLoopMessage>>::Plugin: Send,
  {
    self.handle_mut().runtime_handle_mut().plugin(plugin);
  }
}

/// Extension trait for [`tauri::WebviewBuilder`] when using the [`Wry`] runtime.
pub trait WebviewBuilderWryExt {
  /// Set the environment for the webview (Windows only).
  #[cfg(windows)]
  fn with_environment(
    self,
    environment: webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment,
  ) -> Self;

  /// Creates a new webview sharing the same web process with the provided webview (Linux only).
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
  ))]
  fn with_related_view(self, related_view: webkit2gtk::WebView) -> Self;

  /// Set the webview configuration (macOS only).
  #[cfg(target_os = "macos")]
  fn with_webview_configuration(
    self,
    configuration: objc2::rc::Retained<objc2_web_kit::WKWebViewConfiguration>,
  ) -> Self;
}

impl WebviewBuilderWryExt for tauri::WebviewBuilder<Wry<tauri::EventLoopMessage>> {
  #[cfg(windows)]
  fn with_environment(
    mut self,
    environment: webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment,
  ) -> Self {
    self.platform_specific_attribute(WebviewAttribute::Environment(environment));
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
    self.platform_specific_attribute(WebviewAttribute::RelatedView(related_view));
    self
  }

  #[cfg(target_os = "macos")]
  fn with_webview_configuration(
    mut self,
    configuration: objc2::rc::Retained<objc2_web_kit::WKWebViewConfiguration>,
  ) -> Self {
    self.platform_specific_attribute(WebviewAttribute::WebviewConfiguration(configuration));
    self
  }
}

/// Extension trait for [`tauri::WebviewWindowBuilder`] when using the [`Wry`] runtime.
pub trait WebviewWindowBuilderWryExt {
  /// Set the environment for the webview (Windows only).
  #[cfg(windows)]
  fn with_environment(
    self,
    environment: webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment,
  ) -> Self;

  /// Creates a new webview sharing the same web process with the provided webview (Linux only).
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
  ))]
  fn with_related_view(self, related_view: webkit2gtk::WebView) -> Self;

  /// Set the webview configuration (macOS only).
  #[cfg(target_os = "macos")]
  fn with_webview_configuration(
    self,
    configuration: objc2::rc::Retained<objc2_web_kit::WKWebViewConfiguration>,
  ) -> Self;
}

impl<'a, M: tauri::Manager<Wry<tauri::EventLoopMessage>>> WebviewWindowBuilderWryExt
  for tauri::WebviewWindowBuilder<'a, Wry<tauri::EventLoopMessage>, M>
{
  #[cfg(windows)]
  fn with_environment(
    mut self,
    environment: webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment,
  ) -> Self {
    self
      .webview_builder_mut()
      .platform_specific_attribute(WebviewAttribute::Environment(environment));
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
    self
      .webview_builder_mut()
      .platform_specific_attribute(WebviewAttribute::RelatedView(related_view));
    self
  }

  #[cfg(target_os = "macos")]
  fn with_webview_configuration(
    mut self,
    configuration: objc2::rc::Retained<objc2_web_kit::WKWebViewConfiguration>,
  ) -> Self {
    self
      .webview_builder_mut()
      .platform_specific_attribute(WebviewAttribute::WebviewConfiguration(configuration));
    self
  }
}
