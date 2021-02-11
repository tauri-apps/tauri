pub(crate) mod wry;

pub use crate::{api::config::WindowConfig, plugin::PluginStore};

/// An event to be posted to the webview event loop.
pub enum Event {
  /// Run the given closure.
  Run(crate::SyncTask),
}

/// The window builder.
pub trait WindowBuilderExt: Sized {
  /// The window type.
  type Window;

  /// Initializes a new window builder.
  fn new() -> Self;

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

  /// build the window.
  fn finish(self) -> crate::Result<Self::Window>;
}

pub struct WindowBuilder<T>(T);

impl<T> WindowBuilder<T> {
  pub fn get(self) -> T {
    self.0
  }
}

impl<T: WindowBuilderExt> From<&WindowConfig> for WindowBuilder<T> {
  fn from(config: &WindowConfig) -> Self {
    let mut window = T::new()
      .title(config.title.to_string())
      .width(config.width)
      .height(config.height)
      .visible(config.visible)
      .resizable(config.resizable)
      .decorations(config.decorations)
      .maximized(config.maximized)
      .fullscreen(config.fullscreen)
      .transparent(config.transparent)
      .always_on_top(config.always_on_top);
    if let Some(min_width) = config.min_width {
      window = window.min_width(min_width);
    }
    if let Some(min_height) = config.min_height {
      window = window.min_height(min_height);
    }
    if let Some(max_width) = config.max_width {
      window = window.max_width(max_width);
    }
    if let Some(max_height) = config.max_height {
      window = window.max_height(max_height);
    }
    if let Some(x) = config.x {
      window = window.x(x);
    }
    if let Some(y) = config.y {
      window = window.y(y);
    }

    Self(window)
  }
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

  /// Builds the webview instance.
  fn finish(self) -> crate::Result<Self::Webview>;
}

/// Binds the given callback to a global variable on the window object.
pub struct Callback<D> {
  /// Function name to bind.
  pub name: String,
  /// Function callback handler.
  pub function: Box<dyn FnMut(&D, i32, Vec<String>) -> i32 + Send>,
}

/// Webview dispatcher. A thread-safe handle to the webview API.
pub trait ApplicationDispatcherExt: Clone + Send + Sync + Sized {
  /// Eval a JS string on the webview.
  fn eval(&mut self, js: &str);
  /// Change the window title.
  fn set_title(&mut self, js: &str);
  /// Sends a event to the webview.
  fn send_event(&self, event: Event);
}

/// The application interface.
/// Manages windows and webviews.
pub trait ApplicationExt: Sized {
  /// The webview builder.
  type WebviewBuilder: WebviewBuilderExt;
  /// The window builder.
  type WindowBuilder: WindowBuilderExt;
  /// The window type.
  type Window;
  /// The message dispatcher.
  type Dispatcher: ApplicationDispatcherExt;

  /// Returns the static plugin collection.
  fn plugin_store() -> &'static PluginStore<Self::Dispatcher>;

  /// Creates a new application.
  fn new() -> crate::Result<Self>;

  /// Gets the message dispatcher for the given window.
  fn dispatcher(&self, window: &Self::Window) -> Self::Dispatcher;

  /// Creates a new window.
  fn create_window(&self, window_builder: Self::WindowBuilder) -> crate::Result<Self::Window>;

  /// Creates a new webview.
  fn create_webview(
    &mut self,
    webview_builder: Self::WebviewBuilder,
    window: Self::Window,
    callbacks: Vec<Callback<Self::Dispatcher>>,
  ) -> crate::Result<()>;

  /// Run the application.
  fn run(self);
}
