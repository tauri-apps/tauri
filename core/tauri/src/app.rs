// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(all(desktop, feature = "system-tray"))]
pub(crate) mod tray;

use crate::{
  api::ipc::CallbackFn,
  command::{CommandArg, CommandItem},
  hooks::{
    window_invoke_responder, InvokeHandler, InvokeResponder, OnPageLoad, PageLoadPayload, SetupHook,
  },
  manager::{Asset, CustomProtocol, WindowManager},
  plugin::{Plugin, PluginStore},
  runtime::{
    http::{Request as HttpRequest, Response as HttpResponse},
    webview::WebviewAttributes,
    window::{PendingWindow, WindowEvent as RuntimeWindowEvent},
    ExitRequestedEventAction, RunEvent as RuntimeRunEvent,
  },
  scope::IpcScope,
  sealed::{ManagerBase, RuntimeOrDispatch},
  utils::config::Config,
  utils::{assets::Assets, Env},
  Context, DeviceEventFilter, EventLoopMessage, Icon, Invoke, InvokeError, InvokeResponse, Manager,
  Runtime, Scopes, StateManager, Theme, Window,
};

#[cfg(feature = "protocol-asset")]
use crate::scope::FsScope;

use raw_window_handle::HasRawDisplayHandle;
use tauri_macros::default_runtime;
use tauri_runtime::window::{
  dpi::{PhysicalPosition, PhysicalSize},
  FileDropEvent,
};
use tauri_utils::PackageInfo;

use std::{
  collections::HashMap,
  fmt,
  sync::{mpsc::Sender, Arc, Weak},
};

use crate::runtime::menu::{Menu, MenuId, MenuIdRef};

use crate::runtime::RuntimeHandle;

#[cfg(target_os = "macos")]
use crate::ActivationPolicy;

pub(crate) type GlobalMenuEventListener<R> = Box<dyn Fn(WindowMenuEvent<R>) + Send + Sync>;
pub(crate) type GlobalWindowEventListener<R> = Box<dyn Fn(GlobalWindowEvent<R>) + Send + Sync>;
#[cfg(all(desktop, feature = "system-tray"))]
type SystemTrayEventListener<R> = Box<dyn Fn(&AppHandle<R>, tray::SystemTrayEvent) + Send + Sync>;

/// Api exposed on the `ExitRequested` event.
#[derive(Debug)]
pub struct ExitRequestApi(Sender<ExitRequestedEventAction>);

impl ExitRequestApi {
  /// Prevents the app from exiting
  pub fn prevent_exit(&self) {
    self.0.send(ExitRequestedEventAction::Prevent).unwrap();
  }
}

/// Api exposed on the `CloseRequested` event.
#[derive(Debug, Clone)]
pub struct CloseRequestApi(Sender<bool>);

impl CloseRequestApi {
  /// Prevents the window from being closed.
  pub fn prevent_close(&self) {
    self.0.send(true).unwrap();
  }
}

/// An event from a window.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum WindowEvent {
  /// The size of the window has changed. Contains the client area's new dimensions.
  Resized(PhysicalSize<u32>),
  /// The position of the window has changed. Contains the window's new position.
  Moved(PhysicalPosition<i32>),
  /// The window has been requested to close.
  #[non_exhaustive]
  CloseRequested {
    /// An API modify the behavior of the close requested event.
    api: CloseRequestApi,
  },
  /// The window has been destroyed.
  Destroyed,
  /// The window gained or lost focus.
  ///
  /// The parameter is true if the window has gained focus, and false if it has lost focus.
  Focused(bool),
  /// The window's scale factor has changed.
  ///
  /// The following user actions can cause DPI changes:
  ///
  /// - Changing the display's resolution.
  /// - Changing the display's scale factor (e.g. in Control Panel on Windows).
  /// - Moving the window to a display with a different scale factor.
  #[non_exhaustive]
  ScaleFactorChanged {
    /// The new scale factor.
    scale_factor: f64,
    /// The window inner size.
    new_inner_size: PhysicalSize<u32>,
  },
  /// An event associated with the file drop action.
  FileDrop(FileDropEvent),
  /// The system window theme has changed. Only delivered if the window [`theme`](`crate::window::WindowBuilder#method.theme`) is `None`.
  ///
  /// Applications might wish to react to this to change the theme of the content of the window when the system changes the window theme.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux**: Not supported.
  ThemeChanged(Theme),
}

impl From<RuntimeWindowEvent> for WindowEvent {
  fn from(event: RuntimeWindowEvent) -> Self {
    match event {
      RuntimeWindowEvent::Resized(size) => Self::Resized(size),
      RuntimeWindowEvent::Moved(position) => Self::Moved(position),
      RuntimeWindowEvent::CloseRequested { signal_tx } => Self::CloseRequested {
        api: CloseRequestApi(signal_tx),
      },
      RuntimeWindowEvent::Destroyed => Self::Destroyed,
      RuntimeWindowEvent::Focused(flag) => Self::Focused(flag),
      RuntimeWindowEvent::ScaleFactorChanged {
        scale_factor,
        new_inner_size,
      } => Self::ScaleFactorChanged {
        scale_factor,
        new_inner_size,
      },
      RuntimeWindowEvent::FileDrop(event) => Self::FileDrop(event),
      RuntimeWindowEvent::ThemeChanged(theme) => Self::ThemeChanged(theme),
    }
  }
}

