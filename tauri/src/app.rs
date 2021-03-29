pub use self::{
  webview::{
    wry::WryApplication, Attributes, CustomProtocol, FileDropEvent, FileDropHandler, Icon, Message,
    RpcRequest, WebviewRpcHandler,
  },
  webview_manager::{InnerWindowManager, Tag, Window},
};
pub use crate::api::config::WindowUrl;
use crate::{
  api::{
    assets::Assets,
    rpc::{format_callback, format_callback_result},
  },
  app::{
    webview::{AttributesPrivate, WindowConfig},
    webview_manager::WindowManager,
  },
  event::Listeners,
  flavors::Wry,
  plugin::{Plugin, PluginStore},
  runtime::{Dispatch, Runtime},
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  convert::{TryFrom, TryInto},
  future::Future,
  hash::{Hash, Hasher},
  ops::Deref,
  sync::{Arc, Mutex},
};
use tauri_api::config::Config;

pub(crate) mod event;
mod utils;
pub(crate) mod webview;
mod webview_manager;

pub type InvokeHandler<E, L, R> = dyn Fn(InvokeMessage<E, L, R>) + Send;
/// TODO: pass some listener handle
pub type SetupHook<E, L, R> =
  dyn Fn(&mut App<E, L, R>) -> Result<(), Box<dyn std::error::Error>> + Send;
pub type PageLoadHook<E, L, R> = dyn Fn(Window<E, L, R>, PageLoadPayload) + Send;

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
pub struct InvokeMessage<E: Tag, L: Tag, R: Runtime> {
  window: Window<E, L, R>,
  command: String,
  payload: InvokePayload,
}

impl<E: Tag, L: Tag, R: Runtime> InvokeMessage<E, L, R> {
  pub(crate) fn new(window: Window<E, L, R>, command: String, payload: InvokePayload) -> Self {
    Self {
      window,
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

  pub fn window(&self) -> Window<E, L, R> {
    self.window.clone()
  }

  /// Reply to the invoke promise with a async task.
  pub fn respond_async<
    T: Serialize,
    Err: Serialize,
    F: Future<Output = Result<T, Err>> + Send + 'static,
  >(
    self,
    task: F,
  ) {
    if self.payload.main_thread {
      crate::async_runtime::block_on(async move {
        Self::return_task(self.window, task, self.payload.callback, self.payload.error).await;
      });
    } else {
      crate::async_runtime::spawn(async move {
        Self::return_task(self.window, task, self.payload.callback, self.payload.error).await;
      });
    }
  }

  /// Reply to the invoke promise running the given closure.
  pub fn respond_closure<T: Serialize, Err: Serialize, F: FnOnce() -> Result<T, Err>>(self, f: F) {
    Self::return_closure(self.window, f, self.payload.callback, self.payload.error)
  }

  /// Resolve the invoke promise with a value.
  pub fn resolve<S: Serialize>(self, value: S) {
    Self::return_result(
      self.window,
      Result::<S, ()>::Ok(value),
      self.payload.callback,
      self.payload.error,
    )
  }

  /// Reject the invoke promise with a value.
  pub fn reject<S: Serialize>(self, value: S) {
    Self::return_result(
      self.window,
      Result::<(), S>::Err(value),
      self.payload.callback,
      self.payload.error,
    )
  }

  /// Asynchronously executes the given task
  /// and evaluates its Result to the JS promise described by the `success_callback` and `error_callback` function names.
  ///
  /// If the Result `is_ok()`, the callback will be the `success_callback` function name and the argument will be the Ok value.
  /// If the Result `is_err()`, the callback will be the `error_callback` function name and the argument will be the Err value.
  pub async fn return_task<
    T: Serialize,
    Err: Serialize,
    F: std::future::Future<Output = Result<T, Err>> + Send + 'static,
  >(
    window: Window<E, L, R>,
    task: F,
    success_callback: String,
    error_callback: String,
  ) {
    let result = task.await;
    Self::return_closure(window, || result, success_callback, error_callback)
  }

  pub fn return_closure<T: Serialize, Err: Serialize, F: FnOnce() -> Result<T, Err>>(
    window: Window<E, L, R>,
    f: F,
    success_callback: String,
    error_callback: String,
  ) {
    Self::return_result(window, f(), success_callback, error_callback)
  }

  pub fn return_result<T: Serialize, Err: Serialize>(
    window: Window<E, L, R>,
    result: Result<T, Err>,
    success_callback: String,
    error_callback: String,
  ) {
    let callback_string =
      match format_callback_result(result, success_callback, error_callback.clone()) {
        Ok(callback_string) => callback_string,
        Err(e) => format_callback(error_callback, e.to_string()),
      };

    window.eval(&callback_string);
  }
}

/*
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
 */

/*/// `App` runtime information.
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
}*/
/*
pub(crate) struct Window<R: Runtime> {
  pub(crate) builder: R::WindowBuilder,
  pub(crate) label: String,
  pub(crate) url: WindowUrl,
}*/

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

pub trait AsContext {
  type Assets: Assets + Send + Sync;

  fn config(&self) -> &Config;
  fn assets(&self) -> Self::Assets;
  fn default_window_icon(&self) -> Option<&[u8]>;
}

pub struct Context<A: Assets + Send + Sync> {
  pub config: Config,
  pub assets: A,
  pub default_window_icon: Option<Vec<u8>>,
}

/// Represents all the items needed to spawn a window
pub struct PendingWindow<L, D>
where
  L: Tag,
  D: Dispatch,
{
  pub attributes: D::Attributes,
  pub label: L,
  pub url: WindowUrl,
  pub rpc_handler: Option<WebviewRpcHandler<D, L>>,
  pub custom_protocol: Option<CustomProtocol>,
  pub file_drop_handler: Option<Box<dyn Fn(FileDropEvent, D, L) -> bool + Send>>,
}

impl<L: Tag, D: Dispatch> Hash for PendingWindow<L, D> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.label.hash(state)
  }
}

