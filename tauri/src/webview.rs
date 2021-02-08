pub(crate) mod wry;

pub use crate::plugin::PluginStore;

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

  /// Whether the window is resizable or not.
  fn resizable(self, resizable: bool) -> Self;

  /// The title of the window in the title bar.
  fn title(self, title: String) -> Self;

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
  /// Eval a JS string on the current webview.
  fn eval(&mut self, js: &str);
  /// Eval a JS string on the webview associated with the given window.
  fn eval_on_window(&mut self, window_id: &str, js: &str);
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