/// An application event, triggered from the event loop.
///
/// See [`App::run`](crate::App#method.run) for usage examples.
#[derive(Debug)]
#[non_exhaustive]
pub enum RunEvent {
  /// Event loop is exiting.
  Exit,
  /// The app is about to exit
  #[non_exhaustive]
  ExitRequested {
    /// Event API
    api: ExitRequestApi,
  },
  /// An event associated with a window.
  #[non_exhaustive]
  WindowEvent {
    /// The window label.
    label: String,
    /// The detailed event.
    event: WindowEvent,
  },
  /// Application ready.
  Ready,
  /// Sent if the event loop is being resumed.
  Resumed,
  /// Emitted when all of the event loop’s input events have been processed and redraw processing is about to begin.
  ///
  /// This event is useful as a place to put your code that should be run after all state-changing events have been handled and you want to do stuff (updating state, performing calculations, etc) that happens as the “main body” of your event loop.
  MainEventsCleared,
}

impl From<EventLoopMessage> for RunEvent {
  fn from(event: EventLoopMessage) -> Self {
    match event {}
  }
}

/// A menu event that was triggered on a window.
#[default_runtime(crate::Wry, wry)]
#[derive(Debug)]
pub struct WindowMenuEvent<R: Runtime> {
  pub(crate) menu_item_id: MenuId,
  pub(crate) window: Window<R>,
}

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
#[derive(Debug)]
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

/// The asset resolver is a helper to access the [`tauri_utils::assets::Assets`] interface.
#[derive(Debug, Clone)]
pub struct AssetResolver<R: Runtime> {
  manager: WindowManager<R>,
}

impl<R: Runtime> AssetResolver<R> {
  /// Gets the app asset associated with the given path.
  pub fn get(&self, path: String) -> Option<Asset> {
    self.manager.get_asset(path).ok()
  }
}

/// A handle to the currently running application.
///
/// This type implements [`Manager`] which allows for manipulation of global application items.
#[default_runtime(crate::Wry, wry)]
#[derive(Debug)]
pub struct AppHandle<R: Runtime> {
  pub(crate) runtime_handle: R::Handle,
  pub(crate) manager: WindowManager<R>,
}

/// APIs specific to the wry runtime.
#[cfg(feature = "wry")]
impl AppHandle<crate::Wry> {
  /// Create a new tao window using a callback. The event loop must be running at this point.
  pub fn create_tao_window<
    F: FnOnce() -> (String, tauri_runtime_wry::WryWindowBuilder) + Send + 'static,
  >(
    &self,
    f: F,
  ) -> crate::Result<Weak<tauri_runtime_wry::Window>> {
    self.runtime_handle.create_tao_window(f).map_err(Into::into)
  }

  /// Sends a window message to the event loop.
  pub fn send_tao_window_event(
    &self,
    window_id: tauri_runtime_wry::WindowId,
    message: tauri_runtime_wry::WindowMessage,
  ) -> crate::Result<()> {
    self
      .runtime_handle
      .send_event(tauri_runtime_wry::Message::Window(
        self.runtime_handle.window_id(window_id),
        message,
      ))
      .map_err(Into::into)
  }
}

impl<R: Runtime> Clone for AppHandle<R> {
  fn clone(&self) -> Self {
    Self {
      runtime_handle: self.runtime_handle.clone(),
      manager: self.manager.clone(),
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
  /// Runs the given closure on the main thread.
  pub fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> crate::Result<()> {
    self
      .runtime_handle
      .run_on_main_thread(f)
      .map_err(Into::into)
  }

  /// Adds a Tauri application plugin.
  /// This function can be used to register a plugin that is loaded dynamically e.g. after login.
  /// For plugins that are created when the app is started, prefer [`Builder::plugin`].
  ///
  /// See [`Builder::plugin`] for more information.
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::{plugin::{Builder as PluginBuilder, TauriPlugin}, Runtime};
  ///
  /// fn init_plugin<R: Runtime>() -> TauriPlugin<R> {
  ///   PluginBuilder::new("dummy").build()
  /// }
  ///
  /// tauri::Builder::default()
  ///   .setup(move |app| {
  ///     let handle = app.handle();
  ///     std::thread::spawn(move || {
  ///       handle.plugin(init_plugin());
  ///     });
  ///
  ///     Ok(())
  ///   });
  /// ```
  pub fn plugin<P: Plugin<R> + 'static>(&self, mut plugin: P) -> crate::Result<()> {
    plugin
      .initialize(
        self,
        self
          .config()
          .plugins
          .0
          .get(plugin.name())
          .cloned()
          .unwrap_or_default(),
      )
      .map_err(|e| crate::Error::PluginInitialization(plugin.name().to_string(), e.to_string()))?;
    self
      .manager()
      .inner
      .plugins
      .lock()
      .unwrap()
      .register(plugin);
    Ok(())
  }

  /// Removes the plugin with the given name.
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::{plugin::{Builder as PluginBuilder, TauriPlugin, Plugin}, Runtime};
  ///
  /// fn init_plugin<R: Runtime>() -> TauriPlugin<R> {
  ///   PluginBuilder::new("dummy").build()
  /// }
  ///
  /// let plugin = init_plugin();
  /// // `.name()` requires the `PLugin` trait import
  /// let plugin_name = plugin.name();
  /// tauri::Builder::default()
  ///   .plugin(plugin)
  ///   .setup(move |app| {
  ///     let handle = app.handle();
  ///     std::thread::spawn(move || {
  ///       handle.remove_plugin(plugin_name);
  ///     });
  ///
  ///     Ok(())
  ///   });
  /// ```
  pub fn remove_plugin(&self, plugin: &'static str) -> bool {
    self
      .manager()
      .inner
      .plugins
      .lock()
      .unwrap()
      .unregister(plugin)
  }

  /// Exits the app. This is the same as [`std::process::exit`], but it performs cleanup on this application.
  pub fn exit(&self, exit_code: i32) {
    self.cleanup_before_exit();
    std::process::exit(exit_code);
  }

  /// Restarts the app. This is the same as [`crate::process::restart`], but it performs cleanup on this application.
  pub fn restart(&self) {
    self.cleanup_before_exit();
    crate::process::restart(&self.env());
  }

  /// Runs necessary cleanup tasks before exiting the process
  fn cleanup_before_exit(&self) {
    #[cfg(all(windows, feature = "system-tray"))]
    {
      for tray in self.manager().trays().values() {
        let _ = tray.destroy();
      }
    }
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

  fn managed_app_handle(&self) -> AppHandle<R> {
    self.clone()
  }
}

/// The instance of the currently running application.
///
/// This type implements [`Manager`] which allows for manipulation of global application items.
#[default_runtime(crate::Wry, wry)]
pub struct App<R: Runtime> {
  runtime: Option<R>,
  pending_windows: Option<Vec<PendingWindow<EventLoopMessage, R>>>,
  setup: Option<SetupHook<R>>,
  manager: WindowManager<R>,
  handle: AppHandle<R>,
}

impl<R: Runtime> fmt::Debug for App<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("App")
      .field("runtime", &self.runtime)
      .field("manager", &self.manager)
      .field("handle", &self.handle)
      .finish()
  }
}

