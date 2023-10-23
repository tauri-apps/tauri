// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  command::{CommandArg, CommandItem},
  ipc::{
    channel::ChannelDataIpcQueue, CallbackFn, Invoke, InvokeError, InvokeHandler, InvokeResponder,
    InvokeResponse,
  },
  manager::{Asset, UriSchemeProtocol, WindowManager},
  plugin::{Plugin, PluginStore},
  runtime::{
    webview::WebviewAttributes,
    window::{PendingWindow, WindowEvent as RuntimeWindowEvent},
    ExitRequestedEventAction, RunEvent as RuntimeRunEvent,
  },
  scope,
  sealed::{ManagerBase, RuntimeOrDispatch},
  utils::config::Config,
  utils::{assets::Assets, Env},
  Context, DeviceEventFilter, EventLoopMessage, Icon, Manager, Monitor, Runtime, Scopes,
  StateManager, Theme, Window,
};

#[cfg(desktop)]
use crate::menu::{Menu, MenuEvent};
#[cfg(all(desktop, feature = "tray-icon"))]
use crate::tray::{TrayIcon, TrayIconBuilder, TrayIconEvent, TrayIconId};
#[cfg(desktop)]
use crate::window::WindowMenu;
use raw_window_handle::HasRawDisplayHandle;
use serde::Deserialize;
use serialize_to_javascript::{default_template, DefaultTemplate, Template};
use tauri_macros::default_runtime;
#[cfg(desktop)]
use tauri_runtime::EventLoopProxy;
use tauri_runtime::{
  window::{
    dpi::{PhysicalPosition, PhysicalSize},
    FileDropEvent,
  },
  RuntimeInitArgs,
};
use tauri_utils::PackageInfo;

use std::{
  borrow::Cow,
  collections::HashMap,
  fmt,
  sync::{mpsc::Sender, Arc, Weak},
};

use crate::runtime::RuntimeHandle;

#[cfg(target_os = "macos")]
use crate::ActivationPolicy;

pub(crate) mod plugin;

#[cfg(desktop)]
pub(crate) type GlobalMenuEventListener<T> = Box<dyn Fn(&T, crate::menu::MenuEvent) + Send + Sync>;
#[cfg(all(desktop, feature = "tray-icon"))]
pub(crate) type GlobalTrayIconEventListener<T> =
  Box<dyn Fn(&T, crate::tray::TrayIconEvent) + Send + Sync>;
pub(crate) type GlobalWindowEventListener<R> = Box<dyn Fn(GlobalWindowEvent<R>) + Send + Sync>;
/// A closure that is run when the Tauri application is setting up.
pub type SetupHook<R> =
  Box<dyn FnOnce(&mut App<R>) -> Result<(), Box<dyn std::error::Error>> + Send>;
/// A closure that is run once every time a window is created and loaded.
pub type OnPageLoad<R> = dyn Fn(Window<R>, PageLoadPayload) + Send + Sync + 'static;

/// The payload for the [`OnPageLoad`] hook.
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
  /// Emitted when the user wants to open the specified resource with the app.
  #[cfg(any(target_os = "macos", target_os = "ios"))]
  #[cfg_attr(doc_cfg, doc(cfg(any(target_os = "macos", feature = "ios"))))]
  Opened {
    /// The URL of the resources that is being open.
    urls: Vec<url::Url>,
  },
  /// An event from a menu item, could be on the window menu bar, application menu bar (on macOS) or tray icon menu.
  #[cfg(desktop)]
  #[cfg_attr(doc_cfg, doc(cfg(desktop)))]
  MenuEvent(crate::menu::MenuEvent),
  /// An event from a tray icon.
  #[cfg(all(desktop, feature = "tray-icon"))]
  #[cfg_attr(doc_cfg, doc(cfg(all(desktop, feature = "tray-icon"))))]
  TrayIconEvent(crate::tray::TrayIconEvent),
}

impl From<EventLoopMessage> for RunEvent {
  fn from(event: EventLoopMessage) -> Self {
    match event {
      #[cfg(desktop)]
      EventLoopMessage::MenuEvent(e) => Self::MenuEvent(e),
      #[cfg(all(desktop, feature = "tray-icon"))]
      EventLoopMessage::TrayIconEvent(e) => Self::TrayIconEvent(e),
    }
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
  ///     let handle = app.handle().clone();
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
  ///     let handle = app.handle().clone();
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
}

impl<R: Runtime> Manager<R> for AppHandle<R> {}
impl<R: Runtime> ManagerBase<R> for AppHandle<R> {
  fn manager(&self) -> &WindowManager<R> {
    &self.manager
  }

