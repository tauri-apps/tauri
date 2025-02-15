// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri window types and functions.

pub(crate) mod plugin;

use tauri_runtime::{
  dpi::{PhysicalPosition, PhysicalSize},
  webview::PendingWebview,
};
pub use tauri_utils::{config::Color, WindowEffect as Effect, WindowEffectState as EffectState};

#[cfg(desktop)]
pub use crate::runtime::ProgressBarStatus;

use crate::{
  app::AppHandle,
  event::{Event, EventId, EventTarget},
  ipc::{CommandArg, CommandItem, InvokeError},
  manager::{AppManager, EmitPayload},
  runtime::{
    monitor::Monitor as RuntimeMonitor,
    window::{DetachedWindow, PendingWindow, WindowBuilder as _},
    RuntimeHandle, WindowDispatch,
  },
  sealed::{ManagerBase, RuntimeOrDispatch},
  utils::config::{WindowConfig, WindowEffectsConfig},
  webview::WebviewBuilder,
  Emitter, EventLoopMessage, EventName, Listener, Manager, ResourceTable, Runtime, Theme, Webview,
  WindowEvent,
};
#[cfg(desktop)]
use crate::{
  image::Image,
  menu::{ContextMenu, Menu, MenuId},
  runtime::{
    dpi::{Position, Size},
    UserAttentionType,
  },
  CursorIcon,
};

use serde::Serialize;
#[cfg(windows)]
use windows::Win32::Foundation::HWND;

use tauri_macros::default_runtime;

use std::{
  fmt,
  hash::{Hash, Hasher},
  sync::{Arc, Mutex, MutexGuard},
};

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

macro_rules! unstable_struct {
    (#[doc = $doc:expr] $($tokens:tt)*) => {
      #[cfg(feature = "unstable")]
      #[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
      #[doc = $doc]
      pub $($tokens)*

      #[cfg(not(feature = "unstable"))]
      pub(crate) $($tokens)*
    }
}

unstable_struct!(
  #[doc = "A builder for a window managed by Tauri."]
  struct WindowBuilder<'a, R: Runtime, M: Manager<R>> {
    manager: &'a M,
    pub(crate) label: String,
    pub(crate) window_builder:
      <R::WindowDispatcher as WindowDispatch<EventLoopMessage>>::WindowBuilder,
    #[cfg(desktop)]
    pub(crate) menu: Option<Menu<R>>,
    #[cfg(desktop)]
    on_menu_event: Option<crate::app::GlobalMenuEventListener<Window<R>>>,
    window_effects: Option<WindowEffectsConfig>,
  }
);

impl<R: Runtime, M: Manager<R>> fmt::Debug for WindowBuilder<'_, R, M> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("WindowBuilder")
      .field("label", &self.label)
      .field("window_builder", &self.window_builder)
      .finish()
  }
}

#[cfg_attr(not(feature = "unstable"), allow(dead_code))]
impl<'a, R: Runtime, M: Manager<R>> WindowBuilder<'a, R, M> {
  /// Initializes a window builder with the given window label.
  ///
  /// # Known issues
  ///
  /// On Windows, this function deadlocks when used in a synchronous command, see [the Webview2 issue].
  /// You should use `async` commands when creating windows.
  ///
  /// # Examples
  ///
  /// - Create a window in the setup hook:
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
tauri::Builder::default()
  .setup(|app| {
    let window = tauri::window::WindowBuilder::new(app, "label")
      .build()?;
    Ok(())
  });
```
  "####
  )]
  /// - Create a window in a separate thread:
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
tauri::Builder::default()
  .setup(|app| {
    let handle = app.handle().clone();
    std::thread::spawn(move || {
      let window = tauri::window::WindowBuilder::new(&handle, "label")
        .build()
        .unwrap();
    });
    Ok(())
  });
```
  "####
  )]
  ///
  /// - Create a window in a command:
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
#[tauri::command]
async fn create_window(app: tauri::AppHandle) {
  let window = tauri::window::WindowBuilder::new(&app, "label")
    .build()
    .unwrap();
}
```
  "####
  )]
  ///
  /// [the Webview2 issue]: https://github.com/tauri-apps/wry/issues/583
  pub fn new<L: Into<String>>(manager: &'a M, label: L) -> Self {
    Self {
      manager,
      label: label.into(),
      window_builder: <R::WindowDispatcher as WindowDispatch<EventLoopMessage>>::WindowBuilder::new(
      ),
      #[cfg(desktop)]
      menu: None,
      #[cfg(desktop)]
      on_menu_event: None,
      window_effects: None,
    }
  }

  /// Initializes a window builder from a [`WindowConfig`] from tauri.conf.json.
  /// Keep in mind that you can't create 2 windows with the same `label` so make sure
  /// that the initial window was closed or change the label of the new [`WindowBuilder`].
  ///
  /// # Known issues
  ///
  /// On Windows, this function deadlocks when used in a synchronous command, see [the Webview2 issue].
  /// You should use `async` commands when creating windows.
  ///
  /// # Examples
  ///
  /// - Create a window in a command:
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
#[tauri::command]
async fn reopen_window(app: tauri::AppHandle) {
  let window = tauri::window::WindowBuilder::from_config(&app, &app.config().app.windows.get(0).unwrap().clone())
    .unwrap()
    .build()
    .unwrap();
}
```
  "####
  )]
  ///
  /// [the Webview2 issue]: https://github.com/tauri-apps/wry/issues/583
  pub fn from_config(manager: &'a M, config: &WindowConfig) -> crate::Result<Self> {
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut builder = Self {
      manager,
      label: config.label.clone(),
      window_effects: config.window_effects.clone(),
      window_builder:
        <R::WindowDispatcher as WindowDispatch<EventLoopMessage>>::WindowBuilder::with_config(
          config,
        ),
      #[cfg(desktop)]
      menu: None,
      #[cfg(desktop)]
      on_menu_event: None,
    };

    #[cfg(desktop)]
    if let Some(parent) = &config.parent {
      let window = manager
        .manager()
        .get_window(parent)
        .ok_or(crate::Error::WindowNotFound)?;
      builder = builder.parent(&window)?;
    }

    Ok(builder)
  }

  /// Registers a global menu event listener.
  ///
  /// Note that this handler is called for any menu event,
  /// whether it is coming from this window, another window or from the tray icon menu.
  ///
  /// Also note that this handler will not be called if
  /// the window used to register it was closed.
  ///
  /// # Examples
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
use tauri::menu::{Menu, Submenu, MenuItem};
tauri::Builder::default()
  .setup(|app| {
    let handle = app.handle();
    let save_menu_item = MenuItem::new(handle, "Save", true, None::<&str>)?;
    let menu = Menu::with_items(handle, &[
      &Submenu::with_items(handle, "File", true, &[
        &save_menu_item,
      ])?,
    ])?;
    let window = tauri::window::WindowBuilder::new(app, "editor")
      .menu(menu)
      .on_menu_event(move |window, event| {
        if event.id == save_menu_item.id() {
          // save menu item
        }
      })
      .build()
      .unwrap();
  ///
    Ok(())
  });