impl<R: Runtime> Manager<R> for App<R> {}
impl<R: Runtime> ManagerBase<R> for App<R> {
  fn manager(&self) -> &WindowManager<R> {
    &self.manager
  }

  fn runtime(&self) -> RuntimeOrDispatch<'_, R> {
    if let Some(runtime) = self.runtime.as_ref() {
      RuntimeOrDispatch::Runtime(runtime)
    } else {
      self.handle.runtime()
    }
  }

  fn managed_app_handle(&self) -> AppHandle<R> {
    self.handle()
  }
}

/// APIs specific to the wry runtime.
#[cfg(feature = "wry")]
impl App<crate::Wry> {
  /// Adds a [`tauri_runtime_wry::Plugin`] using its [`tauri_runtime_wry::PluginBuilder`].
  ///
  /// # Stability
  ///
  /// This API is unstable.
  pub fn wry_plugin<P: tauri_runtime_wry::PluginBuilder<EventLoopMessage> + Send + 'static>(
    &mut self,
    plugin: P,
  ) where
    <P as tauri_runtime_wry::PluginBuilder<EventLoopMessage>>::Plugin: Send,
  {
    self.handle.runtime_handle.plugin(plugin);
  }
}

macro_rules! shared_app_impl {
  ($app: ty) => {
    impl<R: Runtime> $app {
      /// Gets a handle to the first system tray.
      ///
      /// Prefer [`Self::tray_handle_by_id`] when multiple system trays are created.
      ///
      /// # Examples
      /// ```
      /// use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu};
      ///
      /// tauri::Builder::default()
      ///   .setup(|app| {
      ///     let app_handle = app.handle();
      ///     SystemTray::new()
      ///       .with_menu(
      ///         SystemTrayMenu::new()
      ///           .add_item(CustomMenuItem::new("quit", "Quit"))
      ///           .add_item(CustomMenuItem::new("open", "Open"))
      ///       )
      ///       .on_event(move |event| {
      ///         let tray_handle = app_handle.tray_handle();
      ///       })
      ///       .build(app)?;
      ///     Ok(())
      ///   });
      /// ```
      #[cfg(all(desktop, feature = "system-tray"))]
      #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
      pub fn tray_handle(&self) -> tray::SystemTrayHandle<R> {
        self
          .manager()
          .trays()
          .values()
          .next()
          .cloned()
          .expect("tray not configured; use the `Builder#system_tray`, `App#system_tray` or `AppHandle#system_tray` APIs first.")
      }


      /// Gets a handle to a system tray by its id.
      ///
      /// ```
      /// use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu};
      ///
      /// tauri::Builder::default()
      ///   .setup(|app| {
      ///     let app_handle = app.handle();
      ///     let tray_id = "my-tray";
      ///     SystemTray::new()
      ///       .with_id(tray_id)
      ///       .with_menu(
      ///         SystemTrayMenu::new()
      ///           .add_item(CustomMenuItem::new("quit", "Quit"))
      ///           .add_item(CustomMenuItem::new("open", "Open"))
      ///       )
      ///       .on_event(move |event| {
      ///         let tray_handle = app_handle.tray_handle_by_id(tray_id).unwrap();
      ///       })
      ///       .build(app)?;
      ///     Ok(())
      ///   });
      /// ```
      #[cfg(all(desktop, feature = "system-tray"))]
      #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
      pub fn tray_handle_by_id(&self, id: &str) -> Option<tray::SystemTrayHandle<R>> {
        self
          .manager()
          .get_tray(id)
      }

      /// Gets the app's configuration, defined on the `tauri.conf.json` file.
      pub fn config(&self) -> Arc<Config> {
        self.manager.config()
      }

      /// Gets the app's package information.
      pub fn package_info(&self) -> &PackageInfo {
        self.manager.package_info()
      }

      /// The application's asset resolver.
      pub fn asset_resolver(&self) -> AssetResolver<R> {
        AssetResolver {
          manager: self.manager.clone(),
        }
      }

      /// Returns the default window icon.
      pub fn default_window_icon(&self) -> Option<&Icon> {
        self.manager.inner.default_window_icon.as_ref()
      }

      /// Shows the application, but does not automatically focus it.
      #[cfg(target_os = "macos")]
      pub fn show(&self) -> crate::Result<()> {
        match self.runtime() {
          RuntimeOrDispatch::Runtime(r) => r.show(),
          RuntimeOrDispatch::RuntimeHandle(h) => h.show()?,
          _ => unreachable!(),
        }
        Ok(())
      }

      /// Hides the application.
      #[cfg(target_os = "macos")]
      pub fn hide(&self) -> crate::Result<()> {
        match self.runtime() {
          RuntimeOrDispatch::Runtime(r) => r.hide(),
          RuntimeOrDispatch::RuntimeHandle(h) => h.hide()?,
          _ => unreachable!(),
        }
        Ok(())
      }
    }
  };
}

