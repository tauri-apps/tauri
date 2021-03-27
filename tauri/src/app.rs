pub use self::{
  webview::{
    wry::WryApplication, Attributes, CustomProtocol, FileDropEvent, FileDropHandler, Icon, Message,
    RpcRequest, WebviewRpcHandler,
  },
  webview_manager::{Label, Window, WindowManager},
};
pub use crate::api::config::WindowUrl;
use crate::runtime::Dispatch;
use crate::{
  api::{
    assets::AssetFetch,
    rpc::{format_callback, format_callback_result},
  },
  app::webview::{AttributesPrivate, WindowConfig},
  event::Listeners,
  flavors::Wry,
  plugin::{Plugin, PluginStore},
  runtime::Runtime,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  convert::TryInto,
  future::Future,
  hash::{Hash, Hasher},
  sync::{Arc, Mutex},
};
use tauri_api::config::Config;

pub(crate) mod event;
mod utils;
pub(crate) mod webview;
mod webview_manager;

pub type InvokeHandler<E, L, R> = dyn Fn(InvokeMessage<E, L, R>) + Send;
/// TODO: pass some listener handle
pub type SetupHook = dyn Fn() + Send;
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
pub struct InvokeMessage<E: Label, L: Label, R: Runtime> {
  window: Window<E, L, R>,
  command: String,
  payload: InvokePayload,
}

impl<E: Label, L: Label, R: Runtime> InvokeMessage<E, L, R> {
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
  type Assets: AssetFetch + Send + Sync;

  fn config(&self) -> &Config;
  fn assets(&self) -> Self::Assets;
  fn default_window_icon(&self) -> Option<&[u8]>;
}

pub struct Context<A: AssetFetch + Send + Sync> {
  pub config: Config,
  pub assets: A,
  pub default_window_icon: Option<Vec<u8>>,
}

/// Represents all the items needed to spawn a window
pub struct PendingWindow<L, R = Wry>
where
  L: Label,
  R: Runtime,
{
  pub attributes: R::Attributes,
  pub label: L,
  pub url: WindowUrl,
  pub rpc_handler: Option<WebviewRpcHandler<R::Dispatcher, L>>,
  pub custom_protocol: Option<CustomProtocol>,
  pub file_drop_handler: Option<Box<dyn Fn(FileDropEvent) -> bool + Send>>,
}

impl<L: Label, R: Runtime> Hash for PendingWindow<L, R> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.label.hash(state)
  }
}

impl<L: Label, R: Runtime> Eq for PendingWindow<L, R> {}
impl<L: Label, R: Runtime> PartialEq for PendingWindow<L, R> {
  fn eq(&self, other: &Self) -> bool {
    self.label.eq(&other.label)
  }
}

impl<L, R> PendingWindow<L, R>
where
  L: Label,
  R: Runtime,
{
  pub fn new(attributes: impl Into<R::Attributes>, label: L, url: WindowUrl) -> Self {
    Self {
      attributes: attributes.into(),
      label,
      url,
      rpc_handler: None,
      custom_protocol: None,
      file_drop_handler: None,
    }
  }

  pub fn set_attributes(&mut self, attributes: R::Attributes) {
    self.attributes = attributes
  }

  pub fn set_rpc_handler(&mut self, rpc: Option<WebviewRpcHandler<R::Dispatcher, L>>) {
    self.rpc_handler = rpc
  }

  pub fn set_custom_protocol(&mut self, protocol: Option<CustomProtocol>) {
    self.custom_protocol = protocol
  }

  pub fn set_file_drop(&mut self, handler: impl Fn(FileDropEvent) -> bool + Send + 'static) {
    self.file_drop_handler = Some(Box::new(handler))
  }
}

/// Items meant to be runnable from anywhere
pub struct Hooks<E: Label, L: Label, R: Runtime> {
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler<E, L, R>>>,

  /// The setup hook, invoked when the webviews have been created.
  setup: Option<Box<SetupHook>>,

  /// The page load hook, invoked when the webview performs a navigation.
  on_page_load: Option<Box<PageLoadHook<E, L, R>>>,
  plugins: PluginStore<E, L, R>,
  listeners: Arc<Mutex<HashMap<L, Listeners<E, L>>>>,
  config: Arc<Config>,
}