```"####
  )]
  #[cfg(desktop)]
  pub fn on_menu_event<F: Fn(&Window<R>, crate::menu::MenuEvent) + Send + Sync + 'static>(
    mut self,
    f: F,
  ) -> Self {
    self.on_menu_event.replace(Box::new(f));
    self
  }

  /// Creates this window with a webview with it.
  #[cfg_attr(
    feature = "tracing",
    tracing::instrument(name = "webview::create", skip_all)
  )]
  pub(crate) fn with_webview(
    self,
    webview: WebviewBuilder<R>,
  ) -> crate::Result<(Window<R>, Webview<R>)> {
    let pending_webview = webview.into_pending_webview(self.manager, &self.label)?;
    let window = self.build_internal(Some(pending_webview))?;

    let webview = window.webviews().first().unwrap().clone();

    Ok((window, webview))
  }

  /// Creates a new window.
  pub fn build(self) -> crate::Result<Window<R>> {
    self.build_internal(None)
  }

  /// Creates a new window with an optional webview.
  fn build_internal(
    self,
    webview: Option<PendingWebview<EventLoopMessage, R>>,
  ) -> crate::Result<Window<R>> {
    let mut pending = PendingWindow::new(self.window_builder.clone(), self.label.clone())?;
    if let Some(webview) = webview {
      pending.set_webview(webview);
    }

    let app_manager = self.manager.manager();

    let pending = app_manager.window.prepare_window(pending)?;

    #[cfg(desktop)]
    let window_menu = {
      let is_app_wide = self.menu.is_none();
      self
        .menu
        .or_else(|| self.manager.app_handle().menu())
        .map(|menu| WindowMenu { is_app_wide, menu })
    };

    #[cfg(desktop)]
    let handler = app_manager
      .menu
      .prepare_window_menu_creation_handler(window_menu.as_ref(), self.window_builder.get_theme());
    #[cfg(not(desktop))]
    #[allow(clippy::type_complexity)]
    let handler: Option<Box<dyn Fn(tauri_runtime::window::RawWindow<'_>) + Send>> = None;

    let window = match &mut self.manager.runtime() {
      RuntimeOrDispatch::Runtime(runtime) => runtime.create_window(pending, handler),
      RuntimeOrDispatch::RuntimeHandle(handle) => handle.create_window(pending, handler),
      RuntimeOrDispatch::Dispatch(dispatcher) => dispatcher.create_window(pending, handler),
    }
    .map(|detached_window| {
      let window = app_manager.window.attach_window(
        self.manager.app_handle().clone(),
        detached_window.clone(),
        #[cfg(desktop)]
        window_menu,
      );

      if let Some(webview) = detached_window.webview {
        app_manager.webview.attach_webview(
          window.clone(),
          webview.webview,
          webview.use_https_scheme,
        );
      }

      window
    })?;

    #[cfg(desktop)]
    if let Some(handler) = self.on_menu_event {
      window.on_menu_event(handler);
    }

    if let Some(effects) = self.window_effects {
      crate::vibrancy::set_window_effects(&window, Some(effects))?;
    }

    let app_manager = self.manager.manager_owned();
    let window_label = window.label().to_string();
    // run on the main thread to fix a deadlock on webview.eval if the tracing feature is enabled
    let _ = window.run_on_main_thread(move || {
      let event = crate::EventName::from_str("tauri://window-created");
      let payload = Some(crate::webview::CreatedEvent {
        label: window_label,
      });
      let _ = app_manager.emit(event, EmitPayload::Serialize(&payload));
    });

    Ok(window)
  }
}

/// Desktop APIs.
#[cfg(desktop)]
#[cfg_attr(not(feature = "unstable"), allow(dead_code))]
impl<'a, R: Runtime, M: Manager<R>> WindowBuilder<'a, R, M> {
  /// Sets the menu for the window.
  #[must_use]
  pub fn menu(mut self, menu: Menu<R>) -> Self {
    self.menu.replace(menu);
    self
  }

  /// Show window in the center of the screen.
  #[must_use]
  pub fn center(mut self) -> Self {
    self.window_builder = self.window_builder.center();
    self
  }

  /// The initial position of the window's.
  #[must_use]
  pub fn position(mut self, x: f64, y: f64) -> Self {
    self.window_builder = self.window_builder.position(x, y);
    self
  }

  /// Window size.
  #[must_use]
  pub fn inner_size(mut self, width: f64, height: f64) -> Self {
    self.window_builder = self.window_builder.inner_size(width, height);
    self
  }

  /// Window min inner size.
  #[must_use]
  pub fn min_inner_size(mut self, min_width: f64, min_height: f64) -> Self {
    self.window_builder = self.window_builder.min_inner_size(min_width, min_height);
    self
  }

  /// Window max inner size.
  #[must_use]
  pub fn max_inner_size(mut self, max_width: f64, max_height: f64) -> Self {
    self.window_builder = self.window_builder.max_inner_size(max_width, max_height);
    self
  }

  /// Window inner size constraints.
  #[must_use]
  pub fn inner_size_constraints(
    mut self,
    constraints: tauri_runtime::window::WindowSizeConstraints,
  ) -> Self {
    self.window_builder = self.window_builder.inner_size_constraints(constraints);
    self
  }

  /// Whether the window is resizable or not.
  /// When resizable is set to false, native window's maximize button is automatically disabled.
  #[must_use]
  pub fn resizable(mut self, resizable: bool) -> Self {
    self.window_builder = self.window_builder.resizable(resizable);
    self
  }

  /// Whether the window's native maximize button is enabled or not.
  /// If resizable is set to false, this setting is ignored.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Disables the "zoom" button in the window titlebar, which is also used to enter fullscreen mode.
  /// - **Linux / iOS / Android:** Unsupported.
  #[must_use]
  pub fn maximizable(mut self, maximizable: bool) -> Self {
    self.window_builder = self.window_builder.maximizable(maximizable);
    self
  }

  /// Whether the window's native minimize button is enabled or not.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / iOS / Android:** Unsupported.
  #[must_use]
  pub fn minimizable(mut self, minimizable: bool) -> Self {
    self.window_builder = self.window_builder.minimizable(minimizable);
    self
  }

  /// Whether the window's native close button is enabled or not.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** "GTK+ will do its best to convince the window manager not to show a close button.
  ///   Depending on the system, this function may not have any effect when called on a window that is already visible"
  /// - **iOS / Android:** Unsupported.
  #[must_use]
  pub fn closable(mut self, closable: bool) -> Self {
    self.window_builder = self.window_builder.closable(closable);
    self
  }

  /// The title of the window in the title bar.
  #[must_use]
  pub fn title<S: Into<String>>(mut self, title: S) -> Self {
    self.window_builder = self.window_builder.title(title);
    self
  }

  /// Whether to start the window in fullscreen or not.
  #[must_use]
  pub fn fullscreen(mut self, fullscreen: bool) -> Self {
    self.window_builder = self.window_builder.fullscreen(fullscreen);
    self
  }

  /// Sets the window to be initially focused.
  #[must_use]
  #[deprecated(
    since = "1.2.0",
    note = "The window is automatically focused by default. This function Will be removed in 3.0.0. Use `focused` instead."
  )]
  pub fn focus(mut self) -> Self {
    self.window_builder = self.window_builder.focused(true);
    self
  }

  /// Whether the window will be initially focused or not.
  #[must_use]
  pub fn focused(mut self, focused: bool) -> Self {
    self.window_builder = self.window_builder.focused(focused);
    self
  }

  /// Whether the window should be maximized upon creation.
  #[must_use]
  pub fn maximized(mut self, maximized: bool) -> Self {
    self.window_builder = self.window_builder.maximized(maximized);
    self
  }

  /// Whether the window should be immediately visible upon creation.
  #[must_use]
  pub fn visible(mut self, visible: bool) -> Self {
    self.window_builder = self.window_builder.visible(visible);
    self
  }

  /// Forces a theme or uses the system settings if None was provided.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS**: Only supported on macOS 10.14+.
  #[must_use]
  pub fn theme(mut self, theme: Option<Theme>) -> Self {
    self.window_builder = self.window_builder.theme(theme);
    self
  }

  /// Whether the window should be transparent. If this is true, writing colors
  /// with alpha values different than `1.0` will produce a transparent window.
  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  #[cfg_attr(
    docsrs,
    doc(cfg(any(not(target_os = "macos"), feature = "macos-private-api")))
  )]
  #[must_use]
  pub fn transparent(mut self, transparent: bool) -> Self {
    self.window_builder = self.window_builder.transparent(transparent);
    self
  }

  /// Whether the window should have borders and bars.
  #[must_use]
  pub fn decorations(mut self, decorations: bool) -> Self {
    self.window_builder = self.window_builder.decorations(decorations);
    self
  }

  /// Whether the window should always be below other windows.
  #[must_use]
  pub fn always_on_bottom(mut self, always_on_bottom: bool) -> Self {
    self.window_builder = self.window_builder.always_on_bottom(always_on_bottom);
    self
  }

  /// Whether the window should always be on top of other windows.
  #[must_use]
  pub fn always_on_top(mut self, always_on_top: bool) -> Self {
    self.window_builder = self.window_builder.always_on_top(always_on_top);
    self
  }

  /// Whether the window will be visible on all workspaces or virtual desktops.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / iOS / Android:** Unsupported.
  #[must_use]
  pub fn visible_on_all_workspaces(mut self, visible_on_all_workspaces: bool) -> Self {
    self.window_builder = self
      .window_builder
      .visible_on_all_workspaces(visible_on_all_workspaces);
    self
  }

  /// Prevents the window contents from being captured by other apps.
  #[must_use]
  pub fn content_protected(mut self, protected: bool) -> Self {
    self.window_builder = self.window_builder.content_protected(protected);
    self
  }

  /// Sets the window icon.
  pub fn icon(mut self, icon: Image<'a>) -> crate::Result<Self> {
    self.window_builder = self.window_builder.icon(icon.into())?;
    Ok(self)
  }

  /// Sets whether or not the window icon should be hidden from the taskbar.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS**: Unsupported.
  #[must_use]
  pub fn skip_taskbar(mut self, skip: bool) -> Self {
    self.window_builder = self.window_builder.skip_taskbar(skip);
    self
  }

  /// Sets custom name for Windows' window class. **Windows only**.
  #[must_use]
  pub fn window_classname<S: Into<String>>(mut self, classname: S) -> Self {
    self.window_builder = self.window_builder.window_classname(classname);
    self
  }

  /// Sets whether or not the window has shadow.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows:**
  ///   - `false` has no effect on decorated window, shadows are always ON.
  ///   - `true` will make undecorated window have a 1px white border,
  ///     and on Windows 11, it will have a rounded corners.
  /// - **Linux:** Unsupported.
  #[must_use]
  pub fn shadow(mut self, enable: bool) -> Self {
    self.window_builder = self.window_builder.shadow(enable);
    self
  }

  /// Sets a parent to the window to be created.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**: This sets the passed parent as an owner window to the window to be created.
  ///   From [MSDN owned windows docs](https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#owned-windows):
  ///     - An owned window is always above its owner in the z-order.
  ///     - The system automatically destroys an owned window when its owner is destroyed.
  ///     - An owned window is hidden when its owner is minimized.
  /// - **Linux**: This makes the new window transient for parent, see <https://docs.gtk.org/gtk3/method.Window.set_transient_for.html>
  /// - **macOS**: This adds the window as a child of parent, see <https://developer.apple.com/documentation/appkit/nswindow/1419152-addchildwindow?language=objc>
  pub fn parent(mut self, parent: &Window<R>) -> crate::Result<Self> {
    #[cfg(windows)]
    {
      self.window_builder = self.window_builder.owner(parent.hwnd()?);
    }

    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "netbsd",
      target_os = "openbsd"
    ))]
    {
      self.window_builder = self.window_builder.transient_for(&parent.gtk_window()?);
    }

    #[cfg(target_os = "macos")]
    {
      self.window_builder = self.window_builder.parent(parent.ns_window()?);
    }

    Ok(self)
  }

  /// Set an owner to the window to be created.
  ///
  /// From MSDN:
  /// - An owned window is always above its owner in the z-order.
  /// - The system automatically destroys an owned window when its owner is destroyed.
  /// - An owned window is hidden when its owner is minimized.
  ///
  /// For more information, see <https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#owned-windows>
  #[cfg(windows)]
  pub fn owner(mut self, owner: &Window<R>) -> crate::Result<Self> {
    self.window_builder = self.window_builder.owner(owner.hwnd()?);
    Ok(self)
  }

  /// Set an owner to the window to be created.
  ///
  /// From MSDN:
  /// - An owned window is always above its owner in the z-order.
  /// - The system automatically destroys an owned window when its owner is destroyed.
  /// - An owned window is hidden when its owner is minimized.
  ///
  /// For more information, see <https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#owned-windows>
  ///
  /// **Note:** This is a low level API. See [`Self::parent`] for a higher level wrapper for Tauri windows.
  #[cfg(windows)]
  #[must_use]
  pub fn owner_raw(mut self, owner: HWND) -> Self {
    self.window_builder = self.window_builder.owner(owner);
    self
  }

  /// Sets a parent to the window to be created.
  ///
  /// A child window has the WS_CHILD style and is confined to the client area of its parent window.
  ///
  /// For more information, see <https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#child-windows>
  ///
  /// **Note:** This is a low level API. See [`Self::parent`] for a higher level wrapper for Tauri windows.
  #[cfg(windows)]
  #[must_use]
  pub fn parent_raw(mut self, parent: HWND) -> Self {
    self.window_builder = self.window_builder.parent(parent);
    self
  }

  /// Sets a parent to the window to be created.
  ///
  /// See <https://developer.apple.com/documentation/appkit/nswindow/1419152-addchildwindow?language=objc>
  ///
  /// **Note:** This is a low level API. See [`Self::parent`] for a higher level wrapper for Tauri windows.
  #[cfg(target_os = "macos")]
  #[must_use]
  pub fn parent_raw(mut self, parent: *mut std::ffi::c_void) -> Self {
    self.window_builder = self.window_builder.parent(parent);
    self
  }

  /// Sets the window to be created transient for parent.
  ///
  /// See <https://docs.gtk.org/gtk3/method.Window.set_transient_for.html>
  ///
  /// **Note:** This is a low level API. See [`Self::parent`] for a higher level wrapper for Tauri windows.
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  pub fn transient_for(mut self, parent: &Window<R>) -> crate::Result<Self> {
    self.window_builder = self.window_builder.transient_for(&parent.gtk_window()?);
    Ok(self)
  }

  /// Sets the window to be created transient for parent.
  ///
  /// See <https://docs.gtk.org/gtk3/method.Window.set_transient_for.html>
  ///
  /// **Note:** This is a low level API. See [`Self::parent`] and [`Self::transient_for`] for higher level wrappers for Tauri windows.
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  #[must_use]
  pub fn transient_for_raw(mut self, parent: &impl gtk::glib::IsA<gtk::Window>) -> Self {
    self.window_builder = self.window_builder.transient_for(parent);
    self
  }

  /// Enables or disables drag and drop support.
  #[cfg(windows)]
  #[must_use]
  pub fn drag_and_drop(mut self, enabled: bool) -> Self {
    self.window_builder = self.window_builder.drag_and_drop(enabled);
    self
  }

  /// Sets the [`crate::TitleBarStyle`].
  #[cfg(target_os = "macos")]
  #[must_use]
  pub fn title_bar_style(mut self, style: crate::TitleBarStyle) -> Self {
    self.window_builder = self.window_builder.title_bar_style(style);
    self
  }

  /// Hide the window title.
  #[cfg(target_os = "macos")]
  #[must_use]
  pub fn hidden_title(mut self, hidden: bool) -> Self {
    self.window_builder = self.window_builder.hidden_title(hidden);
    self
  }

  /// Defines the window [tabbing identifier] for macOS.
  ///
  /// Windows with matching tabbing identifiers will be grouped together.
  /// If the tabbing identifier is not set, automatic tabbing will be disabled.
  ///
  /// [tabbing identifier]: <https://developer.apple.com/documentation/appkit/nswindow/1644704-tabbingidentifier>
  #[cfg(target_os = "macos")]
  #[must_use]
  pub fn tabbing_identifier(mut self, identifier: &str) -> Self {
    self.window_builder = self.window_builder.tabbing_identifier(identifier);
    self
  }

  /// Sets window effects.
  ///
  /// Requires the window to be transparent.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: If using decorations or shadows, you may want to try this workaround <https://github.com/tauri-apps/tao/issues/72#issuecomment-975607891>
  /// - **Linux**: Unsupported
  pub fn effects(mut self, effects: WindowEffectsConfig) -> Self {
    self.window_effects.replace(effects);
    self
  }
}

impl<R: Runtime, M: Manager<R>> WindowBuilder<'_, R, M> {
  /// Set the window and webview background color.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: alpha channel is ignored.
  #[must_use]
  pub fn background_color(mut self, color: Color) -> Self {
    self.window_builder = self.window_builder.background_color(color);
    self
  }
}
/// A wrapper struct to hold the window menu state
/// and whether it is global per-app or specific to this window.
#[cfg(desktop)]
pub(crate) struct WindowMenu<R: Runtime> {
  pub(crate) is_app_wide: bool,
  pub(crate) menu: Menu<R>,
}

// TODO: expand these docs since this is a pretty important type
/// A window managed by Tauri.
///
/// This type also implements [`Manager`] which allows you to manage other windows attached to
/// the same application.
#[default_runtime(crate::Wry, wry)]
pub struct Window<R: Runtime> {
  /// The window created by the runtime.
  pub(crate) window: DetachedWindow<EventLoopMessage, R>,
  /// The manager to associate this window with.
  pub(crate) manager: Arc<AppManager<R>>,
  pub(crate) app_handle: AppHandle<R>,
  // The menu set for this window
  #[cfg(desktop)]
  pub(crate) menu: Arc<Mutex<Option<WindowMenu<R>>>>,
  pub(crate) resources_table: Arc<Mutex<ResourceTable>>,
}

impl<R: Runtime> std::fmt::Debug for Window<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Window")
      .field("window", &self.window)
      .field("manager", &self.manager)
      .field("app_handle", &self.app_handle)
      .finish()
  }
}

impl<R: Runtime> raw_window_handle::HasWindowHandle for Window<R> {
  fn window_handle(
    &self,
  ) -> std::result::Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
    self.window.dispatcher.window_handle()
  }
}

impl<R: Runtime> raw_window_handle::HasDisplayHandle for Window<R> {
  fn display_handle(
    &self,
  ) -> std::result::Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
    self.app_handle.display_handle()
  }
}

impl<R: Runtime> Clone for Window<R> {
  fn clone(&self) -> Self {
    Self {
      window: self.window.clone(),
      manager: self.manager.clone(),
      app_handle: self.app_handle.clone(),
      #[cfg(desktop)]
      menu: self.menu.clone(),
      resources_table: self.resources_table.clone(),
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

impl<R: Runtime> Manager<R> for Window<R> {
  fn resources_table(&self) -> MutexGuard<'_, ResourceTable> {
    self
      .resources_table
      .lock()
      .expect("poisoned window resources table")
  }
}

impl<R: Runtime> ManagerBase<R> for Window<R> {
  fn manager(&self) -> &AppManager<R> {
    &self.manager
  }

  fn manager_owned(&self) -> Arc<AppManager<R>> {
    self.manager.clone()
  }

  fn runtime(&self) -> RuntimeOrDispatch<'_, R> {
    RuntimeOrDispatch::Dispatch(self.window.dispatcher.clone())
  }

  fn managed_app_handle(&self) -> &AppHandle<R> {
    &self.app_handle
  }
}

impl<'de, R: Runtime> CommandArg<'de, R> for Window<R> {
  /// Grabs the [`Window`] from the [`CommandItem`]. This will never fail.
  fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
    Ok(command.message.webview().window())
  }
}

/// Base window functions.
impl<R: Runtime> Window<R> {
  /// Create a new window that is attached to the manager.
  pub(crate) fn new(
    manager: Arc<AppManager<R>>,
    window: DetachedWindow<EventLoopMessage, R>,
    app_handle: AppHandle<R>,
    #[cfg(desktop)] menu: Option<WindowMenu<R>>,
  ) -> Self {
    Self {
      window,
      manager,
      app_handle,
      #[cfg(desktop)]
      menu: Arc::new(std::sync::Mutex::new(menu)),
      resources_table: Default::default(),
    }
  }

  /// Initializes a window builder with the given window label.
  ///
  /// Data URLs are only supported with the `webview-data-url` feature flag.
  #[cfg(feature = "unstable")]
  #[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
  pub fn builder<M: Manager<R>, L: Into<String>>(manager: &M, label: L) -> WindowBuilder<'_, R, M> {
    WindowBuilder::new(manager, label.into())
  }

  /// Adds a new webview as a child of this window.
  #[cfg(any(test, all(desktop, feature = "unstable")))]
  #[cfg_attr(docsrs, doc(cfg(all(desktop, feature = "unstable"))))]
  pub fn add_child<P: Into<Position>, S: Into<Size>>(
    &self,
    webview_builder: WebviewBuilder<R>,
    position: P,
    size: S,
  ) -> crate::Result<Webview<R>> {
    use std::sync::mpsc::channel;

    let (tx, rx) = channel();
    let position = position.into();
    let size = size.into();
    let window_ = self.clone();
    self.run_on_main_thread(move || {
      let res = webview_builder.build(window_, position, size);
      tx.send(res.map_err(Into::into)).unwrap();
    })?;
    rx.recv().unwrap()
  }

  /// List of webviews associated with this window.
  pub fn webviews(&self) -> Vec<Webview<R>> {
    self
      .manager
      .webview
      .webviews_lock()
      .values()
      .filter(|w| w.window_label() == self.label())
      .cloned()
      .collect()
  }

  pub(crate) fn is_webview_window(&self) -> bool {
    self.webviews().iter().all(|w| w.label() == self.label())
  }

  /// Runs the given closure on the main thread.
  pub fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .run_on_main_thread(f)
      .map_err(Into::into)
  }

  /// The label of this window.
  pub fn label(&self) -> &str {
    &self.window.label
  }

  /// Registers a window event listener.
  pub fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) {
    self
      .window
      .dispatcher
      .on_window_event(move |event| f(&event.clone().into()));
  }
}

/// Menu APIs
#[cfg(desktop)]
impl<R: Runtime> Window<R> {
  /// Registers a global menu event listener.
  ///
  /// Note that this handler is called for any menu event,
  /// whether it is coming from this window, another window or from the tray icon menu.
  ///
  /// Also note that this handler will not be called if
  /// the window used to register it was closed.
  ///
  /// # Examples
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
use tauri::menu::{Menu, Submenu, MenuItem};
tauri::Builder::default()
  .setup(|app| {
    let handle = app.handle();
    let save_menu_item = MenuItem::new(handle, "Save", true, None::<&str>)?;
    let menu = Menu::with_items(handle, &[
      &Submenu::with_items(handle, "File", true, &[
        &save_menu_item,
      ])?,
    ])?;
    let window = tauri::window::WindowBuilder::new(app, "editor")
      .menu(menu)
      .build()
      .unwrap();

    window.on_menu_event(move |window, event| {
      if event.id == save_menu_item.id() {
          // save menu item
      }
    });

    Ok(())
  });
```
  "####
  )]
  pub fn on_menu_event<F: Fn(&Window<R>, crate::menu::MenuEvent) + Send + Sync + 'static>(
    &self,
    f: F,
  ) {
    self
      .manager
      .menu
      .event_listeners
      .lock()
      .unwrap()
      .insert(self.label().to_string(), Box::new(f));
  }

  pub(crate) fn menu_lock(&self) -> std::sync::MutexGuard<'_, Option<WindowMenu<R>>> {
    self.menu.lock().expect("poisoned window")
  }

  #[cfg_attr(target_os = "macos", allow(dead_code))]
  pub(crate) fn has_app_wide_menu(&self) -> bool {
    self
      .menu_lock()
      .as_ref()
      .map(|m| m.is_app_wide)
      .unwrap_or(false)
  }

  #[cfg_attr(target_os = "macos", allow(dead_code))]
  pub(crate) fn is_menu_in_use<I: PartialEq<MenuId>>(&self, id: &I) -> bool {
    self
      .menu_lock()
      .as_ref()
      .map(|m| id.eq(m.menu.id()))
      .unwrap_or(false)
  }

  /// Returns this window menu .
  pub fn menu(&self) -> Option<Menu<R>> {
    self.menu_lock().as_ref().map(|m| m.menu.clone())
  }

  /// Sets the window menu and returns the previous one.
  ///
  /// ## Platform-specific:
  ///
  /// - **macOS:** Unsupported. The menu on macOS is app-wide and not specific to one
  ///   window, if you need to set it, use [`AppHandle::set_menu`] instead.
  #[cfg_attr(target_os = "macos", allow(unused_variables))]
  pub fn set_menu(&self, menu: Menu<R>) -> crate::Result<Option<Menu<R>>> {
    let prev_menu = self.remove_menu()?;

    self.manager.menu.insert_menu_into_stash(&menu);

    let window = self.clone();
    let menu_ = menu.clone();
    self.run_on_main_thread(move || {
      #[cfg(windows)]
      if let Ok(hwnd) = window.hwnd() {
        let theme = window
          .theme()
          .map(crate::menu::map_to_menu_theme)
          .unwrap_or(muda::MenuTheme::Auto);

        let _ = unsafe { menu_.inner().init_for_hwnd_with_theme(hwnd.0 as _, theme) };
      }
      #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
      ))]
      if let (Ok(gtk_window), Ok(gtk_box)) = (window.gtk_window(), window.default_vbox()) {
        let _ = menu_
          .inner()
          .init_for_gtk_window(&gtk_window, Some(&gtk_box));
      }
    })?;

    self.menu_lock().replace(WindowMenu {
      is_app_wide: false,
      menu,
    });

    Ok(prev_menu)
  }

  /// Removes the window menu and returns it.
  ///
  /// ## Platform-specific:
  ///
  /// - **macOS:** Unsupported. The menu on macOS is app-wide and not specific to one
  ///   window, if you need to remove it, use [`AppHandle::remove_menu`] instead.
  pub fn remove_menu(&self) -> crate::Result<Option<Menu<R>>> {
    let prev_menu = self.menu_lock().take().map(|m| m.menu);

    // remove from the window
    #[cfg_attr(target_os = "macos", allow(unused_variables))]
    if let Some(menu) = &prev_menu {
      let window = self.clone();
      let menu = menu.clone();
      self.run_on_main_thread(move || {
        #[cfg(windows)]
        if let Ok(hwnd) = window.hwnd() {
          let _ = unsafe { menu.inner().remove_for_hwnd(hwnd.0 as _) };
        }
        #[cfg(any(
          target_os = "linux",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "netbsd",
          target_os = "openbsd"
        ))]
        if let Ok(gtk_window) = window.gtk_window() {
          let _ = menu.inner().remove_for_gtk_window(&gtk_window);
        }
      })?;
    }

    self
      .manager
      .remove_menu_from_stash_by_id(prev_menu.as_ref().map(|m| m.id()));

    Ok(prev_menu)
  }

  /// Hides the window menu.
  pub fn hide_menu(&self) -> crate::Result<()> {
    // remove from the window
    #[cfg_attr(target_os = "macos", allow(unused_variables))]
    if let Some(window_menu) = &*self.menu_lock() {
      let window = self.clone();
      let menu_ = window_menu.menu.clone();
      self.run_on_main_thread(move || {
        #[cfg(windows)]
        if let Ok(hwnd) = window.hwnd() {
          let _ = unsafe { menu_.inner().hide_for_hwnd(hwnd.0 as _) };
        }
        #[cfg(any(
          target_os = "linux",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "netbsd",
          target_os = "openbsd"
        ))]
        if let Ok(gtk_window) = window.gtk_window() {
          let _ = menu_.inner().hide_for_gtk_window(&gtk_window);
        }
      })?;
    }

    Ok(())
  }

  /// Shows the window menu.
  pub fn show_menu(&self) -> crate::Result<()> {
    // remove from the window
    #[cfg_attr(target_os = "macos", allow(unused_variables))]
    if let Some(window_menu) = &*self.menu_lock() {
      let window = self.clone();
      let menu_ = window_menu.menu.clone();
      self.run_on_main_thread(move || {
        #[cfg(windows)]
        if let Ok(hwnd) = window.hwnd() {
          let _ = unsafe { menu_.inner().show_for_hwnd(hwnd.0 as _) };
        }
        #[cfg(any(
          target_os = "linux",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "netbsd",
          target_os = "openbsd"
        ))]
        if let Ok(gtk_window) = window.gtk_window() {
          let _ = menu_.inner().show_for_gtk_window(&gtk_window);
        }
      })?;
    }

    Ok(())
  }

  /// Shows the window menu.
  pub fn is_menu_visible(&self) -> crate::Result<bool> {
    // remove from the window
    #[cfg_attr(target_os = "macos", allow(unused_variables))]
    if let Some(window_menu) = &*self.menu_lock() {
      let (tx, rx) = std::sync::mpsc::channel();
      let window = self.clone();
      let menu_ = window_menu.menu.clone();
      self.run_on_main_thread(move || {
        #[cfg(windows)]
        if let Ok(hwnd) = window.hwnd() {
          let _ = tx.send(unsafe { menu_.inner().is_visible_on_hwnd(hwnd.0 as _) });
        }
        #[cfg(any(
          target_os = "linux",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "netbsd",
          target_os = "openbsd"
        ))]
        if let Ok(gtk_window) = window.gtk_window() {
          let _ = tx.send(menu_.inner().is_visible_on_gtk_window(&gtk_window));
        }
      })?;

      return Ok(rx.recv().unwrap_or(false));
    }

    Ok(false)
  }

  /// Shows the specified menu as a context menu at the cursor position.
  pub fn popup_menu<M: ContextMenu>(&self, menu: &M) -> crate::Result<()> {
    menu.popup(self.clone())
  }

  /// Shows the specified menu as a context menu at the specified position.
  ///
  /// The position is relative to the window's top-left corner.
  pub fn popup_menu_at<M: ContextMenu, P: Into<Position>>(
    &self,
    menu: &M,
    position: P,
  ) -> crate::Result<()> {
    menu.popup_at(self.clone(), position)
  }
}

