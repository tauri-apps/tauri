// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Internal runtime between Tauri and the underlying webview runtime.
//!
//! None of the exposed API of this crate is stable, and it may break semver
//! compatibility in the future. The major version only signifies the intended Tauri version.

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use raw_window_handle::DisplayHandle;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Debug, sync::mpsc::Sender};
use tauri_utils::config::Color;
use tauri_utils::Theme;
use url::Url;
use webview::{DetachedWebview, PendingWebview};

/// Types useful for interacting with a user's monitors.
pub mod monitor;
pub mod webview;
pub mod window;

use dpi::{PhysicalPosition, PhysicalSize, Position, Size};
use monitor::Monitor;
use window::{
  CursorIcon, DetachedWindow, PendingWindow, RawWindow, WebviewEvent, WindowEvent,
  WindowSizeConstraints,
};
use window::{WindowBuilder, WindowId};

use http::{
  header::{InvalidHeaderName, InvalidHeaderValue},
  method::InvalidMethod,
  status::InvalidStatusCode,
};

/// UI scaling utilities.
pub use dpi;

pub type WindowEventId = u32;
pub type WebviewEventId = u32;

/// A rectangular region.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Rect {
  /// Rect position.
  pub position: dpi::Position,
  /// Rect size.
  pub size: dpi::Size,
}

impl Default for Rect {
  fn default() -> Self {
    Self {
      position: Position::Logical((0, 0).into()),
      size: Size::Logical((0, 0).into()),
    }
  }
}

/// Progress bar status.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProgressBarStatus {
  /// Hide progress bar.
  None,
  /// Normal state.
  Normal,
  /// Indeterminate state. **Treated as Normal on Linux and macOS**
  Indeterminate,
  /// Paused state. **Treated as Normal on Linux**
  Paused,
  /// Error state. **Treated as Normal on Linux**
  Error,
}

/// Progress Bar State
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressBarState {
  /// The progress bar status.
  pub status: Option<ProgressBarStatus>,
  /// The progress bar progress. This can be a value ranging from `0` to `100`
  pub progress: Option<u64>,
  /// The `.desktop` filename with the Unity desktop window manager, for example `myapp.desktop` **Linux Only**
  pub desktop_filename: Option<String>,
}

/// Type of user attention requested on a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(tag = "type")]
pub enum DeviceEventFilter {
  /// Always filter out device events.
  Always,
  /// Filter out device events while the window is not focused.
  Unfocused,
  /// Report all device events regardless of window focus.
  Never,
}

impl Default for DeviceEventFilter {
  fn default() -> Self {
    Self::Unfocused
  }
}