  fn runtime(&self) -> RuntimeOrDispatch<'_, R> {
    RuntimeOrDispatch::RuntimeHandle(self.runtime_handle.clone())
  }

  fn managed_app_handle(&self) -> &AppHandle<R> {
    self
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

  fn managed_app_handle(&self) -> &AppHandle<R> {
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
      /// Registers a global menu event listener.
      #[cfg(desktop)]
      pub fn on_menu_event<F: Fn(&AppHandle<R>, MenuEvent) + Send + Sync + 'static>(
        &self,
        handler: F,
      ) {
        self
          .manager
          .inner
          .menu_event_listeners
          .lock()
          .unwrap()
          .push(Box::new(handler));
      }

      /// Registers a global tray icon menu event listener.
      #[cfg(all(desktop, feature = "tray-icon"))]
      #[cfg_attr(doc_cfg, doc(cfg(all(desktop, feature = "tray-icon"))))]
      pub fn on_tray_icon_event<F: Fn(&AppHandle<R>, TrayIconEvent) + Send + Sync + 'static>(
        &self,
        handler: F,
      ) {
        self
          .manager
          .inner
          .global_tray_event_listeners
          .lock()
          .unwrap()
          .push(Box::new(handler));
      }

      /// Gets the first tray icon registered,
      /// usually the one configured in the Tauri configuration file.
      #[cfg(all(desktop, feature = "tray-icon"))]
      #[cfg_attr(doc_cfg, doc(cfg(all(desktop, feature = "tray-icon"))))]
      pub fn tray(&self) -> Option<TrayIcon<R>> {
        self
          .manager
          .inner
          .tray_icons
          .lock()
          .unwrap()
          .first()
          .cloned()
      }

      /// Removes the first tray icon registerd, usually the one configured in
      /// tauri config file, from tauri's internal state and returns it.
      ///
      /// Note that dropping the returned icon, will cause the tray icon to disappear.
      #[cfg(all(desktop, feature = "tray-icon"))]
      #[cfg_attr(doc_cfg, doc(cfg(all(desktop, feature = "tray-icon"))))]
      pub fn remove_tray(&self) -> Option<TrayIcon<R>> {
        let mut tray_icons = self.manager.inner.tray_icons.lock().unwrap();
        if !tray_icons.is_empty() {
          return Some(tray_icons.swap_remove(0));
        }
        None
      }