/// Window getters.
impl<R: Runtime> Window<R> {
  /// Returns the scale factor that can be used to map logical pixels to physical pixels, and vice versa.
  pub fn scale_factor(&self) -> crate::Result<f64> {
    self.window.dispatcher.scale_factor().map_err(Into::into)
  }

  /// Returns the position of the top-left hand corner of the window's client area relative to the top-left hand corner of the desktop.
  pub fn inner_position(&self) -> crate::Result<PhysicalPosition<i32>> {
    self.window.dispatcher.inner_position().map_err(Into::into)
  }

  /// Returns the position of the top-left hand corner of the window relative to the top-left hand corner of the desktop.
  pub fn outer_position(&self) -> crate::Result<PhysicalPosition<i32>> {
    self.window.dispatcher.outer_position().map_err(Into::into)
  }

  /// Returns the physical size of the window's client area.
  ///
  /// The client area is the content of the window, excluding the title bar and borders.
  pub fn inner_size(&self) -> crate::Result<PhysicalSize<u32>> {
    self.window.dispatcher.inner_size().map_err(Into::into)
  }

  /// Returns the physical size of the entire window.
  ///
  /// These dimensions include the title bar and borders. If you don't want that (and you usually don't), use inner_size instead.
  pub fn outer_size(&self) -> crate::Result<PhysicalSize<u32>> {
    self.window.dispatcher.outer_size().map_err(Into::into)
  }