impl<E: Label, L: Label, R: Runtime> Hooks<E, L, R> {}

pub struct App<E, A, L = String, R = Wry>
where
  R: Runtime,
  A: AssetFetch + Send + Sync,
  L: Label,
  E: Label,
{
  runtime: R,
  config: Arc<Config>,
  assets: Arc<A>,
  default_window_icon: Option<Vec<u8>>,
  window_manager: Arc<WindowManager<E, L, R>>,
  pending_windows: HashSet<PendingWindow<L, R>>,
  //hooks: Arc<Hooks<E, L, R>>,
}

impl<E, A, L, R> App<E, A, L, R>
where
  R: Runtime + 'static,
  A: AssetFetch + Send + Sync + 'static,
  L: Label,
  E: Label,
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
    for pending in pending_windows {
      windows.push(self.prepare_window(pending))
    }

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
    todo!()
  }

  fn pending_labels(&self) -> Vec<L> {
    self
      .pending_windows
      .iter()
      .map(|p| p.label.clone())
      .collect()
  }

  // setup content for dev-server
  #[cfg(dev)]
  fn get_url(&self) -> String {
    if self.config.build.dev_path.starts_with("http") {
      self.config.build.dev_path.clone()
    } else {
      let path = "index.html";
      format!(
        "data:text/html;base64,{}",
        base64::encode(
          self
            .assets
            .get(&path)
            .ok_or_else(|| crate::Error::AssetNotFound(path.to_string()))
            .map(std::borrow::Cow::into_owned)
            .expect("Unable to find `index.html` under your devPath folder")
        )
      )
    }
  }

  #[cfg(custom_protocol)]
  fn get_url(&self) -> String {
    format!("tauri://{}", self.config.tauri.bundle.identifier)
  }

  fn prepare_window(
    &mut self,
    mut pending: PendingWindow<L, R>,
  ) -> crate::Result<PendingWindow<L, R>> {
    let (is_local, url) = match &pending.url {
      WindowUrl::App => (true, self.get_url()),
      WindowUrl::Custom(url) => (&url[0..8] == "tauri://", url.clone()),
    };

    let (builder, rpc_handler, custom_protocol) = if is_local {
      let plugin_init = String::from("adsf"); //todo: self.hooks.plugins.initialization_script();
      let is_init_global = self.config.build.with_global_tauri;
      let mut attributes = pending
        .attributes.clone()
        .url(url)
        .initialization_script(&utils::initialization_script(&plugin_init, is_init_global))
        .initialization_script(&format!(
          r#"
              window.__TAURI__.__windows = {window_labels_array}.map(function (label) {{ return {{ label: label }} }});
              window.__TAURI__.__currentWindow = {{ label: "{current_window_label}" }}
            "#,
          window_labels_array =
          serde_json::to_string(&self.pending_labels()).unwrap(),
          current_window_label = pending.label.clone(),
        ));

      if !attributes.has_icon() {
        if let Some(default_window_icon) = &self.default_window_icon {
          let icon = Icon::Raw(default_window_icon.clone());
          let icon = icon.try_into().expect("infallible icon convert failed");
          attributes = attributes.icon(icon);
        }
      }

      let manager = self.window_manager.clone();
      let rpc_handler: Box<dyn Fn(R::Dispatcher, L, RpcRequest) + Send> =
        Box::new(|dispatcher, label, request: RpcRequest| {
          //let window = Window::new(manager.clone(), dispatcher, label);
          let command = request.command.clone();
          let arg = request
            .params
            .unwrap()
            .as_array_mut()
            .unwrap()
            .first_mut()
            .unwrap_or(&mut JsonValue::Null)
            .take();
          match serde_json::from_value::<InvokePayload>(arg) {
            Ok(message) => {
              //let _ = window.on_message(command, message);
            }
            Err(e) => {
              let error: crate::Error = e.into();
              let _ = dispatcher.eval_script(&format!(
                r#"console.error({})"#,
                JsonValue::String(error.to_string())
              ));
            }
          };
        });

      let assets = self.assets.clone();
      let bundle_identifier = self.config.tauri.bundle.identifier.clone();
      let custom_protocol = CustomProtocol {
        name: "tauri".into(),
        handler: Box::new(move |path| {
          let mut path = path
            .to_string()
            .replace(&format!("tauri://{}", bundle_identifier), "");
          if path.ends_with('/') {
            path.pop();
          }
          let path = if path.is_empty() {
            // if the url is `tauri://${appId}`, we should load `index.html`
            "index.html".to_string()
          } else {
            // skip leading `/`
            path.chars().skip(1).collect::<String>()
          };

          let asset_response = assets
            .get(&path)
            .ok_or(crate::Error::AssetNotFound(path))
            .map(Cow::into_owned);
          match asset_response {
            Ok(asset) => Ok(asset),
            Err(e) => {
              #[cfg(debug_assertions)]
              eprintln!("{:?}", e); // TODO log::error!
              Err(e)
            }
          }
        }),
      };
      (attributes, Some(rpc_handler), Some(custom_protocol))
    } else {
      (pending.attributes.clone().url(url), None, None)
    };

    // TODO: one of the signatures needs to change to allow sending events from this closure,
    // or the file_drop handler must be able to be set after getting the window dispatch proxy
    /*let file_drop_handler: Box<dyn Fn(FileDropEvent) -> bool + Send> = Box::new(move |event| {
      crate::async_runtime::block_on(async move {
        let webview = webview_manager.current_webview().unwrap();
        let _ = match event {
          FileDropEvent::Hovered(paths) => webview.emit("tauri://file-drop-hover", Some(paths)),
          FileDropEvent::Dropped(paths) => webview.emit("tauri://file-drop", Some(paths)),
          FileDropEvent::Cancelled => webview.emit("tauri://file-drop-cancelled", Some(())),
        };
      });
      true
    });*/

    pending.set_attributes(builder);
    pending.set_rpc_handler(rpc_handler);
    pending.set_custom_protocol(custom_protocol);
    // TODO: pending.set_file_drop(file_drop_handler);

    Ok(pending)
  }
}

