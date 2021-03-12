use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tauri_api::{config::Config, private::AsTauriContext};

use crate::async_runtime::Mutex;

use std::{collections::HashMap, marker::PhantomData, sync::Arc};

pub(crate) mod event;
mod utils;
pub(crate) mod webview;
mod webview_manager;

pub use crate::api::config::WindowUrl;
use crate::flavors::Wry;
#[cfg(feature = "updater")]
use crate::updater;

pub use webview::{
  wry::WryApplication, ApplicationDispatcherExt, ApplicationExt, CustomProtocol, Icon, Message,
  RpcRequest, WebviewBuilderExt, WebviewRpcHandler,
};
pub use webview_manager::{WebviewDispatcher, WebviewManager};

type InvokeHandler<A> = dyn Fn(WebviewManager<A>, String, JsonValue) -> BoxFuture<'static, crate::Result<InvokeResponse>>
  + Send
  + Sync;
type ManagerHook<A> = dyn Fn(WebviewManager<A>) -> BoxFuture<'static, ()> + Send + Sync;
type PageLoadHook<A> =
  dyn Fn(WebviewManager<A>, PageLoadPayload) -> BoxFuture<'static, ()> + Send + Sync;

/// `App` runtime information.
pub struct Context {
  pub(crate) config: Config,
  pub(crate) tauri_script: &'static str,
  pub(crate) default_window_icon: Option<&'static [u8]>,
  pub(crate) assets: &'static tauri_api::assets::Assets,
  pub(crate) meta: Option<Meta>,
}

/// `App` runtime meta. Expose version and application name
#[derive(Debug, Clone, Deserialize)]
pub struct Meta {
  pub(crate) name: &'static str,
  pub(crate) version: &'static str,
}