  /// Gets the window's current fullscreen state.
  pub fn is_fullscreen(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_fullscreen().map_err(Into::into)
  }

  /// Gets the window's current minimized state.
  pub fn is_minimized(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_minimized().map_err(Into::into)
  }

  /// Gets the window's current maximized state.
  pub fn is_maximized(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_maximized().map_err(Into::into)
  }

  /// Gets the window's current focus state.
  pub fn is_focused(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_focused().map_err(Into::into)
  }

  /// Gets the window's current decoration state.
  pub fn is_decorated(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_decorated().map_err(Into::into)
  }

  /// Gets the window's current resizable state.
  pub fn is_resizable(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_resizable().map_err(Into::into)
  }

  /// Whether the window is enabled or disabled.
  pub fn is_enabled(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_enabled().map_err(Into::into)
  }

  /// Gets the window's native maximize button state
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / iOS / Android:** Unsupported.
  pub fn is_maximizable(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_maximizable().map_err(Into::into)
  }

  /// Gets the window's native minimize button state
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / iOS / Android:** Unsupported.
  pub fn is_minimizable(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_minimizable().map_err(Into::into)
  }

  /// Gets the window's native close button state
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / iOS / Android:** Unsupported.
  pub fn is_closable(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_closable().map_err(Into::into)
  }

