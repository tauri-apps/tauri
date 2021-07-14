// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "menu")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
pub(crate) mod menu;

#[cfg(feature = "menu")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
pub use menu::{MenuEvent, MenuHandle};

use crate::{
  api::config::WindowUrl,
  app::AppHandle,
  command::{CommandArg, CommandItem},
  event::{Event, EventHandler},
  manager::WindowManager,
  runtime::{
    monitor::Monitor as RuntimeMonitor,
    webview::{InvokePayload, WebviewAttributes, WindowBuilder},
    window::{
      dpi::{PhysicalPosition, PhysicalSize, Position, Size},
      DetachedWindow, PendingWindow, WindowEvent,
    },
    Dispatch, Icon, Runtime, UserAttentionType,
  },
  sealed::ManagerBase,
  sealed::RuntimeOrDispatch,
  Invoke, InvokeError, InvokeMessage, InvokeResolver, Manager, PageLoadPayload,
};

use serde::Serialize;

use tauri_macros::default_runtime;

use std::hash::{Hash, Hasher};

/// Monitor descriptor.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
  pub(crate) name: Option<String>,
  pub(crate) size: PhysicalSize<u32>,
  pub(crate) position: PhysicalPosition<i32>,
  pub(crate) scale_factor: f64,
}

impl From<RuntimeMonitor> for Monitor {
  fn from(monitor: RuntimeMonitor) -> Self {
    Self {
      name: monitor.name,
      size: monitor.size,
      position: monitor.position,
      scale_factor: monitor.scale_factor,
    }
  }
}

impl Monitor {
  /// Returns a human-readable name of the monitor.
  /// Returns None if the monitor doesn't exist anymore.
  pub fn name(&self) -> Option<&String> {
    self.name.as_ref()
  }

  /// Returns the monitor's resolution.
  pub fn size(&self) -> &PhysicalSize<u32> {
    &self.size
  }

  /// Returns the top-left corner position of the monitor relative to the larger full screen area.
  pub fn position(&self) -> &PhysicalPosition<i32> {
    &self.position
  }

  /// Returns the scale factor that can be used to map logical pixels to physical pixels, and vice versa.
  pub fn scale_factor(&self) -> f64 {
    self.scale_factor
  }
}

// TODO: expand these docs since this is a pretty important type
/// A webview window managed by Tauri.
///
/// This type also implements [`Manager`] which allows you to manage other windows attached to
/// the same application.
#[default_runtime(crate::Wry, wry)]
pub struct Window<R: Runtime> {
  /// The webview window created by the runtime.
  window: DetachedWindow<R>,
  /// The manager to associate this webview window with.
  manager: WindowManager<R>,
  pub(crate) app_handle: AppHandle<R>,
}

impl<R: Runtime> Clone for Window<R> {
  fn clone(&self) -> Self {
    Self {
      window: self.window.clone(),
      manager: self.manager.clone(),
      app_handle: self.app_handle.clone(),
    }
  }
}

impl<R: Runtime> Hash for Window<R> {
  /// Only use the [`Window`]'s label to represent its hash.
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.window.label.hash(state)
  }
}

impl<R: Runtime> Eq for Window<R> {}
impl<R: Runtime> PartialEq for Window<R> {
  /// Only use the [`Window`]'s label to compare equality.
  fn eq(&self, other: &Self) -> bool {
    self.window.label.eq(&other.window.label)
  }
}

impl<R: Runtime> Manager<R> for Window<R> {}
impl<R: Runtime> ManagerBase<R> for Window<R> {
  fn manager(&self) -> &WindowManager<R> {
    &self.manager
  }

  fn runtime(&self) -> RuntimeOrDispatch<'_, R> {
    RuntimeOrDispatch::Dispatch(self.dispatcher())
  }

  fn app_handle(&self) -> AppHandle<R> {
    self.app_handle.clone()
  }
}

impl<'de, R: Runtime> CommandArg<'de, R> for Window<R> {
  /// Grabs the [`Window`] from the [`CommandItem`]. This will never fail.
  fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
    Ok(command.message.window())
  }
}

