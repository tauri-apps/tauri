pub use self::{
  webview::{
    wry::WryApplication, Attributes, CustomProtocol, FileDropEvent, FileDropHandler, Icon, Message,
    RpcRequest, WebviewRpcHandler,
  },
  webview_manager::{DetachedWindow, Tag, Window},
};
pub use crate::api::config::WindowUrl;
use crate::{
  api::{
    assets::Assets,
    rpc::{format_callback, format_callback_result},
  },
  app::{
    sealed::ManagerExt,
    webview::WindowConfig,
    webview_manager::{InnerWindowManager, WindowManager},
  },
  event::{EventPayload, HandlerId, Listeners},
  flavors::Wry,
  plugin::{Plugin, PluginStore},
  runtime::{Dispatch, Runtime},
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{
  collections::HashSet,
  error::Error as StdError,
  future::Future,
  hash::{Hash, Hasher},
  sync::{Arc, Mutex},
};
use tauri_api::config::Config;

pub(crate) mod event;
mod utils;
pub(crate) mod webview;
mod webview_manager;

/*#[allow(missing_docs)]
pub type InvokeHandler<E, L, R> = dyn Fn(InvokeMessage<E, L, R>) + Send;
/// TODO: pass some listener handle
pub type SetupHook<E, L, R> =
  dyn Fn(&mut App<E, L, R>) -> Result<(), Box<dyn std::error::Error>> + Send;
#[allow(missing_docs)]
pub type PageLoadHook<E, L, R> = dyn Fn(Window<E, L, R>, PageLoadPayload) + Send;*/

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
pub struct InvokeMessage<M: Manager> {
  window: Window<M>,
  command: String,
  payload: InvokePayload,
}

#[allow(missing_docs)]
impl<M: Manager> InvokeMessage<M> {
  pub(crate) fn new(window: Window<M>, command: String, payload: InvokePayload) -> Self {
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

  pub fn window(&self) -> Window<M> {
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
    window: Window<M>,
    task: F,
    success_callback: String,
    error_callback: String,
  ) {
    let result = task.await;
    Self::return_closure(window, || result, success_callback, error_callback)
  }

  pub fn return_closure<T: Serialize, Err: Serialize, F: FnOnce() -> Result<T, Err>>(
    window: Window<M>,
    f: F,
    success_callback: String,
    error_callback: String,
  ) {
    Self::return_result(window, f(), success_callback, error_callback)
  }

  pub fn return_result<T: Serialize, Err: Serialize>(
    window: Window<M>,
    result: Result<T, Err>,
    success_callback: String,
    error_callback: String,
  ) {
    let callback_string =
      match format_callback_result(result, success_callback, error_callback.clone()) {
        Ok(callback_string) => callback_string,
        Err(e) => format_callback(error_callback, e.to_string()),
      };

    let _ = window.eval(&callback_string);
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
#[allow(missing_docs)]
pub struct Context<A: Assets> {
  pub config: Config,
  pub assets: A,
  pub default_window_icon: Option<Vec<u8>>,
}

#[allow(missing_docs)]
pub struct PendingWindow<M: Manager> {
  pub attributes: <<M::Runtime as Runtime>::Dispatcher as Dispatch>::Attributes,
  pub label: M::Label,
  pub url: WindowUrl,
  pub rpc_handler: Option<WebviewRpcHandler<M>>,
  pub custom_protocol: Option<CustomProtocol>,
  pub file_drop_handler: Option<FileDropHandler<M>>,
}

impl<M: Manager> Hash for PendingWindow<M> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.label.hash(state)
  }
}

impl<M: Manager> Eq for PendingWindow<M> {}
impl<M: Manager> PartialEq for PendingWindow<M> {
  fn eq(&self, other: &Self) -> bool {
    self.label.eq(&other.label)
  }
}

#[allow(missing_docs)]
impl<M: Manager> PendingWindow<M> {
  pub fn new(
    attributes: impl Into<<<M::Runtime as Runtime>::Dispatcher as Dispatch>::Attributes>,
    label: M::Label,
    url: WindowUrl,
  ) -> Self {
    Self {
      attributes: attributes.into(),
      label,
      url,
      rpc_handler: None,
      custom_protocol: None,
      file_drop_handler: None,
    }
  }

  pub fn set_attributes(
    &mut self,
    attributes: <<M::Runtime as Runtime>::Dispatcher as Dispatch>::Attributes,
  ) {
    self.attributes = attributes
  }

  pub fn set_rpc_handler(&mut self, rpc: WebviewRpcHandler<M>) {
    self.rpc_handler = Some(rpc)
  }

  pub fn set_custom_protocol(&mut self, protocol: CustomProtocol) {
    self.custom_protocol = Some(protocol)
  }

  pub fn set_file_drop(
    &mut self,
    file_drop: Box<dyn Fn(FileDropEvent, DetachedWindow<M>) -> bool + Send>,
  ) {
    self.file_drop_handler = Some(file_drop)
  }
}

/// A handle to the currently running application.
pub struct App<M: Manager> {
  runtime: M::Runtime,
  manager: M,
}

impl<M: Manager> Managed<M> for App<M> {}
impl<M: Manager> sealed::ManagedExt<M> for App<M> {
  fn manager(&self) -> &M {
    &self.manager
  }

  fn runtime(&mut self) -> RuntimeOrDispatch<'_, M> {
    RuntimeOrDispatch::Runtime(&mut self.runtime)
  }
}

type SetupHook<M> = Box<dyn Fn(&mut App<M>) -> Result<(), Box<dyn StdError>> + Send>;

#[allow(missing_docs)]
pub struct Runner<M: Manager> {
  pending_windows: HashSet<PendingWindow<M>>,
  manager: M,
  setup: SetupHook<M>,
}

impl<M: Manager> Runner<M> {
  /// Consume and run the [`Application`] until it is finished.
  pub fn run(mut self) -> crate::Result<()> {
    // set up all the windows defined in the config
    for config in self.manager.config().tauri.windows.clone() {
      let url = config.url.clone();
      let label = config
        .label
        .parse()
        .unwrap_or_else(|_| panic!("bad label: {}", config.label));

      self
        .pending_windows
        .insert(PendingWindow::new(WindowConfig(config), label, url));
    }

    self.manager.initialize_plugins()?;
    let labels = self.pending_labels();

    let mut app = App {
      runtime: M::Runtime::new()?,
      manager: self.manager,
    };

    let pending_windows = self.pending_windows;
    for pending in pending_windows {
      let pending = app.manager.prepare_window(pending, &labels)?;
      let detached = app.runtime.create_window(pending)?;
      app.manager.attach_window(detached);
    }

    (self.setup)(&mut app)?;
    app.runtime.run();
    Ok(())
  }

  fn pending_labels(&self) -> HashSet<M::Label> {
    self
      .pending_windows
      .iter()
      .map(|p| p.label.clone())
      .collect()
  }
}

#[allow(missing_docs)]
pub type InvokeHandler<M> = dyn Fn(InvokeMessage<M>) + Send + Sync + 'static;

#[allow(missing_docs)]
pub type OnPageLoad<M> = dyn Fn(Window<M>, PageLoadPayload) + Send + Sync + 'static;

/// Traits to be implemented by this crate, but not allowing external implementations.
pub(crate) mod sealed {
  use super::*;

  /// private manager api
  pub trait ManagerExt<M: Manager>: Clone + Send + Sized + 'static {
    /// Pass messages not handled by modules or plugins to the running application
    fn run_invoke_handler(&self, message: InvokeMessage<M>);

    /// Ran once for every window when the page is loaded.
    fn run_on_page_load(&self, window: Window<M>, payload: PageLoadPayload);

    /// Pass a message to be handled by a plugin that expects the command.
    fn extend_api(&self, command: String, message: InvokeMessage<M>);

    /// Initialize all the plugins attached to the [`Manager`].
    fn initialize_plugins(&self) -> crate::Result<()>;

    /// Prepare a [`PendingWindow`] to be created by the [`Runtime`].
    ///
    /// The passed labels should represent either all the windows in the manager. If the application
    /// has not yet been started, the passed labels should represent all windows that will be
    /// created before starting.
    fn prepare_window(
      &self,
      pending: PendingWindow<M>,
      labels: &HashSet<M::Label>,
    ) -> crate::Result<PendingWindow<M>>;

    /// Attach a detached window to the manager.
    fn attach_window(&self, window: DetachedWindow<M>) -> Window<M>;

    /// Emit an event to javascript windows that pass the predicate.
    fn emit_filter<S: Serialize + Clone, F: Fn(&Window<M>) -> bool>(
      &self,
      event: M::Event,
      payload: Option<S>,
      predicate: F,
    ) -> crate::Result<()>;

    /// All current window labels existing.
    fn labels(&self) -> HashSet<M::Label>;

    /// The configuration the [`Manager`] was built with.
    fn config(&self) -> &Config;

    /// Remove the specified event handler.
    fn unlisten(&self, handler_id: HandlerId);

    /// Trigger an event.
    fn trigger(&self, event: M::Event, window: Option<M::Label>, data: Option<String>);

    /// Set up a listener to an event.
    fn listen<F: Fn(EventPayload) + Send + 'static>(
      &self,
      event: M::Event,
      window: Option<M::Label>,
      handler: F,
    ) -> HandlerId;

    /// Set up a listener to and event that is automatically removed after called once.
    fn once<F: Fn(EventPayload) + Send + 'static>(
      &self,
      event: M::Event,
      window: Option<M::Label>,
      handler: F,
    );
  }

  /// Represents a managed handle to the application runner.
  pub trait ManagedExt<M: Manager> {
    /// The manager behind the [`Managed`] item.
    fn manager(&self) -> &M;

    /// The runtime or runtime dispatcher of the [`Managed`] item.
    fn runtime(&mut self) -> RuntimeOrDispatch<'_, M>;
  }
}