/// Defines the orientation that a window resize will be performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ResizeDirection {
  East,
  North,
  NorthEast,
  NorthWest,
  South,
  SouthEast,
  SouthWest,
  West,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
  /// Failed to create webview.
  #[error("failed to create webview: {0}")]
  CreateWebview(Box<dyn std::error::Error + Send + Sync>),
  /// Failed to create window.
  #[error("failed to create window")]
  CreateWindow,
  /// The given window label is invalid.
  #[error("Window labels must only include alphanumeric characters, `-`, `/`, `:` and `_`.")]
  InvalidWindowLabel,
  /// Failed to send message to webview.
  #[error("failed to send message to the webview")]
  FailedToSendMessage,
  /// Failed to receive message from webview.
  #[error("failed to receive message from webview")]
  FailedToReceiveMessage,
  /// Failed to serialize/deserialize.
  #[error("JSON error: {0}")]
  Json(#[from] serde_json::Error),
  /// Failed to load window icon.
  #[error("invalid icon: {0}")]
  InvalidIcon(Box<dyn std::error::Error + Send + Sync>),
  /// Failed to get monitor on window operation.
  #[error("failed to get monitor")]
  FailedToGetMonitor,
  /// Failed to get cursor position.
  #[error("failed to get cursor position")]
  FailedToGetCursorPosition,
  #[error("Invalid header name: {0}")]
  InvalidHeaderName(#[from] InvalidHeaderName),
  #[error("Invalid header value: {0}")]
  InvalidHeaderValue(#[from] InvalidHeaderValue),
  #[error("Invalid status code: {0}")]
  InvalidStatusCode(#[from] InvalidStatusCode),
  #[error("Invalid method: {0}")]
  InvalidMethod(#[from] InvalidMethod),
  #[error("Infallible error, something went really wrong: {0}")]
  Infallible(#[from] std::convert::Infallible),
  #[error("the event loop has been closed")]
  EventLoopClosed,
  #[error("Invalid proxy url")]
  InvalidProxyUrl,
  #[error("window not found")]
  WindowNotFound,
}

/// Result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Window icon.
#[derive(Debug, Clone)]
pub struct Icon<'a> {
  /// RGBA bytes of the icon.
  pub rgba: Cow<'a, [u8]>,
  /// Icon width.
  pub width: u32,
  /// Icon height.
  pub height: u32,
}

/// A type that can be used as an user event.
pub trait UserEvent: Debug + Clone + Send + 'static {}

impl<T: Debug + Clone + Send + 'static> UserEvent for T {}

/// Event triggered on the event loop run.
#[derive(Debug)]
#[non_exhaustive]
pub enum RunEvent<T: UserEvent> {
  /// Event loop is exiting.
  Exit,
  /// Event loop is about to exit
  ExitRequested {
    /// The exit code.
    code: Option<i32>,
    tx: Sender<ExitRequestedEventAction>,
  },
  /// An event associated with a window.
  WindowEvent {
    /// The window label.
    label: String,
    /// The detailed event.
    event: WindowEvent,
  },
  /// An event associated with a webview.
  WebviewEvent {
    /// The webview label.
    label: String,
    /// The detailed event.
    event: WebviewEvent,
  },
  /// Application ready.
  Ready,
  /// Sent if the event loop is being resumed.
  Resumed,
  /// Emitted when all of the event loop's input events have been processed and redraw processing is about to begin.
  ///
  /// This event is useful as a place to put your code that should be run after all state-changing events have been handled and you want to do stuff (updating state, performing calculations, etc) that happens as the “main body” of your event loop.
  MainEventsCleared,
  /// Emitted when the user wants to open the specified resource with the app.
  #[cfg(any(target_os = "macos", target_os = "ios"))]
  Opened { urls: Vec<url::Url> },
  /// Emitted when the NSApplicationDelegate's applicationShouldHandleReopen gets called
  #[cfg(target_os = "macos")]
  Reopen {
    /// Indicates whether the NSApplication object found any visible windows in your application.
    has_visible_windows: bool,
  },
  /// A custom event defined by the user.
  UserEvent(T),
}

/// Action to take when the event loop is about to exit
#[derive(Debug)]
pub enum ExitRequestedEventAction {
  /// Prevent the event loop from exiting
  Prevent,
}

/// Application's activation policy. Corresponds to NSApplicationActivationPolicy.
#[cfg(target_os = "macos")]
#[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
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
pub trait RuntimeHandle<T: UserEvent>: Debug + Clone + Send + Sync + Sized + 'static {
  type Runtime: Runtime<T, Handle = Self>;

  /// Creates an `EventLoopProxy` that can be used to dispatch user events to the main event loop.
  fn create_proxy(&self) -> <Self::Runtime as Runtime<T>>::EventLoopProxy;

  /// Sets the activation policy for the application.
  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn set_activation_policy(&self, activation_policy: ActivationPolicy) -> Result<()>;

  /// Requests an exit of the event loop.
  fn request_exit(&self, code: i32) -> Result<()>;

  /// Create a new window.
  fn create_window<F: Fn(RawWindow) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self::Runtime>,
    before_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>>;

  /// Create a new webview.
  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>>;

  /// Run a task on the main thread.
  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()>;

  fn display_handle(&self) -> std::result::Result<DisplayHandle, raw_window_handle::HandleError>;

  fn primary_monitor(&self) -> Option<Monitor>;
  fn monitor_from_point(&self, x: f64, y: f64) -> Option<Monitor>;
  fn available_monitors(&self) -> Vec<Monitor>;

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>>;

  fn set_theme(&self, theme: Option<Theme>);

  /// Shows the application, but does not automatically focus it.
  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn show(&self) -> Result<()>;

  /// Hides the application.
  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn hide(&self) -> Result<()>;

  /// Finds an Android class in the project scope.
  #[cfg(target_os = "android")]
  fn find_class<'a>(
    &self,
    env: &mut jni::JNIEnv<'a>,
    activity: &jni::objects::JObject<'_>,
    name: impl Into<String>,
  ) -> std::result::Result<jni::objects::JClass<'a>, jni::errors::Error>;

  /// Dispatch a closure to run on the Android context.
  ///
  /// The closure takes the JNI env, the Android activity instance and the possibly null webview.
  #[cfg(target_os = "android")]
  fn run_on_android_context<F>(&self, f: F)
  where
    F: FnOnce(&mut jni::JNIEnv, &jni::objects::JObject, &jni::objects::JObject) + Send + 'static;
}

pub trait EventLoopProxy<T: UserEvent>: Debug + Clone + Send + Sync {
  fn send_event(&self, event: T) -> Result<()>;
}

#[derive(Default)]
pub struct RuntimeInitArgs {
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  pub app_id: Option<String>,
  #[cfg(windows)]
  pub msg_hook: Option<Box<dyn FnMut(*const std::ffi::c_void) -> bool + 'static>>,
}

/// The webview runtime interface.
pub trait Runtime<T: UserEvent>: Debug + Sized + 'static {
  /// The window message dispatcher.
  type WindowDispatcher: WindowDispatch<T, Runtime = Self>;
  /// The webview message dispatcher.
  type WebviewDispatcher: WebviewDispatch<T, Runtime = Self>;
  /// The runtime handle type.
  type Handle: RuntimeHandle<T, Runtime = Self>;
  /// The proxy type.
  type EventLoopProxy: EventLoopProxy<T>;

  /// Creates a new webview runtime. Must be used on the main thread.
  fn new(args: RuntimeInitArgs) -> Result<Self>;

  /// Creates a new webview runtime on any thread.
  #[cfg(any(windows, target_os = "linux"))]
  #[cfg_attr(docsrs, doc(cfg(any(windows, target_os = "linux"))))]
  fn new_any_thread(args: RuntimeInitArgs) -> Result<Self>;

  /// Creates an `EventLoopProxy` that can be used to dispatch user events to the main event loop.
  fn create_proxy(&self) -> Self::EventLoopProxy;

  /// Gets a runtime handle.
  fn handle(&self) -> Self::Handle;

  /// Create a new window.
  fn create_window<F: Fn(RawWindow) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self>>;

  /// Create a new webview.
  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self>,
  ) -> Result<DetachedWebview<T, Self>>;

  fn primary_monitor(&self) -> Option<Monitor>;
  fn monitor_from_point(&self, x: f64, y: f64) -> Option<Monitor>;
  fn available_monitors(&self) -> Vec<Monitor>;

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>>;

  fn set_theme(&self, theme: Option<Theme>);

  /// Sets the activation policy for the application.
  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn set_activation_policy(&mut self, activation_policy: ActivationPolicy);

  /// Shows the application, but does not automatically focus it.
  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn show(&self);

  /// Hides the application.
  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  fn hide(&self);

  /// Change the device event filter mode.
  ///
  /// Since the DeviceEvent capture can lead to high CPU usage for unfocused windows, [`tao`]
  /// will ignore them by default for unfocused windows on Windows. This method allows changing
  /// the filter to explicitly capture them again.
  ///
  /// ## Platform-specific
  ///
  /// - ** Linux / macOS / iOS / Android**: Unsupported.
  ///
  /// [`tao`]: https://crates.io/crates/tao
  fn set_device_event_filter(&mut self, filter: DeviceEventFilter);

  /// Runs an iteration of the runtime event loop and returns control flow to the caller.
  #[cfg(desktop)]
  fn run_iteration<F: FnMut(RunEvent<T>) + 'static>(&mut self, callback: F);

  /// Run the webview runtime.
  fn run<F: FnMut(RunEvent<T>) + 'static>(self, callback: F);
}

