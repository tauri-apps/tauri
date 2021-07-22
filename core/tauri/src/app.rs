// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "system-tray")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
pub(crate) mod tray;

use crate::{
  api::assets::Assets,
  api::config::{Config, WindowUrl},
  command::{CommandArg, CommandItem},
  hooks::{InvokeHandler, OnPageLoad, PageLoadPayload, SetupHook},
  manager::WindowManager,
  plugin::{Plugin, PluginStore},
  runtime::{
    webview::{CustomProtocol, WebviewAttributes, WindowBuilder},
    window::{PendingWindow, WindowEvent},
    Dispatch, RunEvent, Runtime,
  },
  sealed::{ManagerBase, RuntimeOrDispatch},
  Context, Invoke, InvokeError, Manager, StateManager, Window,
};

use tauri_macros::default_runtime;
use tauri_utils::PackageInfo;

use std::{
  collections::HashMap,
  path::PathBuf,
  sync::{mpsc::Sender, Arc},
};

#[cfg(feature = "menu")]
use crate::runtime::menu::{Menu, MenuId, MenuIdRef};

#[cfg(all(windows, feature = "system-tray"))]
use crate::runtime::RuntimeHandle;
#[cfg(feature = "system-tray")]
use crate::runtime::{Icon, SystemTrayEvent as RuntimeSystemTrayEvent};

#[cfg(feature = "updater")]
use crate::updater;

#[cfg(feature = "menu")]
pub(crate) type GlobalMenuEventListener<R> = Box<dyn Fn(WindowMenuEvent<R>) + Send + Sync>;
pub(crate) type GlobalWindowEventListener<R> = Box<dyn Fn(GlobalWindowEvent<R>) + Send + Sync>;
#[cfg(feature = "system-tray")]
type SystemTrayEventListener<R> = Box<dyn Fn(&AppHandle<R>, tray::SystemTrayEvent) + Send + Sync>;

/// Api exposed on the `CloseRequested` event.
pub struct CloseRequestApi(Sender<bool>);

impl CloseRequestApi {
  /// Prevents the window from being closed.
  pub fn prevent_close(&self) {
    self.0.send(true).unwrap();
  }
}

/// An application event, triggered from the event loop.
#[non_exhaustive]
pub enum Event {
  /// Event loop is exiting.
  Exit,
  /// Window close was requested by the user.
  #[non_exhaustive]
  CloseRequested {
    /// The window label.
    label: String,
    /// Event API.
    api: CloseRequestApi,
  },
  /// Window closed.
  WindowClosed(String),
}

/// A menu event that was triggered on a window.
#[cfg(feature = "menu")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
#[default_runtime(crate::Wry, wry)]
pub struct WindowMenuEvent<R: Runtime> {
  pub(crate) menu_item_id: MenuId,
  pub(crate) window: Window<R>,
}

#[cfg(feature = "menu")]
impl<R: Runtime> WindowMenuEvent<R> {
  /// The menu item id.
  pub fn menu_item_id(&self) -> MenuIdRef<'_> {
    &self.menu_item_id
  }

  /// The window that the menu belongs to.
  pub fn window(&self) -> &Window<R> {
    &self.window
  }
}

/// A window event that was triggered on the specified window.
#[default_runtime(crate::Wry, wry)]
pub struct GlobalWindowEvent<R: Runtime> {
  pub(crate) event: WindowEvent,
  pub(crate) window: Window<R>,
}

impl<R: Runtime> GlobalWindowEvent<R> {
  /// The event payload.
  pub fn event(&self) -> &WindowEvent {
    &self.event
  }

  /// The window that the menu belongs to.
  pub fn window(&self) -> &Window<R> {
    &self.window
  }
}

/// The path resolver is a helper for the application-specific [`crate::api::path`] APIs.
#[derive(Debug, Clone)]
pub struct PathResolver {
  config: Arc<Config>,
  package_info: PackageInfo,
}

impl PathResolver {
  /// Returns the path to the resource directory of this app.
  pub fn resource_dir(&self) -> Option<PathBuf> {
    crate::api::path::resource_dir(&self.package_info)
  }

  /// Returns the path to the suggested directory for your app config files.
  pub fn app_dir(&self) -> Option<PathBuf> {
    crate::api::path::app_dir(&self.config)
  }
}

/// A handle to the currently running application.
///
/// This type implements [`Manager`] which allows for manipulation of global application items.
#[default_runtime(crate::Wry, wry)]
pub struct AppHandle<R: Runtime> {
  runtime_handle: R::Handle,
  manager: WindowManager<R>,
  global_shortcut_manager: R::GlobalShortcutManager,
  clipboard_manager: R::ClipboardManager,
  #[cfg(feature = "system-tray")]
  tray_handle: Option<tray::SystemTrayHandle<R>>,
}