  /// Gets the window's current visibility state.
  pub fn is_visible(&self) -> crate::Result<bool> {
    self.window.dispatcher.is_visible().map_err(Into::into)
  }

  /// Gets the window's current title.
  pub fn title(&self) -> crate::Result<String> {
    self.window.dispatcher.title().map_err(Into::into)
  }

  /// Returns the monitor on which the window currently resides.
  ///
  /// Returns None if current monitor can't be detected.
  pub fn current_monitor(&self) -> crate::Result<Option<Monitor>> {
    self
      .window
      .dispatcher
      .current_monitor()
      .map(|m| m.map(Into::into))
      .map_err(Into::into)
  }

  /// Returns the monitor that contains the given point.
  pub fn monitor_from_point(&self, x: f64, y: f64) -> crate::Result<Option<Monitor>> {
    self
      .window
      .dispatcher
      .monitor_from_point(x, y)
      .map(|m| m.map(Into::into))
      .map_err(Into::into)
  }

  /// Returns the primary monitor of the system.
  ///
  /// Returns None if it can't identify any monitor as a primary one.
  pub fn primary_monitor(&self) -> crate::Result<Option<Monitor>> {
    self
      .window
      .dispatcher
      .primary_monitor()
      .map(|m| m.map(Into::into))
      .map_err(Into::into)
  }