impl Context {
  pub(crate) fn new<Context: AsTauriContext>() -> crate::Result<Self> {
    Ok(Self {
      config: serde_json::from_str(Context::raw_config())?,
      tauri_script: Context::raw_tauri_script(),
      default_window_icon: Context::default_window_icon(),
      assets: Context::assets(),
      meta: None,
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

/// The payload for the "page_load" hook.
#[derive(Debug, Clone, Deserialize)]
pub struct PageLoadPayload {
  url: String,
}

impl PageLoadPayload {
  /// The page URL.
  pub fn url(&self) -> &str {
    &self.url
  }
}

/// The application runner.
pub struct App<A: ApplicationExt> {
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler<A>>>,
  /// The page load hook, invoked when the webview performs a navigation.
  on_page_load: Option<Box<PageLoadHook<A>>>,
  /// The setup hook, invoked when the webviews have been created.
  setup: Option<Box<ManagerHook<A>>>,
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

  /// Set application meta.
  pub fn meta(mut self, application_name: &'static str, application_version: &'static str) -> Self {
    self.context.meta = Some(Meta {
      name: application_name,
      version: application_version,
    });
    self
  }

  /// Runs the invoke handler if defined.
  /// Returns whether the message was consumed or not.
  /// The message is considered consumed if the handler exists and returns an Ok Result.
  pub(crate) async fn run_invoke_handler(
    &self,
    dispatcher: &WebviewManager<A>,
    command: String,
    arg: &JsonValue,
  ) -> crate::Result<Option<InvokeResponse>> {
    if let Some(ref invoke_handler) = self.invoke_handler {
      let fut = invoke_handler(dispatcher.clone(), command, arg.clone());
      fut.await.map(Some)
    } else {
      Ok(None)
    }
  }

  /// Runs the setup hook if defined.
  pub(crate) async fn run_setup(&self, dispatcher: WebviewManager<A>) {
    if let Some(ref setup) = self.setup {
      let fut = setup(dispatcher);
      fut.await;
    }
  }

  #[cfg(feature = "updater")]
  /// Runs the updater hook.
  pub(crate) async fn run_updater_dialog(&self, dispatcher: WebviewManager<A>) {
    let mut meta = self.context.meta.clone();
    let updater_config = self.context.config.tauri.updater.clone();
    if meta.is_none() {
      meta = Some(Meta {
        name: "Tauri",
        version: "0.0.0",
      })
    }
    crate::async_runtime::spawn_task(async move {
      updater::check_update_with_dialog(updater_config, meta.unwrap(), &dispatcher).await
    });
  }

  #[cfg(feature = "updater")]
  /// Runs the updater hook.
  pub(crate) async fn listen_updater_events(&self, dispatcher: WebviewManager<A>) {
    let updater_config = self.context.config.tauri.updater.clone();
    let mut meta = self.context.meta.clone();
    if meta.is_none() {
      meta = Some(Meta {
        name: "Tauri",
        version: "0.0.0",
      })
    }
    updater::listener(updater_config, meta.unwrap(), &dispatcher);
  }

  /// Runs the on page load hook if defined.
  pub(crate) async fn run_on_page_load(
    &self,
    dispatcher: &WebviewManager<A>,
    payload: PageLoadPayload,
  ) {
    if let Some(ref on_page_load) = self.on_page_load {
      let fut = on_page_load(dispatcher.clone(), payload);
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
    Option<WebviewRpcHandler<A::Dispatcher>>,
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
    Option<WebviewRpcHandler<A::Dispatcher>>,
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
      &self.context,
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
pub struct AppBuilder<C: AsTauriContext, A = Wry>
where
  A: ApplicationExt,
{
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler<A>>>,
  /// The setup hook.
  setup: Option<Box<ManagerHook<A>>>,
  /// Page load hook.
  on_page_load: Option<Box<PageLoadHook<A>>>,
  config: PhantomData<C>,
  /// The webview dispatchers.
  dispatchers: Arc<Mutex<HashMap<String, WebviewDispatcher<A::Dispatcher>>>>,
  /// The created webviews.
  webviews: Vec<Webview<A>>,
}

impl<A: ApplicationExt + 'static, C: AsTauriContext> AppBuilder<C, A> {
  /// Creates a new App builder.
  pub fn new() -> Self {
    Self {
      invoke_handler: None,
      setup: None,
      on_page_load: None,
      config: Default::default(),
      dispatchers: Default::default(),
      webviews: Default::default(),
    }
  }

  /// Defines the JS message handler callback.
  pub fn invoke_handler<
    T: futures::Future<Output = crate::Result<InvokeResponse>> + Send + Sync + 'static,
    F: Fn(WebviewManager<A>, String, JsonValue) -> T + Send + Sync + 'static,
  >(
    mut self,
    invoke_handler: F,
  ) -> Self {
    self.invoke_handler = Some(Box::new(move |webview_manager, command, args| {
      Box::pin(invoke_handler(webview_manager, command, args))
    }));
    self
  }

  /// Defines the setup hook.
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

  /// Defines the page load hook.
  pub fn on_page_load<
    T: futures::Future<Output = ()> + Send + Sync + 'static,
    F: Fn(WebviewManager<A>, PageLoadPayload) -> T + Send + Sync + 'static,
  >(
    mut self,
    on_page_load: F,
  ) -> Self {
    self.on_page_load = Some(Box::new(move |webview_manager, payload| {
      Box::pin(on_page_load(webview_manager, payload))
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
    let url = utils::get_url(&context);

    Ok(App {
      invoke_handler: self.invoke_handler,
      setup: self.setup,
      on_page_load: self.on_page_load,
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
  let mut main_webview_manager = None;

  for webview in webviews {
    let webview_label = webview.label.to_string();
    let webview_manager = WebviewManager::new(
      application.clone(),
      application.dispatchers.clone(),
      webview_label.to_string(),
    );
    if main_webview_manager.is_none() {
      main_webview_manager = Some(webview_manager.clone());
    }
    let (webview_builder, rpc_handler, custom_protocol) =
      crate::async_runtime::block_on(application.init_webview(webview))?;

    let dispatcher = webview_app.create_webview(webview_builder, rpc_handler, custom_protocol)?;
    crate::async_runtime::block_on(application.on_webview_created(
      webview_label,
      dispatcher,
      webview_manager,
    ));
  }

  if let Some(main_webview_manager) = main_webview_manager.clone() {
    crate::async_runtime::block_on(application.run_setup(main_webview_manager));
  }

  #[cfg(feature = "updater")]
  if let Some(main_webview_manager) = main_webview_manager {
    let updater_config = application.context.config.tauri.updater.clone();
    let event_webview_manager = main_webview_manager.clone();

    if updater_config.dialog && updater_config.active {
      // if updater dialog is enabled spawn a new task
      crate::async_runtime::block_on(application.run_updater_dialog(main_webview_manager.clone()));
      // When dialog is enabled, if user want to recheck
      // if an update is available after first start
      // invoke the Event `tauri://update` from JS or rust side.
      main_webview_manager.listen(crate::updater::EVENT_CHECK_UPDATE, move |_msg| {
        let webview_manager = event_webview_manager.clone();
        let config = application.context.config.tauri.updater.clone();
        let mut meta = application.context.meta.clone();
        // re-spawn task inside tokyo to launch the download
        // we don't need to emit anything as everything is handled
        // by the process (user is asked to restart at the end)
        // and it's handled by the updater
        if meta.is_none() {
          meta = Some(Meta {
            name: "Tauri",
            version: "0.0.0",
          });
        }
        crate::async_runtime::spawn_task(async move {
          updater::check_update_with_dialog(config, meta.unwrap(), &webview_manager).await
        });
      });
    } else if updater_config.active {
      // we only listen for `tauri://update`
      // once we receive the call, we check if an update is available or not
      // if there is a new update we emit `tauri://update-available` with details
      // this is the user responsabilities to display dialog and ask if user want to install
      // to install the update you need to invoke the Event `tauri://update-install`
      crate::async_runtime::block_on(application.listen_updater_events(main_webview_manager));
    }
  }

  // Lock the thread
  webview_app.run();

  Ok(())
}
