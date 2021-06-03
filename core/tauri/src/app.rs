// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "system-tray")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
pub(crate) mod tray;

use crate::{
  api::assets::Assets,
  api::config::WindowUrl,
  hooks::{InvokeHandler, OnPageLoad, PageLoadPayload, SetupHook},
  manager::{Args, WindowManager},
  plugin::{Plugin, PluginStore},
  runtime::{
    tag::Tag,
    webview::{CustomProtocol, WebviewAttributes, WindowBuilder},
    window::{PendingWindow, WindowEvent},
    Dispatch, MenuId, Params, Runtime,
  },
  sealed::{ManagerBase, RuntimeOrDispatch},
  Context, Invoke, Manager, StateManager, Window,
};

use std::{collections::HashMap, sync::Arc};

#[cfg(feature = "menu")]
use crate::runtime::menu::Menu;

#[cfg(feature = "system-tray")]
use crate::runtime::{Icon, SystemTrayEvent as RuntimeSystemTrayEvent};

#[cfg(feature = "updater")]
use crate::updater;

#[cfg(feature = "menu")]
pub(crate) type GlobalMenuEventListener<P> = Box<dyn Fn(WindowMenuEvent<P>) + Send + Sync>;
pub(crate) type GlobalWindowEventListener<P> = Box<dyn Fn(GlobalWindowEvent<P>) + Send + Sync>;
#[cfg(feature = "system-tray")]
type SystemTrayEventListener<P> =
  Box<dyn Fn(&AppHandle<P>, tray::SystemTrayEvent<<P as Params>::SystemTrayMenuId>) + Send + Sync>;

crate::manager::default_args! {
  /// A menu event that was triggered on a window.
  #[cfg(feature = "menu")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
  pub struct WindowMenuEvent<P: Params> {
    pub(crate) menu_item_id: P::MenuId,
    pub(crate) window: Window<P>,
  }
}

#[cfg(feature = "menu")]
impl<P: Params> WindowMenuEvent<P> {
  /// The menu item id.
  pub fn menu_item_id(&self) -> &P::MenuId {
    &self.menu_item_id
  }

  /// The window that the menu belongs to.
  pub fn window(&self) -> &Window<P> {
    &self.window
  }
}

crate::manager::default_args! {
  /// A window event that was triggered on the specified window.
  pub struct GlobalWindowEvent<P: Params> {
    pub(crate) event: WindowEvent,
    pub(crate) window: Window<P>,
  }
}

impl<P: Params> GlobalWindowEvent<P> {
  /// The eventpayload.
  pub fn event(&self) -> &WindowEvent {
    &self.event
  }

  /// The window that the menu belongs to.
  pub fn window(&self) -> &Window<P> {
    &self.window
  }
}

crate::manager::default_args! {
  /// A handle to the currently running application.
  ///
  /// This type implements [`Manager`] which allows for manipulation of global application items.
  pub struct AppHandle<P: Params> {
    runtime_handle: <P::Runtime as Runtime>::Handle,
    manager: WindowManager<P>,
    #[cfg(feature = "system-tray")]
    tray_handle: Option<tray::SystemTrayHandle<P>>,
  }
}

impl<P: Params> Clone for AppHandle<P> {
  fn clone(&self) -> Self {
    Self {
      runtime_handle: self.runtime_handle.clone(),
      manager: self.manager.clone(),
      tray_handle: self.tray_handle.clone(),
    }
  }
}

impl<P: Params> Manager<P> for AppHandle<P> {}
impl<P: Params> ManagerBase<P> for AppHandle<P> {
  fn manager(&self) -> &WindowManager<P> {
    &self.manager
  }

  fn runtime(&self) -> RuntimeOrDispatch<'_, P> {
    RuntimeOrDispatch::RuntimeHandle(self.runtime_handle.clone())
  }
}

crate::manager::default_args! {
  /// The instance of the currently running application.
  ///
  /// This type implements [`Manager`] which allows for manipulation of global application items.
  pub struct App<P: Params> {
    runtime: Option<P::Runtime>,
    manager: WindowManager<P>,
    #[cfg(shell_execute)]
    cleanup_on_drop: bool,
    #[cfg(feature = "system-tray")]
    tray_handle: Option<tray::SystemTrayHandle<P>>,
  }
}

