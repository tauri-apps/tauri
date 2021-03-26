use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tauri_api::{
  config::Config,
  private::AsTauriContext,
  rpc::{format_callback, format_callback_result},
};

use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

pub(crate) mod event;
mod utils;
pub(crate) mod webview;
mod webview_manager;

pub use crate::api::config::WindowUrl;
use crate::flavors::Wry;
pub use webview::{
  wry::WryApplication, ApplicationDispatcherExt, ApplicationExt, CustomProtocol, FileDropEvent,
  FileDropHandler, Icon, Message, RpcRequest, WebviewBuilderExt, WebviewRpcHandler,
};
pub use webview_manager::{WebviewDispatcher, WebviewManager};

type InvokeHandler<A> = dyn Fn(WebviewManager<A>, InvokeMessage<A>) + Send;
type ManagerHook<A> = dyn Fn(WebviewManager<A>) + Send;
type PageLoadHook<A> = dyn Fn(WebviewManager<A>, PageLoadPayload) + Send;

/// Payload from an invoke call.
#[derive(Debug, Deserialize)]
pub(crate) struct InvokePayload {
  #[serde(rename = "__tauriModule")]
  tauri_module: Option<String>,
  callback: String,
  error: String,
  #[serde(rename = "mainThread", default)]
  pub(crate) main_thread: bool,
  #[serde(flatten)]
  inner: JsonValue,
}

/// An invoke message.
pub struct InvokeMessage<A: ApplicationExt> {
  webview_manager: WebviewManager<A>,
  command: String,
  payload: InvokePayload,
}

impl<A: ApplicationExt + 'static> InvokeMessage<A> {
  pub(crate) fn new(
    webview_manager: WebviewManager<A>,
    command: String,
    payload: InvokePayload,
  ) -> Self {
    Self {
      webview_manager,
      command,
      payload,
    }
  }

  /// The invoke command.
  pub fn command(&self) -> &str {
    &self.command
  }

  /// The invoke payload.
  pub fn payload(&self) -> JsonValue {
    self.payload.inner.clone()
  }

  /// Reply to the invoke promise with a async task.
  pub fn respond_async<
    F: std::future::Future<Output = crate::Result<InvokeResponse>> + Send + 'static,
  >(
    self,
    task: F,
  ) {
    if self.payload.main_thread {
      crate::async_runtime::block_on(async move {
        return_task(
          &self.webview_manager,
          task,
          self.payload.callback,
          self.payload.error,
        )
        .await;
      });
    } else {
      crate::async_runtime::spawn(async move {
        return_task(
          &self.webview_manager,
          task,
          self.payload.callback,
          self.payload.error,
        )
        .await;
      });
    }
  }

  /// Reply to the invoke promise running the given closure.
  pub fn respond_closure<
    O: Serialize,
    E: Serialize,
    F: FnOnce() -> Result<O, E> + Send + 'static,
  >(
    self,
    f: F,
  ) {
    return_closure(
      &self.webview_manager,
      f,
      self.payload.callback,
      self.payload.error,
    )
  }

  /// Resolve the invoke promise with a value.
  pub fn resolve<S: Serialize>(self, value: S) {
    return_result(
      &self.webview_manager,
      Result::<S, ()>::Ok(value),
      self.payload.callback,
      self.payload.error,
    )
  }

  /// Reject the invoke promise with a value.
  pub fn reject<S: Serialize>(self, value: S) {
    return_result(
      &self.webview_manager,
      Result::<(), S>::Err(value),
      self.payload.callback,
      self.payload.error,
    )
  }
}

/// Asynchronously executes the given task
/// and evaluates its Result to the JS promise described by the `success_callback` and `error_callback` function names.
///
/// If the Result `is_ok()`, the callback will be the `success_callback` function name and the argument will be the Ok value.
/// If the Result `is_err()`, the callback will be the `error_callback` function name and the argument will be the Err value.
async fn return_task<
  A: ApplicationExt + 'static,
  F: std::future::Future<Output = crate::Result<InvokeResponse>> + Send + 'static,
>(
  webview_manager: &crate::WebviewManager<A>,
  task: F,
  success_callback: String,
  error_callback: String,
) {
  let result = task
    .await
    .and_then(|response| response.json)
    .map_err(|err| err.to_string());
  return_closure(webview_manager, || result, success_callback, error_callback)
}