  /// Returns the list of all the monitors available on the system.
  pub fn available_monitors(&self) -> crate::Result<Vec<Monitor>> {
    self
      .window
      .dispatcher
      .available_monitors()
      .map(|m| m.into_iter().map(Into::into).collect())
      .map_err(Into::into)
  }

  /// Returns the native handle that is used by this window.
  #[cfg(target_os = "macos")]
  pub fn ns_window(&self) -> crate::Result<*mut std::ffi::c_void> {
    self
      .window
      .dispatcher
      .window_handle()
      .map_err(Into::into)
      .and_then(|handle| {
        if let raw_window_handle::RawWindowHandle::AppKit(h) = handle.as_raw() {
          let view: &objc2_app_kit::NSView = unsafe { h.ns_view.cast().as_ref() };
          let ns_window = view.window().expect("view to be installed in window");
          Ok(objc2::rc::Retained::autorelease_ptr(ns_window).cast())
        } else {
          Err(crate::Error::InvalidWindowHandle)
        }
      })
  }

  /// Returns the pointer to the content view of this window.
  #[cfg(target_os = "macos")]
  pub fn ns_view(&self) -> crate::Result<*mut std::ffi::c_void> {
    self
      .window
      .dispatcher
      .window_handle()
      .map_err(Into::into)
      .and_then(|handle| {
        if let raw_window_handle::RawWindowHandle::AppKit(h) = handle.as_raw() {
          Ok(h.ns_view.as_ptr())
        } else {
          Err(crate::Error::InvalidWindowHandle)
        }
      })
  }

  /// Returns the native handle that is used by this window.
  #[cfg(windows)]
  pub fn hwnd(&self) -> crate::Result<HWND> {
    self
      .window
      .dispatcher
      .window_handle()
      .map_err(Into::into)
      .and_then(|handle| {
        if let raw_window_handle::RawWindowHandle::Win32(h) = handle.as_raw() {
          Ok(HWND(h.hwnd.get() as _))
        } else {
          Err(crate::Error::InvalidWindowHandle)
        }
      })
  }

  /// Returns the `ApplicationWindow` from gtk crate that is used by this window.
  ///
  /// Note that this type can only be used on the main thread.
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

  /// Returns the vertical [`gtk::Box`] that is added by default as the sole child of this window.
  ///
  /// Note that this type can only be used on the main thread.
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  pub fn default_vbox(&self) -> crate::Result<gtk::Box> {
    self.window.dispatcher.default_vbox().map_err(Into::into)
  }