impl<R: Runtime> Clone for AppHandle<R> {
  fn clone(&self) -> Self {
    Self {
      runtime_handle: self.runtime_handle.clone(),
      manager: self.manager.clone(),
      global_shortcut_manager: self.global_shortcut_manager.clone(),
      clipboard_manager: self.clipboard_manager.clone(),
      #[cfg(feature = "system-tray")]
      tray_handle: self.tray_handle.clone(),
    }
  }
}

impl<'de, R: Runtime> CommandArg<'de, R> for AppHandle<R> {
  /// Grabs the [`Window`] from the [`CommandItem`] and returns the associated [`AppHandle`]. This will never fail.
  fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
    Ok(command.message.window().app_handle)
  }
}

impl<R: Runtime> AppHandle<R> {
  /// Removes the system tray.
  #[cfg(all(windows, feature = "system-tray"))]
  #[cfg_attr(doc_cfg, doc(cfg(all(windows, feature = "system-tray"))))]
  fn remove_system_tray(&self) -> crate::Result<()> {
    self.runtime_handle.remove_system_tray().map_err(Into::into)
  }
}

impl<R: Runtime> Manager<R> for AppHandle<R> {}
impl<R: Runtime> ManagerBase<R> for AppHandle<R> {
  fn manager(&self) -> &WindowManager<R> {
    &self.manager
  }

  fn runtime(&self) -> RuntimeOrDispatch<'_, R> {
    RuntimeOrDispatch::RuntimeHandle(self.runtime_handle.clone())
  }

  fn app_handle(&self) -> AppHandle<R> {
    self.clone()
  }
}

/// The instance of the currently running application.
///
/// This type implements [`Manager`] which allows for manipulation of global application items.
#[default_runtime(crate::Wry, wry)]
pub struct App<R: Runtime> {
  runtime: Option<R>,
  manager: WindowManager<R>,
  global_shortcut_manager: R::GlobalShortcutManager,
  clipboard_manager: R::ClipboardManager,
  #[cfg(feature = "system-tray")]
  tray_handle: Option<tray::SystemTrayHandle<R>>,
  handle: AppHandle<R>,
}

impl<R: Runtime> Manager<R> for App<R> {}
impl<R: Runtime> ManagerBase<R> for App<R> {
  fn manager(&self) -> &WindowManager<R> {
    &self.manager
  }

  fn runtime(&self) -> RuntimeOrDispatch<'_, R> {
    RuntimeOrDispatch::Runtime(self.runtime.as_ref().unwrap())
  }

  fn app_handle(&self) -> AppHandle<R> {
    self.handle()
  }
}

macro_rules! shared_app_impl {
  ($app: ty) => {
    impl<R: Runtime> $app {
      /// Creates a new webview window.
      pub fn create_window<F>(&self, label: String, url: WindowUrl, setup: F) -> crate::Result<()>
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
      pub fn tray_handle(&self) -> tray::SystemTrayHandle<R> {
        self
          .tray_handle
          .clone()
          .expect("tray not configured; use the `Builder#system_tray` API first.")
      }

      /// The path resolver for the application.
      pub fn path_resolver(&self) -> PathResolver {
        PathResolver {
          config: self.manager.config(),
          package_info: self.manager.package_info().clone(),
        }
      }

      /// Gets a copy of the global shortcut manager instance.
      pub fn global_shortcut_manager(&self) -> R::GlobalShortcutManager {
        self.global_shortcut_manager.clone()
      }

      /// Gets a copy of the clipboard manager instance.
      pub fn clipboard_manager(&self) -> R::ClipboardManager {
        self.clipboard_manager.clone()
      }

      /// Gets the app's configuration, defined on the `tauri.conf.json` file.
      pub fn config(&self) -> Arc<Config> {
        self.manager.config()
      }

      /// Gets the app's package information.
      pub fn package_info(&self) -> &PackageInfo {
        self.manager.package_info()
      }
    }
  };
}

shared_app_impl!(App<R>);
shared_app_impl!(AppHandle<R>);

impl<R: Runtime> App<R> {
  /// Gets a handle to the application instance.
  pub fn handle(&self) -> AppHandle<R> {
    self.handle.clone()
  }