impl<L: Tag, D: Dispatch> Eq for PendingWindow<L, D> {}
impl<L: Tag, D: Dispatch> PartialEq for PendingWindow<L, D> {
  fn eq(&self, other: &Self) -> bool {
    self.label.eq(&other.label)
  }
}

impl<L, D> PendingWindow<L, D>
where
  L: Tag,
  D: Dispatch,
{
  pub fn new(attributes: impl Into<D::Attributes>, label: L, url: WindowUrl) -> Self {
    Self {
      attributes: attributes.into(),
      label,
      url,
      rpc_handler: None,
      custom_protocol: None,
      file_drop_handler: None,
    }
  }

  pub fn set_attributes(&mut self, attributes: D::Attributes) {
    self.attributes = attributes
  }

  pub fn set_rpc_handler(&mut self, rpc: Option<WebviewRpcHandler<D, L>>) {
    self.rpc_handler = rpc
  }

  pub fn set_custom_protocol(&mut self, protocol: Option<CustomProtocol>) {
    self.custom_protocol = protocol
  }

  pub fn set_file_drop(&mut self, handler: Box<dyn Fn(FileDropEvent, D, L) -> bool + Send>) {
    self.file_drop_handler = Some(handler)
  }
}

pub struct App<E: Tag, L: Tag, R: Runtime> {
  runtime: R,
  windows: InnerWindowManager<E, L, R>,
}

impl<E: Tag, L: Tag, R: Runtime> App<E, L, R> {
  pub fn create_window(&mut self, pending: PendingWindow<L, R::Dispatcher>) -> crate::Result<()> {
    let manager = self.windows.clone();
    let label = pending.label.clone();
    println!("runtime create window");
    let dispatcher = self.runtime.create_window(pending)?;
    println!("runtime attach window");
    manager.attach_window(dispatcher, label);
    Ok(())
  }

  fn run(self) {
    self.runtime.run()
  }
}

pub struct Application<E, A, L = String, R = Wry>
where
  R: Runtime,
  A: Assets + Send + Sync,
  L: Tag,
  E: Tag,
{
  config: Arc<Config>,
  assets: Arc<A>,
  default_window_icon: Option<Vec<u8>>,
  inner_window_manager: InnerWindowManager<E, L, R>,
  pending_windows: HashSet<PendingWindow<L, R::Dispatcher>>,
  setup: Box<SetupHook<E, L, R>>,
}