      /// Gets a tray icon using the provided id.
      #[cfg(all(desktop, feature = "tray-icon"))]
      #[cfg_attr(doc_cfg, doc(cfg(all(desktop, feature = "tray-icon"))))]
      pub fn tray_by_id<'a, I>(&self, id: &'a I) -> Option<TrayIcon<R>>
      where
        I: ?Sized,
        TrayIconId: PartialEq<&'a I>,
      {
        self
          .manager
          .inner
          .tray_icons
          .lock()
          .unwrap()
          .iter()
          .find(|t| t.id() == &id)
          .cloned()
      }

      /// Removes a tray icon using the provided id from tauri's internal state and returns it.
      ///
      /// Note that dropping the returned icon, will cause the tray icon to disappear.
      #[cfg(all(desktop, feature = "tray-icon"))]
      #[cfg_attr(doc_cfg, doc(cfg(all(desktop, feature = "tray-icon"))))]
      pub fn remove_tray_by_id<'a, I>(&self, id: &'a I) -> Option<TrayIcon<R>>
      where
        I: ?Sized,
        TrayIconId: PartialEq<&'a I>,
      {
        let mut tray_icons = self.manager.inner.tray_icons.lock().unwrap();
        let idx = tray_icons.iter().position(|t| t.id() == &id);
        if let Some(idx) = idx {
          return Some(tray_icons.swap_remove(idx));
        }
        None
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

      /// Returns the primary monitor of the system.
      ///
      /// Returns None if it can't identify any monitor as a primary one.
      pub fn primary_monitor(&self) -> crate::Result<Option<Monitor>> {
        Ok(match self.runtime() {
          RuntimeOrDispatch::Runtime(h) => h.primary_monitor().map(Into::into),
          RuntimeOrDispatch::RuntimeHandle(h) => h.primary_monitor().map(Into::into),
          _ => unreachable!(),
        })
      }

      /// Returns the list of all the monitors available on the system.
      pub fn available_monitors(&self) -> crate::Result<Vec<Monitor>> {
        Ok(match self.runtime() {
          RuntimeOrDispatch::Runtime(h) => {
            h.available_monitors().into_iter().map(Into::into).collect()
          }
          RuntimeOrDispatch::RuntimeHandle(h) => {
            h.available_monitors().into_iter().map(Into::into).collect()
          }
          _ => unreachable!(),
        })
      }
      /// Returns the default window icon.
      pub fn default_window_icon(&self) -> Option<&Icon> {
        self.manager.inner.default_window_icon.as_ref()
      }

      /// Returns the app-wide menu.
      #[cfg(desktop)]
      pub fn menu(&self) -> Option<Menu<R>> {
        self.manager.menu_lock().clone()
      }

      /// Sets the app-wide menu and returns the previous one.
      ///
      /// If a window was not created with an explicit menu or had one set explicitly,
      /// this menu will be assigned to it.
      #[cfg(desktop)]
      pub fn set_menu(&self, menu: Menu<R>) -> crate::Result<Option<Menu<R>>> {
        let prev_menu = self.remove_menu()?;

        self.manager.insert_menu_into_stash(&menu);

        self.manager.menu_lock().replace(menu.clone());

        // set it on all windows that don't have one or previously had the app-wide menu
        #[cfg(not(target_os = "macos"))]
        {
          for window in self.manager.windows().values() {
            let has_app_wide_menu = window.has_app_wide_menu() || window.menu().is_none();
            if has_app_wide_menu {
              window.set_menu(menu.clone())?;
              window.menu_lock().replace(WindowMenu {
                is_app_wide: true,
                menu: menu.clone(),
              });
            }
          }
        }

        // set it app-wide for macos
        #[cfg(target_os = "macos")]
        {
          let menu_ = menu.clone();
          self.run_on_main_thread(move || {
            let _ = init_app_menu(&menu_);
          })?;
        }

        Ok(prev_menu)
      }

      /// Remove the app-wide menu and returns it.
      ///
      /// If a window was not created with an explicit menu or had one set explicitly,
      /// this will remove the menu from it.
      #[cfg(desktop)]
      pub fn remove_menu(&self) -> crate::Result<Option<Menu<R>>> {
        let menu = self.manager.menu_lock().as_ref().cloned();
        #[allow(unused_variables)]
        if let Some(menu) = menu {
          // remove from windows that have the app-wide menu
          #[cfg(not(target_os = "macos"))]
          {
            for window in self.manager.windows().values() {
              let has_app_wide_menu = window.has_app_wide_menu();
              if has_app_wide_menu {
                window.remove_menu()?;
                *window.menu_lock() = None;
              }
            }
          }

          // remove app-wide for macos
          #[cfg(target_os = "macos")]
          {
            self.run_on_main_thread(move || {
              menu.inner().remove_for_nsapp();
            })?;
          }
        }

        let prev_menu = self.manager.menu_lock().take();

        self
          .manager
          .remove_menu_from_stash_by_id(prev_menu.as_ref().map(|m| m.id()));

        Ok(prev_menu)
      }

      /// Hides the app-wide menu from windows that have it.
      ///
      /// If a window was not created with an explicit menu or had one set explicitly,
      /// this will hide the menu from it.
      #[cfg(desktop)]
      pub fn hide_menu(&self) -> crate::Result<()> {
        #[cfg(not(target_os = "macos"))]
        {
          let is_app_menu_set = self.manager.menu_lock().is_some();
          if is_app_menu_set {
            for window in self.manager.windows().values() {
              if window.has_app_wide_menu() {
                window.hide_menu()?;
              }
            }
          }
        }

        Ok(())
      }

      /// Shows the app-wide menu for windows that have it.
      ///
      /// If a window was not created with an explicit menu or had one set explicitly,
      /// this will show the menu for it.
      #[cfg(desktop)]
      pub fn show_menu(&self) -> crate::Result<()> {
        #[cfg(not(target_os = "macos"))]
        {
          let is_app_menu_set = self.manager.menu_lock().is_some();
          if is_app_menu_set {
            for window in self.manager.windows().values() {
              if window.has_app_wide_menu() {
                window.show_menu()?;
              }
            }
          }
        }

        Ok(())
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

      /// Runs necessary cleanup tasks before exiting the process.
      /// **You should always exit the tauri app immediately after this function returns and not use any tauri-related APIs.**
      pub fn cleanup_before_exit(&self) {
        #[cfg(all(desktop, feature = "tray-icon"))]
        self.manager.inner.tray_icons.lock().unwrap().clear()
      }
    }
  };
}

