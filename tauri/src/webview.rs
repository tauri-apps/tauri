pub(crate) mod official;
pub(crate) mod wry;

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
  /// Binds a new API on the webview.
  fn bind<F>(self, name: &str, f: F) -> Self
  where
    F: FnMut(
        &<<Self as WebviewBuilder>::WebviewObject as Webview>::Dispatcher,
        i8,
        Vec<String>,
      ) -> i32
      + Send
      + 'static;
  /// Sets the debug flag.
  fn debug(self, debug: bool) -> Self;
  /// Sets the window title.
  fn title(self, title: &str) -> Self;
  /// Sets the initial url.
  fn url(self, url: &str) -> Self;
  /// Sets the init script.
  fn init(self, init: &str) -> Self;
  /// Sets the window width.
  fn width(self, width: usize) -> Self;
  /// Sets the window height.
  fn height(self, height: usize) -> Self;
  /// Whether the window is resizable or not.
  fn resizable(self, resizable: SizeHint) -> Self;
  /// Builds the webview instance.
  fn finish(self) -> crate::Result<Self::WebviewObject>;
}

/// Webview dispatcher. A thread-safe handle to the webview API.
pub trait WebviewDispatcher: Clone + Send + Sync + Sized {
  /// Eval a JS string.
  fn eval(&mut self, js: &str);
}

/// Webview core API.
pub trait Webview: Sized {
  /// The builder type.
  type Builder: WebviewBuilder<WebviewObject = Self>;
  /// The dispatcher type.
  type Dispatcher: WebviewDispatcher;

  /// Returns the static plugin collection.
  fn plugin_store() -> &'static PluginStore<Self::Dispatcher>;

  /// Sets the window title.
  fn set_title(&mut self, title: &str);

  /// Sets the window size.
  fn set_size(&mut self, width: i32, height: i32, hint: SizeHint);

  /// terminate the webview.
  fn terminate(&mut self);

  /// eval a string as JS code.
  fn eval(&mut self, js: &str);

  /// Gets an instance of the webview dispatcher.
  fn dispatcher(&mut self) -> Self::Dispatcher;

  /// Run the webview event loop.
  fn run(&mut self);
}