shared_app_impl!(App<R>);
shared_app_impl!(AppHandle<R>);

impl<R: Runtime> App<R> {
  fn register_core_plugins(&self) -> crate::Result<()> {
    self.handle.plugin(crate::path::init())?;
    self.handle.plugin(crate::event::init())?;
    Ok(())
  }

  /// Gets a handle to the application instance.
  pub fn handle(&self) -> AppHandle<R> {
    self.handle.clone()
  }

  /// Sets the activation policy for the application. It is set to `NSApplicationActivationPolicyRegular` by default.
  ///
  /// # Examples
  /// ```,no_run
  /// let mut app = tauri::Builder::default()
  ///   // on an actual app, remove the string argument
  ///   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
  ///   .expect("error while building tauri application");
  /// #[cfg(target_os = "macos")]
  /// app.set_activation_policy(tauri::ActivationPolicy::Accessory);
  /// app.run(|_app_handle, _event| {});
  /// ```
  #[cfg(target_os = "macos")]
  #[cfg_attr(doc_cfg, doc(cfg(target_os = "macos")))]
  pub fn set_activation_policy(&mut self, activation_policy: ActivationPolicy) {
    self
      .runtime
      .as_mut()
      .unwrap()
      .set_activation_policy(activation_policy);
  }

  /// Change the device event filter mode.
  ///
  /// Since the DeviceEvent capture can lead to high CPU usage for unfocused windows, [`tao`]
  /// will ignore them by default for unfocused windows on Windows. This method allows changing
  /// the filter to explicitly capture them again.
  ///
  /// ## Platform-specific
  ///
  /// - ** Linux / macOS / iOS / Android**: Unsupported.
  ///
  /// # Examples
  /// ```,no_run
  /// let mut app = tauri::Builder::default()
  ///   // on an actual app, remove the string argument
  ///   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
  ///   .expect("error while building tauri application");
  /// app.set_device_event_filter(tauri::DeviceEventFilter::Always);
  /// app.run(|_app_handle, _event| {});
  /// ```
  ///
  /// [`tao`]: https://crates.io/crates/tao
  pub fn set_device_event_filter(&mut self, filter: DeviceEventFilter) {
    self
      .runtime
      .as_mut()
      .unwrap()
      .set_device_event_filter(filter);
  }

  /// Runs the application.
  ///
  /// # Examples
  /// ```,no_run
  /// let app = tauri::Builder::default()
  ///   // on an actual app, remove the string argument
  ///   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
  ///   .expect("error while building tauri application");
  /// app.run(|_app_handle, event| match event {
  ///   tauri::RunEvent::ExitRequested { api, .. } => {
  ///     api.prevent_exit();
  ///   }
  ///   _ => {}
  /// });
  /// ```
  pub fn run<F: FnMut(&AppHandle<R>, RunEvent) + 'static>(mut self, mut callback: F) {
    let app_handle = self.handle();
    let manager = self.manager.clone();
    self.runtime.take().unwrap().run(move |event| match event {
      RuntimeRunEvent::Ready => {
        if let Err(e) = setup(&mut self) {
          panic!("Failed to setup app: {e}");
        }
        on_event_loop_event(
          &app_handle,
          RuntimeRunEvent::Ready,
          &manager,
          Some(&mut callback),
        );
      }
      RuntimeRunEvent::Exit => {
        on_event_loop_event(
          &app_handle,
          RuntimeRunEvent::Exit,
          &manager,
          Some(&mut callback),
        );
        app_handle.cleanup_before_exit();
      }
      _ => {
        on_event_loop_event(&app_handle, event, &manager, Some(&mut callback));
      }
    });
  }

  /// Runs a iteration of the runtime event loop and immediately return.
  ///
  /// Note that when using this API, app cleanup is not automatically done.
  /// The cleanup calls [`crate::process::kill_children`] so you may want to call that function before exiting the application.
  /// Additionally, the cleanup calls [AppHandle#remove_system_tray](`AppHandle#method.remove_system_tray`) (Windows only).
  ///
  /// # Examples
  /// ```no_run
  /// let mut app = tauri::Builder::default()
  ///   // on an actual app, remove the string argument
  ///   .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
  ///   .expect("error while building tauri application");
  /// loop {
  ///   let iteration = app.run_iteration();
  ///   if iteration.window_count == 0 {
  ///     break;
  ///   }
  /// }
  /// ```
  #[cfg(desktop)]
  pub fn run_iteration(&mut self) -> crate::runtime::RunIteration {
    let manager = self.manager.clone();
    let app_handle = self.handle();
    self.runtime.as_mut().unwrap().run_iteration(move |event| {
      on_event_loop_event(
        &app_handle,
        event,
        &manager,
        Option::<&mut Box<dyn FnMut(&AppHandle<R>, RunEvent)>>::None,
      )
    })
  }
}

/// Builds a Tauri application.
///
/// # Examples
/// ```,no_run
/// tauri::Builder::default()
///   // on an actual app, remove the string argument
///   .run(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
///  .expect("error while running tauri application");
/// ```
#[allow(clippy::type_complexity)]
pub struct Builder<R: Runtime> {
  /// A flag indicating that the runtime must be started on an environment that supports the event loop not on the main thread.
  #[cfg(any(windows, target_os = "linux"))]
  runtime_any_thread: bool,

  /// The JS message handler.
  invoke_handler: Box<InvokeHandler<R>>,