  /// Runs the application.
  pub fn run<F: Fn(&AppHandle<R>, Event) + 'static>(mut self, callback: F) {
    let app_handle = self.handle();
    let manager = self.manager.clone();
    self.runtime.take().unwrap().run(move |event| match event {
      RunEvent::Exit => {
        #[cfg(shell_execute)]
        {
          crate::api::process::kill_children();
        }
        #[cfg(all(windows, feature = "system-tray"))]
        {
          let _ = app_handle.remove_system_tray();
        }
        callback(&app_handle, Event::Exit);
      }
      _ => {
        on_event_loop_event(&event, &manager);
        callback(
          &app_handle,
          match event {
            RunEvent::Exit => Event::Exit,
            RunEvent::CloseRequested { label, signal_tx } => Event::CloseRequested {
              label: label.parse().unwrap_or_else(|_| unreachable!()),
              api: CloseRequestApi(signal_tx),
            },
            RunEvent::WindowClose(label) => {
              Event::WindowClosed(label.parse().unwrap_or_else(|_| unreachable!()))
            }
            _ => unimplemented!(),
          },
        );
      }
    });
  }

  /// Runs a iteration of the runtime event loop and immediately return.
  ///
  /// Note that when using this API, app cleanup is not automatically done.
  /// The cleanup calls [`crate::api::process::kill_children`] so you may want to call that function before exiting the application.
  /// Additionally, the cleanup calls [AppHandle#remove_system_tray](`AppHandle#method.remove_system_tray`) (Windows only).
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
    let manager = self.manager.clone();
    self
      .runtime
      .as_mut()
      .unwrap()
      .run_iteration(move |event| on_event_loop_event(&event, &manager))
  }
}

#[cfg(feature = "updater")]
impl<R: Runtime> App<R> {
  /// Runs the updater hook with built-in dialog.
  fn run_updater_dialog(&self, window: Window<R>) {
    let updater_config = self.manager.config().tauri.updater.clone();
    let package_info = self.manager.package_info().clone();

    #[cfg(not(target_os = "linux"))]
    crate::async_runtime::spawn(async move {
      updater::check_update_with_dialog(updater_config, package_info, window).await
    });

    #[cfg(target_os = "linux")]
    {
      let context = glib::MainContext::default();
      context.spawn_with_priority(glib::PRIORITY_HIGH, async move {
        updater::check_update_with_dialog(updater_config, package_info, window).await
      });
    }
  }

  /// Listen updater events when dialog are disabled.
  fn listen_updater_events(&self, window: Window<R>) {
    let updater_config = self.manager.config().tauri.updater.clone();
    updater::listener(updater_config, self.manager.package_info().clone(), &window);
  }

  fn run_updater(&self, main_window: Option<Window<R>>) {
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
        main_window.listen(updater::EVENT_CHECK_UPDATE, move |_msg| {
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
        });
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
pub struct Builder<R: Runtime> {
  /// The JS message handler.
  invoke_handler: Box<InvokeHandler<R>>,

  /// The setup hook.
  setup: SetupHook<R>,

  /// Page load hook.
  on_page_load: Box<OnPageLoad<R>>,

  /// windows to create when starting up.
  pending_windows: Vec<PendingWindow<R>>,

  /// All passed plugins
  plugins: PluginStore<R>,

  /// The webview protocols available to all windows.
  uri_scheme_protocols: HashMap<String, Arc<CustomProtocol>>,

  /// App state.
  state: StateManager,

  /// The menu set to all windows.
  #[cfg(feature = "menu")]
  menu: Option<Menu>,

  /// Menu event handlers that listens to all windows.
  #[cfg(feature = "menu")]
  menu_event_listeners: Vec<GlobalMenuEventListener<R>>,

  /// Window event handlers that listens to all windows.
  window_event_listeners: Vec<GlobalWindowEventListener<R>>,

  /// The app system tray.
  #[cfg(feature = "system-tray")]
  system_tray: Option<tray::SystemTray>,

  /// System tray event handlers.
  #[cfg(feature = "system-tray")]
  system_tray_event_listeners: Vec<SystemTrayEventListener<R>>,
}

impl<R: Runtime> Builder<R> {
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
    }
  }

  /// Defines the JS message handler callback.
  pub fn invoke_handler<F>(mut self, invoke_handler: F) -> Self
  where
    F: Fn(Invoke<R>) + Send + Sync + 'static,
  {
    self.invoke_handler = Box::new(invoke_handler);
    self
  }

  /// Defines the setup hook.
  pub fn setup<F>(mut self, setup: F) -> Self
  where
    F: Fn(&mut App<R>) -> Result<(), Box<dyn std::error::Error + Send>> + Send + 'static,
  {
    self.setup = Box::new(setup);
    self
  }

  /// Defines the page load hook.
  pub fn on_page_load<F>(mut self, on_page_load: F) -> Self
  where
    F: Fn(Window<R>, PageLoadPayload) + Send + Sync + 'static,
  {
    self.on_page_load = Box::new(on_page_load);
    self
  }