fn return_closure<
  A: ApplicationExt + 'static,
  O: Serialize,
  E: Serialize,
  F: FnOnce() -> Result<O, E> + Send + 'static,
>(
  webview_manager: &crate::WebviewManager<A>,
  f: F,
  success_callback: String,
  error_callback: String,
) {
  return_result(webview_manager, f(), success_callback, error_callback)
}

fn return_result<A: ApplicationExt + 'static, O: Serialize, E: Serialize>(
  webview_manager: &crate::WebviewManager<A>,
  result: Result<O, E>,
  success_callback: String,
  error_callback: String,
) {
  let callback_string =
    match format_callback_result(result, success_callback, error_callback.clone()) {
      Ok(callback_string) => callback_string,
      Err(e) => format_callback(error_callback, e.to_string()),
    };
  if let Ok(dispatcher) = webview_manager.current_webview() {
    let _ = dispatcher.eval(callback_string.as_str());
  }
}

/// `App` runtime information.
pub struct Context {
  pub(crate) config: &'static Config,
  pub(crate) default_window_icon: Option<&'static [u8]>,
  pub(crate) assets: &'static tauri_api::assets::EmbeddedAssets,
}

impl Context {
  pub(crate) fn new<Context: AsTauriContext>(_: Context) -> Self {
    Self {
      config: Context::config(),
      default_window_icon: Context::default_window_icon(),
      assets: Context::assets(),
    }
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
      let mut window_labels = self.window_labels.lock().unwrap();
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
  pub(crate) fn run_invoke_handler(
    &self,
    dispatcher: &WebviewManager<A>,
    message: InvokeMessage<A>,
  ) {
    if let Some(ref invoke_handler) = self.invoke_handler {
      invoke_handler(dispatcher.clone(), message);
    }
  }

  /// Runs the setup hook if defined.
  pub(crate) fn run_setup(&self, dispatcher: WebviewManager<A>) {
    if let Some(ref setup) = self.setup {
      setup(dispatcher);
    }
  }

  /// Runs the on page load hook if defined.
  pub(crate) fn run_on_page_load(&self, dispatcher: &WebviewManager<A>, payload: PageLoadPayload) {
    if let Some(ref on_page_load) = self.on_page_load {
      on_page_load(dispatcher.clone(), payload);
    }
  }
}

type WebviewContext<A> = (
  <A as ApplicationExt>::WebviewBuilder,
  Option<WebviewRpcHandler<<A as ApplicationExt>::Dispatcher>>,
  Option<CustomProtocol>,
  Option<FileDropHandler>,
);

trait WebviewInitializer<A: ApplicationExt> {
  fn init_webview(&self, webview: Webview<A>) -> crate::Result<WebviewContext<A>>;

  fn on_webview_created(&self, webview_label: String, dispatcher: A::Dispatcher);
}

impl<A: ApplicationExt + 'static> WebviewInitializer<A> for Arc<Mutex<App<A>>> {
  fn init_webview(&self, webview: Webview<A>) -> crate::Result<WebviewContext<A>> {
    let application = self.lock().unwrap();
    let webview_manager = WebviewManager::new(
      self.clone(),
      application.dispatchers.clone(),
      webview.label.to_string(),
    );
    let (webview_builder, rpc_handler, custom_protocol) = utils::build_webview(
      self.clone(),
      webview,
      &webview_manager,
      &application.url,
      &application.window_labels.lock().unwrap(),
      &application.plugin_initialization_script,
      &application.context,
    )?;
    let file_drop_handler: Box<dyn Fn(FileDropEvent) -> bool + Send> = Box::new(move |event| {
      let webview_manager = webview_manager.clone();
      crate::async_runtime::block_on(async move {
        let webview = webview_manager.current_webview().unwrap();
        let _ = match event {
          FileDropEvent::Hovered(paths) => webview.emit("tauri://file-drop-hover", Some(paths)),
          FileDropEvent::Dropped(paths) => webview.emit("tauri://file-drop", Some(paths)),
          FileDropEvent::Cancelled => webview.emit("tauri://file-drop-cancelled", Some(())),
        };
      });
      true
    });
    Ok((
      webview_builder,
      rpc_handler,
      custom_protocol,
      Some(file_drop_handler),
    ))
  }