/// Represents either a [`Runtime`] or its dispatcher.
pub enum RuntimeOrDispatch<'m, M: Manager> {
  /// Mutable reference to the [`Runtime`].
  Runtime(&'m mut M::Runtime),

  /// Copy of the [`Runtime`]'s dispatcher.
  Dispatch(<M::Runtime as Runtime>::Dispatcher),
}

/// Represents a managed handle to the application runner
pub trait Managed<M: Manager>: sealed::ManagedExt<M> {
  /// The [`Config`] the manager was created with.
  fn config(&self) -> &Config {
    self.manager().config()
  }

  /// Emits a event to all windows.
  fn emit_all<S: Serialize + Clone>(
    &self,
    event: M::Event,
    payload: Option<S>,
  ) -> crate::Result<()> {
    self.manager().emit_filter(event, payload, |_| true)
  }

  /// Emits an event to a window with the specified label.
  fn emit_to<S: Serialize + Clone>(
    &self,
    label: &M::Label,
    event: M::Event,
    payload: Option<S>,
  ) -> crate::Result<()> {
    self
      .manager()
      .emit_filter(event, payload, |w| w.label() == label)
  }

  /// Creates a new [`Window`] on the [`Runtime`] and attaches it to the [`Manager`].
  fn create_window(&mut self, pending: PendingWindow<M>) -> crate::Result<Window<M>> {
    let labels = self.manager().labels();
    let pending = self.manager().prepare_window(pending, &labels)?;
    match self.runtime() {
      RuntimeOrDispatch::Runtime(runtime) => runtime.create_window(pending),
      RuntimeOrDispatch::Dispatch(mut dispatcher) => dispatcher.create_window(pending),
    }
    .map(|window| self.manager().attach_window(window))
  }