/// Webview dispatcher. A thread-safe handle to the webview APIs.
pub trait WebviewDispatch<T: UserEvent>: Debug + Clone + Send + Sync + Sized + 'static {
  /// The runtime this [`WebviewDispatch`] runs under.
  type Runtime: Runtime<T>;

  /// Run a task on the main thread.
  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()>;

  /// Registers a webview event handler.
  fn on_webview_event<F: Fn(&WebviewEvent) + Send + 'static>(&self, f: F) -> WebviewEventId;

  /// Runs a closure with the platform webview object as argument.
  fn with_webview<F: FnOnce(Box<dyn std::any::Any>) + Send + 'static>(&self, f: F) -> Result<()>;

  /// Open the web inspector which is usually called devtools.
  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn open_devtools(&self);

  /// Close the web inspector which is usually called devtools.
  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn close_devtools(&self);

  /// Gets the devtools window's current open state.
  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn is_devtools_open(&self) -> Result<bool>;

  // GETTERS

  /// Returns the webview's current URL.
  fn url(&self) -> Result<String>;

  /// Returns the webview's bounds.
  fn bounds(&self) -> Result<Rect>;

  /// Returns the position of the top-left hand corner of the webviews's client area relative to the top-left hand corner of the window.
  fn position(&self) -> Result<PhysicalPosition<i32>>;

  /// Returns the physical size of the webviews's client area.
  fn size(&self) -> Result<PhysicalSize<u32>>;

  // SETTER

  /// Navigate to the given URL.
  fn navigate(&self, url: Url) -> Result<()>;

  /// Opens the dialog to prints the contents of the webview.
  fn print(&self) -> Result<()>;

  /// Closes the webview.
  fn close(&self) -> Result<()>;

  /// Sets the webview's bounds.
  fn set_bounds(&self, bounds: Rect) -> Result<()>;

  /// Resizes the webview.
  fn set_size(&self, size: Size) -> Result<()>;

  /// Updates the webview position.
  fn set_position(&self, position: Position) -> Result<()>;

  /// Bring the window to front and focus the webview.
  fn set_focus(&self) -> Result<()>;

  /// Hide the webview
  fn hide(&self) -> Result<()>;

  /// Show the webview
  fn show(&self) -> Result<()>;

  /// Executes javascript on the window this [`WindowDispatch`] represents.
  fn eval_script<S: Into<String>>(&self, script: S) -> Result<()>;

  /// Moves the webview to the given window.
  fn reparent(&self, window_id: WindowId) -> Result<()>;

  /// Sets whether the webview should automatically grow and shrink its size and position when the parent window resizes.
  fn set_auto_resize(&self, auto_resize: bool) -> Result<()>;

  /// Set the webview zoom level
  fn set_zoom(&self, scale_factor: f64) -> Result<()>;

  /// Set the webview background.
  fn set_background_color(&self, color: Option<Color>) -> Result<()>;

  /// Clear all browsing data for this webview.
  fn clear_all_browsing_data(&self) -> Result<()>;
}