impl<P: Params> Drop for App<P> {
  fn drop(&mut self) {
    #[cfg(shell_execute)]
    {
      if self.cleanup_on_drop {
        crate::api::process::kill_children();
      }
    }
  }
}

impl<P: Params> Manager<P> for App<P> {}
impl<P: Params> ManagerBase<P> for App<P> {
  fn manager(&self) -> &WindowManager<P> {
    &self.manager
  }

  fn runtime(&self) -> RuntimeOrDispatch<'_, P> {
    RuntimeOrDispatch::Runtime(self.runtime.as_ref().unwrap())
  }
}

macro_rules! shared_app_impl {
  ($app: ty) => {
    impl<P: Params> $app {
      /// Creates a new webview window.
      pub fn create_window<F>(&self, label: P::Label, url: WindowUrl, setup: F) -> crate::Result<()>
      where
        F: FnOnce(
          <<P::Runtime as Runtime>::Dispatcher as Dispatch>::WindowBuilder,
          WebviewAttributes,
        ) -> (
          <<P::Runtime as Runtime>::Dispatcher as Dispatch>::WindowBuilder,
          WebviewAttributes,
        ),
      {
        let (window_builder, webview_attributes) = setup(
          <<P::Runtime as Runtime>::Dispatcher as Dispatch>::WindowBuilder::new(),
          WebviewAttributes::new(url),
        );
        self.create_new_window(PendingWindow::new(
          window_builder,
          webview_attributes,
          label,
        ))?;
        Ok(())
      }

      #[cfg(feature = "system-tray")]
      #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
      /// Gets a handle handle to the system tray.
      pub fn tray_handle(&self) -> tray::SystemTrayHandle<P> {
        self
          .tray_handle
          .clone()
          .expect("tray not configured; use the `Builder#system_tray` API first.")
      }
    }
  };
}

shared_app_impl!(App<P>);
shared_app_impl!(AppHandle<P>);

impl<P: Params> App<P> {
  /// Gets a handle to the application instance.
  pub fn handle(&self) -> AppHandle<P> {
    AppHandle {
      runtime_handle: self.runtime.as_ref().unwrap().handle(),
      manager: self.manager.clone(),
      #[cfg(feature = "system-tray")]
      tray_handle: self.tray_handle.clone(),
    }
  }

  /// Runs a iteration of the runtime event loop and immediately return.
  ///
  /// # Example
  /// ```rust,ignore
  /// fn main() {
  ///   let mut app = tauri::Builder::default()
  ///     .build(tauri::generate_context!())
  ///     .expect("error while building tauri application");
  ///   loop {
  ///     let iteration = app.run_iteration();
  ///     if iteration.webview_count == 0 {
  ///       break;
  ///     }
  ///   }
  /// }
  #[cfg(any(target_os = "windows", target_os = "macos"))]
  pub fn run_iteration(&mut self) -> crate::runtime::RunIteration {
    self.runtime.as_mut().unwrap().run_iteration()
  }
}

#[cfg(feature = "updater")]
impl<P: Params> App<P> {
  /// Runs the updater hook with built-in dialog.
  fn run_updater_dialog(&self, window: Window<P>) {
    let updater_config = self.manager.config().tauri.updater.clone();
    let package_info = self.manager.package_info().clone();
    crate::async_runtime::spawn(async move {
      updater::check_update_with_dialog(updater_config, package_info, window).await
    });
  }

  /// Listen updater events when dialog are disabled.
  fn listen_updater_events(&self, window: Window<P>) {
    let updater_config = self.manager.config().tauri.updater.clone();
    updater::listener(updater_config, self.manager.package_info().clone(), &window);
  }

