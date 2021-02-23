use futures::future::BoxFuture;
use serde::Serialize;
use serde_json::Value as JsonValue;
use tauri_api::{config::Config, private::AsTauriContext};

use crate::async_runtime::Mutex;

use std::{collections::HashMap, marker::PhantomData, sync::Arc};

pub(crate) mod event;
mod utils;
pub(crate) mod webview;
mod webview_manager;

pub use crate::api::config::WindowUrl;
pub use webview::{
  wry::WryApplication, ApplicationDispatcherExt, ApplicationExt, Callback, CustomProtocol, Icon,
  Message, WebviewBuilderExt,
};
pub use webview_manager::{WebviewDispatcher, WebviewManager};

type InvokeHandler<A> = dyn Fn(WebviewManager<A>, String) -> BoxFuture<'static, crate::Result<InvokeResponse>>
  + Send
  + Sync;
type Setup<A> = dyn Fn(WebviewManager<A>) -> BoxFuture<'static, ()> + Send + Sync;

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

/// The response for a JS `invoke` call.
pub struct InvokeResponse {
  json: crate::Result<JsonValue>,
}

impl<T: Serialize> From<T> for InvokeResponse {
  fn from(value: T) -> Self {
    Self {
      json: serde_json::to_value(value).map_err(Into::into),
    }
  }
}

/// The application runner.
pub struct App<A: ApplicationExt> {
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler<A>>>,
  /// The setup callback, invoked when the webview is ready.
  setup: Option<Box<Setup<A>>>,
  /// The context the App was created with
  pub(crate) context: Context,
  pub(crate) dispatchers: Arc<Mutex<HashMap<String, WebviewDispatcher<A::Dispatcher>>>>,
  pub(crate) webviews: Option<Vec<Webview<A>>>,
  url: String,
  window_labels: Arc<Mutex<Vec<String>>>,
  plugin_initialization_script: String,
}

impl<A: ApplicationExt + 'static> App<A> {
  /// Runs the app until it finishes.
  pub fn run(mut self) {
    {
      let mut window_labels = crate::async_runtime::block_on(self.window_labels.lock());
      for window_config in self.context.config.tauri.windows.clone() {
        let window_url = window_config.url.clone();
        let window_label = window_config.label.to_string();
        window_labels.push(window_label.to_string());
        let webview = A::WebviewBuilder::from(webview::WindowConfig(window_config));
        let mut webviews = self.webviews.take().unwrap();
        webviews.push(Webview {
          label: window_label,
          builder: webview,
          url: window_url,
        });
        self.webviews = Some(webviews);
      }
    }

    run(self).expect("failed to run application");
  }

  /// Runs the invoke handler if defined.
  /// Returns whether the message was consumed or not.
  /// The message is considered consumed if the handler exists and returns an Ok Result.
  pub(crate) async fn run_invoke_handler(
    &self,
    dispatcher: &WebviewManager<A>,
    arg: &JsonValue,
  ) -> crate::Result<Option<InvokeResponse>> {
    if let Some(ref invoke_handler) = self.invoke_handler {
      let fut = invoke_handler(dispatcher.clone(), arg.to_string());
      fut.await.map(Some)
    } else {
      Ok(None)
    }
  }

  /// Runs the setup callback if defined.
  pub(crate) async fn run_setup(&self, dispatcher: &WebviewManager<A>) {
    if let Some(ref setup) = self.setup {
      let fut = setup(dispatcher.clone());
      fut.await;
    }
  }
}

#[async_trait::async_trait]
trait WebviewInitializer<A: ApplicationExt> {
  async fn init_webview(
    &self,
    webview: Webview<A>,
  ) -> crate::Result<(
    <A as ApplicationExt>::WebviewBuilder,
    Vec<Callback<A::Dispatcher>>,
    Option<CustomProtocol>,
  )>;

  async fn on_webview_created(
    &self,
    webview_label: String,
    dispatcher: A::Dispatcher,
    manager: WebviewManager<A>,
  );
}