  fn on_webview_created(&self, webview_label: String, dispatcher: A::Dispatcher) {
    self.lock().unwrap().dispatchers.lock().unwrap().insert(
      webview_label.to_string(),
      WebviewDispatcher::new(dispatcher, webview_label),
    );
  }
}

/// The App builder.
pub struct AppBuilder<A = Wry>
where
  A: ApplicationExt,
{
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler<A>>>,
  /// The setup hook.
  setup: Option<Box<ManagerHook<A>>>,
  /// Page load hook.
  on_page_load: Option<Box<PageLoadHook<A>>>,
  /// The webview dispatchers.
  dispatchers: Arc<Mutex<HashMap<String, WebviewDispatcher<A::Dispatcher>>>>,
  /// The created webviews.
  webviews: Vec<Webview<A>>,
}

impl<A: ApplicationExt + 'static> AppBuilder<A> {
  /// Creates a new App builder.
  pub fn new() -> Self {
    Self {
      invoke_handler: None,
      setup: None,
      on_page_load: None,
      dispatchers: Default::default(),
      webviews: Default::default(),
    }
  }

  /// Defines the JS message handler callback.
  pub fn invoke_handler<F: Fn(WebviewManager<A>, InvokeMessage<A>) + Send + Sync + 'static>(
    mut self,
    invoke_handler: F,
  ) -> Self {
    self.invoke_handler = Some(Box::new(invoke_handler));
    self
  }

  /// Defines the setup hook.
  pub fn setup<F: Fn(WebviewManager<A>) + Send + Sync + 'static>(mut self, setup: F) -> Self {
    self.setup = Some(Box::new(setup));
    self
  }

  /// Defines the page load hook.
  pub fn on_page_load<F: Fn(WebviewManager<A>, PageLoadPayload) + Send + Sync + 'static>(
    mut self,
    on_page_load: F,
  ) -> Self {
    self.on_page_load = Some(Box::new(on_page_load));
    self
  }

  /// Adds a plugin to the runtime.
  pub fn plugin(
    self,
    plugin: impl crate::plugin::Plugin<A> + Send + Sync + Sync + 'static,
  ) -> Self {
    crate::plugin::register(A::plugin_store(), plugin);
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
  pub fn build(self, context: impl AsTauriContext) -> App<A> {
    let window_labels: Vec<String> = self.webviews.iter().map(|w| w.label.to_string()).collect();
    let plugin_initialization_script = crate::plugin::initialization_script(A::plugin_store());

    let context = Context::new(context);
    let url = utils::get_url(&context);

    App {
      invoke_handler: self.invoke_handler,
      setup: self.setup,
      on_page_load: self.on_page_load,
      context,
      dispatchers: self.dispatchers,
      webviews: Some(self.webviews),
      url,
      window_labels: Arc::new(Mutex::new(window_labels)),
      plugin_initialization_script,
    }
  }
}

/// Make `Wry` the default `ApplicationExt` for `AppBuilder`
impl Default for AppBuilder<Wry> {
  fn default() -> Self {
    Self::new()
  }
}

fn run<A: ApplicationExt + 'static>(mut application: App<A>) -> crate::Result<()> {
  let plugin_config = application.context.config.plugins.clone();
  crate::plugin::initialize(A::plugin_store(), plugin_config)?;

  let webviews = application.webviews.take().unwrap();

  let dispatchers = application.dispatchers.clone();
  let application = Arc::new(Mutex::new(application));
  let mut webview_app = A::new()?;
  let mut main_webview_manager = None;

  for webview in webviews {
    let webview_label = webview.label.to_string();
    let webview_manager = WebviewManager::new(
      application.clone(),
      dispatchers.clone(),
      webview_label.to_string(),
    );
    if main_webview_manager.is_none() {
      main_webview_manager = Some(webview_manager.clone());
    }
    let (webview_builder, rpc_handler, custom_protocol, file_drop_handler) =
      application.init_webview(webview)?;

    let dispatcher = webview_app.create_webview(
      webview_builder,
      rpc_handler,
      custom_protocol,
      file_drop_handler,
    )?;
    application.on_webview_created(webview_label, dispatcher);
    crate::plugin::created(A::plugin_store(), &webview_manager);
  }

  if let Some(main_webview_manager) = main_webview_manager {
    application.lock().unwrap().run_setup(main_webview_manager);
  }

  webview_app.run();

  Ok(())
}