  fn run_updater(&self, main_window: Option<Window<P>>) {
    if let Some(main_window) = main_window {
      let event_window = main_window.clone();
      let updater_config = self.manager.config().tauri.updater.clone();
      // check if updater is active or not
      if updater_config.dialog && updater_config.active {
        // if updater dialog is enabled spawn a new task
        self.run_updater_dialog(main_window.clone());
        let config = self.manager.config().tauri.updater.clone();
        let package_info = self.manager.package_info().clone();
        // When dialog is enabled, if user want to recheck
        // if an update is available after first start
        // invoke the Event `tauri://update` from JS or rust side.
        main_window.listen(
          updater::EVENT_CHECK_UPDATE
            .parse::<P::Event>()
            .unwrap_or_else(|_| panic!("bad label")),
          move |_msg| {
            let window = event_window.clone();
            let package_info = package_info.clone();
            let config = config.clone();
            // re-spawn task inside tokyo to launch the download
            // we don't need to emit anything as everything is handled
            // by the process (user is asked to restart at the end)
            // and it's handled by the updater
            crate::async_runtime::spawn(async move {
              updater::check_update_with_dialog(config, package_info, window).await
            });
          },
        );
      } else if updater_config.active {
        // we only listen for `tauri://update`
        // once we receive the call, we check if an update is available or not
        // if there is a new update we emit `tauri://update-available` with details
        // this is the user responsabilities to display dialog and ask if user want to install
        // to install the update you need to invoke the Event `tauri://update-install`
        self.listen_updater_events(main_window);
      }
    }
  }
}

/// Builds a Tauri application.
#[allow(clippy::type_complexity)]
pub struct Builder<E, L, MID, TID, A, R>
where
  E: Tag,
  L: Tag,
  MID: MenuId,
  TID: MenuId,
  A: Assets,
  R: Runtime,
{
  /// The JS message handler.
  invoke_handler: Box<InvokeHandler<Args<E, L, MID, TID, A, R>>>,

  /// The setup hook.
  setup: SetupHook<Args<E, L, MID, TID, A, R>>,

  /// Page load hook.
  on_page_load: Box<OnPageLoad<Args<E, L, MID, TID, A, R>>>,

  /// windows to create when starting up.
  pending_windows: Vec<PendingWindow<Args<E, L, MID, TID, A, R>>>,

  /// All passed plugins
  plugins: PluginStore<Args<E, L, MID, TID, A, R>>,

  /// The webview protocols available to all windows.
  uri_scheme_protocols: HashMap<String, Arc<CustomProtocol>>,

  /// App state.
  state: StateManager,

  /// The menu set to all windows.
  #[cfg(feature = "menu")]
  menu: Option<Menu<MID>>,

  /// Menu event handlers that listens to all windows.
  #[cfg(feature = "menu")]
  menu_event_listeners: Vec<GlobalMenuEventListener<Args<E, L, MID, TID, A, R>>>,

  /// Window event handlers that listens to all windows.
  window_event_listeners: Vec<GlobalWindowEventListener<Args<E, L, MID, TID, A, R>>>,

  /// The app system tray.
  #[cfg(feature = "system-tray")]
  system_tray: Option<tray::SystemTray<TID>>,

  /// System tray event handlers.
  #[cfg(feature = "system-tray")]
  system_tray_event_listeners: Vec<SystemTrayEventListener<Args<E, L, MID, TID, A, R>>>,

  #[cfg(shell_execute)]
  cleanup_on_drop: bool,
}