#[async_trait::async_trait]
impl<A: ApplicationExt + 'static> WebviewInitializer<A> for Arc<App<A>> {
  async fn init_webview(
    &self,
    webview: Webview<A>,
  ) -> crate::Result<(
    <A as ApplicationExt>::WebviewBuilder,
    Vec<Callback<A::Dispatcher>>,
    Option<CustomProtocol>,
  )> {
    let webview_manager = WebviewManager::new(
      self.clone(),
      self.dispatchers.clone(),
      webview.label.to_string(),
    );
    utils::build_webview(
      self.clone(),
      webview,
      &webview_manager,
      &self.url,
      &self.window_labels.lock().await,
      &self.plugin_initialization_script,
      &self.context.tauri_script,
      self.context.assets.clone(),
    )
  }

  async fn on_webview_created(
    &self,
    webview_label: String,
    dispatcher: A::Dispatcher,
    manager: WebviewManager<A>,
  ) {
    self.dispatchers.lock().await.insert(
      webview_label.to_string(),
      WebviewDispatcher::new(dispatcher.clone(), webview_label),
    );

    crate::async_runtime::spawn_task(async move {
      crate::plugin::created(A::plugin_store(), &manager).await
    });
  }
}

/// The App builder.
#[derive(Default)]
pub struct AppBuilder<A: ApplicationExt, C: AsTauriContext> {
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler<A>>>,
  /// The setup callback, invoked when the webview is ready.
  setup: Option<Box<Setup<A>>>,
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
    T: futures::Future<Output = crate::Result<InvokeResponse>> + Send + Sync + 'static,
    F: Fn(WebviewManager<A>, String) -> T + Send + Sync + 'static,
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
    F: Fn(WebviewManager<A>) -> T + Send + Sync + 'static,
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
    plugin: impl crate::plugin::Plugin<A> + Send + Sync + Sync + 'static,
  ) -> Self {
    crate::async_runtime::block_on(crate::plugin::register(A::plugin_store(), plugin));
    self
  }

  /// Creates a new webview.
  pub fn create_webview<F: FnOnce(A::WebviewBuilder) -> crate::Result<A::WebviewBuilder>>(
    mut self,
    label: String,
    url: WindowUrl,
    f: F,
  ) -> crate::Result<Self> {
    let builder = f(A::WebviewBuilder::new())?;
    self.webviews.push(Webview {
      label,
      builder,
      url,
    });
    Ok(self)
  }

  /// Builds the App.
  pub fn build(self) -> crate::Result<App<A>> {
    let window_labels: Vec<String> = self.webviews.iter().map(|w| w.label.to_string()).collect();
    let plugin_initialization_script =
      crate::async_runtime::block_on(crate::plugin::initialization_script(A::plugin_store()));

    let context = Context::new::<C>()?;
    let url = utils::get_url(&context)?;

    Ok(App {
      invoke_handler: self.invoke_handler,
      setup: self.setup,
      context,
      dispatchers: self.dispatchers,
      webviews: Some(self.webviews),
      url,
      window_labels: Arc::new(Mutex::new(window_labels)),
      plugin_initialization_script,
    })
  }
}

fn run<A: ApplicationExt + 'static>(mut application: App<A>) -> crate::Result<()> {
  let plugin_config = application.context.config.plugins.clone();
  crate::async_runtime::block_on(async move {
    crate::plugin::initialize(A::plugin_store(), plugin_config).await
  })?;

  let webviews = application.webviews.take().unwrap();

  let application = Arc::new(application);
  let mut webview_app = A::new()?;

  for webview in webviews {
    let webview_label = webview.label.to_string();
    let webview_manager = WebviewManager::new(
      application.clone(),
      application.dispatchers.clone(),
      webview_label.to_string(),
    );
    let (webview_builder, callbacks, custom_protocol) =
      crate::async_runtime::block_on(application.init_webview(webview))?;

    let dispatcher = webview_app.create_webview(webview_builder, callbacks, custom_protocol)?;
    crate::async_runtime::block_on(application.on_webview_created(
      webview_label,
      dispatcher,
      webview_manager,
    ));
  }

  webview_app.run();

  Ok(())
}