  /// Returns the current window theme.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS**: Only supported on macOS 10.14+.
  pub fn theme(&self) -> crate::Result<Theme> {
    self.window.dispatcher.theme().map_err(Into::into)
  }
}

/// Desktop window getters.
#[cfg(desktop)]
impl<R: Runtime> Window<R> {
  /// Get the cursor position relative to the top-left hand corner of the desktop.
  ///
  /// Note that the top-left hand corner of the desktop is not necessarily the same as the screen.
  /// If the user uses a desktop with multiple monitors,
  /// the top-left hand corner of the desktop is the top-left hand corner of the main monitor on Windows and macOS
  /// or the top-left of the leftmost monitor on X11.
  ///
  /// The coordinates can be negative if the top-left hand corner of the window is outside of the visible screen region.
  pub fn cursor_position(&self) -> crate::Result<PhysicalPosition<f64>> {
    self.app_handle.cursor_position()
  }
}

/// Desktop window setters and actions.
#[cfg(desktop)]
impl<R: Runtime> Window<R> {
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
  /// - **Linux:** Urgency levels have the same effect.
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

  /// Determines if this window should be resizable.
  /// When resizable is set to false, native window's maximize button is automatically disabled.
  pub fn set_resizable(&self, resizable: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_resizable(resizable)
      .map_err(Into::into)
  }

  /// Determines if this window's native maximize button should be enabled.
  /// If resizable is set to false, this setting is ignored.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Disables the "zoom" button in the window titlebar, which is also used to enter fullscreen mode.
  /// - **Linux / iOS / Android:** Unsupported.
  pub fn set_maximizable(&self, maximizable: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_maximizable(maximizable)
      .map_err(Into::into)
  }

  /// Determines if this window's native minimize button should be enabled.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / iOS / Android:** Unsupported.
  pub fn set_minimizable(&self, minimizable: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_minimizable(minimizable)
      .map_err(Into::into)
  }

  /// Determines if this window's native close button should be enabled.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** "GTK+ will do its best to convince the window manager not to show a close button.
  ///   Depending on the system, this function may not have any effect when called on a window that is already visible"
  /// - **iOS / Android:** Unsupported.
  pub fn set_closable(&self, closable: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_closable(closable)
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

  /// Enable or disable the window.
  pub fn set_enabled(&self, enabled: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_enabled(enabled)
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

  /// Closes this window. It emits [`crate::RunEvent::CloseRequested`] first like a user-initiated close request so you can intercept it.
  pub fn close(&self) -> crate::Result<()> {
    self.window.dispatcher.close().map_err(Into::into)
  }

  /// Destroys this window. Similar to [`Self::close`] but does not emit any events and force close the window instead.
  pub fn destroy(&self) -> crate::Result<()> {
    self.window.dispatcher.destroy().map_err(Into::into)
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

  /// Determines if this window should have shadow.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows:**
  ///   - `false` has no effect on decorated window, shadow are always ON.
  ///   - `true` will make undecorated window have a 1px white border,
  ///     and on Windows 11, it will have a rounded corners.
  /// - **Linux:** Unsupported.
  pub fn set_shadow(&self, enable: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_shadow(enable)
      .map_err(Into::into)
  }

  /// Sets window effects, pass [`None`] to clear any effects applied if possible.
  ///
  /// Requires the window to be transparent.
  ///
  /// See [`EffectsBuilder`] for a convenient builder for [`WindowEffectsConfig`].
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```rust,no_run
use tauri::{Manager, window::{Color, Effect, EffectState, EffectsBuilder}};
tauri::Builder::default()
  .setup(|app| {
    let window = app.get_window("main").unwrap();
    window.set_effects(
      EffectsBuilder::new()
        .effect(Effect::Popover)
        .state(EffectState::Active)
        .radius(5.)
        .color(Color(0, 0, 0, 255))
        .build(),
    )?;
    Ok(())
  });
```
  "####
  )]
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: If using decorations or shadows, you may want to try this workaround <https://github.com/tauri-apps/tao/issues/72#issuecomment-975607891>
  /// - **Linux**: Unsupported
  pub fn set_effects<E: Into<Option<WindowEffectsConfig>>>(&self, effects: E) -> crate::Result<()> {
    let effects = effects.into();
    let window = self.clone();
    self.run_on_main_thread(move || {
      let _ = crate::vibrancy::set_window_effects(&window, effects);
    })
  }

  /// Determines if this window should always be below other windows.
  pub fn set_always_on_bottom(&self, always_on_bottom: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_always_on_bottom(always_on_bottom)
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

  /// Sets whether the window should be visible on all workspaces or virtual desktops.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / iOS / Android:** Unsupported.
  pub fn set_visible_on_all_workspaces(
    &self,
    visible_on_all_workspaces: bool,
  ) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_visible_on_all_workspaces(visible_on_all_workspaces)
      .map_err(Into::into)
  }

  /// Sets the window background color.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows:** alpha channel is ignored.
  /// - **iOS / Android:** Unsupported.
  pub fn set_background_color(&self, color: Option<Color>) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_background_color(color)
      .map_err(Into::into)
  }

  /// Prevents the window contents from being captured by other apps.
  pub fn set_content_protected(&self, protected: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_content_protected(protected)
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

  /// Sets this window's minimum inner size.
  pub fn set_min_size<S: Into<Size>>(&self, size: Option<S>) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_min_size(size.map(|s| s.into()))
      .map_err(Into::into)
  }