  /// The JS message responder.
  pub(crate) invoke_responder: Arc<InvokeResponder<R>>,

  /// The script that initializes the `window.__TAURI_POST_MESSAGE__` function.
  invoke_initialization_script: String,

  /// The setup hook.
  setup: SetupHook<R>,

  /// Page load hook.
  on_page_load: Box<OnPageLoad<R>>,

  /// windows to create when starting up.
  pending_windows: Vec<PendingWindow<EventLoopMessage, R>>,

  /// All passed plugins
  plugins: PluginStore<R>,

  /// The webview protocols available to all windows.
  uri_scheme_protocols: HashMap<String, Arc<CustomProtocol<R>>>,

  /// App state.
  state: StateManager,

  /// The menu set to all windows.
  menu: Option<Menu>,

  /// Enable macOS default menu creation.
  #[allow(unused)]
  enable_macos_default_menu: bool,

  /// Menu event handlers that listens to all windows.
  menu_event_listeners: Vec<GlobalMenuEventListener<R>>,

  /// Window event handlers that listens to all windows.
  window_event_listeners: Vec<GlobalWindowEventListener<R>>,

  /// The app system tray.
  #[cfg(all(desktop, feature = "system-tray"))]
  system_tray: Option<tray::SystemTray>,

  /// System tray event handlers.
  #[cfg(all(desktop, feature = "system-tray"))]
  system_tray_event_listeners: Vec<SystemTrayEventListener<R>>,

  /// The device event filter.
  device_event_filter: DeviceEventFilter,
}

impl<R: Runtime> Builder<R> {
  /// Creates a new App builder.
  pub fn new() -> Self {
    Self {
      #[cfg(any(windows, target_os = "linux"))]
      runtime_any_thread: false,
      setup: Box::new(|_| Ok(())),
      invoke_handler: Box::new(|_| false),
      invoke_responder: Arc::new(window_invoke_responder),
      invoke_initialization_script:
        format!("Object.defineProperty(window, '__TAURI_POST_MESSAGE__', {{ value: (message) => window.ipc.postMessage({}(message)) }})", crate::manager::STRINGIFY_IPC_MESSAGE_FN),
      on_page_load: Box::new(|_, _| ()),
      pending_windows: Default::default(),
      plugins: PluginStore::default(),
      uri_scheme_protocols: Default::default(),
      state: StateManager::new(),
      menu: None,
      enable_macos_default_menu: true,
      menu_event_listeners: Vec::new(),
      window_event_listeners: Vec::new(),
      #[cfg(all(desktop, feature = "system-tray"))]
      system_tray: None,
      #[cfg(all(desktop, feature = "system-tray"))]
      system_tray_event_listeners: Vec::new(),
      device_event_filter: Default::default(),
    }
  }

  /// Builds a new Tauri application running on any thread, bypassing the main thread requirement.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** on macOS the application *must* be executed on the main thread, so this function is not exposed.
  #[cfg(any(windows, target_os = "linux"))]
  #[cfg_attr(doc_cfg, doc(cfg(any(windows, target_os = "linux"))))]
  #[must_use]
  pub fn any_thread(mut self) -> Self {
    self.runtime_any_thread = true;
    self
  }

  /// Defines the JS message handler callback.
  ///
  /// # Examples
  /// ```
  /// #[tauri::command]
  /// fn command_1() -> String {
  ///   return "hello world".to_string();
  /// }
  /// tauri::Builder::default()
  ///   .invoke_handler(tauri::generate_handler![
  ///     command_1,
  ///     // etc...
  ///   ]);
  /// ```
  #[must_use]
  pub fn invoke_handler<F>(mut self, invoke_handler: F) -> Self
  where
    F: Fn(Invoke<R>) -> bool + Send + Sync + 'static,
  {
    self.invoke_handler = Box::new(invoke_handler);
    self
  }

  /// Defines a custom JS message system.
  ///
  /// The `responder` is a function that will be called when a command has been executed and must send a response to the JS layer.
  ///
  /// The `initialization_script` is a script that initializes `window.__TAURI_POST_MESSAGE__`.
  /// That function must take the `message: object` argument and send it to the backend.
  #[must_use]
  pub fn invoke_system<F>(mut self, initialization_script: String, responder: F) -> Self
  where
    F: Fn(Window<R>, InvokeResponse, CallbackFn, CallbackFn) + Send + Sync + 'static,
  {
    self.invoke_initialization_script = initialization_script;
    self.invoke_responder = Arc::new(responder);
    self
  }

  /// Defines the setup hook.
  ///
  /// # Examples
  /// ```
  /// use tauri::Manager;
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let main_window = app.get_window("main").unwrap();
  ///     main_window.set_title("Tauri!");
  ///     Ok(())
  ///   });
  /// ```
  #[must_use]
  pub fn setup<F>(mut self, setup: F) -> Self
  where
    F: FnOnce(&mut App<R>) -> Result<(), Box<dyn std::error::Error>> + Send + 'static,
  {
    self.setup = Box::new(setup);
    self
  }

  /// Defines the page load hook.
  #[must_use]
  pub fn on_page_load<F>(mut self, on_page_load: F) -> Self
  where
    F: Fn(Window<R>, PageLoadPayload) + Send + Sync + 'static,
  {
    self.on_page_load = Box::new(on_page_load);
    self
  }