impl<E, A, L, R> Application<E, A, L, R>
where
  R: Runtime + 'static,
  A: Assets + Send + Sync + 'static,
  L: Tag,
  E: Tag,
{
  /// Consume and run the [`App`] until it is finished.
  pub fn run(mut self) -> crate::Result<()> {
    // set up all the windows defined in the config
    for config in self.config.tauri.windows.clone() {
      let url = config.url.clone();
      let label = config
        .label
        .parse()
        .unwrap_or_else(|_| panic!("bad label: {}", config.label));

      self
        .pending_windows
        .insert(PendingWindow::new(WindowConfig(config), label, url));
    }

    //self.hooks.plugins.initialize(&self.config.plugins);

    let pending_windows = std::mem::take(&mut self.pending_windows);
    let mut windows = Vec::new();
    let labels = self.pending_labels();
    for pending in pending_windows {
      let manager = self.inner_window_manager.clone();
      let res = manager.prepare_window(
        pending,
        self.default_window_icon.clone(),
        self.assets.clone(),
        &labels,
      )?;
      windows.push(res);
    }

    let mut app = App {
      runtime: R::new()?,
      windows: self.inner_window_manager,
    };

    dbg!(windows.len());

    //let live = Vec::new();
    for window in windows {
      app.create_window(window)?;
    }

    println!("not yolo");

    (self.setup)(&mut app)?;

    /*

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

    Ok(())*/
    println!("yolo?");
    Ok(app.run())
  }

  fn pending_labels(&self) -> Vec<String> {
    self
      .pending_windows
      .iter()
      .map(|p| p.label.to_string())
      .collect()
  }
}

/// The App builder.
pub struct AppBuilder<E, L = String, R = Wry>
where
  R: Runtime,
  L: Tag,
  E: Tag,
{
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler<E, L, R>>>,

  /// The setup hook.
  setup: Box<SetupHook<E, L, R>>,

  /// Page load hook.
  on_page_load: Option<Box<PageLoadHook<E, L, R>>>,

  /// windows to create when starting up.
  pending_windows: HashSet<PendingWindow<L, R::Dispatcher>>,

  /// All passed plugins
  plugins: PluginStore<E, L, R>,
}

impl<E, L, R> AppBuilder<E, L, R>
where
  R: Runtime,
  L: Tag,
  E: Tag,
{
  /// Creates a new App builder.
  pub fn new() -> Self {
    Self {
      setup: Box::new(|_| Ok(())),
      invoke_handler: None,
      on_page_load: None,
      pending_windows: Default::default(),
      plugins: PluginStore::new(),
    }
  }

  /// Defines the JS message handler callback.
  pub fn invoke_handler<F>(mut self, invoke_handler: F) -> Self
  where
    F: Fn(InvokeMessage<E, L, R>) + Send + 'static,
  {
    self.invoke_handler = Some(Box::new(invoke_handler));
    self
  }

  /// Defines the setup hook.
  pub fn setup<F>(mut self, setup: F) -> Self
  where
    F: Fn(&mut App<E, L, R>) -> Result<(), Box<dyn std::error::Error>> + Send + 'static,
  {
    self.setup = Box::new(setup);
    self
  }

  /// Defines the page load hook.
  pub fn on_page_load<F>(mut self, on_page_load: F) -> Self
  where
    F: Fn(Window<E, L, R>, PageLoadPayload) + Send + 'static,
  {
    self.on_page_load = Some(Box::new(on_page_load));
    self
  }

  /// Adds a plugin to the runtime.
  pub fn plugin<P: Plugin<E, L, R> + 'static>(mut self, plugin: P) -> Self {
    self.plugins.register(plugin);
    self
  }

  /// Creates a new webview.
  pub fn create_window<F>(mut self, label: L, url: WindowUrl, setup: F) -> Self
  where
    F: FnOnce(
      <<R as Runtime>::Dispatcher as Dispatch>::Attributes,
    ) -> <<R as Runtime>::Dispatcher as Dispatch>::Attributes,
  {
    let attributes = setup(<<R as Runtime>::Dispatcher as Dispatch>::Attributes::new());
    self
      .pending_windows
      .insert(PendingWindow::new(attributes, label, url));
    self
  }

  /// Builds the [`App`] and the underlying [`Runtime`].
  pub fn build<A, C>(self, context: C) -> Application<E, A, L, R>
  where
    A: Assets + Send + Sync,
    C: Into<Context<A>>,
  {
    let Context {
      config,
      assets,
      default_window_icon,
    } = context.into();
    let config = Arc::new(config);
    let window_manager =
      InnerWindowManager::new(config.clone(), self.invoke_handler, self.on_page_load);
    let pending_windows = self.pending_windows;

    Application {
      setup: self.setup,
      config: config.clone(),
      assets: Arc::new(assets),
      default_window_icon,
      inner_window_manager: window_manager,
      pending_windows,
    }
  }
}

/// Make `Wry` the default `ApplicationExt` for `AppBuilder`
impl Default for AppBuilder<String, String, Wry> {
  fn default() -> Self {
    Self::new()
  }
}
