pub mod wry;

use crate::plugin::PluginStore;

/// A icon definition.
pub enum Icon {
  /// Icon from file path.
  File(String),
  /// Icon from raw bytes.
  Raw(Vec<u8>),
}

/// Messages to dispatch to the application.
pub enum Message {
  // webview messages
  /// Eval a script on the webview.
  EvalScript(String),
  // window messages
  /// Updates the window resizable flag.
  SetResizable(bool),
  /// Updates the window title.
  SetTitle(String),
  /// Maximizes the window.
  Maximize,
  /// Unmaximizes the window.
  Unmaximize,
  /// Minimizes the window.
  Minimize,
  /// Unminimizes the window.
  Unminimize,
  /// Shows the window.
  Show,
  /// Hides the window.
  Hide,
  /// Updates the transparency flag.
  SetTransparent(bool),
  /// Updates the hasDecorations flag.
  SetDecorations(bool),
  /// Updates the window alwaysOnTop flag.
  SetAlwaysOnTop(bool),
  /// Updates the window width.
  SetWidth(f64),
  /// Updates the window height.
  SetHeight(f64),
  /// Resizes the window.
  Resize {
    /// New width.
    width: f64,
    /// New height.
    height: f64,
  },
  /// Updates the window min size.
  SetMinSize {
    /// New value for the window min width.
    min_width: f64,
    /// New value for the window min height.
    min_height: f64,
  },
  /// Updates the window max size.
  SetMaxSize {
    /// New value for the window max width.
    max_width: f64,
    /// New value for the window max height.
    max_height: f64,
  },
  /// Updates the X position.
  SetX(f64),
  /// Updates the Y position.
  SetY(f64),
  /// Updates the window position.
  SetPosition {
    /// New value for the window X coordinate.
    x: f64,
    /// New value for the window Y coordinate.
    y: f64,
  },
  /// Updates the window fullscreen state.
  SetFullscreen(bool),
  /// Updates the window icon.
  SetIcon(Icon),
}

/// The webview builder.
pub trait WebviewBuilderExt: Sized {
  /// The webview object that this builder creates.
  type Webview;

  /// Initializes a new webview builder.
  fn new() -> Self;

  /// Sets the webview url.
  fn url(self, url: String) -> Self;

  /// Sets the init script.
  fn initialization_script(self, init: &str) -> Self;

  /// The horizontal position of the window's top left corner.
  fn x(self, x: f64) -> Self;

  /// The vertical position of the window's top left corner.
  fn y(self, y: f64) -> Self;

  /// Window width.
  fn width(self, width: f64) -> Self;

  /// Window height.
  fn height(self, height: f64) -> Self;

  /// Window min width.
  fn min_width(self, min_width: f64) -> Self;

  /// Window min height.
  fn min_height(self, min_height: f64) -> Self;

  /// Window max width.
  fn max_width(self, max_width: f64) -> Self;

  /// Window max height.
  fn max_height(self, max_height: f64) -> Self;

  /// Whether the window is resizable or not.
  fn resizable(self, resizable: bool) -> Self;

  /// The title of the window in the title bar.
  fn title(self, title: String) -> Self;

  /// Whether to start the window in fullscreen or not.
  fn fullscreen(self, fullscreen: bool) -> Self;

  /// Whether the window should be maximized upon creation.
  fn maximized(self, maximized: bool) -> Self;

  /// Whether the window should be immediately visible upon creation.
  fn visible(self, visible: bool) -> Self;

  /// Whether the the window should be transparent. If this is true, writing colors
  /// with alpha values different than `1.0` will produce a transparent window.
  fn transparent(self, transparent: bool) -> Self;

  /// Whether the window should have borders and bars.
  fn decorations(self, decorations: bool) -> Self;

  /// Whether the window should always be on top of other windows.
  fn always_on_top(self, always_on_top: bool) -> Self;

  /// Builds the webview instance.
  fn finish(self) -> crate::Result<Self::Webview>;
}

/// Binds the given callback to a global variable on the window object.
pub struct Callback<D> {
  /// Function name to bind.
  pub name: String,
  /// Function callback handler.
  pub function: Box<dyn FnMut(D, i32, Vec<String>) -> i32 + Send>,
}

/// Webview dispatcher. A thread-safe handle to the webview API.
pub trait ApplicationDispatcherExt: Clone + Send + Sync + Sized {
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

  /// Updates the transparency flag.
  fn set_transparent(&self, resizable: bool) -> crate::Result<()>;

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
  fn set_icon(&self, icon: Icon) -> crate::Result<()>;

  /// Evals a script on the webview.
  fn eval_script<S: Into<String>>(&self, script: S) -> crate::Result<()>;
}

/// The application interface.
/// Manages windows and webviews.
pub trait ApplicationExt: Sized {
  /// The webview builder.
  type WebviewBuilder: WebviewBuilderExt;
  /// The message dispatcher.
  type Dispatcher: ApplicationDispatcherExt;

  /// Returns the static plugin collection.
  fn plugin_store() -> &'static PluginStore<Self::Dispatcher>;

  /// Creates a new application.
  fn new() -> crate::Result<Self>;

  /// Creates a new webview.
  fn create_webview(
    &mut self,
    webview_builder: Self::WebviewBuilder,
    callbacks: Vec<Callback<Self::Dispatcher>>,
  ) -> crate::Result<Self::Dispatcher>;

  /// Run the application.
  fn run(self);
}
