use futures::future::BoxFuture;
use serde_json::Value as JsonValue;
use tauri_api::{config::Config, private::AsTauriContext};

use crate::async_runtime::Mutex;

use std::{collections::HashMap, marker::PhantomData, sync::Arc};

pub(crate) mod event;
mod runner;
pub(crate) mod webview;
mod webview_manager;

pub use crate::api::config::WindowUrl;
pub use webview::{
  wry::WryApplication, ApplicationDispatcherExt, ApplicationExt, Callback, Icon, Message,
  WebviewBuilderExt,
};
pub use webview_manager::{WebviewDispatcher, WebviewManager};

type InvokeHandler<D> =
  dyn Fn(WebviewManager<D>, String) -> BoxFuture<'static, crate::Result<JsonValue>> + Send + Sync;
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

pub(crate) struct Webview<A: ApplicationExt> {
  pub(crate) builder: A::WebviewBuilder,
  pub(crate) label: String,
  pub(crate) url: WindowUrl,
}

/// The application runner.
pub struct App<A: ApplicationExt> {
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler<A::Dispatcher>>>,
  /// The setup callback, invoked when the webview is ready.
  setup: Option<Box<Setup<A::Dispatcher>>>,
  /// The context the App was created with
  pub(crate) context: Context,
  pub(crate) dispatchers: Arc<Mutex<HashMap<String, WebviewDispatcher<A::Dispatcher>>>>,
  pub(crate) webviews: Option<Vec<Webview<A>>>,
}

impl<A: ApplicationExt + 'static> App<A> {
  /// Runs the app until it finishes.
  pub fn run(mut self) {
    for window_config in self.context.config.tauri.windows.clone() {
      let mut webview = A::WebviewBuilder::new()
        .title(window_config.title.to_string())
        .width(window_config.width)
        .height(window_config.height)
        .visible(window_config.visible)
        .resizable(window_config.resizable)
        .decorations(window_config.decorations)
        .maximized(window_config.maximized)
        .fullscreen(window_config.fullscreen)
        .transparent(window_config.transparent)
        .always_on_top(window_config.always_on_top);
      if let Some(min_width) = window_config.min_width {
        webview = webview.min_width(min_width);
      }
      if let Some(min_height) = window_config.min_height {
        webview = webview.min_height(min_height);
      }
      if let Some(max_width) = window_config.max_width {
        webview = webview.max_width(max_width);
      }
      if let Some(max_height) = window_config.max_height {
        webview = webview.max_height(max_height);
      }
      if let Some(x) = window_config.x {
        webview = webview.x(x);
      }
      if let Some(y) = window_config.y {
        webview = webview.y(y);
      }
      let mut webviews = self.webviews.take().unwrap();
      webviews.push(Webview {
        label: window_config.label.to_string(),
        builder: webview,
        url: window_config.url,
      });
      self.webviews = Some(webviews);
    }
    runner::run(self).expect("Failed to build webview");
  }

  /// Runs the invoke handler if defined.
  /// Returns whether the message was consumed or not.
  /// The message is considered consumed if the handler exists and returns an Ok Result.
  pub(crate) async fn run_invoke_handler(
    &self,
    dispatcher: &WebviewManager<A::Dispatcher>,
    arg: &JsonValue,
  ) -> crate::Result<Option<JsonValue>> {
    if let Some(ref invoke_handler) = self.invoke_handler {
      let fut = invoke_handler(dispatcher.clone(), arg.to_string());
      fut.await.map(Some)
    } else {
      Ok(None)
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
  /// The webview dispatchers.
  dispatchers: Arc<Mutex<HashMap<String, WebviewDispatcher<A::Dispatcher>>>>,
  /// The created webviews.
  webviews: Vec<Webview<A>>,
}

impl<A: ApplicationExt + 'static, C: AsTauriContext> AppBuilder<A, C> {
  /// Creates a new App builder.
  pub fn new() -> Self {
    Self {
      invoke_handler: None,
      setup: None,
      config: Default::default(),
      dispatchers: Default::default(),
      webviews: Default::default(),
    }
  }

  /// Defines the JS message handler callback.
  pub fn invoke_handler<
    T: futures::Future<Output = crate::Result<JsonValue>> + Send + Sync + 'static,
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

  /// Creates a new webview.
  pub fn create_webview<F: FnOnce(A::WebviewBuilder) -> crate::Result<A::WebviewBuilder>>(
    &mut self,
    label: String,
    url: WindowUrl,
    f: F,
  ) -> crate::Result<WebviewManager<A::Dispatcher>> {
    let builder = f(A::WebviewBuilder::new())?;
    self.webviews.push(Webview {
      label: label.to_string(),
      builder,
      url,
    });
    let manager = WebviewManager::new(self.dispatchers.clone(), label);
    Ok(manager)
  }

  /// Builds the App.
  pub fn build(self) -> crate::Result<App<A>> {
    Ok(App {
      invoke_handler: self.invoke_handler,
      setup: self.setup,
      context: Context::new::<C>()?,
      dispatchers: self.dispatchers,
      webviews: Some(self.webviews),
    })
  }
}
