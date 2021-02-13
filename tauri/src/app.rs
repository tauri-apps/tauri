use crate::ApplicationExt;
use futures::future::BoxFuture;
use std::marker::PhantomData;
use tauri_api::{config::Config, private::AsTauriContext};

pub(crate) mod event;
mod runner;
mod webview_manager;

pub use webview_manager::{WebviewDispatcher, WebviewManager};

type InvokeHandler<D> =
  dyn Fn(WebviewManager<D>, String) -> BoxFuture<'static, crate::Result<()>> + Send + Sync;
type Setup<D> = dyn Fn(WebviewManager<D>) -> BoxFuture<'static, ()> + Send + Sync;

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
    dispatcher: &WebviewManager<A::Dispatcher>,
    arg: &str,
  ) -> crate::Result<bool> {
    if let Some(ref invoke_handler) = self.invoke_handler {
      let fut = invoke_handler(dispatcher.clone(), arg.to_string());
      fut.await.map(|_| true)
    } else {
      Ok(false)
    }
  }

  /// Runs the setup callback if defined.
  pub(crate) async fn run_setup(&self, dispatcher: &WebviewManager<A::Dispatcher>) {
    if let Some(ref setup) = self.setup {
      let fut = setup(dispatcher.clone());
      fut.await;
    }
  }
}

/// The App builder.
#[derive(Default)]
pub struct AppBuilder<A: ApplicationExt, C: AsTauriContext> {
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler<A::Dispatcher>>>,
  /// The setup callback, invoked when the webview is ready.
  setup: Option<Box<Setup<A::Dispatcher>>>,
  /// The configuration used
  config: PhantomData<C>,
}

impl<A: ApplicationExt + 'static, C: AsTauriContext> AppBuilder<A, C> {
  /// Creates a new App builder.
  pub fn new() -> Self {
    Self {
      invoke_handler: None,
      setup: None,
      config: Default::default(),
    }
  }

  /// Defines the JS message handler callback.
  pub fn invoke_handler<
    T: futures::Future<Output = crate::Result<()>> + Send + Sync + 'static,
    F: Fn(WebviewManager<A::Dispatcher>, String) -> T + Send + Sync + 'static,
  >(
    mut self,
    invoke_handler: F,
  ) -> Self {
    self.invoke_handler = Some(Box::new(move |webview_manager, arg| {
      Box::pin(invoke_handler(webview_manager, arg))
    }));
    self
  }

  /// Defines the setup callback.
  pub fn setup<
    T: futures::Future<Output = ()> + Send + Sync + 'static,
    F: Fn(WebviewManager<A::Dispatcher>) -> T + Send + Sync + 'static,
  >(
    mut self,
    setup: F,
  ) -> Self {
    self.setup = Some(Box::new(move |webview_manager| {
      Box::pin(setup(webview_manager))
    }));
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
      context: Context::new::<C>()?,
    })
  }
}