impl<R: Runtime> Window<R> {
  /// Create a new window that is attached to the manager.
  pub(crate) fn new(
    manager: WindowManager<R>,
    window: DetachedWindow<R>,
    app_handle: AppHandle<R>,
  ) -> Self {
    Self {
      window,
      manager,
      app_handle,
    }
  }

  /// Creates a new webview window.
  pub fn create_window<F>(
    &mut self,
    label: String,
    url: WindowUrl,
    setup: F,
  ) -> crate::Result<Window<R>>
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
    ))
  }

  /// The current window's dispatcher.
  pub(crate) fn dispatcher(&self) -> R::Dispatcher {
    self.window.dispatcher.clone()
  }

  #[allow(dead_code)]
  pub(crate) fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .run_on_main_thread(f)
      .map_err(Into::into)
  }

  /// How to handle this window receiving an [`InvokeMessage`].
  pub(crate) fn on_message(self, command: String, payload: InvokePayload) -> crate::Result<()> {
    let manager = self.manager.clone();
    match command.as_str() {
      "__initialized" => {
        let payload: PageLoadPayload = serde_json::from_value(payload.inner)?;
        manager.run_on_page_load(self, payload);
      }
      _ => {
        let message = InvokeMessage::new(
          self.clone(),
          manager.state(),
          command.to_string(),
          payload.inner,
        );
        let resolver = InvokeResolver::new(self, payload.callback, payload.error);
        let invoke = Invoke { message, resolver };
        if manager.verify_invoke_key(payload.key) {
          if let Some(module) = &payload.tauri_module {
            let module = module.to_string();
            crate::endpoints::handle(module, invoke, manager.config(), manager.package_info());
          } else if command.starts_with("plugin:") {
            manager.extend_api(invoke);
          } else {
            manager.run_invoke_handler(invoke);
          }
        } else {
          panic!(
            r#"The invoke key "{}" is invalid. This means that an external, possible malicious script is trying to access the system interface."#,
            payload.key
          );
        }
      }
    }

    Ok(())
  }

  /// The label of this window.
  pub fn label(&self) -> &str {
    &self.window.label
  }

  /// Emits an event to the current window.
  pub fn emit<S: Serialize>(&self, event: &str, payload: S) -> crate::Result<()> {
    self.eval(&format!(
      "window['{}']({{event: {}, payload: {}}}, '{}')",
      self.manager.event_emit_function_name(),
      serde_json::to_string(event)?,
      serde_json::to_value(payload)?,
      self.manager.generate_salt(),
    ))?;

    Ok(())
  }

  /// Emits an event on all windows except this one.
  pub fn emit_others<S: Serialize + Clone>(&self, event: &str, payload: S) -> crate::Result<()> {
    self.manager.emit_filter(event, payload, |w| w != self)
  }

  /// Listen to an event on this window.
  pub fn listen<F>(&self, event: impl Into<String>, handler: F) -> EventHandler
  where
    F: Fn(Event) + Send + 'static,
  {
    let label = self.window.label.clone();
    self.manager.listen(event.into(), Some(label), handler)
  }

  /// Listen to a an event on this window a single time.
  pub fn once<F>(&self, event: impl Into<String>, handler: F) -> EventHandler
  where
    F: Fn(Event) + Send + 'static,
  {
    let label = self.window.label.clone();
    self.manager.once(event.into(), Some(label), handler)
  }

  /// Triggers an event on this window.
  pub fn trigger(&self, event: &str, data: Option<String>) {
    let label = self.window.label.clone();
    self.manager.trigger(event, Some(label), data)
  }

  /// Evaluates JavaScript on this window.
  pub fn eval(&self, js: &str) -> crate::Result<()> {
    self.window.dispatcher.eval_script(js).map_err(Into::into)
  }

  /// Registers a window event listener.
  pub fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) {
    self.window.dispatcher.on_window_event(f);
  }

  /// Registers a menu event listener.
  #[cfg(feature = "menu")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
  pub fn on_menu_event<F: Fn(MenuEvent) + Send + 'static>(&self, f: F) -> uuid::Uuid {
    let menu_ids = self.manager.menu_ids();
    self.window.dispatcher.on_menu_event(move |event| {
      f(MenuEvent {
        menu_item_id: menu_ids.get(&event.menu_item_id).unwrap().clone(),
      })
    })
  }

  // Getters

  /// Gets a handle to the window menu.
  #[cfg(feature = "menu")]
  pub fn menu_handle(&self) -> MenuHandle<R> {
    MenuHandle {
      ids: self.manager.menu_ids(),
      dispatcher: self.dispatcher(),
    }
  }

  /// Returns the scale factor that can be used to map logical pixels to physical pixels, and vice versa.
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn scale_factor(&self) -> crate::Result<f64> {
    self.window.dispatcher.scale_factor().map_err(Into::into)
  }

  /// Returns the position of the top-left hand corner of the window's client area relative to the top-left hand corner of the desktop.
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn inner_position(&self) -> crate::Result<PhysicalPosition<i32>> {
    self.window.dispatcher.inner_position().map_err(Into::into)
  }

  /// Returns the position of the top-left hand corner of the window relative to the top-left hand corner of the desktop.
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn outer_position(&self) -> crate::Result<PhysicalPosition<i32>> {
    self.window.dispatcher.outer_position().map_err(Into::into)
  }

  /// Returns the physical size of the window's client area.
  ///
  /// The client area is the content of the window, excluding the title bar and borders.
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn inner_size(&self) -> crate::Result<PhysicalSize<u32>> {
    self.window.dispatcher.inner_size().map_err(Into::into)
  }

  /// Returns the physical size of the entire window.
  ///
  /// These dimensions include the title bar and borders. If you don't want that (and you usually don't), use inner_size instead.
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn outer_size(&self) -> crate::Result<PhysicalSize<u32>> {
    self.window.dispatcher.outer_size().map_err(Into::into)
  }

  /// Gets the window's current fullscreen state.
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn is_fullscreen(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_fullscreen().map_err(Into::into)
  }

  /// Gets the window's current maximized state.
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn is_maximized(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_maximized().map_err(Into::into)
  }

  /// Gets the window’s current decoration state.
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn is_decorated(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_decorated().map_err(Into::into)
  }

  /// Gets the window’s current resizable state.
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn is_resizable(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_resizable().map_err(Into::into)
  }

  /// Gets the window's current vibility state.
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn is_visible(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_visible().map_err(Into::into)
  }

  /// Returns the monitor on which the window currently resides.
  ///
  /// Returns None if current monitor can't be detected.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Unsupported
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn current_monitor(&self) -> crate::Result<Option<Monitor>> {
    self
      .window
      .dispatcher
      .current_monitor()
      .map(|m| m.map(Into::into))
      .map_err(Into::into)
  }

  /// Returns the primary monitor of the system.
  ///
  /// Returns None if it can't identify any monitor as a primary one.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Unsupported
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn primary_monitor(&self) -> crate::Result<Option<Monitor>> {
    self
      .window
      .dispatcher
      .primary_monitor()
      .map(|m| m.map(Into::into))
      .map_err(Into::into)
  }

  /// Returns the list of all the monitors available on the system.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Unsupported
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn available_monitors(&self) -> crate::Result<Vec<Monitor>> {
    self
      .window
      .dispatcher
      .available_monitors()
      .map(|m| m.into_iter().map(Into::into).collect())
      .map_err(Into::into)
  }

  /// Returns the native handle that is used by this window.
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  #[cfg(windows)]
  pub fn hwnd(&self) -> crate::Result<*mut std::ffi::c_void> {
    self
      .window
      .dispatcher
      .hwnd()
      .map(|hwnd| hwnd as *mut _)
      .map_err(Into::into)
  }

  /// Returns the `ApplicatonWindow` from gtk crate that is used by this window.
  ///
  /// Note that this can only be used on the main thread.
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  pub fn gtk_window(&self) -> crate::Result<gtk::ApplicationWindow> {
    self.window.dispatcher.gtk_window().map_err(Into::into)
  }

  // Setters

  /// Centers the window.
  pub fn center(&self) -> crate::Result<()> {
    self.window.dispatcher.center().map_err(Into::into)
  }

  /// Requests user attention to the window, this has no effect if the application
  /// is already focused. How requesting for user attention manifests is platform dependent,
  /// see `UserAttentionType` for details.
  ///
  /// Providing `None` will unset the request for user attention. Unsetting the request for
  /// user attention might not be done automatically by the WM when the window receives input.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** `None` has no effect.
  pub fn request_user_attention(
    &self,
    request_type: Option<UserAttentionType>,
  ) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .request_user_attention(request_type)
      .map_err(Into::into)
  }

  /// Opens the dialog to prints the contents of the webview.
  /// Currently only supported on macOS on `wry`.
  /// `window.print()` works on all platforms.
  pub fn print(&self) -> crate::Result<()> {
    self.window.dispatcher.print().map_err(Into::into)
  }

  /// Determines if this window should be resizable.
  pub fn set_resizable(&self, resizable: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_resizable(resizable)
      .map_err(Into::into)
  }

  /// Set this window's title.
  pub fn set_title(&self, title: &str) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_title(title.to_string())
      .map_err(Into::into)
  }

  /// Maximizes this window.
  pub fn maximize(&self) -> crate::Result<()> {
    self.window.dispatcher.maximize().map_err(Into::into)
  }

  /// Un-maximizes this window.
  pub fn unmaximize(&self) -> crate::Result<()> {
    self.window.dispatcher.unmaximize().map_err(Into::into)
  }

  /// Minimizes this window.
  pub fn minimize(&self) -> crate::Result<()> {
    self.window.dispatcher.minimize().map_err(Into::into)
  }

  /// Un-minimizes this window.
  pub fn unminimize(&self) -> crate::Result<()> {
    self.window.dispatcher.unminimize().map_err(Into::into)
  }

  /// Show this window.
  pub fn show(&self) -> crate::Result<()> {
    self.window.dispatcher.show().map_err(Into::into)
  }

  /// Hide this window.
  pub fn hide(&self) -> crate::Result<()> {
    self.window.dispatcher.hide().map_err(Into::into)
  }

  /// Closes this window.
  pub fn close(&self) -> crate::Result<()> {
    self.window.dispatcher.close().map_err(Into::into)
  }

  /// Determines if this window should be [decorated].
  ///
  /// [decorated]: https://en.wikipedia.org/wiki/Window_(computing)#Window_decoration
  pub fn set_decorations(&self, decorations: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_decorations(decorations)
      .map_err(Into::into)
  }

  /// Determines if this window should always be on top of other windows.
  pub fn set_always_on_top(&self, always_on_top: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_always_on_top(always_on_top)
      .map_err(Into::into)
  }

  /// Resizes this window.
  pub fn set_size<S: Into<Size>>(&self, size: S) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_size(size.into())
      .map_err(Into::into)
  }

  /// Sets this window's minimum size.
  pub fn set_min_size<S: Into<Size>>(&self, size: Option<S>) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_min_size(size.map(|s| s.into()))
      .map_err(Into::into)
  }

  /// Sets this window's maximum size.
  pub fn set_max_size<S: Into<Size>>(&self, size: Option<S>) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_max_size(size.map(|s| s.into()))
      .map_err(Into::into)
  }

  /// Sets this window's position.
  pub fn set_position<Pos: Into<Position>>(&self, position: Pos) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_position(position.into())
      .map_err(Into::into)
  }

  /// Determines if this window should be fullscreen.
  pub fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_fullscreen(fullscreen)
      .map_err(Into::into)
  }

  /// Bring the window to front and focus.
  pub fn set_focus(&self) -> crate::Result<()> {
    self.window.dispatcher.set_focus().map_err(Into::into)
  }

  /// Sets this window' icon.
  pub fn set_icon(&self, icon: Icon) -> crate::Result<()> {
    self.window.dispatcher.set_icon(icon).map_err(Into::into)
  }

  /// Whether to show the window icon in the task bar or not.
  pub fn set_skip_taskbar(&self, skip: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_skip_taskbar(skip)
      .map_err(Into::into)
  }

  /// Starts dragging the window.
  pub fn start_dragging(&self) -> crate::Result<()> {
    self.window.dispatcher.start_dragging().map_err(Into::into)
  }

  pub(crate) fn verify_salt(&self, salt: String) -> bool {
    self.manager.verify_salt(salt)
  }
}