  /// Adds a plugin to the runtime.
  pub fn plugin<P: Plugin<R> + 'static>(mut self, plugin: P) -> Self {
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
  pub fn create_window<F>(mut self, label: impl Into<String>, url: WindowUrl, setup: F) -> Self
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
  pub fn system_tray(mut self, system_tray: tray::SystemTray) -> Self {
    self.system_tray.replace(system_tray);
    self
  }

  /// Sets the menu to use on all windows.
  #[cfg(feature = "menu")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
  pub fn menu(mut self, menu: Menu) -> Self {
    self.menu.replace(menu);
    self
  }

  /// Registers a menu event handler for all windows.
  #[cfg(feature = "menu")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
  pub fn on_menu_event<F: Fn(WindowMenuEvent<R>) + Send + Sync + 'static>(
    mut self,
    handler: F,
  ) -> Self {
    self.menu_event_listeners.push(Box::new(handler));
    self
  }

  /// Registers a window event handler for all windows.
  pub fn on_window_event<F: Fn(GlobalWindowEvent<R>) + Send + Sync + 'static>(
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
    F: Fn(&AppHandle<R>, tray::SystemTrayEvent) + Send + Sync + 'static,
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

  /// Builds the application.
  #[allow(clippy::type_complexity)]
  pub fn build<A: Assets>(mut self, context: Context<A>) -> crate::Result<App<R>> {
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
      let label = config.label.clone();
      let file_drop_enabled = config.file_drop_enabled;

      let mut webview_attributes = WebviewAttributes::new(url);
      if !file_drop_enabled {
        webview_attributes = webview_attributes.disable_file_drop_handler();
      }

      self.pending_windows.push(PendingWindow::with_config(
        config,
        webview_attributes,
        label,
      ));
    }

    let runtime = R::new()?;
    let runtime_handle = runtime.handle();
    let global_shortcut_manager = runtime.global_shortcut_manager();
    let clipboard_manager = runtime.clipboard_manager();

    let mut app = App {
      runtime: Some(runtime),
      manager: manager.clone(),
      global_shortcut_manager: global_shortcut_manager.clone(),
      clipboard_manager: clipboard_manager.clone(),
      #[cfg(feature = "system-tray")]
      tray_handle: None,
      handle: AppHandle {
        runtime_handle,
        manager,
        global_shortcut_manager,
        clipboard_manager,
        #[cfg(feature = "system-tray")]
        tray_handle: None,
      },
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
      let pending = app
        .manager
        .prepare_window(app.handle.clone(), pending, &pending_labels)?;
      let detached = app.runtime.as_ref().unwrap().create_window(pending)?;
      let _window = app.manager.attach_window(app.handle(), detached);
      #[cfg(feature = "updater")]
      if main_window.is_none() {
        main_window = Some(_window);
      }
    }

    (self.setup)(&mut app).map_err(|e| crate::Error::Setup(e))?;

    #[cfg(feature = "system-tray")]
    if let Some(system_tray) = self.system_tray {
      let mut ids = HashMap::new();
      if let Some(menu) = system_tray.menu() {
        tray::get_menu_ids(&mut ids, menu);
      }
      let mut tray = tray::SystemTray::new();
      if let Some(menu) = system_tray.menu {
        tray = tray.with_menu(menu);
      }
      let tray_handler = app
        .runtime
        .as_ref()
        .unwrap()
        .system_tray(
          tray.with_icon(
            system_tray
              .icon
              .or(system_tray_icon)
              .expect("tray icon not found; please configure it on tauri.conf.json"),
          ),
        )
        .expect("failed to run tray");
      let tray_handle = tray::SystemTrayHandle {
        ids: Arc::new(ids.clone()),
        inner: tray_handler,
      };
      app.tray_handle.replace(tray_handle.clone());
      app.handle.tray_handle.replace(tray_handle);
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
                id: ids.get(id).unwrap().clone(),
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

    #[cfg(feature = "updater")]
    app.run_updater(main_window);

    Ok(app)
  }

  /// Runs the configured Tauri application.
  pub fn run<A: Assets>(self, context: Context<A>) -> crate::Result<()> {
    self.build(context)?.run(|_, _| {});
    Ok(())
  }
}

fn on_event_loop_event<R: Runtime>(event: &RunEvent, manager: &WindowManager<R>) {
  if let RunEvent::WindowClose(label) = event {
    manager.on_window_close(label);
  }
}

/// Make `Wry` the default `Runtime` for `Builder`
#[cfg(feature = "wry")]
impl Default for Builder<crate::Wry> {
  fn default() -> Self {
    Self::new()
  }
}