impl<E, L, MID, TID, A, R> Builder<E, L, MID, TID, A, R>
where
  E: Tag,
  L: Tag,
  MID: MenuId,
  TID: MenuId,
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
      uri_scheme_protocols: Default::default(),
      state: StateManager::new(),
      #[cfg(feature = "menu")]
      menu: None,
      #[cfg(feature = "menu")]
      menu_event_listeners: Vec::new(),
      window_event_listeners: Vec::new(),
      #[cfg(feature = "system-tray")]
      system_tray: None,
      #[cfg(feature = "system-tray")]
      system_tray_event_listeners: Vec::new(),
      #[cfg(shell_execute)]
      cleanup_on_drop: true,
    }
  }

  /// Defines the JS message handler callback.
  pub fn invoke_handler<F>(mut self, invoke_handler: F) -> Self
  where
    F: Fn(Invoke<Args<E, L, MID, TID, A, R>>) + Send + Sync + 'static,
  {
    self.invoke_handler = Box::new(invoke_handler);
    self
  }

  /// Defines the setup hook.
  pub fn setup<F>(mut self, setup: F) -> Self
  where
    F: Fn(&mut App<Args<E, L, MID, TID, A, R>>) -> Result<(), Box<dyn std::error::Error + Send>>
      + Send
      + 'static,
  {
    self.setup = Box::new(setup);
    self
  }

  /// Defines the page load hook.
  pub fn on_page_load<F>(mut self, on_page_load: F) -> Self
  where
    F: Fn(Window<Args<E, L, MID, TID, A, R>>, PageLoadPayload) + Send + Sync + 'static,
  {
    self.on_page_load = Box::new(on_page_load);
    self
  }

  /// Adds a plugin to the runtime.
  pub fn plugin<P: Plugin<Args<E, L, MID, TID, A, R>> + 'static>(mut self, plugin: P) -> Self {
    self.plugins.register(plugin);
    self
  }

  /// Add `state` to the state managed by the application.
  ///
  /// This method can be called any number of times as long as each call
  /// refers to a different `T`.
  ///
  /// Managed state can be retrieved by any request handler via the
  /// [`State`](crate::State) request guard. In particular, if a value of type `T`
  /// is managed by Tauri, adding `State<T>` to the list of arguments in a
  /// request handler instructs Tauri to retrieve the managed value.
  ///
  /// # Panics
  ///
  /// Panics if state of type `T` is already being managed.
  ///
  /// # Mutability
  ///
  /// Since the managed state is global and must be [`Send`] + [`Sync`], mutations can only happen through interior mutability:
  ///
  /// ```rust,ignore
  /// use std::{collections::HashMap, sync::Mutex};
  /// use tauri::State;
  /// // here we use Mutex to achieve interior mutability
  /// struct Storage(Mutex<HashMap<u64, String>>);
  /// struct Connection;
  /// struct DbConnection(Mutex<Option<Connection>>);
  ///
  /// #[tauri::command]
  /// fn connect(connection: State<DbConnection>) {
  ///   // initialize the connection, mutating the state with interior mutability
  ///   *connection.0.lock().unwrap() = Some(Connection {});
  /// }
  ///
  /// #[tauri::command]
  /// fn storage_insert(key: u64, value: String, storage: State<Storage>) {
  ///   // mutate the storage behind the Mutex
  ///   storage.0.lock().unwrap().insert(key, value);
  /// }
  ///
  /// fn main() {
  ///   Builder::default()
  ///     .manage(Storage(Default::default()))
  ///     .manage(DbConnection(Default::default()))
  ///     .invoke_handler(tauri::generate_handler![connect, storage_insert])
  ///     .run(tauri::generate_context!())
  ///     .expect("error while running tauri application");
  /// }
  /// ```
  ///
  /// # Example
  ///
  /// ```rust,ignore
  /// use tauri::State;
  ///
  /// struct MyInt(isize);
  /// struct MyString(String);
  ///
  /// #[tauri::command]
  /// fn int_command(state: State<MyInt>) -> String {
  ///     format!("The stateful int is: {}", state.0)
  /// }
  ///
  /// #[tauri::command]
  /// fn string_command<'r>(state: State<'r, MyString>) {
  ///     println!("state: {}", state.inner().0);
  /// }
  ///
  /// fn main() {
  ///     tauri::Builder::default()
  ///         .manage(MyInt(10))
  ///         .manage(MyString("Hello, managed state!".to_string()))
  ///         .invoke_handler(tauri::generate_handler![int_command, string_command])
  ///         .run(tauri::generate_context!())
  ///         .expect("error while running tauri application");
  /// }
  /// ```
  pub fn manage<T>(self, state: T) -> Self
  where
    T: Send + Sync + 'static,
  {
    let type_name = std::any::type_name::<T>();
    if !self.state.set(state) {
      panic!("state for type '{}' is already being managed", type_name);
    }

    self
  }

  /// Creates a new webview window.
  pub fn create_window<F>(mut self, label: L, url: WindowUrl, setup: F) -> Self
  where
    F: FnOnce(
      <R::Dispatcher as Dispatch>::WindowBuilder,
      WebviewAttributes,
    ) -> (
      <R::Dispatcher as Dispatch>::WindowBuilder,
      WebviewAttributes,
    ),
  {
    let (window_builder, webview_attributes) = setup(
      <R::Dispatcher as Dispatch>::WindowBuilder::new(),
      WebviewAttributes::new(url),
    );
    self.pending_windows.push(PendingWindow::new(
      window_builder,
      webview_attributes,
      label,
    ));
    self
  }

  /// Adds the icon configured on `tauri.conf.json` to the system tray with the specified menu items.
  #[cfg(feature = "system-tray")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
  pub fn system_tray(mut self, system_tray: tray::SystemTray<TID>) -> Self {
    self.system_tray.replace(system_tray);
    self
  }

  /// Sets the menu to use on all windows.
  #[cfg(feature = "menu")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
  pub fn menu(mut self, menu: Menu<MID>) -> Self {
    self.menu.replace(menu);
    self
  }

  /// Registers a menu event handler for all windows.
  #[cfg(feature = "menu")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
  pub fn on_menu_event<
    F: Fn(WindowMenuEvent<Args<E, L, MID, TID, A, R>>) + Send + Sync + 'static,
  >(
    mut self,
    handler: F,
  ) -> Self {
    self.menu_event_listeners.push(Box::new(handler));
    self
  }

  /// Registers a window event handler for all windows.
  pub fn on_window_event<
    F: Fn(GlobalWindowEvent<Args<E, L, MID, TID, A, R>>) + Send + Sync + 'static,
  >(
    mut self,
    handler: F,
  ) -> Self {
    self.window_event_listeners.push(Box::new(handler));
    self
  }

  /// Registers a system tray event handler.
  #[cfg(feature = "system-tray")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
  pub fn on_system_tray_event<
    F: Fn(&AppHandle<Args<E, L, MID, TID, A, R>>, tray::SystemTrayEvent<TID>) + Send + Sync + 'static,
  >(
    mut self,
    handler: F,
  ) -> Self {
    self.system_tray_event_listeners.push(Box::new(handler));
    self
  }

  /// Registers a URI scheme protocol available to all webviews.
  /// Leverages [setURLSchemeHandler](https://developer.apple.com/documentation/webkit/wkwebviewconfiguration/2875766-seturlschemehandler) on macOS,
  /// [AddWebResourceRequestedFilter](https://docs.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2.addwebresourcerequestedfilter?view=webview2-dotnet-1.0.774.44) on Windows
  /// and [webkit-web-context-register-uri-scheme](https://webkitgtk.org/reference/webkit2gtk/stable/WebKitWebContext.html#webkit-web-context-register-uri-scheme) on Linux.
  ///
  /// # Arguments
  ///
  /// * `uri_scheme` The URI scheme to register, such as `example`.
  /// * `protocol` the protocol associated with the given URI scheme. It's a function that takes an URL such as `example://localhost/asset.css`.
  pub fn register_global_uri_scheme_protocol<
    N: Into<String>,
    H: Fn(&str) -> Result<Vec<u8>, Box<dyn std::error::Error>> + Send + Sync + 'static,
  >(
    mut self,
    uri_scheme: N,
    protocol: H,
  ) -> Self {
    self.uri_scheme_protocols.insert(
      uri_scheme.into(),
      Arc::new(CustomProtocol {
        protocol: Box::new(protocol),
      }),
    );
    self
  }

  /// Skips Tauri cleanup on [`App`] drop. Useful if your application has multiple [`App`] instances.
  ///
  /// The cleanup calls [`crate::api::process::kill_children`] so you may want to call that function before exiting the application.
  #[cfg(shell_execute)]
  pub fn skip_cleanup_on_drop(mut self) -> Self {
    self.cleanup_on_drop = false;
    self
  }

  /// Builds the application.
  #[allow(clippy::type_complexity)]
  pub fn build(mut self, context: Context<A>) -> crate::Result<App<Args<E, L, MID, TID, A, R>>> {
    #[cfg(feature = "system-tray")]
    let system_tray_icon = {
      let icon = context.system_tray_icon.clone();

      // check the icon format if the system tray is configured
      if self.system_tray.is_some() {
        use std::io::{Error, ErrorKind};
        #[cfg(target_os = "linux")]
        if let Some(Icon::Raw(_)) = icon {
          return Err(crate::Error::InvalidIcon(Box::new(Error::new(
            ErrorKind::InvalidInput,
            "system tray icons on linux must be a file path",
          ))));
        }

        #[cfg(not(target_os = "linux"))]
        if let Some(Icon::File(_)) = icon {
          return Err(crate::Error::InvalidIcon(Box::new(Error::new(
            ErrorKind::InvalidInput,
            "system tray icons on non-linux platforms must be the raw bytes",
          ))));
        }
      }

      icon
    };

    let manager = WindowManager::with_handlers(
      context,
      self.plugins,
      self.invoke_handler,
      self.on_page_load,
      self.uri_scheme_protocols,
      self.state,
      self.window_event_listeners,
      #[cfg(feature = "menu")]
      (self.menu, self.menu_event_listeners),
    );

    // set up all the windows defined in the config
    for config in manager.config().tauri.windows.clone() {
      let url = config.url.clone();
      let label = config
        .label
        .parse()
        .unwrap_or_else(|_| panic!("bad label found in config: {}", config.label));

      self.pending_windows.push(PendingWindow::with_config(
        config,
        WebviewAttributes::new(url),
        label,
      ));
    }

    let mut app = App {
      runtime: Some(R::new()?),
      manager,
      #[cfg(shell_execute)]
      cleanup_on_drop: self.cleanup_on_drop,
      #[cfg(feature = "system-tray")]
      tray_handle: None,
    };

    app.manager.initialize_plugins(&app)?;

    let pending_labels = self
      .pending_windows
      .iter()
      .map(|p| p.label.clone())
      .collect::<Vec<_>>();

    #[cfg(feature = "updater")]
    let mut main_window = None;

    for pending in self.pending_windows {
      let pending = app.manager.prepare_window(pending, &pending_labels)?;
      let detached = app.runtime.as_ref().unwrap().create_window(pending)?;
      let _window = app.manager.attach_window(detached);
      #[cfg(feature = "updater")]
      if main_window.is_none() {
        main_window = Some(_window);
      }
    }

    #[cfg(feature = "updater")]
    app.run_updater(main_window);

    (self.setup)(&mut app).map_err(|e| crate::Error::Setup(e))?;

    #[cfg(feature = "system-tray")]
    if let Some(system_tray) = self.system_tray {
      let mut ids = HashMap::new();
      if let Some(menu) = system_tray.menu() {
        tray::get_menu_ids(&mut ids, menu);
      }
      let tray_handler = app
        .runtime
        .as_ref()
        .unwrap()
        .system_tray(
          system_tray_icon.expect("tray icon not found; please configure it on tauri.conf.json"),
          system_tray,
        )
        .expect("failed to run tray");
      app.tray_handle.replace(tray::SystemTrayHandle {
        ids: Arc::new(ids.clone()),
        inner: Arc::new(std::sync::Mutex::new(tray_handler)),
      });
      for listener in self.system_tray_event_listeners {
        let app_handle = app.handle();
        let ids = ids.clone();
        let listener = Arc::new(std::sync::Mutex::new(listener));
        app
          .runtime
          .as_mut()
          .unwrap()
          .on_system_tray_event(move |event| {
            let app_handle = app_handle.clone();
            let event = match event {
              RuntimeSystemTrayEvent::MenuItemClick(id) => tray::SystemTrayEvent::MenuItemClick {
                id: ids.get(&id).unwrap().clone(),
              },
              RuntimeSystemTrayEvent::LeftClick { position, size } => {
                tray::SystemTrayEvent::LeftClick {
                  position: *position,
                  size: *size,
                }
              }
              RuntimeSystemTrayEvent::RightClick { position, size } => {
                tray::SystemTrayEvent::RightClick {
                  position: *position,
                  size: *size,
                }
              }
              RuntimeSystemTrayEvent::DoubleClick { position, size } => {
                tray::SystemTrayEvent::DoubleClick {
                  position: *position,
                  size: *size,
                }
              }
            };
            let listener = listener.clone();
            crate::async_runtime::spawn(async move {
              listener.lock().unwrap()(&app_handle, event);
            });
          });
      }
    }

    Ok(app)
  }

  /// Runs the configured Tauri application.
  pub fn run(self, context: Context<A>) -> crate::Result<()> {
    let mut app = self.build(context)?;
    app.runtime.take().unwrap().run();
    Ok(())
  }
}

/// Make `Wry` the default `Runtime` for `Builder`
#[cfg(feature = "wry")]
impl<A: Assets> Default for Builder<String, String, String, String, A, crate::Wry> {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(not(feature = "wry"))]
impl<A: Assets, R: Runtime> Default for Builder<String, String, String, String, A, R> {
  fn default() -> Self {
    Self::new()
  }
}
