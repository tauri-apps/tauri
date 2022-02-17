// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Internal runtime between Tauri and the underlying webview runtime.

#![cfg_attr(doc_cfg, feature(doc_cfg))]

use serde::Deserialize;
use std::{fmt::Debug, path::PathBuf, sync::mpsc::Sender};
use uuid::Uuid;

#[cfg(windows)]
use windows::Win32::Foundation::HWND;

pub mod http;
/// Create window and system tray menus.
pub mod menu;
/// Types useful for interacting with a user's monitors.
pub mod monitor;
pub mod webview;
pub mod window;

use monitor::Monitor;
use webview::WindowBuilder;
use window::{
  dpi::{PhysicalPosition, PhysicalSize, Position, Size},
  DetachedWindow, PendingWindow, WindowEvent,
};

use crate::http::{
  header::{InvalidHeaderName, InvalidHeaderValue},
  method::InvalidMethod,
  status::InvalidStatusCode,
  InvalidUri,
};

#[cfg(feature = "system-tray")]
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct SystemTray {
  pub icon: Option<Icon>,
  pub menu: Option<menu::SystemTrayMenu>,
  #[cfg(target_os = "macos")]
  pub icon_as_template: bool,
}

#[cfg(feature = "system-tray")]
impl SystemTray {
  /// Creates a new system tray that only renders an icon.
  pub fn new() -> Self {
    Default::default()
  }

  pub fn menu(&self) -> Option<&menu::SystemTrayMenu> {
    self.menu.as_ref()
  }

  /// Sets the tray icon. Must be a [`Icon::File`] on Linux and a [`Icon::Raw`] on Windows and macOS.
  #[must_use]
  pub fn with_icon(mut self, icon: Icon) -> Self {
    self.icon.replace(icon);
    self
  }

  /// Sets the tray icon as template.
  #[cfg(target_os = "macos")]
  #[must_use]
  pub fn with_icon_as_template(mut self, is_template: bool) -> Self {
    self.icon_as_template = is_template;
    self
  }

  /// Sets the menu to show when the system tray is right clicked.
  #[must_use]
  pub fn with_menu(mut self, menu: menu::SystemTrayMenu) -> Self {
    self.menu.replace(menu);
    self
  }
}