  /// Listen to a global event.
  fn listen_global<F>(&self, event: M::Event, handler: F) -> HandlerId
  where
    F: Fn(EventPayload) + Send + 'static,
  {
    self.manager().listen(event, None, handler)
  }

  /// Listen to a global event only once.
  fn once_global<F>(&self, event: M::Event, handler: F)
  where
    F: Fn(EventPayload) + Send + 'static,
  {
    self.manager().once(event, None, handler)
  }

  /// Trigger a global event.
  fn trigger_global(&self, event: M::Event, data: Option<String>) {
    self.manager().trigger(event, None, data)
  }

  /// Remove an event listener.
  fn unlisten(&self, handler_id: HandlerId) {
    self.manager().unlisten(handler_id)
  }
}

/// public manager api
#[allow(missing_docs)]
pub trait Manager: ManagerExt<Self> {
  type Event: Tag;
  type Label: Tag;
  type Assets: Assets;
  type Runtime: Runtime;
}

/// The App builder.
pub struct AppBuilder<E, L, A, R>
where
  E: Tag,
  L: Tag,
  A: Assets,
  R: Runtime,
{
  /// The JS message handler.
  invoke_handler: Box<InvokeHandler<WindowManager<E, L, A, R>>>,

  /// The setup hook.
  setup: SetupHook<WindowManager<E, L, A, R>>,

  /// Page load hook.
  on_page_load: Box<OnPageLoad<WindowManager<E, L, A, R>>>,

  /// windows to create when starting up.
  pending_windows: HashSet<PendingWindow<WindowManager<E, L, A, R>>>,

  /// All passed plugins
  plugins: PluginStore<WindowManager<E, L, A, R>>,
}