  /// Sets this window's maximum inner size.
  pub fn set_max_size<S: Into<Size>>(&self, size: Option<S>) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_max_size(size.map(|s| s.into()))
      .map_err(Into::into)
  }

  /// Sets this window's minimum inner width.
  pub fn set_size_constraints(
    &self,
    constriants: tauri_runtime::window::WindowSizeConstraints,
  ) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_size_constraints(constriants)
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
  pub fn set_icon(&self, icon: Image<'_>) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_icon(icon.into())
      .map_err(Into::into)
  }

  /// Whether to hide the window icon from the taskbar or not.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Unsupported.
  pub fn set_skip_taskbar(&self, skip: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_skip_taskbar(skip)
      .map_err(Into::into)
  }

  /// Grabs the cursor, preventing it from leaving the window.
  ///
  /// There's no guarantee that the cursor will be hidden. You should
  /// hide it by yourself if you want so.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Unsupported.
  /// - **macOS:** This locks the cursor in a fixed location, which looks visually awkward.
  pub fn set_cursor_grab(&self, grab: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_cursor_grab(grab)
      .map_err(Into::into)
  }

  /// Modifies the cursor's visibility.
  ///
  /// If `false`, this will hide the cursor. If `true`, this will show the cursor.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows:** The cursor is only hidden within the confines of the window.
  /// - **macOS:** The cursor is hidden as long as the window has input focus, even if the cursor is
  ///   outside of the window.
  pub fn set_cursor_visible(&self, visible: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_cursor_visible(visible)
      .map_err(Into::into)
  }

  /// Modifies the cursor icon of the window.
  pub fn set_cursor_icon(&self, icon: CursorIcon) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_cursor_icon(icon)
      .map_err(Into::into)
  }

  /// Changes the position of the cursor in window coordinates.
  pub fn set_cursor_position<Pos: Into<Position>>(&self, position: Pos) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_cursor_position(position)
      .map_err(Into::into)
  }

  /// Ignores the window cursor events.
  pub fn set_ignore_cursor_events(&self, ignore: bool) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_ignore_cursor_events(ignore)
      .map_err(Into::into)
  }

  /// Starts dragging the window.
  pub fn start_dragging(&self) -> crate::Result<()> {
    self.window.dispatcher.start_dragging().map_err(Into::into)
  }

  /// Starts resize-dragging the window.
  pub fn start_resize_dragging(
    &self,
    direction: tauri_runtime::ResizeDirection,
  ) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .start_resize_dragging(direction)
      .map_err(Into::into)
  }

  /// Sets the overlay icon on the taskbar **Windows only**. Using `None` to remove the overlay icon
  ///
  /// The overlay icon can be unique for each window.
  #[cfg(target_os = "windows")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "windows")))]
  pub fn set_overlay_icon(&self, icon: Option<Image<'_>>) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_overlay_icon(icon.map(|x| x.into()))
      .map_err(Into::into)
  }

  /// Sets the taskbar badge count. Using `0` or `None` will remove the badge
  ///
  /// ## Platform-specific
  /// - **Windows:** Unsupported, use [`Window::set_overlay_icon`] instead.
  /// - **iOS:** iOS expects i32, the value will be clamped to i32::MIN, i32::MAX.
  /// - **Android:** Unsupported.
  pub fn set_badge_count(&self, count: Option<i64>) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_badge_count(count, Some(format!("{}.desktop", self.package_info().name)))
      .map_err(Into::into)
  }

  /// Sets the taskbar badge label **macOS only**. Using `None` will remove the badge
  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  pub fn set_badge_label(&self, label: Option<String>) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_badge_label(label)
      .map_err(Into::into)
  }

  /// Sets the taskbar progress state.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / macOS**: Progress bar is app-wide and not specific to this window.
  /// - **Linux**: Only supported desktop environments with `libunity` (e.g. GNOME).
  /// - **iOS / Android:** Unsupported.
  pub fn set_progress_bar(&self, progress_state: ProgressBarState) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_progress_bar(crate::runtime::ProgressBarState {
        status: progress_state.status,
        progress: progress_state.progress,
        desktop_filename: Some(format!("{}.desktop", self.package_info().name)),
      })
      .map_err(Into::into)
  }

  /// Sets the title bar style. **macOS only**.
  pub fn set_title_bar_style(&self, style: tauri_utils::TitleBarStyle) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_title_bar_style(style)
      .map_err(Into::into)
  }

  /// Sets the theme for this window.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / macOS**: Theme is app-wide and not specific to this window.
  /// - **iOS / Android:** Unsupported.
  pub fn set_theme(&self, theme: Option<Theme>) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_theme(theme)
      .map_err(Into::<crate::Error>::into)?;
    #[cfg(windows)]
    if let (Some(menu), Ok(hwnd)) = (self.menu(), self.hwnd()) {
      let raw_hwnd = hwnd.0 as isize;
      self.run_on_main_thread(move || {
        let _ = unsafe {
          menu.inner().set_theme_for_hwnd(
            raw_hwnd,
            theme
              .map(crate::menu::map_to_menu_theme)
              .unwrap_or(muda::MenuTheme::Auto),
          )
        };
      })?;
    };
    Ok(())
  }
}

/// Progress bar state.
#[cfg(desktop)]
#[cfg_attr(
  docsrs,
  doc(cfg(any(target_os = "macos", target_os = "linux", windows)))
)]
#[derive(serde::Deserialize, Debug)]
pub struct ProgressBarState {
  /// The progress bar status.
  pub status: Option<ProgressBarStatus>,
  /// The progress bar progress. This can be a value ranging from `0` to `100`
  pub progress: Option<u64>,
}

impl<R: Runtime> Listener<R> for Window<R> {
  /// Listen to an event on this window.
  ///
  /// # Examples
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
use tauri::{Manager, Listener};

tauri::Builder::default()
  .setup(|app| {
    let window = app.get_window("main").unwrap();
    window.listen("component-loaded", move |event| {
      println!("window just loaded a component");
    });

    Ok(())
  });
```
  "####
  )]
  fn listen<F>(&self, event: impl Into<String>, handler: F) -> EventId
  where
    F: Fn(Event) + Send + 'static,
  {
    let event = EventName::new(event.into()).unwrap();
    self.manager.listen(
      event,
      EventTarget::Window {
        label: self.label().to_string(),
      },
      handler,
    )
  }

  /// Listen to an event on this window only once.
  ///
  /// See [`Self::listen`] for more information.
  fn once<F>(&self, event: impl Into<String>, handler: F) -> EventId
  where
    F: FnOnce(Event) + Send + 'static,
  {
    let event = EventName::new(event.into()).unwrap();
    self.manager.once(
      event,
      EventTarget::Window {
        label: self.label().to_string(),
      },
      handler,
    )
  }

  /// Unlisten to an event on this window.
  ///
  /// # Examples
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
use tauri::{Manager, Listener};

tauri::Builder::default()
  .setup(|app| {
    let window = app.get_window("main").unwrap();
    let window_ = window.clone();
    let handler = window.listen("component-loaded", move |event| {
      println!("window just loaded a component");

      // we no longer need to listen to the event
      // we also could have used `window.once` instead
      window_.unlisten(event.id());
    });

    // stop listening to the event when you do not need it anymore
    window.unlisten(handler);

    Ok(())
  });
```
  "####
  )]
  fn unlisten(&self, id: EventId) {
    self.manager.unlisten(id)
  }
}

impl<R: Runtime> Emitter<R> for Window<R> {}

/// The [`WindowEffectsConfig`] object builder
#[derive(Default)]
pub struct EffectsBuilder(WindowEffectsConfig);
impl EffectsBuilder {
  /// Create a new [`WindowEffectsConfig`] builder
  pub fn new() -> Self {
    Self(WindowEffectsConfig::default())
  }

  /// Adds effect to the [`WindowEffectsConfig`] `effects` field
  pub fn effect(mut self, effect: Effect) -> Self {
    self.0.effects.push(effect);
    self
  }

  /// Adds effects to the [`WindowEffectsConfig`] `effects` field
  pub fn effects<I: IntoIterator<Item = Effect>>(mut self, effects: I) -> Self {
    self.0.effects.extend(effects);
    self
  }

  /// Clears the [`WindowEffectsConfig`] `effects` field
  pub fn clear_effects(mut self) -> Self {
    self.0.effects.clear();
    self
  }

  /// Sets `state` field for the [`WindowEffectsConfig`] **macOS Only**
  pub fn state(mut self, state: EffectState) -> Self {
    self.0.state = Some(state);
    self
  }
  /// Sets `radius` field fo the [`WindowEffectsConfig`] **macOS Only**
  pub fn radius(mut self, radius: f64) -> Self {
    self.0.radius = Some(radius);
    self
  }
  /// Sets `color` field fo the [`WindowEffectsConfig`] **Windows Only**
  pub fn color(mut self, color: Color) -> Self {
    self.0.color = Some(color);
    self
  }

  /// Builds a [`WindowEffectsConfig`]
  pub fn build(self) -> WindowEffectsConfig {
    self.0
  }
}

impl From<WindowEffectsConfig> for EffectsBuilder {
  fn from(value: WindowEffectsConfig) -> Self {
    Self(value)
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn window_is_send_sync() {
    crate::test_utils::assert_send::<super::Window>();
    crate::test_utils::assert_sync::<super::Window>();
  }
}
