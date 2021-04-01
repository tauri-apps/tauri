use crate::{
  app::webview::{Attributes, AttributesPrivate, Icon, WindowConfig},
  DetachedWindow, Manager, PendingWindow,
};
use std::convert::TryFrom;

/// Webview dispatcher. A thread-safe handle to the webview API.
#[allow(missing_docs)]
pub trait Dispatch: Clone + Send + Sized + 'static {
  type Runtime: Runtime;
  type Icon: TryFrom<Icon, Error = crate::Error>;

  /// The webview builder type.
  type Attributes: Attributes<Icon = Self::Icon>
    + AttributesPrivate
    + From<WindowConfig>
    + Clone
    + Send;

  /// Creates a new webview window.
  fn create_window<M: Manager<Runtime = Self::Runtime>>(
    &mut self,
    pending: PendingWindow<M>,
  ) -> crate::Result<DetachedWindow<M>>;
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

  /// Updates the window width.
  fn set_width(&self, width: f64) -> crate::Result<()>;

  /// Updates the window height.
  fn set_height(&self, height: f64) -> crate::Result<()>;

  /// Resizes the window.
  fn resize(&self, width: f64, height: f64) -> crate::Result<()>;

  /// Updates the window min size.
  fn set_min_size(&self, min_width: f64, min_height: f64) -> crate::Result<()>;

  /// Updates the window max size.
  fn set_max_size(&self, max_width: f64, max_height: f64) -> crate::Result<()>;

  /// Updates the X position.
  fn set_x(&self, x: f64) -> crate::Result<()>;

  /// Updates the Y position.
  fn set_y(&self, y: f64) -> crate::Result<()>;

  /// Updates the window position.
  fn set_position(&self, x: f64, y: f64) -> crate::Result<()>;

  /// Updates the window fullscreen state.
  fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()>;

  /// Updates the window icon.
  fn set_icon(&self, icon: Self::Icon) -> crate::Result<()>;

  /// Evals a script on the webview.
  fn eval_script<S: Into<String>>(&self, script: S) -> crate::Result<()>;
}

/// The application interface.
/// Manages windows and webviews.
#[allow(missing_docs)]
pub trait Runtime: Sized + 'static {
  /// The message dispatcher.
  type Dispatcher: Dispatch<Runtime = Self>;

  /// Creates a new application.
  fn new() -> crate::Result<Self>;

  /// Creates a new webview window.
  fn create_window<M: Manager<Runtime = Self>>(
    &mut self,
    pending: PendingWindow<M>,
  ) -> crate::Result<DetachedWindow<M>>;

  /// Run the application.
  fn run(self);
}
