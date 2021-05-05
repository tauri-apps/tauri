// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Internal runtime between Tauri and the underlying webview runtime.

use crate::{
  runtime::window::{DetachedWindow, PendingWindow},
  Icon, Params, WindowBuilder,
};

use serde::{Deserialize, Serialize};

pub(crate) mod app;
pub mod flavors;
pub(crate) mod manager;
pub mod tag;
pub mod webview;
pub mod window;

/// The webview runtime interface.
pub trait Runtime: Sized + 'static {
  /// The message dispatcher.
  type Dispatcher: Dispatch<Runtime = Self>;

  /// Creates a new webview runtime.
  fn new() -> crate::Result<Self>;

  /// Create a new webview window.
  fn create_window<P: Params<Runtime = Self>>(
    &mut self,
    pending: PendingWindow<P>,
  ) -> crate::Result<DetachedWindow<P>>;

  /// Run the webview runtime.
  fn run(self);
}

/// Physical position descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalPosition<P> {
  /// Vertical axis value.
  pub x: P,
  /// Horizontal axis value.
  pub y: P,
}

/// Logical position descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalPosition<P> {
  /// Vertical axis value.
  pub x: P,
  /// Horizontal axis value.
  pub y: P,
}

/// A position that's either physical or logical.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Position {
  /// Physical position.
  Physical(PhysicalPosition<i32>),
  /// Logical position.
  Logical(LogicalPosition<f64>),
}

/// Physical size descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalSize<T> {
  /// Width.
  pub width: T,
  /// Height.
  pub height: T,
}

/// Logical size descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalSize<T> {
  /// Width.
  pub width: T,
  /// Height.
  pub height: T,
}

/// A size that's either physical or logical.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Size {
  /// Physical size.
  Physical(PhysicalSize<i32>),
  /// Logical size.
  Logical(LogicalSize<f64>),
}

/// Monitor descriptor.
#[derive(Debug, Clone, Serialize)]
pub struct Monitor {
  pub(crate) name: Option<String>,
  pub(crate) size: PhysicalSize<u32>,
  pub(crate) position: PhysicalPosition<i32>,
  pub(crate) scale_factor: f64,
}

impl Monitor {
  /// Returns a human-readable name of the monitor.
  /// Returns None if the monitor doesn't exist anymore.
  pub fn name(&self) -> Option<&String> {
    self.name.as_ref()
  }

  /// Returns the monitor's resolution.
  pub fn size(&self) -> &PhysicalSize<u32> {
    &self.size
  }

  /// Returns the top-left corner position of the monitor relative to the larger full screen area.
  pub fn position(&self) -> &PhysicalPosition<i32> {
    &self.position
  }

  /// Returns the scale factor that can be used to map logical pixels to physical pixels, and vice versa.
  pub fn scale_factor(&self) -> f64 {
    self.scale_factor
  }
}

/// Webview dispatcher. A thread-safe handle to the webview API.
pub trait Dispatch: Clone + Send + Sized + 'static {
  /// The runtime this [`Dispatch`] runs under.
  type Runtime: Runtime;

  /// The winoow builder type.
  type WindowBuilder: WindowBuilder + Clone;

  /// Run a task on the main thread.
  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> crate::Result<()>;

  // GETTERS

  /// Returns the scale factor that can be used to map logical pixels to physical pixels, and vice versa.
  fn scale_factor(&self) -> f64;

  /// Returns the position of the top-left hand corner of the window's client area relative to the top-left hand corner of the desktop.
  fn inner_position(&self) -> crate::Result<PhysicalPosition<i32>>;

  /// Returns the position of the top-left hand corner of the window relative to the top-left hand corner of the desktop.
  fn outer_position(&self) -> crate::Result<PhysicalPosition<i32>>;

  /// Returns the physical size of the window's client area.
  ///
  /// The client area is the content of the window, excluding the title bar and borders.
  fn inner_size(&self) -> crate::Result<PhysicalSize<f64>>;

  /// Returns the physical size of the entire window.
  ///
  /// These dimensions include the title bar and borders. If you don't want that (and you usually don't), use inner_size instead.
  fn outer_size(&self) -> crate::Result<PhysicalSize<f64>>;

  /// Gets the window's current fullscreen state.
  fn is_fullscreen(&self) -> bool;

  /// Gets the window's current maximized state.
  fn is_maximized(&self) -> bool;

  /// Returns the monitor on which the window currently resides.
  ///
  /// Returns None if current monitor can't be detected.
  fn current_monitor(&self) -> Option<Monitor>;

  /// Returns the primary monitor of the system.
  ///
  /// Returns None if it can't identify any monitor as a primary one.
  fn primary_monitor(&self) -> Option<Monitor>;

  /// Returns the list of all the monitors available on the system.
  fn available_monitors(&self) -> Vec<Monitor>;

  // SETTERS

  /// Create a new webview window.
  fn create_window<P: Params<Runtime = Self::Runtime>>(
    &mut self,
    pending: PendingWindow<P>,
  ) -> crate::Result<DetachedWindow<P>>;

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
  fn resize(&self, size: Size) -> crate::Result<()>;

  /// Updates the window min size.
  fn set_min_size(&self, size: Option<Size>) -> crate::Result<()>;

  /// Updates the window max size.
  fn set_max_size(&self, size: Option<Size>) -> crate::Result<()>;

  /// Updates the window position.
  fn set_position(&self, position: Position) -> crate::Result<()>;

  /// Updates the window fullscreen state.
  fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()>;

  /// Updates the window icon.
  fn set_icon(&self, icon: Icon) -> crate::Result<()>;

  /// Starts dragging the window.
  fn start_dragging(&self) -> crate::Result<()>;

  /// Executes javascript on the window this [`Dispatch`] represents.
  fn eval_script<S: Into<String>>(&self, script: S) -> crate::Result<()>;
}