impl<E, L, A, R> AppBuilder<E, L, A, R>
where
  E: Tag,
  L: Tag,
  A: Assets,
  R: Runtime,
{
  /// Creates a new App builder.
  pub fn new() -> Self {
    Self {
      setup: Box::new(|_| Ok(())),
      invoke_handler: Box::new(|_| ()),
      on_page_load: Box::new(|_, _| ()),
      pending_windows: Default::default(),
      plugins: PluginStore::default(),
    }
  }

  /// Defines the JS message handler callback.
  pub fn invoke_handler<F>(mut self, invoke_handler: F) -> Self
  where
    F: Fn(InvokeMessage<WindowManager<E, L, A, R>>) + Send + Sync + 'static,
  {
    self.invoke_handler = Box::new(invoke_handler);
    self
  }

  /// Defines the setup hook.
  pub fn setup<F>(mut self, setup: F) -> Self
  where
    F: Fn(&mut App<WindowManager<E, L, A, R>>) -> Result<(), Box<dyn StdError>> + Send + 'static,
  {
    self.setup = Box::new(setup);
    self
  }

  /// Defines the page load hook.
  pub fn on_page_load<F>(mut self, on_page_load: F) -> Self
  where
    F: Fn(Window<WindowManager<E, L, A, R>>, PageLoadPayload) + Send + Sync + 'static,
  {
    self.on_page_load = Box::new(on_page_load);
    self
  }

  /// Adds a plugin to the runtime.
  pub fn plugin<P: Plugin<WindowManager<E, L, A, R>> + 'static>(mut self, plugin: P) -> Self {
    self.plugins.register(plugin);
    self
  }

  /// Creates a new webview.
  pub fn create_window<F>(mut self, label: L, url: WindowUrl, setup: F) -> Self
  where
    F: FnOnce(<R::Dispatcher as Dispatch>::Attributes) -> <R::Dispatcher as Dispatch>::Attributes,
  {
    let attributes = setup(<R::Dispatcher as Dispatch>::Attributes::new());
    self
      .pending_windows
      .insert(PendingWindow::new(attributes, label, url));
    self
  }

  /// Builds the [`App`] and the underlying [`Runtime`].
  pub fn build<C: Into<Context<A>>>(self, context: C) -> Runner<WindowManager<E, L, A, R>> {
    let Context {
      config,
      assets,
      default_window_icon,
    } = context.into();

    Runner {
      pending_windows: self.pending_windows,
      setup: self.setup,
      manager: WindowManager {
        inner: Arc::new(InnerWindowManager {
          windows: Mutex::default(),
          plugins: Mutex::default(),
          listeners: Listeners::default(),
          invoke_handler: self.invoke_handler,
          on_page_load: self.on_page_load,
          config,
          assets: Arc::new(assets),
          default_window_icon,
        }),
      },
    }
  }
}

/// Make `Wry` the default `ApplicationExt` for `AppBuilder`
impl<A: Assets> Default for AppBuilder<String, String, A, Wry> {
  fn default() -> Self {
    Self::new()
  }
}
