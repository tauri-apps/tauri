use crate::ApplicationExt;
use futures::future::BoxFuture;
use std::marker::PhantomData;
use tauri_api::{config::Config, private::AsTauriContext};

mod runner;

type InvokeHandler<W> = dyn Fn(W, String) -> BoxFuture<'static, Result<(), String>> + Send + Sync;
type Setup<W> = dyn Fn(W, String) -> BoxFuture<'static, ()> + Send + Sync;

/// `App` runtime information.
pub struct Context {
  pub(crate) config: Config,
  pub(crate) tauri_script: &'static str,
  pub(crate) assets: &'static tauri_api::assets::Assets,
}

impl Context {
  pub(crate) fn new<Context: AsTauriContext>() -> crate::Result<Self> {
    Ok(Self {
      config: serde_json::from_str(Context::raw_config())?,
      tauri_script: Context::raw_tauri_script(),
      assets: Context::assets(),
    })
  }
}

/// The application runner.
pub struct App<A: ApplicationExt> {
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler<A::Dispatcher>>>,
  /// The setup callback, invoked when the webview is ready.
  setup: Option<Box<Setup<A::Dispatcher>>>,
  /// The HTML of the splashscreen to render.
  splashscreen_html: Option<String>,
  /// The context the App was created with
  pub(crate) context: Context,
}

impl<A: ApplicationExt + 'static> App<A> {
  /// Runs the app until it finishes.
  pub fn run(self) {
    runner::run(self).expect("Failed to build webview");
  }

  /// Runs the invoke handler if defined.
  /// Returns whether the message was consumed or not.
  /// The message is considered consumed if the handler exists and returns an Ok Result.
  pub(crate) async fn run_invoke_handler(
    &self,
    dispatcher: &mut A::Dispatcher,
    arg: &str,
  ) -> Result<bool, String> {
    if let Some(ref invoke_handler) = self.invoke_handler {
      let fut = invoke_handler(dispatcher.clone(), arg.to_string());
      fut.await.map(|_| true)
    } else {
      Ok(false)
    }
  }

  /// Runs the setup callback if defined.
  pub(crate) async fn run_setup(&self, dispatcher: &mut A::Dispatcher, source: String) {
    if let Some(ref setup) = self.setup {
      let fut = setup(dispatcher.clone(), source);
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
pub struct AppBuilder<A: ApplicationExt, C: AsTauriContext> {
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler<A::Dispatcher>>>,
  /// The setup callback, invoked when the webview is ready.
  setup: Option<Box<Setup<A::Dispatcher>>>,
  /// The HTML of the splashscreen to render.
  splashscreen_html: Option<String>,
  /// The configuration used
  config: PhantomData<C>,
}

impl<A: ApplicationExt + 'static, C: AsTauriContext> AppBuilder<A, C> {
  /// Creates a new App builder.
  pub fn new() -> Self {
    Self {
      invoke_handler: None,
      setup: None,
      splashscreen_html: None,
      config: Default::default(),
    }
  }

  /// Defines the JS message handler callback.
  pub fn invoke_handler<
    T: futures::Future<Output = Result<(), String>> + Send + Sync + 'static,
    F: Fn(A::Dispatcher, String) -> T + Send + Sync + 'static,
  >(
    mut self,
    invoke_handler: F,
  ) -> Self {
    self.invoke_handler = Some(Box::new(move |dispatcher, arg| {
      Box::pin(invoke_handler(dispatcher, arg))
    }));
    self
  }

  /// Defines the setup callback.
  pub fn setup<
    T: futures::Future<Output = ()> + Send + Sync + 'static,
    F: Fn(A::Dispatcher, String) -> T + Send + Sync + 'static,
  >(
    mut self,
    setup: F,
  ) -> Self {
    self.setup = Some(Box::new(move |dispatcher, source| {
      Box::pin(setup(dispatcher, source))
    }));
    self
  }

  /// Defines the splashscreen HTML to render.
  pub fn splashscreen_html(mut self, html: &str) -> Self {
    self.splashscreen_html = Some(html.to_string());
    self
  }

  /// Adds a plugin to the runtime.
  pub fn plugin(
    self,
    plugin: impl crate::plugin::Plugin<A::Dispatcher> + Send + Sync + Sync + 'static,
  ) -> Self {
    crate::async_runtime::block_on(crate::plugin::register(A::plugin_store(), plugin));
    self
  }

  /// Builds the App.
  pub fn build(self) -> crate::Result<App<A>> {
    Ok(App {
      invoke_handler: self.invoke_handler,
      setup: self.setup,
      splashscreen_html: self.splashscreen_html,
      context: Context::new::<C>()?,
    })
  }
}
