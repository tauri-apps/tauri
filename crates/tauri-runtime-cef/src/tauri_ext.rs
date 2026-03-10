//! Extension traits for [`tauri`] types when using the [`CefRuntime`].

use crate::{CefRuntime, DevToolsProtocol, RuntimeInitAttribute, RuntimeStyle, WebviewAtribute};
use tauri_runtime::Result;

/// Extension trait for [`tauri::Builder`] when using the [`CefRuntime`].
pub trait BuilderCefExt {
  /// Command line arguments passed to CEF at startup.
  fn command_line_args(self, args: Vec<(String, Option<String>)>) -> Self;
}

impl BuilderCefExt for tauri::Builder<CefRuntime<tauri::EventLoopMessage>> {
  fn command_line_args(mut self, args: Vec<(String, Option<String>)>) -> Self {
    self.platform_specific_attribute(RuntimeInitAttribute::CommandLineArgs { args });
    self
  }
}

/// Extension trait for [`tauri::WebviewBuilder`] when using the [`CefRuntime`].
pub trait WebviewBuilderCefExt {
  /// Sets the browser runtime style (Alloy or Chrome).
  fn browser_runtime_style(self, style: RuntimeStyle) -> Self;
}

impl WebviewBuilderCefExt for tauri::WebviewBuilder<CefRuntime<tauri::EventLoopMessage>> {
  fn browser_runtime_style(mut self, style: RuntimeStyle) -> Self {
    self.platform_specific_attribute(WebviewAtribute::RuntimeStyle { style });
    self
  }
}

/// Extension trait for [`tauri::WebviewWindowBuilder`] when using the [`CefRuntime`].
pub trait WebviewWindowBuilderCefExt {
  /// Sets the browser runtime style (Alloy or Chrome).
  fn browser_runtime_style(self, style: RuntimeStyle) -> Self;

  /// Create a browser window (CEF browser without native window chrome).
  fn browser_window(self) -> Self;
}

impl<'a, M: tauri::Manager<CefRuntime<tauri::EventLoopMessage>>> WebviewWindowBuilderCefExt
  for tauri::WebviewWindowBuilder<'a, CefRuntime<tauri::EventLoopMessage>, M>
{
  fn browser_runtime_style(mut self, style: RuntimeStyle) -> Self {
    self
      .webview_builder_mut()
      .platform_specific_attribute(WebviewAtribute::RuntimeStyle { style });
    self
  }

  fn browser_window(mut self) -> Self {
    self
      .window_builder_mut()
      .inner_window_builder_mut()
      .set_browser_window(true);
    self
  }
}

/// Extension trait for [`tauri::Webview`] when using the [`CefRuntime`].
pub trait WebviewCefExt {
  /// Send a message to the DevTools agent.
  fn send_dev_tools_message(&self, message: &[u8]) -> Result<()>;

  /// Register a callback to receive DevTools protocol messages.
  fn on_dev_tools_protocol<F: Fn(DevToolsProtocol) + Send + Sync + 'static>(
    &self,
    f: F,
  ) -> Result<()>;
}

impl WebviewCefExt for tauri::Webview<CefRuntime<tauri::EventLoopMessage>> {
  fn send_dev_tools_message(&self, message: &[u8]) -> Result<()> {
    self.dispatcher().send_dev_tools_message(message)
  }

  fn on_dev_tools_protocol<F: Fn(DevToolsProtocol) + Send + Sync + 'static>(
    &self,
    f: F,
  ) -> Result<()> {
    self.dispatcher().on_dev_tools_protocol(f)
  }
}

/// Extension trait for [`tauri::WebviewWindow`] when using the [`CefRuntime`].
pub trait WebviewWindowCefExt {
  /// Send a message to the DevTools agent.
  fn send_dev_tools_message(&self, message: &[u8]) -> Result<()>;

  /// Register a callback to receive DevTools protocol messages.
  fn on_dev_tools_protocol<F: Fn(DevToolsProtocol) + Send + Sync + 'static>(
    &self,
    f: F,
  ) -> Result<()>;
}

impl WebviewWindowCefExt for tauri::WebviewWindow<CefRuntime<tauri::EventLoopMessage>> {
  fn send_dev_tools_message(&self, message: &[u8]) -> Result<()> {
    self.dispatcher().send_dev_tools_message(message)
  }

  fn on_dev_tools_protocol<F: Fn(DevToolsProtocol) + Send + Sync + 'static>(
    &self,
    f: F,
  ) -> Result<()> {
    self.dispatcher().on_dev_tools_protocol(f)
  }
}