  /// Adds a Tauri application plugin.
  ///
  /// A plugin is created using the [`crate::plugin::Builder`] struct.Check its documentation for more information.
  ///
  /// # Examples
  ///
  /// ```
  /// mod plugin {
  ///   use tauri::{plugin::{Builder as PluginBuilder, TauriPlugin}, RunEvent, Runtime};
  ///
  ///   // this command can be called in the frontend using `invoke('plugin:window|do_something')`.
  ///   #[tauri::command]
  ///   async fn do_something<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>) -> Result<(), String> {
  ///     println!("command called");
  ///     Ok(())
  ///   }
  ///   pub fn init<R: Runtime>() -> TauriPlugin<R> {
  ///     PluginBuilder::new("window")
  ///       .setup(|app, api| {
  ///         // initialize the plugin here
  ///         Ok(())
  ///       })
  ///       .on_event(|app, event| {
  ///         match event {
  ///           RunEvent::Ready => {
  ///             println!("app is ready");
  ///           }
  ///           RunEvent::WindowEvent { label, event, .. } => {
  ///             println!("window {} received an event: {:?}", label, event);
  ///           }
  ///           _ => (),
  ///         }
  ///       })
  ///       .invoke_handler(tauri::generate_handler![do_something])
  ///       .build()
  ///   }
  /// }
  ///
  /// tauri::Builder::default()
  ///   .plugin(plugin::init());
  /// ```
  #[must_use]
  pub fn plugin<P: Plugin<R> + 'static>(mut self, plugin: P) -> Self {
    self.plugins.register(plugin);
    self
  }

  /// Add `state` to the state managed by the application.
  ///
  /// This method can be called any number of times as long as each call
  /// refers to a different `T`.
  ///
  /// Managed state can be retrieved by any command handler via the
  /// [`State`](crate::State) guard. In particular, if a value of type `T`
  /// is managed by Tauri, adding `State<T>` to the list of arguments in a
  /// command handler instructs Tauri to retrieve the managed value.
  ///
  /// # Panics
  ///
  /// Panics if state of type `T` is already being managed.
  ///
  /// # Mutability
  ///
  /// Since the managed state is global and must be [`Send`] + [`Sync`], mutations can only happen through interior mutability:
  ///
  /// ```,no_run
  /// use std::{collections::HashMap, sync::Mutex};
  /// use tauri::State;
  /// // here we use Mutex to achieve interior mutability
  /// struct Storage {
  ///   store: Mutex<HashMap<u64, String>>,
  /// }
  /// struct Connection;
  /// struct DbConnection {
  ///   db: Mutex<Option<Connection>>,
  /// }
  ///
  /// #[tauri::command]
  /// fn connect(connection: State<DbConnection>) {
  ///   // initialize the connection, mutating the state with interior mutability
  ///   *connection.db.lock().unwrap() = Some(Connection {});
  /// }
  ///
  /// #[tauri::command]
  /// fn storage_insert(key: u64, value: String, storage: State<Storage>) {
  ///   // mutate the storage behind the Mutex
  ///   storage.store.lock().unwrap().insert(key, value);
  /// }
  ///
  /// tauri::Builder::default()
  ///   .manage(Storage { store: Default::default() })
  ///   .manage(DbConnection { db: Default::default() })
  ///   .invoke_handler(tauri::generate_handler![connect, storage_insert])
  ///   // on an actual app, remove the string argument
  ///   .run(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
  ///   .expect("error while running tauri application");
  /// ```
  ///
  /// # Examples
  ///
  /// ```,no_run
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
  /// tauri::Builder::default()
  ///   .manage(MyInt(10))
  ///   .manage(MyString("Hello, managed state!".to_string()))
  ///   .invoke_handler(tauri::generate_handler![int_command, string_command])
  ///   // on an actual app, remove the string argument
  ///   .run(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
  ///   .expect("error while running tauri application");
  /// ```
  #[must_use]
  pub fn manage<T>(self, state: T) -> Self
  where
    T: Send + Sync + 'static,
  {
    let type_name = std::any::type_name::<T>();
    assert!(
      self.state.set(state),
      "state for type '{type_name}' is already being managed",
    );
    self
  }

  /// Sets the given system tray to be built before the app runs.
  ///
  /// Prefer the [`SystemTray#method.build`](crate::SystemTray#method.build) method to create the tray at runtime instead.
  ///
  /// # Examples
  /// ```
  /// use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu};
  ///
  /// tauri::Builder::default()
  ///   .system_tray(SystemTray::new().with_menu(
  ///     SystemTrayMenu::new()
  ///       .add_item(CustomMenuItem::new("quit", "Quit"))
  ///       .add_item(CustomMenuItem::new("open", "Open"))
  ///   ));
  /// ```
  #[cfg(all(desktop, feature = "system-tray"))]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
  #[must_use]
  pub fn system_tray(mut self, system_tray: tray::SystemTray) -> Self {
    self.system_tray.replace(system_tray);
    self
  }

  /// Sets the menu to use on all windows.
  ///
  /// # Examples
  /// ```
  /// use tauri::{MenuEntry, Submenu, MenuItem, Menu, CustomMenuItem};
  ///
  /// tauri::Builder::default()
  ///   .menu(Menu::with_items([
  ///     MenuEntry::Submenu(Submenu::new(
  ///       "File",
  ///       Menu::with_items([
  ///         MenuItem::CloseWindow.into(),
  ///         #[cfg(target_os = "macos")]
  ///         CustomMenuItem::new("hello", "Hello").into(),
  ///       ]),
  ///     )),
  ///   ]));
  /// ```
  #[must_use]
  pub fn menu(mut self, menu: Menu) -> Self {
    self.menu.replace(menu);
    self
  }

  /// Enable or disable the default menu on macOS. Enabled by default.
  ///
  /// # Examples
  /// ```
  /// use tauri::{MenuEntry, Submenu, MenuItem, Menu, CustomMenuItem};
  ///
  /// tauri::Builder::default()
  ///   .enable_macos_default_menu(false);
  /// ```
  #[must_use]
  pub fn enable_macos_default_menu(mut self, enable: bool) -> Self {
    self.enable_macos_default_menu = enable;
    self
  }

  /// Registers a menu event handler for all windows.
  ///
  /// # Examples
  /// ```
  /// use tauri::{Menu, MenuEntry, Submenu, CustomMenuItem, api, Manager};
  /// tauri::Builder::default()
  ///   .menu(Menu::with_items([
  ///     MenuEntry::Submenu(Submenu::new(
  ///       "File",
  ///       Menu::with_items([
  ///         CustomMenuItem::new("new", "New").into(),
  ///         CustomMenuItem::new("learn-more", "Learn More").into(),
  ///       ]),
  ///     )),
  ///   ]))
  ///   .on_menu_event(|event| {
  ///     match event.menu_item_id() {
  ///       "learn-more" => {
  ///         // open a link in the browser using tauri-plugin-shell
  ///       }
  ///       id => {
  ///         // do something with other events
  ///         println!("got menu event: {}", id);
  ///       }
  ///     }
  ///   });
  /// ```
  #[must_use]
  pub fn on_menu_event<F: Fn(WindowMenuEvent<R>) + Send + Sync + 'static>(
    mut self,
    handler: F,
  ) -> Self {
    self.menu_event_listeners.push(Box::new(handler));
    self
  }

  /// Registers a window event handler for all windows.
  ///
  /// # Examples
  /// ```
  /// tauri::Builder::default()
  ///   .on_window_event(|event| match event.event() {
  ///     tauri::WindowEvent::Focused(focused) => {
  ///       // hide window whenever it loses focus
  ///       if !focused {
  ///         event.window().hide().unwrap();
  ///       }
  ///     }
  ///     _ => {}
  ///   });
  /// ```
  #[must_use]
  pub fn on_window_event<F: Fn(GlobalWindowEvent<R>) + Send + Sync + 'static>(
    mut self,
    handler: F,
  ) -> Self {
    self.window_event_listeners.push(Box::new(handler));
    self
  }

  /// Registers a system tray event handler.
  ///
  /// Prefer the [`SystemTray#method.on_event`](crate::SystemTray#method.on_event) method when creating a tray at runtime instead.
  ///
  /// # Examples
  /// ```
  /// use tauri::{Manager, SystemTrayEvent};
  /// tauri::Builder::default()
  ///   .on_system_tray_event(|app, event| match event {
  ///     // show window with id "main" when the tray is left clicked
  ///     SystemTrayEvent::LeftClick { .. } => {
  ///       let window = app.get_window("main").unwrap();
  ///       window.show().unwrap();
  ///       window.set_focus().unwrap();
  ///     }
  ///     _ => {}
  ///   });
  /// ```
  #[cfg(all(desktop, feature = "system-tray"))]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
  #[must_use]
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
  #[must_use]
  pub fn register_uri_scheme_protocol<
    N: Into<String>,
    H: Fn(&AppHandle<R>, &HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>>
      + Send
      + Sync
      + 'static,
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

  /// Change the device event filter mode.
  ///
  /// Since the DeviceEvent capture can lead to high CPU usage for unfocused windows, [`tao`]
  /// will ignore them by default for unfocused windows on Windows. This method allows changing
  /// the filter to explicitly capture them again.
  ///
  /// ## Platform-specific
  ///
  /// - ** Linux / macOS / iOS / Android**: Unsupported.
  ///
  /// # Examples
  /// ```,no_run
  /// tauri::Builder::default()
  ///   .device_event_filter(tauri::DeviceEventFilter::Always);
  /// ```
  ///
  /// [`tao`]: https://crates.io/crates/tao
  pub fn device_event_filter(mut self, filter: DeviceEventFilter) -> Self {
    self.device_event_filter = filter;
    self
  }

  /// Builds the application.
  #[allow(clippy::type_complexity)]
  pub fn build<A: Assets>(mut self, context: Context<A>) -> crate::Result<App<R>> {
    #[cfg(target_os = "macos")]
    if self.menu.is_none() && self.enable_macos_default_menu {
      self.menu = Some(Menu::os_default(&context.package_info().name));
    }

    let manager = WindowManager::with_handlers(
      context,
      self.plugins,
      self.invoke_handler,
      self.on_page_load,
      self.uri_scheme_protocols,
      self.state,
      self.window_event_listeners,
      (self.menu, self.menu_event_listeners),
      (self.invoke_responder, self.invoke_initialization_script),
    );

    // set up all the windows defined in the config
    for config in manager.config().tauri.windows.clone() {
      let label = config.label.clone();
      let webview_attributes = WebviewAttributes::from(&config);
      self.pending_windows.push(PendingWindow::with_config(
        config,
        webview_attributes,
        label,
      )?);
    }

    #[cfg(any(windows, target_os = "linux"))]
    let mut runtime = if self.runtime_any_thread {
      R::new_any_thread()?
    } else {
      R::new()?
    };
    #[cfg(not(any(windows, target_os = "linux")))]
    let mut runtime = R::new()?;

    runtime.set_device_event_filter(self.device_event_filter);

    let runtime_handle = runtime.handle();

    #[allow(unused_mut)]
    let mut app = App {
      runtime: Some(runtime),
      pending_windows: Some(self.pending_windows),
      setup: Some(self.setup),
      manager: manager.clone(),
      handle: AppHandle {
        runtime_handle,
        manager,
      },
    };

    app.register_core_plugins()?;

    let env = Env::default();
    app.manage(env);

    app.manage(Scopes {
      ipc: IpcScope::new(&app.config()),
      #[cfg(feature = "protocol-asset")]
      asset_protocol: FsScope::for_fs_api(&app, &app.config().tauri.security.asset_protocol.scope)?,
    });

    #[cfg(windows)]
    {
      if let crate::utils::config::WebviewInstallMode::FixedRuntime { path } = &app
        .manager
        .config()
        .tauri
        .bundle
        .windows
        .webview_install_mode
      {
        if let Ok(resource_dir) = app.path().resource_dir() {
          std::env::set_var(
            "WEBVIEW2_BROWSER_EXECUTABLE_FOLDER",
            resource_dir.join(path),
          );
        } else {
          #[cfg(debug_assertions)]
          eprintln!(
            "failed to resolve resource directory; fallback to the installed Webview2 runtime."
          );
        }
      }
    }

    #[cfg(all(desktop, feature = "system-tray"))]
    {
      if let Some(tray) = self.system_tray {
        tray.build(&app)?;
      }

      for listener in self.system_tray_event_listeners {
        let app_handle = app.handle();
        let listener = Arc::new(std::sync::Mutex::new(listener));
        app
          .runtime
          .as_mut()
          .unwrap()
          .on_system_tray_event(move |tray_id, event| {
            if let Some((tray_id, tray)) = app_handle.manager().get_tray_by_runtime_id(tray_id) {
              let app_handle = app_handle.clone();
              let event = tray::SystemTrayEvent::from_runtime_event(event, tray_id, &tray.ids);
              let listener = listener.clone();
              listener.lock().unwrap()(&app_handle, event);
            }
          });
      }
    }

    app.manager.initialize_plugins(&app.handle())?;

    Ok(app)
  }

  /// Runs the configured Tauri application.
  pub fn run<A: Assets>(self, context: Context<A>) -> crate::Result<()> {
    self.build(context)?.run(|_, _| {});
    Ok(())
  }
}

