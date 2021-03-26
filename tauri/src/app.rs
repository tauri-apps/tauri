use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tauri_api::{
  config::Config,
  private::AsTauriContext,
  rpc::{format_callback, format_callback_result},
};

use std::{
  collections::HashMap,
  path::PathBuf,
  sync::{Arc, Mutex},
};

pub(crate) mod event;
mod utils;
pub(crate) mod webview;
mod webview_manager;

pub use crate::api::config::WindowUrl;
use crate::flavors::Wry;
#[cfg(feature = "updater")]
use crate::updater;

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
    T: Serialize,
    E: Serialize,
    F: std::future::Future<Output = Result<T, E>> + Send + 'static,
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
  pub fn respond_closure<T: Serialize, E: Serialize, F: FnOnce() -> Result<T, E>>(self, f: F) {
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
  T: Serialize,
  E: Serialize,
  F: std::future::Future<Output = Result<T, E>> + Send + 'static,
>(
  webview_manager: &crate::WebviewManager<A>,
  task: F,
  success_callback: String,
  error_callback: String,
) {
  let result = task.await;
  return_closure(webview_manager, || result, success_callback, error_callback)
}

fn return_closure<
  A: ApplicationExt + 'static,
  T: Serialize,
  E: Serialize,
  F: FnOnce() -> Result<T, E>,
>(
  webview_manager: &crate::WebviewManager<A>,
  f: F,
  success_callback: String,
  error_callback: String,
) {
  return_result(webview_manager, f(), success_callback, error_callback)
}

fn return_result<A: ApplicationExt + 'static, T: Serialize, E: Serialize>(
  webview_manager: &crate::WebviewManager<A>,
  result: Result<T, E>,
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
  pub(crate) package_info: tauri_api::PackageInfo,
}

impl Context {
  pub(crate) fn new<Context: AsTauriContext>(_: Context) -> Self {
    Self {
      config: Context::config(),
      default_window_icon: Context::default_window_icon(),
      assets: Context::assets(),
      package_info: Context::package_info(),
    }
  }
}

pub(crate) struct Webview<A: ApplicationExt> {
  pub(crate) builder: A::WebviewBuilder,
  pub(crate) label: String,
  pub(crate) url: WindowUrl,
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
        // Default user_data_path is `None`.
        let mut user_data_path: Option<PathBuf> = None;
        // If we are on windows use App Data Local as webview temp dir
        // to prevent any bundled application to failed.
        // Fix: https://github.com/tauri-apps/tauri/issues/1365
        // Using cfg! prevent warning for the user_data_path mutability on others platforms.
        if cfg!(windows) {
          // Should return a path similar to C:\Users\<User>\AppData\Local\<AppName>
          let local_app_data = tauri_api::path::resolve_path(
            self.context.package_info.name,
            Some(tauri_api::path::BaseDirectory::LocalData),
          );
          // Make sure the directory exist without panic
          if let Ok(user_data_dir) = local_app_data {
            if let Ok(()) = std::fs::create_dir_all(&user_data_dir) {
              user_data_path = Some(user_data_dir);
            }
          }
        }

        // Build our webview from the WindowConfig and inject user_data_path, default is `None`.
        let webview = A::WebviewBuilder::from(webview::WindowConfig(window_config))
          .with_user_data_path(user_data_path);

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

  /// Runs the updater hook with built-in dialog.
  #[cfg(feature = "updater")]
  pub(crate) fn run_updater_dialog(&self, dispatcher: WebviewManager<A>) {
    let updater_config = self.context.config.tauri.updater.clone();
    let package_info = self.context.package_info.clone();
    crate::async_runtime::spawn(async move {
      updater::check_update_with_dialog(updater_config, package_info, &dispatcher).await
    });
  }

  /// Listen updater events when dialog are disabled.
  #[cfg(feature = "updater")]
  pub(crate) fn listen_updater_events(&self, dispatcher: WebviewManager<A>) {
    let updater_config = self.context.config.tauri.updater.clone();
    updater::listener(
      updater_config,
      self.context.package_info.clone(),
      &dispatcher,
    );
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
  pub fn invoke_handler<F: Fn(WebviewManager<A>, InvokeMessage<A>) + Send + 'static>(
    mut self,
    invoke_handler: F,
  ) -> Self {
    self.invoke_handler = Some(Box::new(invoke_handler));
    self
  }

  /// Defines the setup hook.
  pub fn setup<F: Fn(WebviewManager<A>) + Send + 'static>(mut self, setup: F) -> Self {
    self.setup = Some(Box::new(setup));
    self
  }

  /// Defines the page load hook.
  pub fn on_page_load<F: Fn(WebviewManager<A>, PageLoadPayload) + Send + 'static>(
    mut self,
    on_page_load: F,
  ) -> Self {
    self.on_page_load = Some(Box::new(on_page_load));
    self
  }

  /// Adds a plugin to the runtime.
  pub fn plugin(self, plugin: impl crate::plugin::Plugin<A> + Send + 'static) -> Self {
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

  if let Some(main_webview_manager) = main_webview_manager.clone() {
    application.lock().unwrap().run_setup(main_webview_manager);
  }

  #[cfg(feature = "updater")]
  if let Some(main_webview_manager) = main_webview_manager {
    let event_webview_manager = main_webview_manager.clone();
    let updater_config = application
      .lock()
      .unwrap()
      .context
      .config
      .tauri
      .updater
      .clone();
    // check if updater is active or not
    if updater_config.dialog && updater_config.active {
      // if updater dialog is enabled spawn a new task
      application
        .lock()
        .unwrap()
        .run_updater_dialog(main_webview_manager.clone());
      // When dialog is enabled, if user want to recheck
      // if an update is available after first start
      // invoke the Event `tauri://update` from JS or rust side.
      main_webview_manager.listen(crate::updater::EVENT_CHECK_UPDATE, move |_msg| {
        let webview_manager = event_webview_manager.clone();
        let config = application
          .lock()
          .unwrap()
          .context
          .config
          .tauri
          .updater
          .clone();
        // re-spawn task inside tokyo to launch the download
        // we don't need to emit anything as everything is handled
        // by the process (user is asked to restart at the end)
        // and it's handled by the updater
        let package_info = application.lock().unwrap().context.package_info.clone();
        crate::async_runtime::spawn(async move {
          updater::check_update_with_dialog(config, package_info, &webview_manager).await
        });
      });
    } else if updater_config.active {
      // we only listen for `tauri://update`
      // once we receive the call, we check if an update is available or not
      // if there is a new update we emit `tauri://update-available` with details
      // this is the user responsabilities to display dialog and ask if user want to install
      // to install the update you need to invoke the Event `tauri://update-install`
      application
        .lock()
        .unwrap()
        .listen_updater_events(main_webview_manager);
    }
  }

  // Lock the thread
  webview_app.run();

  Ok(())
}