/// Window dispatcher. A thread-safe handle to the window APIs.
pub trait WindowDispatch<T: UserEvent>: Debug + Clone + Send + Sync + Sized + 'static {
  /// The runtime this [`WindowDispatch`] runs under.
  type Runtime: Runtime<T>;

  /// The window builder type.
  type WindowBuilder: WindowBuilder;

  /// Run a task on the main thread.
  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()>;

  /// Registers a window event handler.
  fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) -> WindowEventId;

  // GETTERS

  /// Returns the scale factor that can be used to map logical pixels to physical pixels, and vice versa.
  fn scale_factor(&self) -> Result<f64>;

  /// Returns the position of the top-left hand corner of the window's client area relative to the top-left hand corner of the desktop.
  fn inner_position(&self) -> Result<PhysicalPosition<i32>>;

  /// Returns the position of the top-left hand corner of the window relative to the top-left hand corner of the desktop.
  fn outer_position(&self) -> Result<PhysicalPosition<i32>>;

  /// Returns the physical size of the window's client area.
  ///
  /// The client area is the content of the window, excluding the title bar and borders.
  fn inner_size(&self) -> Result<PhysicalSize<u32>>;

  /// Returns the physical size of the entire window.
  ///
  /// These dimensions include the title bar and borders. If you don't want that (and you usually don't), use inner_size instead.
  fn outer_size(&self) -> Result<PhysicalSize<u32>>;

  /// Gets the window's current fullscreen state.
  fn is_fullscreen(&self) -> Result<bool>;

  /// Gets the window's current minimized state.
  fn is_minimized(&self) -> Result<bool>;

  /// Gets the window's current maximized state.
  fn is_maximized(&self) -> Result<bool>;

  /// Gets the window's current focus state.
  fn is_focused(&self) -> Result<bool>;

  /// Gets the window's current decoration state.
  fn is_decorated(&self) -> Result<bool>;

  /// Gets the window's current resizable state.
  fn is_resizable(&self) -> Result<bool>;

  /// Gets the window's native maximize button state.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / iOS / Android:** Unsupported.
  fn is_maximizable(&self) -> Result<bool>;

  /// Gets the window's native minimize button state.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / iOS / Android:** Unsupported.
  fn is_minimizable(&self) -> Result<bool>;

  /// Gets the window's native close button state.
  ///
  /// ## Platform-specific
  ///
  /// - **iOS / Android:** Unsupported.
  fn is_closable(&self) -> Result<bool>;

  /// Gets the window's current visibility state.
  fn is_visible(&self) -> Result<bool>;

  /// Whether the window is enabled or disable.
  fn is_enabled(&self) -> Result<bool>;

  /// Gets the window's current title.
  fn title(&self) -> Result<String>;

  /// Returns the monitor on which the window currently resides.
  ///
  /// Returns None if current monitor can't be detected.
  fn current_monitor(&self) -> Result<Option<Monitor>>;

  /// Returns the primary monitor of the system.
  ///
  /// Returns None if it can't identify any monitor as a primary one.
  fn primary_monitor(&self) -> Result<Option<Monitor>>;

  /// Returns the monitor that contains the given point.
  fn monitor_from_point(&self, x: f64, y: f64) -> Result<Option<Monitor>>;

  /// Returns the list of all the monitors available on the system.
  fn available_monitors(&self) -> Result<Vec<Monitor>>;

  /// Returns the `ApplicationWindow` from gtk crate that is used by this window.
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn gtk_window(&self) -> Result<gtk::ApplicationWindow>;

  /// Returns the vertical [`gtk::Box`] that is added by default as the sole child of this window.
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn default_vbox(&self) -> Result<gtk::Box>;

  /// Raw window handle.
  fn window_handle(
    &self,
  ) -> std::result::Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError>;

  /// Returns the current window theme.
  fn theme(&self) -> Result<Theme>;

  // SETTERS

  /// Centers the window.
  fn center(&self) -> Result<()>;

  /// Requests user attention to the window.
  ///
  /// Providing `None` will unset the request for user attention.
  fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> Result<()>;

  /// Create a new window.
  fn create_window<F: Fn(RawWindow) + Send + 'static>(
    &mut self,
    pending: PendingWindow<T, Self::Runtime>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>>;

  /// Create a new webview.
  fn create_webview(
    &mut self,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>>;

  /// Updates the window resizable flag.
  fn set_resizable(&self, resizable: bool) -> Result<()>;

  /// Enable or disable the window.
  ///
  /// ## Platform-specific
  ///
  /// - **Android / iOS**: Unsupported.
  fn set_enabled(&self, enabled: bool) -> Result<()>;

  /// Updates the window's native maximize button state.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Disables the "zoom" button in the window titlebar, which is also used to enter fullscreen mode.
  /// - **Linux / iOS / Android:** Unsupported.
  fn set_maximizable(&self, maximizable: bool) -> Result<()>;

  /// Updates the window's native minimize button state.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / iOS / Android:** Unsupported.
  fn set_minimizable(&self, minimizable: bool) -> Result<()>;

  /// Updates the window's native close button state.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** "GTK+ will do its best to convince the window manager not to show a close button.
  ///   Depending on the system, this function may not have any effect when called on a window that is already visible"
  /// - **iOS / Android:** Unsupported.
  fn set_closable(&self, closable: bool) -> Result<()>;

  /// Updates the window title.
  fn set_title<S: Into<String>>(&self, title: S) -> Result<()>;

  /// Maximizes the window.
  fn maximize(&self) -> Result<()>;

  /// Unmaximizes the window.
  fn unmaximize(&self) -> Result<()>;

  /// Minimizes the window.
  fn minimize(&self) -> Result<()>;

  /// Unminimizes the window.
  fn unminimize(&self) -> Result<()>;

  /// Shows the window.
  fn show(&self) -> Result<()>;

  /// Hides the window.
  fn hide(&self) -> Result<()>;

  /// Closes the window.
  fn close(&self) -> Result<()>;

  /// Destroys the window.
  fn destroy(&self) -> Result<()>;

  /// Updates the decorations flag.
  fn set_decorations(&self, decorations: bool) -> Result<()>;

  /// Updates the shadow flag.
  fn set_shadow(&self, enable: bool) -> Result<()>;

  /// Updates the window alwaysOnBottom flag.
  fn set_always_on_bottom(&self, always_on_bottom: bool) -> Result<()>;

  /// Updates the window alwaysOnTop flag.
  fn set_always_on_top(&self, always_on_top: bool) -> Result<()>;

  /// Updates the window visibleOnAllWorkspaces flag.
  fn set_visible_on_all_workspaces(&self, visible_on_all_workspaces: bool) -> Result<()>;

  /// Set the window background.
  fn set_background_color(&self, color: Option<Color>) -> Result<()>;

  /// Prevents the window contents from being captured by other apps.
  fn set_content_protected(&self, protected: bool) -> Result<()>;

  /// Resizes the window.
  fn set_size(&self, size: Size) -> Result<()>;

  /// Updates the window min inner size.
  fn set_min_size(&self, size: Option<Size>) -> Result<()>;

  /// Updates the window max inner size.
  fn set_max_size(&self, size: Option<Size>) -> Result<()>;

  /// Sets this window's minimum inner width.
  fn set_size_constraints(&self, constraints: WindowSizeConstraints) -> Result<()>;

  /// Updates the window position.
  fn set_position(&self, position: Position) -> Result<()>;

  /// Updates the window fullscreen state.
  fn set_fullscreen(&self, fullscreen: bool) -> Result<()>;

  /// Bring the window to front and focus.
  fn set_focus(&self) -> Result<()>;

  /// Updates the window icon.
  fn set_icon(&self, icon: Icon) -> Result<()>;

  /// Whether to hide the window icon from the taskbar or not.
  fn set_skip_taskbar(&self, skip: bool) -> Result<()>;

  /// Grabs the cursor, preventing it from leaving the window.
  ///
  /// There's no guarantee that the cursor will be hidden. You should
  /// hide it by yourself if you want so.
  fn set_cursor_grab(&self, grab: bool) -> Result<()>;

  /// Modifies the cursor's visibility.
  ///
  /// If `false`, this will hide the cursor. If `true`, this will show the cursor.
  fn set_cursor_visible(&self, visible: bool) -> Result<()>;

  // Modifies the cursor icon of the window.
  fn set_cursor_icon(&self, icon: CursorIcon) -> Result<()>;

  /// Changes the position of the cursor in window coordinates.
  fn set_cursor_position<Pos: Into<Position>>(&self, position: Pos) -> Result<()>;

  /// Ignores the window cursor events.
  fn set_ignore_cursor_events(&self, ignore: bool) -> Result<()>;

  /// Starts dragging the window.
  fn start_dragging(&self) -> Result<()>;

  /// Starts resize-dragging the window.
  fn start_resize_dragging(&self, direction: ResizeDirection) -> Result<()>;

  /// Sets the badge count on the taskbar
  /// The badge count appears as a whole for the application
  /// Using `0` or using `None` will remove the badge
  ///
  /// ## Platform-specific
  /// - **Windows:** Unsupported, use [`WindowDispatch::set_overlay_icon`] instead.
  /// - **Android:** Unsupported.
  /// - **iOS:** iOS expects i32, if the value is larger than i32::MAX, it will be clamped to i32::MAX.
  fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>) -> Result<()>;

  /// Sets the badge count on the taskbar **macOS only**. Using `None` will remove the badge
  fn set_badge_label(&self, label: Option<String>) -> Result<()>;

  /// Sets the overlay icon on the taskbar **Windows only**. Using `None` will remove the icon
  ///
  /// The overlay icon can be unique for each window.
  fn set_overlay_icon(&self, icon: Option<Icon>) -> Result<()>;

  /// Sets the taskbar progress state.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / macOS**: Progress bar is app-wide and not specific to this window. Only supported desktop environments with `libunity` (e.g. GNOME).
  /// - **iOS / Android:** Unsupported.
  fn set_progress_bar(&self, progress_state: ProgressBarState) -> Result<()>;

  /// Sets the title bar style. Available on macOS only.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / Windows / iOS / Android:** Unsupported.
  fn set_title_bar_style(&self, style: tauri_utils::TitleBarStyle) -> Result<()>;

  /// Sets the theme for this window.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / macOS**: Theme is app-wide and not specific to this window.
  /// - **iOS / Android:** Unsupported.
  fn set_theme(&self, theme: Option<Theme>) -> Result<()>;
}