unsafe impl<R: Runtime> HasRawDisplayHandle for AppHandle<R> {
  fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
    self.runtime_handle.raw_display_handle()
  }
}

unsafe impl<R: Runtime> HasRawDisplayHandle for App<R> {
  fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
    self.handle.raw_display_handle()
  }
}

fn setup<R: Runtime>(app: &mut App<R>) -> crate::Result<()> {
  let pending_windows = app.pending_windows.take();
  if let Some(pending_windows) = pending_windows {
    let window_labels = pending_windows
      .iter()
      .map(|p| p.label.clone())
      .collect::<Vec<_>>();

    for pending in pending_windows {
      let pending = app
        .manager
        .prepare_window(app.handle.clone(), pending, &window_labels)?;
      let window_effects = pending.webview_attributes.window_effects.clone();
      let detached = if let RuntimeOrDispatch::RuntimeHandle(runtime) = app.handle().runtime() {
        runtime.create_window(pending)?
      } else {
        // the AppHandle's runtime is always RuntimeOrDispatch::RuntimeHandle
        unreachable!()
      };
      let window = app.manager.attach_window(app.handle(), detached);

      if let Some(effects) = window_effects {
        crate::vibrancy::set_window_effects(&window, Some(effects))?;
      }
    }
  }

  if let Some(setup) = app.setup.take() {
    (setup)(app).map_err(|e| crate::Error::Setup(e.into()))?;
  }

  Ok(())
}

