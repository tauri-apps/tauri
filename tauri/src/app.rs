use crate::Webview;
use futures::future::BoxFuture;

mod runner;

type InvokeHandler<W> = dyn Fn(W, String) -> BoxFuture<'static, Result<(), String>> + Send + Sync;
type Setup<W> = dyn Fn(W, String) -> BoxFuture<'static, ()> + Send + Sync;

/// The application runner.
pub struct App<W: Webview> {
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler<W>>>,
  /// The setup callback, invoked when the webview is ready.
  setup: Option<Box<Setup<W>>>,
  /// The HTML of the splashscreen to render.
  splashscreen_html: Option<String>,
}

impl<W: Webview + 'static> App<W> {
  /// Runs the app until it finishes.
  pub fn run(self) {
    runner::run(self).expect("Failed to build webview");
  }

  /// Runs the invoke handler if defined.
  /// Returns whether the message was consumed or not.
  /// The message is considered consumed if the handler exists and returns an Ok Result.
  pub(crate) async fn run_invoke_handler(
    &self,
    webview: &mut W,
    arg: &str,
  ) -> Result<bool, String> {
    if let Some(ref invoke_handler) = self.invoke_handler {
      let fut = invoke_handler(webview.clone(), arg.to_string());
      fut.await.map(|_| true)
    } else {
      Ok(false)
    }
  }

  /// Runs the setup callback if defined.
  pub(crate) async fn run_setup(&self, webview: &mut W, source: String) {
    if let Some(ref setup) = self.setup {
      let fut = setup(webview.clone(), source);
      fut.await;
    }
  }

  /// Returns the splashscreen HTML.
  pub fn splashscreen_html(&self) -> Option<&String> {
    self.splashscreen_html.as_ref()
  }
}

/// The App builder.
#[derive(Default)]
pub struct AppBuilder<W: Webview> {
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler<W>>>,
  /// The setup callback, invoked when the webview is ready.
  setup: Option<Box<Setup<W>>>,
  /// The HTML of the splashscreen to render.
  splashscreen_html: Option<String>,
}

impl<W: Webview + 'static> AppBuilder<W> {
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
    T: futures::Future<Output = Result<(), String>> + Send + Sync + 'static,
    F: Fn(W, String) -> T + Send + Sync + 'static,
  >(
    mut self,
    invoke_handler: F,
  ) -> Self {
    self.invoke_handler = Some(Box::new(move |webview, arg| {
      Box::pin(invoke_handler(webview, arg))
    }));
    self
  }

  /// Defines the setup callback.
  pub fn setup<
    T: futures::Future<Output = ()> + Send + Sync + 'static,
    F: Fn(W, String) -> T + Send + Sync + 'static,
  >(
    mut self,
    setup: F,
  ) -> Self {
    self.setup = Some(Box::new(move |webview, source| {
      Box::pin(setup(webview, source))
    }));
    self
  }

  /// Defines the splashscreen HTML to render.
  pub fn splashscreen_html(mut self, html: &str) -> Self {
    self.splashscreen_html = Some(format!("data:text/html,{}", urlencoding::encode(html)));
    self
  }

  /// Adds a plugin to the runtime.
  pub fn plugin(
    self,
    plugin: impl crate::plugin::Plugin<W> + Send + Sync + Sync + 'static,
  ) -> Self {
    crate::async_runtime::block_on(crate::plugin::register(W::plugin_store(), plugin));
    self
  }

  /// Builds the App.
  pub fn build(self) -> App<W> {
    App {
      invoke_handler: self.invoke_handler,
      setup: self.setup,
      splashscreen_html: self.splashscreen_html,
    }
  }
}