shared_app_impl!(App<R>);
shared_app_impl!(AppHandle<R>);

impl<R: Runtime> App<R> {
  fn register_core_plugins(&self) -> crate::Result<()> {
    self.handle.plugin(crate::path::plugin::init())?;
    self.handle.plugin(crate::event::plugin::init())?;
    self.handle.plugin(crate::window::plugin::init())?;
    self.handle.plugin(crate::app::plugin::init())?;
    Ok(())
  }

  /// Runs the given closure on the main thread.
  pub fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> crate::Result<()> {
    self.app_handle().run_on_main_thread(f)
  }

  /// Gets a handle to the application instance.
  pub fn handle(&self) -> &AppHandle<R> {
    &self.handle
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
    let app_handle = self.handle().clone();
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
  /// The cleanup calls [`App::cleanup_before_exit`] so you may want to call that function before exiting the application.
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
  ///     app.cleanup_before_exit();
  ///     break;
  ///   }
  /// }
  /// ```
  #[cfg(desktop)]
  pub fn run_iteration(&mut self) -> crate::runtime::RunIteration {
    let manager = self.manager.clone();
    let app_handle = self.handle().clone();
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
  invoke_responder: Option<Arc<InvokeResponder<R>>>,

  /// The script that initializes the `window.__TAURI_INTERNALS__.postMessage` function.
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
  uri_scheme_protocols: HashMap<String, Arc<UriSchemeProtocol<R>>>,

  /// App state.
  state: StateManager,

  /// A closure that returns the menu set to all windows.
  #[cfg(desktop)]
  menu: Option<Box<dyn FnOnce(&AppHandle<R>) -> crate::Result<Menu<R>> + Send>>,

  /// Enable macOS default menu creation.
  #[allow(unused)]
  enable_macos_default_menu: bool,

  /// Window event handlers that listens to all windows.
  window_event_listeners: Vec<GlobalWindowEventListener<R>>,

  /// The device event filter.
  device_event_filter: DeviceEventFilter,
}

#[derive(Template)]
#[default_template("../scripts/ipc-protocol.js")]
struct InvokeInitializationScript<'a> {
  /// The function that processes the IPC message.
  #[raw]
  process_ipc_message_fn: &'a str,
  os_name: &'a str,
  fetch_channel_data_command: &'a str,
  use_custom_protocol: bool,
}

impl<R: Runtime> Builder<R> {
  /// Creates a new App builder.
  pub fn new() -> Self {
    Self {
      #[cfg(any(windows, target_os = "linux"))]
      runtime_any_thread: false,
      setup: Box::new(|_| Ok(())),
      invoke_handler: Box::new(|_| false),
      invoke_responder: None,
      invoke_initialization_script: InvokeInitializationScript {
        process_ipc_message_fn: crate::manager::PROCESS_IPC_MESSAGE_FN,
        os_name: std::env::consts::OS,
        fetch_channel_data_command: crate::ipc::channel::FETCH_CHANNEL_DATA_COMMAND,
        use_custom_protocol: cfg!(ipc_custom_protocol),
      }
      .render_default(&Default::default())
      .unwrap()
      .into_string(),
      on_page_load: Box::new(|_, _| ()),
      pending_windows: Default::default(),
      plugins: PluginStore::default(),
      uri_scheme_protocols: Default::default(),
      state: StateManager::new(),
      #[cfg(desktop)]
      menu: None,
      enable_macos_default_menu: true,
      window_event_listeners: Vec::new(),
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
  /// The `initialization_script` is a script that initializes `window.__TAURI_INTERNALS__.postMessage`.
  /// That function must take the `(message: object, options: object)` arguments and send it to the backend.
  #[must_use]
  pub fn invoke_system<F>(mut self, initialization_script: String, responder: F) -> Self
  where
    F: Fn(&Window<R>, &str, &InvokeResponse, CallbackFn, CallbackFn) + Send + Sync + 'static,
  {
    self.invoke_initialization_script = initialization_script;
    self.invoke_responder.replace(Arc::new(responder));
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
  /// Additionally, [`state`](crate::Manager#method.state) can be used to retrieve the value manually.
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

  /// Sets the menu to use on all windows.
  ///
  /// # Examples
  /// ```
  /// use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
  ///
  /// tauri::Builder::default()
  ///   .menu(|handle| Menu::with_items(handle, &[
  ///     &Submenu::with_items(
  ///       handle,
  ///       "File",
  ///       true,
  ///       &[
  ///         &PredefinedMenuItem::close_window(handle, None),
  ///         #[cfg(target_os = "macos")]
  ///         &MenuItem::new(handle, "Hello", true, None),
  ///       ],
  ///     )?
  ///   ]));
  /// ```
  #[must_use]
  #[cfg(desktop)]
  pub fn menu<F: FnOnce(&AppHandle<R>) -> crate::Result<Menu<R>> + Send + 'static>(
    mut self,
    f: F,
  ) -> Self {
    self.menu.replace(Box::new(f));
    self
  }

  /// Enable or disable the default menu on macOS. Enabled by default.
  ///
  /// # Examples
  /// ```
  /// tauri::Builder::default()
  ///   .enable_macos_default_menu(false);
  /// ```
  #[must_use]
  pub fn enable_macos_default_menu(mut self, enable: bool) -> Self {
    self.enable_macos_default_menu = enable;
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

  /// Registers a URI scheme protocol available to all webviews.
  /// Leverages [setURLSchemeHandler](https://developer.apple.com/documentation/webkit/wkwebviewconfiguration/2875766-seturlschemehandler) on macOS,
  /// [AddWebResourceRequestedFilter](https://docs.microsoft.com/en-us/dotnet/api/microsoft.web.webview2.core.corewebview2.addwebresourcerequestedfilter?view=webview2-dotnet-1.0.774.44) on Windows
  /// and [webkit-web-context-register-uri-scheme](https://webkitgtk.org/reference/webkit2gtk/stable/WebKitWebContext.html#webkit-web-context-register-uri-scheme) on Linux.
  ///
  /// # Arguments
  ///
  /// * `uri_scheme` The URI scheme to register, such as `example`.
  /// * `protocol` the protocol associated with the given URI scheme. It's a function that takes a request and returns a response.
  ///
  /// # Examples
  /// ```
  /// tauri::Builder::default()
  ///   .register_uri_scheme_protocol("app-files", |_app, request| {
  ///     // skip leading `/`
  ///     if let Ok(data) = std::fs::read(&request.uri().path()[1..]) {
  ///       http::Response::builder()
  ///         .body(data)
  ///         .unwrap()
  ///     } else {
  ///       http::Response::builder()
  ///         .status(http::StatusCode::BAD_REQUEST)
  ///         .header(http::header::CONTENT_TYPE, mime::TEXT_PLAIN.essence_str())
  ///         .body("failed to read file".as_bytes().to_vec())
  ///         .unwrap()
  ///     }
  ///   });
  /// ```
  #[must_use]
  pub fn register_uri_scheme_protocol<
    N: Into<String>,
    T: Into<Cow<'static, [u8]>>,
    H: Fn(&AppHandle<R>, http::Request<Vec<u8>>) -> http::Response<T> + Send + Sync + 'static,
  >(
    mut self,
    uri_scheme: N,
    protocol: H,
  ) -> Self {
    self.uri_scheme_protocols.insert(
      uri_scheme.into(),
      Arc::new(UriSchemeProtocol {
        protocol: Box::new(move |app, request, responder| {
          responder.respond(protocol(app, request))
        }),
      }),
    );
    self
  }

  /// Similar to [`Self::register_uri_scheme_protocol`] but with an asynchronous responder that allows you
  /// to process the request in a separate thread and respond asynchronously.
  ///
  /// # Arguments
  ///
  /// * `uri_scheme` The URI scheme to register, such as `example`.
  /// * `protocol` the protocol associated with the given URI scheme. It's a function that takes an URL such as `example://localhost/asset.css`.
  ///
  /// # Examples
  /// ```
  /// tauri::Builder::default()
  ///   .register_asynchronous_uri_scheme_protocol("app-files", |_app, request, responder| {
  ///     // skip leading `/`
  ///     let path = request.uri().path()[1..].to_string();
  ///     std::thread::spawn(move || {
  ///       if let Ok(data) = std::fs::read(path) {
  ///         responder.respond(
  ///           http::Response::builder()
  ///             .body(data)
  ///             .unwrap()
  ///         );
  ///       } else {
  ///         responder.respond(
  ///           http::Response::builder()
  ///             .status(http::StatusCode::BAD_REQUEST)
  ///             .header(http::header::CONTENT_TYPE, mime::TEXT_PLAIN.essence_str())
  ///             .body("failed to read file".as_bytes().to_vec())
  ///             .unwrap()
  ///         );
  ///     }
  ///   });
  ///   });
  /// ```
  #[must_use]
  pub fn register_asynchronous_uri_scheme_protocol<
    N: Into<String>,
    H: Fn(&AppHandle<R>, http::Request<Vec<u8>>, UriSchemeResponder) + Send + Sync + 'static,
  >(
    mut self,
    uri_scheme: N,
    protocol: H,
  ) -> Self {
    self.uri_scheme_protocols.insert(
      uri_scheme.into(),
      Arc::new(UriSchemeProtocol {
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
      self.menu = Some(Box::new(|app_handle| {
        crate::menu::Menu::default(app_handle)
      }));
    }

    let manager = WindowManager::with_handlers(
      context,
      self.plugins,
      self.invoke_handler,
      self.on_page_load,
      self.uri_scheme_protocols,
      self.state,
      self.window_event_listeners,
      #[cfg(desktop)]
      HashMap::new(),
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

    let runtime_args = RuntimeInitArgs {
      #[cfg(windows)]
      msg_hook: {
        let menus = manager.inner.menus.clone();
        Some(Box::new(move |msg| {
          use windows::Win32::UI::WindowsAndMessaging::{TranslateAcceleratorW, HACCEL, MSG};
          unsafe {
            let msg = msg as *const MSG;
            for menu in menus.lock().unwrap().values() {
              let translated =
                TranslateAcceleratorW((*msg).hwnd, HACCEL(menu.inner().haccel()), msg);
              if translated == 1 {
                return true;
              }
            }

            false
          }
        }))
      },
    };

    #[cfg(any(windows, target_os = "linux"))]
    let mut runtime = if self.runtime_any_thread {
      R::new_any_thread(runtime_args)?
    } else {
      R::new(runtime_args)?
    };
    #[cfg(not(any(windows, target_os = "linux")))]
    let mut runtime = R::new(runtime_args)?;

    #[cfg(desktop)]
    {
      // setup menu event handler
      let proxy = runtime.create_proxy();
      muda::MenuEvent::set_event_handler(Some(move |e: muda::MenuEvent| {
        let _ = proxy.send_event(EventLoopMessage::MenuEvent(e.into()));
      }));

      // setup tray event handler
      #[cfg(feature = "tray-icon")]
      {
        let proxy = runtime.create_proxy();
        tray_icon::TrayIconEvent::set_event_handler(Some(move |e: tray_icon::TrayIconEvent| {
          let _ = proxy.send_event(EventLoopMessage::TrayIconEvent(e.into()));
        }));
      }
    }

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

    #[cfg(desktop)]
    if let Some(menu) = self.menu {
      let menu = menu(&app.handle)?;
      app
        .manager
        .menus_stash_lock()
        .insert(menu.id().clone(), menu.clone());

      #[cfg(target_os = "macos")]
      init_app_menu(&menu)?;

      app.manager.menu_lock().replace(menu);
    }

    app.register_core_plugins()?;

    let env = Env::default();
    app.manage(env);

    app.manage(Scopes {
      ipc: scope::ipc::Scope::new(&app.config()),
      #[cfg(feature = "protocol-asset")]
      asset_protocol: scope::fs::Scope::for_fs_api(
        &app,
        &app.config().tauri.security.asset_protocol.scope,
      )?,
    });

    app.manage(ChannelDataIpcQueue::default());
    app.handle.plugin(crate::ipc::channel::plugin())?;

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

    let handle = app.handle();

    // initialize default tray icon if defined
    #[cfg(all(desktop, feature = "tray-icon"))]
    {
      let config = app.config();
      if let Some(tray_config) = &config.tauri.tray_icon {
        let mut tray =
          TrayIconBuilder::with_id(tray_config.id.clone().unwrap_or_else(|| "main".into()))
            .icon_as_template(tray_config.icon_as_template)
            .menu_on_left_click(tray_config.menu_on_left_click);
        if let Some(icon) = &app.manager.inner.tray_icon {
          tray = tray.icon(icon.clone());
        }
        if let Some(title) = &tray_config.title {
          tray = tray.title(title);
        }
        if let Some(tooltip) = &tray_config.tooltip {
          tray = tray.tooltip(tooltip);
        }
        let tray = tray.build(handle)?;
        app.manager.inner.tray_icons.lock().unwrap().push(tray);
      }
    }

    app.manager.initialize_plugins(handle)?;

    Ok(app)
  }

  /// Runs the configured Tauri application.
  pub fn run<A: Assets>(self, context: Context<A>) -> crate::Result<()> {
    self.build(context)?.run(|_, _| {});
    Ok(())
  }
}

pub(crate) type UriSchemeResponderFn = Box<dyn FnOnce(http::Response<Cow<'static, [u8]>>) + Send>;
pub struct UriSchemeResponder(pub(crate) UriSchemeResponderFn);

impl UriSchemeResponder {
  /// Resolves the request with the given response.
  pub fn respond<T: Into<Cow<'static, [u8]>>>(self, response: http::Response<T>) {
    let (parts, body) = response.into_parts();
    (self.0)(http::Response::from_parts(parts, body.into()))
  }
}

#[cfg(target_os = "macos")]
fn init_app_menu<R: Runtime>(menu: &Menu<R>) -> crate::Result<()> {
  menu.inner().init_for_nsapp();

  if let Some(window_menu) = menu.get(crate::menu::WINDOW_SUBMENU_ID) {
    if let Some(m) = window_menu.as_submenu() {
      m.set_as_windows_menu_for_nsapp()?;
    }
  }
  if let Some(help_menu) = menu.get(crate::menu::HELP_SUBMENU_ID) {
    if let Some(m) = help_menu.as_submenu() {
      m.set_as_help_menu_for_nsapp()?;
    }
  }

  Ok(())
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

    let app_handle = app.handle();
    let manager = app.manager();

    for pending in pending_windows {
      let pending = manager.prepare_window(app_handle.clone(), pending, &window_labels)?;

      #[cfg(desktop)]
      let window_menu = app.manager.menu_lock().as_ref().map(|m| WindowMenu {
        is_app_wide: true,
        menu: m.clone(),
      });

      #[cfg(desktop)]
      let handler = manager.prepare_window_menu_creation_handler(window_menu.as_ref());
      #[cfg(not(desktop))]
      #[allow(clippy::type_complexity)]
      let handler: Option<Box<dyn Fn(tauri_runtime::window::RawWindow<'_>) + Send>> = None;

      let window_effects = pending.webview_attributes.window_effects.clone();
      let detached = if let RuntimeOrDispatch::RuntimeHandle(runtime) = app_handle.runtime() {
        runtime.create_window(pending, handler)?
      } else {
        // the AppHandle's runtime is always RuntimeOrDispatch::RuntimeHandle
        unreachable!()
      };
      let window = manager.attach_window(
        app_handle.clone(),
        detached,
        #[cfg(desktop)]
        None,
      );

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
    RuntimeRunEvent::UserEvent(t) => {
      match t {
        #[cfg(desktop)]
        EventLoopMessage::MenuEvent(ref e) => {
          for listener in &*app_handle
            .manager
            .inner
            .menu_event_listeners
            .lock()
            .unwrap()
          {
            listener(app_handle, e.clone());
          }
          for (label, listener) in &*app_handle
            .manager
            .inner
            .window_menu_event_listeners
            .lock()
            .unwrap()
          {
            if let Some(w) = app_handle.get_window(label) {
              listener(&w, e.clone());
            }
          }
        }
        #[cfg(all(desktop, feature = "tray-icon"))]
        EventLoopMessage::TrayIconEvent(ref e) => {
          for listener in &*app_handle
            .manager
            .inner
            .global_tray_event_listeners
            .lock()
            .unwrap()
          {
            listener(app_handle, e.clone());
          }

          for (id, listener) in &*app_handle
            .manager
            .inner
            .tray_event_listeners
            .lock()
            .unwrap()
          {
            if e.id == id {
              if let Some(tray) = app_handle.tray_by_id(id) {
                listener(&tray, e.clone());
              }
            }
          }
        }
      }

      #[allow(unreachable_code)]
      t.into()
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    RuntimeRunEvent::Opened { urls } => RunEvent::Opened { urls },
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