fn on_event_loop_event<R: Runtime, F: FnMut(&AppHandle<R>, RunEvent) + 'static>(
  app_handle: &AppHandle<R>,
  event: RuntimeRunEvent<EventLoopMessage>,
  manager: &WindowManager<R>,
  callback: Option<&mut F>,
) {
  if let RuntimeRunEvent::WindowEvent {
    label,
    event: RuntimeWindowEvent::Destroyed,
  } = &event
  {
    manager.on_window_close(label);
  }

  let event = match event {
    RuntimeRunEvent::Exit => RunEvent::Exit,
    RuntimeRunEvent::ExitRequested { tx } => RunEvent::ExitRequested {
      api: ExitRequestApi(tx),
    },
    RuntimeRunEvent::WindowEvent { label, event } => RunEvent::WindowEvent {
      label,
      event: event.into(),
    },
    RuntimeRunEvent::Ready => {
      // set the app icon in development
      #[cfg(all(dev, target_os = "macos"))]
      unsafe {
        use cocoa::{
          appkit::NSImage,
          base::{id, nil},
          foundation::NSData,
        };
        use objc::*;
        if let Some(icon) = app_handle.manager.inner.app_icon.clone() {
          let ns_app: id = msg_send![class!(NSApplication), sharedApplication];
          let data = NSData::dataWithBytes_length_(
            nil,
            icon.as_ptr() as *const std::os::raw::c_void,
            icon.len() as u64,
          );
          let app_icon = NSImage::initWithData_(NSImage::alloc(nil), data);
          let _: () = msg_send![ns_app, setApplicationIconImage: app_icon];
        }
      }
      RunEvent::Ready
    }
    RuntimeRunEvent::Resumed => RunEvent::Resumed,
    RuntimeRunEvent::MainEventsCleared => RunEvent::MainEventsCleared,
    RuntimeRunEvent::UserEvent(t) => t.into(),
    _ => unimplemented!(),
  };

  manager
    .inner
    .plugins
    .lock()
    .expect("poisoned plugin store")
    .on_event(app_handle, &event);

  if let Some(c) = callback {
    c(app_handle, event);
  }
}

/// Make `Wry` the default `Runtime` for `Builder`
#[cfg(feature = "wry")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "wry")))]
impl Default for Builder<crate::Wry> {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn is_send_sync() {
    crate::test_utils::assert_send::<super::AppHandle>();
    crate::test_utils::assert_sync::<super::AppHandle>();

    #[cfg(feature = "wry")]
    {
      crate::test_utils::assert_send::<super::AssetResolver<crate::Wry>>();
      crate::test_utils::assert_sync::<super::AssetResolver<crate::Wry>>();
    }
  }
}