/// The App builder.
pub struct AppBuilder<E, L = String, R = Wry>
where
  R: Runtime,
  L: Label,
  E: Label,
{
  /// The JS message handler.
  invoke_handler: Option<Box<InvokeHandler<E, L, R>>>,

  /// The setup hook.
  setup: Option<Box<SetupHook>>,

  /// Page load hook.
  on_page_load: Option<Box<PageLoadHook<E, L, R>>>,

  /// windows to create when starting up.
  pending_windows: HashSet<PendingWindow<L, R>>,

  /// All passed plugins
  plugins: PluginStore<E, L, R>,
}

impl<E, L, R> AppBuilder<E, L, R>
where
  R: Runtime,
  L: Label,
  E: Label,
{
  /// Creates a new App builder.
  pub fn new() -> Self {
    Self {
      invoke_handler: None,
      setup: None,
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
  pub fn setup<C, F>(mut self, setup: F) -> Self
  where
    C: AsContext,
    F: Fn() + Send + 'static,
  {
    self.setup = Some(Box::new(setup));
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
    F: FnOnce(R::Attributes) -> R::Attributes,
  {
    let attributes = setup(R::Attributes::new());
    self
      .pending_windows
      .insert(PendingWindow::new(attributes, label, url));
    self
  }

  /// Builds the [`App`] and the underlying [`Runtime`].
  pub fn build<A, C>(self, context: C) -> crate::Result<App<E, A, L, R>>
  where
    A: AssetFetch + Send + Sync,
    C: Into<Context<A>>,
  {
    let Context {
      config,
      assets,
      default_window_icon,
    } = context.into();
    let config = Arc::new(config);
    let window_manager = WindowManager::new(config.clone(), self.invoke_handler, self.on_page_load);
    let pending_windows = self.pending_windows;

    R::new().map(|runtime| App {
      runtime,
      config: config.clone(),
      assets: Arc::new(assets),
      default_window_icon,
      window_manager: Arc::new(window_manager),
      pending_windows,
    })
  }
}

/// Make `Wry` the default `ApplicationExt` for `AppBuilder`
impl Default for AppBuilder<String, String, Wry> {
  fn default() -> Self {
    Self::new()
  }
}
