mod official;

/// Size hints.
pub enum SizeHint {
  /// None
  NONE = 0,
  /// Min
  MIN = 1,
  /// Max
  MAX = 2,
  /// Fixed
  FIXED = 3,
}

impl Default for SizeHint {
  fn default() -> Self {
    Self::NONE
  }
}

pub use crate::plugin::PluginStore;

/// The webview builder.
pub trait WebviewBuilder: Sized {
  /// The webview object that this builder creates.
  type WebviewObject: Webview<Builder = Self>;

  /// Initializes a new instance of the builder.
  fn new() -> Self;
  /// Sets the debug flag.
  fn debug(&mut self, debug: bool) -> &mut Self;
  /// Sets the window title.
  fn title(&mut self, title: &str) -> &mut Self;
  /// Sets the initial url.
  fn url(&mut self, url: &str) -> &mut Self;
  /// Sets the init script.
  fn init(&mut self, init: &str) -> &mut Self;
  /// Sets the window width.
  fn width(&mut self, width: usize) -> &mut Self;
  /// Sets the window height.
  fn height(&mut self, height: usize) -> &mut Self;
  /// Whether the window is resizable or not.
  fn resizable(&mut self, resizable: SizeHint) -> &mut Self;
  /// Builds the webview instance.
  fn finish(self) -> Self::WebviewObject;
}

/// Webview core API.
pub trait Webview: Sized {
  /// The `as_mut` type.
  type Mut: WebviewMut;
  /// The builder type.
  type Builder: WebviewBuilder<WebviewObject = Self>;

  /// Returns the static plugin collection.
  fn plugin_store() -> &'static PluginStore<Self::Mut>;

  /// Adds an init JS code.
  fn init(&mut self, js: &str);

  /// Sets the window title.
  fn set_title(&mut self, title: &str);

  /// Get a handle to a thread safe webview value.
  fn as_mut(&mut self) -> Self::Mut;

  /// Sets the window size.
  fn set_size(&mut self, width: i32, height: i32, hint: SizeHint);

  /// terminate the webview.
  fn terminate(&mut self);

  /// eval a string as JS code.
  fn eval(&mut self, js: &str);

  /// Dispatches a closure to run on the main thread.
  fn dispatch<F>(&mut self, f: F)
  where
    F: FnOnce(&mut Self) + Send + 'static;

  /// Binds a new API on the webview.
  fn bind<F>(&mut self, name: &str, f: F)
  where
    F: FnMut(&str, &str);

  /// Run the webview event loop.
  fn run(&mut self);
}

/// A thread safe webview handle.
pub trait WebviewMut: Clone + Send + Sync {
  /// The error type for the APIs.
  type Error: std::error::Error + Send + Sync + 'static;
  /// The parent webview type, used for Dispatch.
  type WebviewObject: Webview;

  /// terminate the webview.
  fn terminate(&mut self) -> Result<(), Self::Error>;

  /// eval a string as JS code.
  fn eval(&mut self, js: &str) -> Result<(), Self::Error>;

  /// Dispatches a closure to run on the main thread.
  fn dispatch<F>(&mut self, f: F) -> Result<(), Self::Error>
  where
    F: FnOnce(&mut Self::WebviewObject) + Send + 'static;

  /// Binds a new API on the webview.
  fn bind<F>(&mut self, name: &str, f: F) -> Result<(), Self::Error>
  where
    F: FnMut(&str, &str) + 'static;
}
