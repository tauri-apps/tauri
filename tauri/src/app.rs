use futures::future::BoxFuture;
use webview_official::WebviewMut;

mod runner;

type InvokeHandler = dyn FnMut(WebviewMut, String) -> BoxFuture<'static, Result<(), String>>;
type Setup = dyn FnMut(WebviewMut, String) -> BoxFuture<'static, ()>;

/// The application runner.
pub struct App {
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler>>,
  /// The setup callback, invoked when the webview is ready.
  setup: Option<Box<Setup>>,
  /// The HTML of the splashscreen to render.
  splashscreen_html: Option<String>,
}

impl App {
  /// Runs the app until it finishes.
  pub fn run(mut self) {
    runner::run(&mut self).expect("Failed to build webview");
  }

  /// Runs the invoke handler if defined.
  /// Returns whether the message was consumed or not.
  /// The message is considered consumed if the handler exists and returns an Ok Result.
  pub(crate) async fn run_invoke_handler(
    &mut self,
    webview: &mut WebviewMut,
    arg: &str,
  ) -> Result<bool, String> {
    if let Some(ref mut invoke_handler) = self.invoke_handler {
      invoke_handler(webview.clone(), arg.to_string())
        .await
        .map(|_| true)
    } else {
      Ok(false)
    }
  }

  /// Runs the setup callback if defined.
  pub(crate) async fn run_setup(&mut self, webview: &mut WebviewMut, source: String) {
    if let Some(ref mut setup) = self.setup {
      setup(webview.clone(), source).await;
    }
  }

  /// Returns the splashscreen HTML.
  pub fn splashscreen_html(&self) -> Option<&String> {
    self.splashscreen_html.as_ref()
  }
}

/// The App builder.
#[derive(Default)]
pub struct AppBuilder {
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler>>,
  /// The setup callback, invoked when the webview is ready.
  setup: Option<Box<Setup>>,
  /// The HTML of the splashscreen to render.
  splashscreen_html: Option<String>,
}

impl AppBuilder {
  /// Creates a new App builder.
  pub fn new() -> Self {
    Self {
      invoke_handler: None,
      setup: None,
      splashscreen_html: None,
    }
  }

  /// Defines the JS message handler callback.
  pub fn invoke_handler<
    T: futures::Future<Output = Result<(), String>> + Send + 'static,
    F: FnMut(WebviewMut, String) -> T + Send + 'static,
  >(
    mut self,
    mut invoke_handler: F,
  ) -> Self {
    self.invoke_handler = Some(Box::new(move |webview, arg| {
      Box::pin(invoke_handler(webview, arg))
    }));
    self
  }

  /// Defines the setup callback.
  pub fn setup<
    T: futures::Future<Output = ()> + Send + 'static,
    F: FnMut(WebviewMut, String) -> T + Send + 'static,
  >(
    mut self,
    mut setup: F,
  ) -> Self {
    self.setup = Some(Box::new(move |webview, source| {
      Box::pin(setup(webview, source))
    }));
    self
  }

  /// Defines the splashscreen HTML to render.
  pub fn splashscreen_html(mut self, html: &str) -> Self {
    self.splashscreen_html = Some(html.to_string());
    self
  }

  /// Adds a plugin to the runtime.
  pub fn plugin(self, plugin: impl crate::plugin::Plugin + Send + Sync + 'static) -> Self {
    crate::async_runtime::block_on(crate::plugin::register(plugin));
    self
  }

  /// Builds the App.
  pub fn build(self) -> App {
    App {
      invoke_handler: self.invoke_handler,
      setup: self.setup,
      splashscreen_html: self.splashscreen_html,
    }
  }
}
