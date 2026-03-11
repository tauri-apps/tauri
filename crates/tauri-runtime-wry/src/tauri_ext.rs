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

/// Extension trait for [`tauri::plugin::PluginApi`] when using the [`Wry`] runtime on iOS.
#[cfg(target_os = "ios")]
pub trait PluginApiWryExt {
  /// Registers an iOS plugin.
  fn register_ios_plugin(
    &self,
    init_fn: unsafe fn() -> *const std::ffi::c_void,
  ) -> std::result::Result<
    tauri::plugin::PluginHandle<Wry<tauri::EventLoopMessage>>,
    tauri::plugin::mobile::PluginInvokeError,
  >;
}

#[cfg(target_os = "ios")]
impl<C: serde::de::DeserializeOwned> PluginApiWryExt
  for tauri::plugin::PluginApi<Wry<tauri::EventLoopMessage>, C>
{
  fn register_ios_plugin(
    &self,
    init_fn: unsafe fn() -> *const std::ffi::c_void,
  ) -> std::result::Result<
    tauri::plugin::PluginHandle<Wry<tauri::EventLoopMessage>>,
    tauri::plugin::mobile::PluginInvokeError,
  > {
    use std::sync::mpsc::channel;

    let name = self.name();
    let raw_config = self.raw_config().clone();

    if let Some(webview) = self.app().webviews().values().next() {
      let (tx, rx) = channel();
      let name_ = name;
      let config_str = serde_json::to_string(&*raw_config).unwrap();
      webview
        .with_webview(move |w| {
          if let Ok(w) = w.downcast::<wry::WebView>() {
            unsafe {
              tauri::ios::register_plugin(
                &name_.into(),
                init_fn(),
                &config_str.as_str().into(),
                objc2::rc::Retained::into_raw(w.webview()) as *mut objc2::runtime::AnyObject
                  as *mut std::ffi::c_void,
              );
            }
          }
          let _ = tx.send(());
        })
        .map_err(|_| tauri::plugin::mobile::PluginInvokeError::UnreachableWebview)?;
      rx.recv().unwrap();
    } else {
      let config_str = serde_json::to_string(&*raw_config).unwrap();
      unsafe {
        tauri::ios::register_plugin(
          &name.into(),
          init_fn(),
          &config_str.as_str().into(),
          std::ptr::null(),
        );
      }
    }
    Ok(tauri::plugin::PluginHandle::new(name, self.app().clone()))
  }
}