/// Type of user attention requested on a window.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(tag = "type")]
pub enum UserAttentionType {
  /// ## Platform-specific
  /// - **macOS:** Bounces the dock icon until the application is in focus.
  /// - **Windows:** Flashes both the window and the taskbar button until the application is in focus.
  Critical,
  /// ## Platform-specific
  /// - **macOS:** Bounces the dock icon once.
  /// - **Windows:** Flashes the taskbar button until the application is in focus.
  Informational,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
  /// Failed to create webview.
  #[error("failed to create webview: {0}")]
  CreateWebview(Box<dyn std::error::Error + Send>),
  /// Failed to create window.
  #[error("failed to create window")]
  CreateWindow,
  /// Failed to send message to webview.
  #[error("failed to send message to the webview")]
  FailedToSendMessage,
  /// Failed to receive message from webview.
  #[error("failed to receive message from webview")]
  FailedToReceiveMessage,
  /// Failed to serialize/deserialize.
  #[error("JSON error: {0}")]
  Json(#[from] serde_json::Error),
  /// Encountered an error creating the app system tray.
  #[cfg(feature = "system-tray")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
  #[error("error encountered during tray setup: {0}")]
  SystemTray(Box<dyn std::error::Error + Send>),
  /// Failed to load window icon.
  #[error("invalid icon: {0}")]
  InvalidIcon(Box<dyn std::error::Error + Send>),
  /// Failed to get monitor on window operation.
  #[error("failed to get monitor")]
  FailedToGetMonitor,
  /// Global shortcut error.
  #[error(transparent)]
  GlobalShortcut(Box<dyn std::error::Error + Send>),
  #[error("Invalid header name: {0}")]
  InvalidHeaderName(#[from] InvalidHeaderName),
  #[error("Invalid header value: {0}")]
  InvalidHeaderValue(#[from] InvalidHeaderValue),
  #[error("Invalid uri: {0}")]
  InvalidUri(#[from] InvalidUri),
  #[error("Invalid status code: {0}")]
  InvalidStatusCode(#[from] InvalidStatusCode),
  #[error("Invalid method: {0}")]
  InvalidMethod(#[from] InvalidMethod),
  #[error("Infallible error, something went really wrong: {0}")]
  Infallible(#[from] std::convert::Infallible),
}

/// Result type.
pub type Result<T> = std::result::Result<T, Error>;

/// A icon definition.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Icon {
  /// Icon from file path.
  File(PathBuf),
  /// Icon from raw bytes.
  Raw(Vec<u8>),
}

impl Icon {
  /// Converts the icon to a the expected system tray format.
  /// We expect the code that passes the Icon enum to have already checked the platform.
  #[cfg(target_os = "linux")]
  pub fn into_tray_icon(self) -> PathBuf {
    match self {
      Icon::File(path) => path,
      Icon::Raw(_) => {
        panic!("linux requires the system menu icon to be a file path, not bytes.")
      }
    }
  }

  /// Converts the icon to a the expected system tray format.
  /// We expect the code that passes the Icon enum to have already checked the platform.
  #[cfg(not(target_os = "linux"))]
  pub fn into_tray_icon(self) -> Vec<u8> {
    match self {
      Icon::Raw(bytes) => bytes,
      Icon::File(_) => {
        panic!("non-linux system menu icons must be bytes, not a file path.")
      }
    }
  }
}

/// Event triggered on the event loop run.
#[non_exhaustive]
pub enum RunEvent {
  /// Event loop is exiting.
  Exit,
  /// Event loop is about to exit
  ExitRequested {
    /// Label of the last window managed by the runtime.
    window_label: String,
    tx: Sender<ExitRequestedEventAction>,
  },
  /// Window close was requested by the user.
  CloseRequested {
    /// The window label.
    label: String,
    /// A signal sender. If a `true` value is emitted, the window won't be closed.
    signal_tx: Sender<bool>,
  },
  /// Window closed.
  WindowClose(String),
  /// Application ready.
  Ready,
  /// Sent if the event loop is being resumed.
  Resumed,
  /// Emitted when all of the event loop’s input events have been processed and redraw processing is about to begin.
  ///
  /// This event is useful as a place to put your code that should be run after all state-changing events have been handled and you want to do stuff (updating state, performing calculations, etc) that happens as the “main body” of your event loop.
  MainEventsCleared,
}

/// Action to take when the event loop is about to exit
#[derive(Debug)]
pub enum ExitRequestedEventAction {
  /// Prevent the event loop from exiting
  Prevent,
}

/// A system tray event.
#[derive(Debug)]
pub enum SystemTrayEvent {
  MenuItemClick(u16),
  LeftClick {
    position: PhysicalPosition<f64>,
    size: PhysicalSize<f64>,
  },
  RightClick {
    position: PhysicalPosition<f64>,
    size: PhysicalSize<f64>,
  },
  DoubleClick {
    position: PhysicalPosition<f64>,
    size: PhysicalSize<f64>,
  },
}

/// Metadata for a runtime event loop iteration on `run_iteration`.
#[derive(Debug, Clone, Default)]
pub struct RunIteration {
  pub window_count: usize,
}

/// Application's activation policy. Corresponds to NSApplicationActivationPolicy.
#[cfg(target_os = "macos")]
#[cfg_attr(doc_cfg, doc(cfg(target_os = "macos")))]
#[non_exhaustive]
pub enum ActivationPolicy {
  /// Corresponds to NSApplicationActivationPolicyRegular.
  Regular,
  /// Corresponds to NSApplicationActivationPolicyAccessory.
  Accessory,
  /// Corresponds to NSApplicationActivationPolicyProhibited.
  Prohibited,
}

/// A [`Send`] handle to the runtime.
pub trait RuntimeHandle: Debug + Clone + Send + Sync + Sized + 'static {
  type Runtime: Runtime<Handle = Self>;
  /// Create a new webview window.
  fn create_window(
    &self,
    pending: PendingWindow<Self::Runtime>,
  ) -> crate::Result<DetachedWindow<Self::Runtime>>;

  /// Run a task on the main thread.
  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> crate::Result<()>;

  #[cfg(all(windows, feature = "system-tray"))]
  #[cfg_attr(doc_cfg, doc(cfg(all(windows, feature = "system-tray"))))]
  fn remove_system_tray(&self) -> crate::Result<()>;
}

/// A global shortcut manager.
pub trait GlobalShortcutManager: Debug + Clone + Send + Sync {
  /// Whether the application has registered the given `accelerator`.
  fn is_registered(&self, accelerator: &str) -> crate::Result<bool>;

  /// Register a global shortcut of `accelerator`.
  fn register<F: Fn() + Send + 'static>(
    &mut self,
    accelerator: &str,
    handler: F,
  ) -> crate::Result<()>;

  /// Unregister all accelerators registered by the manager instance.
  fn unregister_all(&mut self) -> crate::Result<()>;

  /// Unregister the provided `accelerator`.
  fn unregister(&mut self, accelerator: &str) -> crate::Result<()>;
}

/// Clipboard manager.
pub trait ClipboardManager: Debug + Clone + Send + Sync {
  /// Writes the text into the clipboard as plain text.
  fn write_text<T: Into<String>>(&mut self, text: T) -> Result<()>;
  /// Read the content in the clipboard as plain text.
  fn read_text(&self) -> Result<Option<String>>;
}

/// The webview runtime interface.
pub trait Runtime: Sized + 'static {
  /// The message dispatcher.
  type Dispatcher: Dispatch<Runtime = Self>;
  /// The runtime handle type.
  type Handle: RuntimeHandle<Runtime = Self>;
  /// The global shortcut manager type.
  type GlobalShortcutManager: GlobalShortcutManager;
  /// The clipboard manager type.
  type ClipboardManager: ClipboardManager;
  /// The tray handler type.
  #[cfg(feature = "system-tray")]
  type TrayHandler: menu::TrayHandle;

  /// Creates a new webview runtime. Must be used on the main thread.
  fn new() -> crate::Result<Self>;

  /// Creates a new webview runtime on any thread.
  #[cfg(any(windows, target_os = "linux"))]
  #[cfg_attr(doc_cfg, doc(cfg(any(windows, target_os = "linux"))))]
  fn new_any_thread() -> crate::Result<Self>;

  /// Gets a runtime handle.
  fn handle(&self) -> Self::Handle;

  /// Gets the global shortcut manager.
  fn global_shortcut_manager(&self) -> Self::GlobalShortcutManager;

  /// Gets the clipboard manager.
  fn clipboard_manager(&self) -> Self::ClipboardManager;

  /// Create a new webview window.
  fn create_window(&self, pending: PendingWindow<Self>) -> crate::Result<DetachedWindow<Self>>;

  /// Adds the icon to the system tray with the specified menu items.
  #[cfg(feature = "system-tray")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
  fn system_tray(&self, system_tray: SystemTray) -> crate::Result<Self::TrayHandler>;

  /// Registers a system tray event handler.
  #[cfg(feature = "system-tray")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
  fn on_system_tray_event<F: Fn(&SystemTrayEvent) + Send + 'static>(&mut self, f: F) -> Uuid;

  /// Sets the activation policy for the application. It is set to `NSApplicationActivationPolicyRegular` by default.
  #[cfg(target_os = "macos")]
  #[cfg_attr(doc_cfg, doc(cfg(target_os = "macos")))]
  fn set_activation_policy(&mut self, activation_policy: ActivationPolicy);

  /// Runs the one step of the webview runtime event loop and returns control flow to the caller.
  fn run_iteration<F: Fn(RunEvent) + 'static>(&mut self, callback: F) -> RunIteration;

  /// Run the webview runtime.
  fn run<F: FnMut(RunEvent) + 'static>(self, callback: F);
}

/// Webview dispatcher. A thread-safe handle to the webview API.
pub trait Dispatch: Debug + Clone + Send + Sync + Sized + 'static {
  /// The runtime this [`Dispatch`] runs under.
  type Runtime: Runtime;

  /// The winoow builder type.
  type WindowBuilder: WindowBuilder + Clone;

  /// Run a task on the main thread.
  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> crate::Result<()>;

  /// Registers a window event handler.
  fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) -> Uuid;

  /// Registers a window event handler.
  fn on_menu_event<F: Fn(&window::MenuEvent) + Send + 'static>(&self, f: F) -> Uuid;

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn open_devtools(&self);

  // GETTERS

  /// Returns the scale factor that can be used to map logical pixels to physical pixels, and vice versa.
  fn scale_factor(&self) -> crate::Result<f64>;

  /// Returns the position of the top-left hand corner of the window's client area relative to the top-left hand corner of the desktop.
  fn inner_position(&self) -> crate::Result<PhysicalPosition<i32>>;

  /// Returns the position of the top-left hand corner of the window relative to the top-left hand corner of the desktop.
  fn outer_position(&self) -> crate::Result<PhysicalPosition<i32>>;

  /// Returns the physical size of the window's client area.
  ///
  /// The client area is the content of the window, excluding the title bar and borders.
  fn inner_size(&self) -> crate::Result<PhysicalSize<u32>>;

  /// Returns the physical size of the entire window.
  ///
  /// These dimensions include the title bar and borders. If you don't want that (and you usually don't), use inner_size instead.
  fn outer_size(&self) -> crate::Result<PhysicalSize<u32>>;

  /// Gets the window's current fullscreen state.
  fn is_fullscreen(&self) -> crate::Result<bool>;

  /// Gets the window's current maximized state.
  fn is_maximized(&self) -> crate::Result<bool>;

  /// Gets the window’s current decoration state.
  fn is_decorated(&self) -> crate::Result<bool>;

  /// Gets the window’s current resizable state.
  fn is_resizable(&self) -> crate::Result<bool>;

  /// Gets the window's current vibility state.
  fn is_visible(&self) -> crate::Result<bool>;

  /// Gets the window menu current visibility state.
  fn is_menu_visible(&self) -> crate::Result<bool>;

  /// Returns the monitor on which the window currently resides.
  ///
  /// Returns None if current monitor can't be detected.
  fn current_monitor(&self) -> crate::Result<Option<Monitor>>;

  /// Returns the primary monitor of the system.
  ///
  /// Returns None if it can't identify any monitor as a primary one.
  fn primary_monitor(&self) -> crate::Result<Option<Monitor>>;

  /// Returns the list of all the monitors available on the system.
  fn available_monitors(&self) -> crate::Result<Vec<Monitor>>;

  /// Returns the native handle that is used by this window.
  #[cfg(windows)]
  fn hwnd(&self) -> crate::Result<HWND>;

  /// Returns the native handle that is used by this window.
  #[cfg(target_os = "macos")]
  fn ns_window(&self) -> crate::Result<*mut std::ffi::c_void>;

  /// Returns the `ApplicatonWindow` from gtk crate that is used by this window.
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn gtk_window(&self) -> crate::Result<gtk::ApplicationWindow>;

  // SETTERS

  /// Centers the window.
  fn center(&self) -> crate::Result<()>;

  /// Opens the dialog to prints the contents of the webview.
  fn print(&self) -> crate::Result<()>;

  /// Requests user attention to the window.
  ///
  /// Providing `None` will unset the request for user attention.
  fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> crate::Result<()>;

  /// Create a new webview window.
  fn create_window(
    &mut self,
    pending: PendingWindow<Self::Runtime>,
  ) -> crate::Result<DetachedWindow<Self::Runtime>>;

  /// Updates the window resizable flag.
  fn set_resizable(&self, resizable: bool) -> crate::Result<()>;

  /// Updates the window title.
  fn set_title<S: Into<String>>(&self, title: S) -> crate::Result<()>;

  /// Maximizes the window.
  fn maximize(&self) -> crate::Result<()>;

  /// Unmaximizes the window.
  fn unmaximize(&self) -> crate::Result<()>;

  /// Minimizes the window.
  fn minimize(&self) -> crate::Result<()>;

  /// Unminimizes the window.
  fn unminimize(&self) -> crate::Result<()>;

  /// Shows the window menu.
  fn show_menu(&self) -> crate::Result<()>;

  /// Hides the window menu.
  fn hide_menu(&self) -> crate::Result<()>;

  /// Shows the window.
  fn show(&self) -> crate::Result<()>;

  /// Hides the window.
  fn hide(&self) -> crate::Result<()>;

  /// Closes the window.
  fn close(&self) -> crate::Result<()>;

  /// Updates the hasDecorations flag.
  fn set_decorations(&self, decorations: bool) -> crate::Result<()>;

  /// Updates the window alwaysOnTop flag.
  fn set_always_on_top(&self, always_on_top: bool) -> crate::Result<()>;

  /// Resizes the window.
  fn set_size(&self, size: Size) -> crate::Result<()>;

  /// Updates the window min size.
  fn set_min_size(&self, size: Option<Size>) -> crate::Result<()>;

  /// Updates the window max size.
  fn set_max_size(&self, size: Option<Size>) -> crate::Result<()>;

  /// Updates the window position.
  fn set_position(&self, position: Position) -> crate::Result<()>;

  /// Updates the window fullscreen state.
  fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()>;

  /// Bring the window to front and focus.
  fn set_focus(&self) -> crate::Result<()>;

  /// Updates the window icon.
  fn set_icon(&self, icon: Icon) -> crate::Result<()>;

  /// Whether to show the window icon in the task bar or not.
  fn set_skip_taskbar(&self, skip: bool) -> crate::Result<()>;

  /// Starts dragging the window.
  fn start_dragging(&self) -> crate::Result<()>;

  /// Executes javascript on the window this [`Dispatch`] represents.
  fn eval_script<S: Into<String>>(&self, script: S) -> crate::Result<()>;

  /// Applies the specified `update` to the menu item associated with the given `id`.
  fn update_menu_item(&self, id: u16, update: menu::MenuUpdate) -> crate::Result<()>;
}
