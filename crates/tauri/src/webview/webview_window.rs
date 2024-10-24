// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [`Window`] that hosts a single [`Webview`].

use std::{
  borrow::Cow,
  path::PathBuf,
  sync::{Arc, MutexGuard},
};

use crate::{
  event::EventTarget,
  ipc::ScopeObject,
  runtime::dpi::{PhysicalPosition, PhysicalSize},
  window::Monitor,
  Emitter, Listener, ResourceTable, Window,
};
#[cfg(desktop)]
use crate::{
  image::Image,
  menu::{ContextMenu, Menu},
  runtime::{
    dpi::{Position, Size},
    window::CursorIcon,
    UserAttentionType,
  },
};
use serde::Serialize;
use tauri_utils::{
  config::{Color, WebviewUrl, WindowConfig},
  Theme,
};
use url::Url;

use crate::{
  ipc::{CommandArg, CommandItem, InvokeError, OwnedInvokeResponder},
  manager::AppManager,
  sealed::{ManagerBase, RuntimeOrDispatch},
  webview::PageLoadPayload,
  webview::WebviewBuilder,
  window::WindowBuilder,
  AppHandle, Event, EventId, Manager, Runtime, Webview, WindowEvent,
};

use tauri_macros::default_runtime;

#[cfg(windows)]
use windows::Win32::Foundation::HWND;

use super::{DownloadEvent, ResolvedScope};

/// A builder for [`WebviewWindow`], a window that hosts a single webview.
pub struct WebviewWindowBuilder<'a, R: Runtime, M: Manager<R>> {
  window_builder: WindowBuilder<'a, R, M>,
  webview_builder: WebviewBuilder<R>,
}

impl<'a, R: Runtime, M: Manager<R>> WebviewWindowBuilder<'a, R, M> {
  /// Initializes a webview window builder with the given window label.
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
  /// ```
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let webview_window = tauri::WebviewWindowBuilder::new(app, "label", tauri::WebviewUrl::App("index.html".into()))
  ///       .build()?;
  ///     Ok(())
  ///   });
  /// ```
  ///
  /// - Create a window in a separate thread:
  ///
  /// ```
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let handle = app.handle().clone();
  ///     std::thread::spawn(move || {
  ///       let webview_window = tauri::WebviewWindowBuilder::new(&handle, "label", tauri::WebviewUrl::App("index.html".into()))
  ///         .build()
  ///         .unwrap();
  ///     });
  ///     Ok(())
  ///   });
  /// ```
  ///
  /// - Create a window in a command:
  ///
  /// ```
  /// #[tauri::command]
  /// async fn create_window(app: tauri::AppHandle) {
  ///   let webview_window = tauri::WebviewWindowBuilder::new(&app, "label", tauri::WebviewUrl::App("index.html".into()))
  ///     .build()
  ///     .unwrap();
  /// }
  /// ```
  ///
  /// [the Webview2 issue]: https://github.com/tauri-apps/wry/issues/583
  pub fn new<L: Into<String>>(manager: &'a M, label: L, url: WebviewUrl) -> Self {
    let label = label.into();
    Self {
      window_builder: WindowBuilder::new(manager, &label),
      webview_builder: WebviewBuilder::new(&label, url),
    }
  }

  /// Initializes a webview window builder from a [`WindowConfig`] from tauri.conf.json.
  /// Keep in mind that you can't create 2 windows with the same `label` so make sure
  /// that the initial window was closed or change the label of the new [`WebviewWindowBuilder`].
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
  /// ```
  /// #[tauri::command]
  /// async fn reopen_window(app: tauri::AppHandle) {
  ///   let webview_window = tauri::WebviewWindowBuilder::from_config(&app, &app.config().app.windows.get(0).unwrap().clone())
  ///     .unwrap()
  ///     .build()
  ///     .unwrap();
  /// }
  /// ```
  ///
  /// [the Webview2 issue]: https://github.com/tauri-apps/wry/issues/583
  pub fn from_config(manager: &'a M, config: &WindowConfig) -> crate::Result<Self> {
    Ok(Self {
      window_builder: WindowBuilder::from_config(manager, config)?,
      webview_builder: WebviewBuilder::from_config(config),
    })
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
  /// ```
  /// use tauri::menu::{Menu, Submenu, MenuItem};
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let handle = app.handle();
  ///     let save_menu_item = MenuItem::new(handle, "Save", true, None::<&str>)?;
  ///     let menu = Menu::with_items(handle, &[
  ///       &Submenu::with_items(handle, "File", true, &[
  ///         &save_menu_item,
  ///       ])?,
  ///     ])?;
  ///     let webview_window = tauri::WebviewWindowBuilder::new(app, "editor", tauri::WebviewUrl::App("index.html".into()))
  ///       .menu(menu)
  ///       .on_menu_event(move |window, event| {
  ///         if event.id == save_menu_item.id() {
  ///           // save menu item
  ///         }
  ///       })
  ///       .build()
  ///       .unwrap();
  ///
  ///     Ok(())
  ///   });
  /// ```
  #[cfg(desktop)]
  pub fn on_menu_event<F: Fn(&crate::Window<R>, crate::menu::MenuEvent) + Send + Sync + 'static>(
    mut self,
    f: F,
  ) -> Self {
    self.window_builder = self.window_builder.on_menu_event(f);
    self
  }

  /// Defines a closure to be executed when the webview makes an HTTP request for a web resource, allowing you to modify the response.
  ///
  /// Currently only implemented for the `tauri` URI protocol.
  ///
  /// **NOTE:** Currently this is **not** executed when using external URLs such as a development server,
  /// but it might be implemented in the future. **Always** check the request URL.
  ///
  /// # Examples
  /// ```rust,no_run
  /// use tauri::{
  ///   utils::config::{Csp, CspDirectiveSources, WebviewUrl},
  ///   webview::WebviewWindowBuilder,
  /// };
  /// use http::header::HeaderValue;
  /// use std::collections::HashMap;
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let webview_window = WebviewWindowBuilder::new(app, "core", WebviewUrl::App("index.html".into()))
  ///       .on_web_resource_request(|request, response| {
  ///         if request.uri().scheme_str() == Some("tauri") {
  ///           // if we have a CSP header, Tauri is loading an HTML file
  ///           //  for this example, let's dynamically change the CSP
  ///           if let Some(csp) = response.headers_mut().get_mut("Content-Security-Policy") {
  ///             // use the tauri helper to parse the CSP policy to a map
  ///             let mut csp_map: HashMap<String, CspDirectiveSources> = Csp::Policy(csp.to_str().unwrap().to_string()).into();
  ///             csp_map.entry("script-src".to_string()).or_insert_with(Default::default).push("'unsafe-inline'");
  ///             // use the tauri helper to get a CSP string from the map
  ///             let csp_string = Csp::from(csp_map).to_string();
  ///             *csp = HeaderValue::from_str(&csp_string).unwrap();
  ///           }
  ///         }
  ///       })
  ///       .build()?;
  ///     Ok(())
  ///   });
  /// ```
  pub fn on_web_resource_request<
    F: Fn(http::Request<Vec<u8>>, &mut http::Response<Cow<'static, [u8]>>) + Send + Sync + 'static,
  >(
    mut self,
    f: F,
  ) -> Self {
    self.webview_builder = self.webview_builder.on_web_resource_request(f);
    self
  }

  /// Defines a closure to be executed when the webview navigates to a URL. Returning `false` cancels the navigation.
  ///
  /// # Examples
  /// ```rust,no_run
  /// use tauri::{
  ///   utils::config::{Csp, CspDirectiveSources, WebviewUrl},
  ///   webview::WebviewWindowBuilder,
  /// };
  /// use http::header::HeaderValue;
  /// use std::collections::HashMap;
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let webview_window = WebviewWindowBuilder::new(app, "core", WebviewUrl::App("index.html".into()))
  ///       .on_navigation(|url| {
  ///         // allow the production URL or localhost on dev
  ///         url.scheme() == "tauri" || (cfg!(dev) && url.host_str() == Some("localhost"))
  ///       })
  ///       .build()?;
  ///     Ok(())
  ///   });
  /// ```
  pub fn on_navigation<F: Fn(&Url) -> bool + Send + 'static>(mut self, f: F) -> Self {
    self.webview_builder = self.webview_builder.on_navigation(f);
    self
  }

  /// Set a download event handler to be notified when a download is requested or finished.
  ///
  /// Returning `false` prevents the download from happening on a [`DownloadEvent::Requested`] event.
  ///
  /// # Examples
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```rust,no_run
use tauri::{
  utils::config::{Csp, CspDirectiveSources, WebviewUrl},
  webview::{DownloadEvent, WebviewWindowBuilder},
};

tauri::Builder::default()
  .setup(|app| {
    let handle = app.handle();
    let webview_window = WebviewWindowBuilder::new(handle, "core", WebviewUrl::App("index.html".into()))
      .on_download(|webview, event| {
        match event {
          DownloadEvent::Requested { url, destination } => {
            println!("downloading {}", url);
            *destination = "/home/tauri/target/path".into();
          }
          DownloadEvent::Finished { url, path, success } => {
            println!("downloaded {} to {:?}, success: {}", url, path, success);
          }
          _ => (),
        }
        // let the download start
        true
      })
      .build()?;

    Ok(())
  });
```
  "####
  )]
  pub fn on_download<F: Fn(Webview<R>, DownloadEvent<'_>) -> bool + Send + Sync + 'static>(
    mut self,
    f: F,
  ) -> Self {
    self.webview_builder.download_handler.replace(Arc::new(f));
    self
  }

  /// Defines a closure to be executed when a page load event is triggered.
  /// The event can be either [`tauri_runtime::webview::PageLoadEvent::Started`] if the page has started loading
  /// or [`tauri_runtime::webview::PageLoadEvent::Finished`] when the page finishes loading.
  ///
  /// # Examples
  /// ```rust,no_run
  /// use tauri::{
  ///   utils::config::{Csp, CspDirectiveSources, WebviewUrl},
  ///   webview::{PageLoadEvent, WebviewWindowBuilder},
  /// };
  /// use http::header::HeaderValue;
  /// use std::collections::HashMap;
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let webview_window = WebviewWindowBuilder::new(app, "core", WebviewUrl::App("index.html".into()))
  ///       .on_page_load(|window, payload| {
  ///         match payload.event() {
  ///           PageLoadEvent::Started => {
  ///             println!("{} finished loading", payload.url());
  ///           }
  ///           PageLoadEvent::Finished => {
  ///             println!("{} finished loading", payload.url());
  ///           }
  ///         }
  ///       })
  ///       .build()?;
  ///     Ok(())
  ///   });
  /// ```
  pub fn on_page_load<F: Fn(WebviewWindow<R>, PageLoadPayload<'_>) + Send + Sync + 'static>(
    mut self,
    f: F,
  ) -> Self {
    self.webview_builder = self.webview_builder.on_page_load(move |webview, payload| {
      f(
        WebviewWindow {
          window: webview.window(),
          webview,
        },
        payload,
      )
    });
    self
  }

  /// Creates a new window.
  pub fn build(self) -> crate::Result<WebviewWindow<R>> {
    let (window, webview) = self.window_builder.with_webview(self.webview_builder)?;
    Ok(WebviewWindow { window, webview })
  }
}

/// Desktop APIs.
#[cfg(desktop)]
impl<'a, R: Runtime, M: Manager<R>> WebviewWindowBuilder<'a, R, M> {
  /// Sets the menu for the window.
  #[must_use]
  pub fn menu(mut self, menu: crate::menu::Menu<R>) -> Self {
    self.window_builder = self.window_builder.menu(menu);
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
    note = "The window is automatically focused by default. This function Will be removed in 2.0.0. Use `focused` instead."
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
  pub fn theme(mut self, theme: Option<crate::Theme>) -> Self {
    self.window_builder = self.window_builder.theme(theme);
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
    self.window_builder = self.window_builder.icon(icon)?;
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
  pub fn parent(mut self, parent: &WebviewWindow<R>) -> crate::Result<Self> {
    self.window_builder = self.window_builder.parent(&parent.window)?;
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
  pub fn owner(mut self, owner: &WebviewWindow<R>) -> crate::Result<Self> {
    self.window_builder = self.window_builder.owner(&owner.window)?;
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
  #[must_use]
  pub fn owner_raw(mut self, owner: HWND) -> Self {
    self.window_builder = self.window_builder.owner_raw(owner);
    self
  }

  /// Sets a parent to the window to be created.
  ///
  /// A child window has the WS_CHILD style and is confined to the client area of its parent window.
  ///
  /// For more information, see <https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#child-windows>
  #[cfg(windows)]
  #[must_use]
  pub fn parent_raw(mut self, parent: HWND) -> Self {
    self.window_builder = self.window_builder.parent_raw(parent);
    self
  }

  /// Sets a parent to the window to be created.
  ///
  /// See <https://developer.apple.com/documentation/appkit/nswindow/1419152-addchildwindow?language=objc>
  #[cfg(target_os = "macos")]
  #[must_use]
  pub fn parent_raw(mut self, parent: *mut std::ffi::c_void) -> Self {
    self.window_builder = self.window_builder.parent_raw(parent);
    self
  }

  /// Sets the window to be created transient for parent.
  ///
  /// See <https://docs.gtk.org/gtk3/method.Window.set_transient_for.html>
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  pub fn transient_for(mut self, parent: &WebviewWindow<R>) -> crate::Result<Self> {
    self.window_builder = self.window_builder.transient_for(&parent.window)?;
    Ok(self)
  }

  /// Sets the window to be created transient for parent.
  ///
  /// See <https://docs.gtk.org/gtk3/method.Window.set_transient_for.html>
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  #[must_use]
  pub fn transient_for_raw(mut self, parent: &impl gtk::glib::IsA<gtk::Window>) -> Self {
    self.window_builder = self.window_builder.transient_for_raw(parent);
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
  pub fn effects(mut self, effects: crate::utils::config::WindowEffectsConfig) -> Self {
    self.window_builder = self.window_builder.effects(effects);
    self
  }
}

/// Webview attributes.
impl<'a, R: Runtime, M: Manager<R>> WebviewWindowBuilder<'a, R, M> {
  /// Sets whether clicking an inactive window also clicks through to the webview.
  #[must_use]
  pub fn accept_first_mouse(mut self, accept: bool) -> Self {
    self.webview_builder = self.webview_builder.accept_first_mouse(accept);
    self
  }

  /// Adds the provided JavaScript to a list of scripts that should be run after the global object has been created,
  /// but before the HTML document has been parsed and before any other script included by the HTML document is run.
  ///
  /// Since it runs on all top-level document and child frame page navigations,
  /// it's recommended to check the `window.location` to guard your script from running on unexpected origins.
  ///
  /// # Examples
  ///
  /// ```rust
  /// use tauri::{WebviewWindowBuilder, Runtime};
  ///
  /// const INIT_SCRIPT: &str = r#"
  ///   if (window.location.origin === 'https://tauri.app') {
  ///     console.log("hello world from js init script");
  ///
  ///     window.__MY_CUSTOM_PROPERTY__ = { foo: 'bar' };
  ///   }
  /// "#;
  ///
  /// fn main() {
  ///   tauri::Builder::default()
  ///     .setup(|app| {
  ///       let webview = tauri::WebviewWindowBuilder::new(app, "label", tauri::WebviewUrl::App("index.html".into()))
  ///         .initialization_script(INIT_SCRIPT)
  ///         .build()?;
  ///       Ok(())
  ///     });
  /// }
  /// ```
  #[must_use]
  pub fn initialization_script(mut self, script: &str) -> Self {
    self.webview_builder = self.webview_builder.initialization_script(script);
    self
  }

  /// Set the user agent for the webview
  #[must_use]
  pub fn user_agent(mut self, user_agent: &str) -> Self {
    self.webview_builder = self.webview_builder.user_agent(user_agent);
    self
  }

  /// Set additional arguments for the webview.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS / Linux / Android / iOS**: Unsupported.
  ///
  /// ## Warning
  ///
  /// By default wry passes `--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection`
  /// so if you use this method, you also need to disable these components by yourself if you want.
  #[must_use]
  pub fn additional_browser_args(mut self, additional_args: &str) -> Self {
    self.webview_builder = self
      .webview_builder
      .additional_browser_args(additional_args);
    self
  }

  /// Data directory for the webview.
  #[must_use]
  pub fn data_directory(mut self, data_directory: PathBuf) -> Self {
    self.webview_builder = self.webview_builder.data_directory(data_directory);
    self
  }

  /// Disables the drag and drop handler. This is required to use HTML5 drag and drop APIs on the frontend on Windows.
  #[must_use]
  pub fn disable_drag_drop_handler(mut self) -> Self {
    self.webview_builder = self.webview_builder.disable_drag_drop_handler();
    self
  }

  /// Enables clipboard access for the page rendered on **Linux** and **Windows**.
  ///
  /// **macOS** doesn't provide such method and is always enabled by default,
  /// but you still need to add menu item accelerators to use shortcuts.
  #[must_use]
  pub fn enable_clipboard_access(mut self) -> Self {
    self.webview_builder = self.webview_builder.enable_clipboard_access();
    self
  }

  /// Enable or disable incognito mode for the WebView..
  ///
  ///  ## Platform-specific:
  ///
  ///  **Android**: Unsupported.
  #[must_use]
  pub fn incognito(mut self, incognito: bool) -> Self {
    self.webview_builder = self.webview_builder.incognito(incognito);
    self
  }

  /// Sets the webview to automatically grow and shrink its size and position when the parent window resizes.
  #[must_use]
  pub fn auto_resize(mut self) -> Self {
    self.webview_builder = self.webview_builder.auto_resize();
    self
  }

  /// Set a proxy URL for the WebView for all network requests.
  ///
  /// Must be either a `http://` or a `socks5://` URL.
  #[must_use]
  pub fn proxy_url(mut self, url: Url) -> Self {
    self.webview_builder = self.webview_builder.proxy_url(url);
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
    #[cfg(desktop)]
    {
      self.window_builder = self.window_builder.transparent(transparent);
    }
    self.webview_builder = self.webview_builder.transparent(transparent);
    self
  }

  /// Whether page zooming by hotkeys is enabled
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: Controls WebView2's [`IsZoomControlEnabled`](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2settings?view=webview2-winrt-1.0.2420.47#iszoomcontrolenabled) setting.
  /// - **MacOS / Linux**: Injects a polyfill that zooms in and out with `ctrl/command` + `-/=`,
  ///   20% in each step, ranging from 20% to 1000%. Requires `webview:allow-set-webview-zoom` permission
  ///
  /// - **Android / iOS**: Unsupported.
  #[must_use]
  pub fn zoom_hotkeys_enabled(mut self, enabled: bool) -> Self {
    self.webview_builder = self.webview_builder.zoom_hotkeys_enabled(enabled);
    self
  }

  /// Whether browser extensions can be installed for the webview process
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: Enables the WebView2 environment's [`AreBrowserExtensionsEnabled`](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2environmentoptions?view=webview2-winrt-1.0.2739.15#arebrowserextensionsenabled)
  /// - **MacOS / Linux / iOS / Android** - Unsupported.
  #[must_use]
  pub fn browser_extensions_enabled(mut self, enabled: bool) -> Self {
    self.webview_builder = self.webview_builder.browser_extensions_enabled(enabled);
    self
  }

  /// Set the window and webview background color.
  ///
  /// ## Platform-specific:
  ///
  /// - **Android / iOS:** Unsupported for the window layer.
  /// - **macOS / iOS**: Not implemented for the webview layer.
  /// - **Windows**:
  ///   - alpha channel is ignored for the window layer.
  ///   - On Windows 7, alpha channel is ignored for the webview layer.
  ///   - On Windows 8 and newer, if alpha channel is not `0`, it will be ignored.
  #[must_use]
  pub fn background_color(mut self, color: Color) -> Self {
    self.window_builder = self.window_builder.background_color(color);
    self.webview_builder = self.webview_builder.background_color(color);
    self
  }
}

/// A type that wraps a [`Window`] together with a [`Webview`].
#[default_runtime(crate::Wry, wry)]
#[derive(Debug)]
pub struct WebviewWindow<R: Runtime> {
  pub(crate) window: Window<R>,
  pub(crate) webview: Webview<R>,
}

impl<R: Runtime> AsRef<Webview<R>> for WebviewWindow<R> {
  fn as_ref(&self) -> &Webview<R> {
    &self.webview
  }
}

impl<R: Runtime> Clone for WebviewWindow<R> {
  fn clone(&self) -> Self {
    Self {
      window: self.window.clone(),
      webview: self.webview.clone(),
    }
  }
}

impl<R: Runtime> Eq for WebviewWindow<R> {}
impl<R: Runtime> PartialEq for WebviewWindow<R> {
  /// Only use the [`Window`]'s label to compare equality.
  fn eq(&self, other: &Self) -> bool {
    self.webview.eq(&other.webview)
  }
}

impl<R: Runtime> raw_window_handle::HasWindowHandle for WebviewWindow<R> {
  fn window_handle(
    &self,
  ) -> std::result::Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
    Ok(unsafe {
      raw_window_handle::WindowHandle::borrow_raw(self.window.window_handle()?.as_raw())
    })
  }
}

impl<R: Runtime> raw_window_handle::HasDisplayHandle for WebviewWindow<R> {
  fn display_handle(
    &self,
  ) -> std::result::Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
    self.webview.app_handle.display_handle()
  }
}

impl<'de, R: Runtime> CommandArg<'de, R> for WebviewWindow<R> {
  /// Grabs the [`Window`] from the [`CommandItem`]. This will never fail.
  fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
    let webview = command.message.webview();
    let window = webview.window();
    if window.is_webview_window() {
      return Ok(Self {
        window: window.clone(),
        webview,
      });
    }

    Err(InvokeError::from("current webview is not a WebviewWindow"))
  }
}

/// Base webview window functions.
impl<R: Runtime> WebviewWindow<R> {
  /// Initializes a [`WebviewWindowBuilder`] with the given window label and webview URL.
  ///
  /// Data URLs are only supported with the `webview-data-url` feature flag.
  pub fn builder<M: Manager<R>, L: Into<String>>(
    manager: &M,
    label: L,
    url: WebviewUrl,
  ) -> WebviewWindowBuilder<'_, R, M> {
    WebviewWindowBuilder::new(manager, label, url)
  }

  /// Runs the given closure on the main thread.
  pub fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> crate::Result<()> {
    self.webview.run_on_main_thread(f)
  }

  /// The webview label.
  pub fn label(&self) -> &str {
    self.webview.label()
  }

  /// Registers a window event listener.
  pub fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) {
    self.window.on_window_event(f);
  }

  /// Resolves the given command scope for this webview on the currently loaded URL.
  ///
  /// If the command is not allowed, returns None.
  ///
  /// If the scope cannot be deserialized to the given type, an error is returned.
  ///
  /// In a command context this can be directly resolved from the command arguments via [crate::ipc::CommandScope]:
  ///
  /// ```
  /// use tauri::ipc::CommandScope;
  ///
  /// #[derive(Debug, serde::Deserialize)]
  /// struct ScopeType {
  ///   some_value: String,
  /// }
  /// #[tauri::command]
  /// fn my_command(scope: CommandScope<ScopeType>) {
  ///   // check scope
  /// }
  /// ```
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::Manager;
  ///
  /// #[derive(Debug, serde::Deserialize)]
  /// struct ScopeType {
  ///   some_value: String,
  /// }
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let webview = app.get_webview_window("main").unwrap();
  ///     let scope = webview.resolve_command_scope::<ScopeType>("my-plugin", "read");
  ///     Ok(())
  ///   });
  /// ```
  pub fn resolve_command_scope<T: ScopeObject>(
    &self,
    plugin: &str,
    command: &str,
  ) -> crate::Result<Option<ResolvedScope<T>>> {
    self.webview.resolve_command_scope(plugin, command)
  }
}

/// Menu APIs
#[cfg(desktop)]
impl<R: Runtime> WebviewWindow<R> {
  /// Registers a global menu event listener.
  ///
  /// Note that this handler is called for any menu event,
  /// whether it is coming from this window, another window or from the tray icon menu.
  ///
  /// Also note that this handler will not be called if
  /// the window used to register it was closed.
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::menu::{Menu, Submenu, MenuItem};
  /// use tauri::{WebviewWindowBuilder, WebviewUrl};
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let handle = app.handle();
  ///     let save_menu_item = MenuItem::new(handle, "Save", true, None::<&str>)?;
  ///     let menu = Menu::with_items(handle, &[
  ///       &Submenu::with_items(handle, "File", true, &[
  ///         &save_menu_item,
  ///       ])?,
  ///     ])?;
  ///     let webview_window = WebviewWindowBuilder::new(app, "editor", WebviewUrl::default())
  ///       .menu(menu)
  ///       .build()
  ///       .unwrap();
  ///
  ///     webview_window.on_menu_event(move |window, event| {
  ///       if event.id == save_menu_item.id() {
  ///           // save menu item
  ///       }
  ///     });
  ///
  ///     Ok(())
  ///   });
  /// ```
  pub fn on_menu_event<F: Fn(&crate::Window<R>, crate::menu::MenuEvent) + Send + Sync + 'static>(
    &self,
    f: F,
  ) {
    self.window.on_menu_event(f)
  }

  /// Returns this window menu.
  pub fn menu(&self) -> Option<Menu<R>> {
    self.window.menu()
  }

  /// Sets the window menu and returns the previous one.
  ///
  /// ## Platform-specific:
  ///
  /// - **macOS:** Unsupported. The menu on macOS is app-wide and not specific to one
  ///   window, if you need to set it, use [`AppHandle::set_menu`] instead.
  #[cfg_attr(target_os = "macos", allow(unused_variables))]
  pub fn set_menu(&self, menu: Menu<R>) -> crate::Result<Option<Menu<R>>> {
    self.window.set_menu(menu)
  }

  /// Removes the window menu and returns it.
  ///
  /// ## Platform-specific:
  ///
  /// - **macOS:** Unsupported. The menu on macOS is app-wide and not specific to one
  ///   window, if you need to remove it, use [`AppHandle::remove_menu`] instead.
  pub fn remove_menu(&self) -> crate::Result<Option<Menu<R>>> {
    self.window.remove_menu()
  }

  /// Hides the window menu.
  pub fn hide_menu(&self) -> crate::Result<()> {
    self.window.hide_menu()
  }

  /// Shows the window menu.
  pub fn show_menu(&self) -> crate::Result<()> {
    self.window.show_menu()
  }

  /// Shows the window menu.
  pub fn is_menu_visible(&self) -> crate::Result<bool> {
    self.window.is_menu_visible()
  }

  /// Shows the specified menu as a context menu at the cursor position.
  pub fn popup_menu<M: ContextMenu>(&self, menu: &M) -> crate::Result<()> {
    self.window.popup_menu(menu)
  }

  /// Shows the specified menu as a context menu at the specified position.
  ///
  /// The position is relative to the window's top-left corner.
  pub fn popup_menu_at<M: ContextMenu, P: Into<Position>>(
    &self,
    menu: &M,
    position: P,
  ) -> crate::Result<()> {
    self.window.popup_menu_at(menu, position)
  }
}

/// Window getters.
impl<R: Runtime> WebviewWindow<R> {
  /// Returns the scale factor that can be used to map logical pixels to physical pixels, and vice versa.
  pub fn scale_factor(&self) -> crate::Result<f64> {
    self.window.scale_factor()
  }

  /// Returns the position of the top-left hand corner of the window's client area relative to the top-left hand corner of the desktop.
  pub fn inner_position(&self) -> crate::Result<PhysicalPosition<i32>> {
    self.window.inner_position()
  }

  /// Returns the position of the top-left hand corner of the window relative to the top-left hand corner of the desktop.
  pub fn outer_position(&self) -> crate::Result<PhysicalPosition<i32>> {
    self.window.outer_position()
  }

  /// Returns the physical size of the window's client area.
  ///
  /// The client area is the content of the window, excluding the title bar and borders.
  pub fn inner_size(&self) -> crate::Result<PhysicalSize<u32>> {
    self.window.inner_size()
  }

  /// Returns the physical size of the entire window.
  ///
  /// These dimensions include the title bar and borders. If you don't want that (and you usually don't), use inner_size instead.
  pub fn outer_size(&self) -> crate::Result<PhysicalSize<u32>> {
    self.window.outer_size()
  }

  /// Gets the window's current fullscreen state.
  pub fn is_fullscreen(&self) -> crate::Result<bool> {
    self.window.is_fullscreen()
  }

  /// Gets the window's current minimized state.
  pub fn is_minimized(&self) -> crate::Result<bool> {
    self.window.is_minimized()
  }

  /// Gets the window's current maximized state.
  pub fn is_maximized(&self) -> crate::Result<bool> {
    self.window.is_maximized()
  }

  /// Gets the window's current focus state.
  pub fn is_focused(&self) -> crate::Result<bool> {
    self.window.is_focused()
  }

  /// Gets the window's current decoration state.
  pub fn is_decorated(&self) -> crate::Result<bool> {
    self.window.is_decorated()
  }

  /// Gets the window's current resizable state.
  pub fn is_resizable(&self) -> crate::Result<bool> {
    self.window.is_resizable()
  }

  /// Whether the window is enabled or disabled.
  pub fn is_enabled(&self) -> crate::Result<bool> {
    self.webview.window().is_enabled()
  }

  /// Gets the window's native maximize button state
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / iOS / Android:** Unsupported.
  pub fn is_maximizable(&self) -> crate::Result<bool> {
    self.window.is_maximizable()
  }

  /// Gets the window's native minimize button state
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / iOS / Android:** Unsupported.
  pub fn is_minimizable(&self) -> crate::Result<bool> {
    self.window.is_minimizable()
  }

  /// Gets the window's native close button state
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / iOS / Android:** Unsupported.
  pub fn is_closable(&self) -> crate::Result<bool> {
    self.window.is_closable()
  }

  /// Gets the window's current visibility state.
  pub fn is_visible(&self) -> crate::Result<bool> {
    self.window.is_visible()
  }

  /// Gets the window's current title.
  pub fn title(&self) -> crate::Result<String> {
    self.window.title()
  }

  /// Returns the monitor on which the window currently resides.
  ///
  /// Returns None if current monitor can't be detected.
  pub fn current_monitor(&self) -> crate::Result<Option<Monitor>> {
    self.window.current_monitor()
  }

  /// Returns the primary monitor of the system.
  ///
  /// Returns None if it can't identify any monitor as a primary one.
  pub fn primary_monitor(&self) -> crate::Result<Option<Monitor>> {
    self.window.primary_monitor()
  }

  /// Returns the monitor that contains the given point.
  pub fn monitor_from_point(&self, x: f64, y: f64) -> crate::Result<Option<Monitor>> {
    self.window.monitor_from_point(x, y)
  }

  /// Returns the list of all the monitors available on the system.
  pub fn available_monitors(&self) -> crate::Result<Vec<Monitor>> {
    self.window.available_monitors()
  }

  /// Returns the native handle that is used by this window.
  #[cfg(target_os = "macos")]
  pub fn ns_window(&self) -> crate::Result<*mut std::ffi::c_void> {
    self.window.ns_window()
  }

  /// Returns the pointer to the content view of this window.
  #[cfg(target_os = "macos")]
  pub fn ns_view(&self) -> crate::Result<*mut std::ffi::c_void> {
    self.window.ns_view()
  }

  /// Returns the native handle that is used by this window.
  #[cfg(windows)]
  pub fn hwnd(&self) -> crate::Result<HWND> {
    self.window.hwnd()
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
    self.window.gtk_window()
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
    self.window.default_vbox()
  }

  /// Returns the current window theme.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS**: Only supported on macOS 10.14+.
  pub fn theme(&self) -> crate::Result<crate::Theme> {
    self.window.theme()
  }
}

/// Desktop window getters.
#[cfg(desktop)]
impl<R: Runtime> WebviewWindow<R> {
  /// Get the cursor position relative to the top-left hand corner of the desktop.
  ///
  /// Note that the top-left hand corner of the desktop is not necessarily the same as the screen.
  /// If the user uses a desktop with multiple monitors,
  /// the top-left hand corner of the desktop is the top-left hand corner of the main monitor on Windows and macOS
  /// or the top-left of the leftmost monitor on X11.
  ///
  /// The coordinates can be negative if the top-left hand corner of the window is outside of the visible screen region.
  pub fn cursor_position(&self) -> crate::Result<PhysicalPosition<f64>> {
    self.webview.cursor_position()
  }
}

/// Desktop window setters and actions.
#[cfg(desktop)]
impl<R: Runtime> WebviewWindow<R> {
  /// Centers the window.
  pub fn center(&self) -> crate::Result<()> {
    self.window.center()
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
    self.window.request_user_attention(request_type)
  }

  /// Determines if this window should be resizable.
  /// When resizable is set to false, native window's maximize button is automatically disabled.
  pub fn set_resizable(&self, resizable: bool) -> crate::Result<()> {
    self.window.set_resizable(resizable)
  }

  /// Enable or disable the window.
  pub fn set_enabled(&self, enabled: bool) -> crate::Result<()> {
    self.webview.window().set_enabled(enabled)
  }

  /// Determines if this window's native maximize button should be enabled.
  /// If resizable is set to false, this setting is ignored.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Disables the "zoom" button in the window titlebar, which is also used to enter fullscreen mode.
  /// - **Linux / iOS / Android:** Unsupported.
  pub fn set_maximizable(&self, maximizable: bool) -> crate::Result<()> {
    self.window.set_maximizable(maximizable)
  }

  /// Determines if this window's native minimize button should be enabled.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / iOS / Android:** Unsupported.
  pub fn set_minimizable(&self, minimizable: bool) -> crate::Result<()> {
    self.window.set_minimizable(minimizable)
  }

  /// Determines if this window's native close button should be enabled.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** "GTK+ will do its best to convince the window manager not to show a close button.
  ///   Depending on the system, this function may not have any effect when called on a window that is already visible"
  /// - **iOS / Android:** Unsupported.
  pub fn set_closable(&self, closable: bool) -> crate::Result<()> {
    self.window.set_closable(closable)
  }

  /// Set this window's title.
  pub fn set_title(&self, title: &str) -> crate::Result<()> {
    self.window.set_title(title)
  }

  /// Maximizes this window.
  pub fn maximize(&self) -> crate::Result<()> {
    self.window.maximize()
  }

  /// Un-maximizes this window.
  pub fn unmaximize(&self) -> crate::Result<()> {
    self.window.unmaximize()
  }

  /// Minimizes this window.
  pub fn minimize(&self) -> crate::Result<()> {
    self.window.minimize()
  }

  /// Un-minimizes this window.
  pub fn unminimize(&self) -> crate::Result<()> {
    self.window.unminimize()
  }

  /// Show this window.
  pub fn show(&self) -> crate::Result<()> {
    self.window.show()
  }

  /// Hide this window.
  pub fn hide(&self) -> crate::Result<()> {
    self.window.hide()
  }

  /// Closes this window. It emits [`crate::RunEvent::CloseRequested`] first like a user-initiated close request so you can intercept it.
  pub fn close(&self) -> crate::Result<()> {
    self.window.close()
  }

  /// Destroys this window. Similar to [`Self::close`] but does not emit any events and force close the window instead.
  pub fn destroy(&self) -> crate::Result<()> {
    self.window.destroy()
  }

  /// Determines if this window should be [decorated].
  ///
  /// [decorated]: https://en.wikipedia.org/wiki/Window_(computing)#Window_decoration
  pub fn set_decorations(&self, decorations: bool) -> crate::Result<()> {
    self.window.set_decorations(decorations)
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
    self.window.set_shadow(enable)
  }

  /// Sets window effects, pass [`None`] to clear any effects applied if possible.
  ///
  /// Requires the window to be transparent.
  ///
  /// See [`crate::window::EffectsBuilder`] for a convenient builder for [`crate::utils::config::WindowEffectsConfig`].
  ///
  ///
  /// ```rust,no_run
  /// use tauri::{Manager, window::{Color, Effect, EffectState, EffectsBuilder}};
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let webview_window = app.get_webview_window("main").unwrap();
  ///     webview_window.set_effects(
  ///       EffectsBuilder::new()
  ///         .effect(Effect::Popover)
  ///         .state(EffectState::Active)
  ///         .radius(5.)
  ///         .color(Color(0, 0, 0, 255))
  ///         .build(),
  ///     )?;
  ///     Ok(())
  ///   });
  /// ```
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: If using decorations or shadows, you may want to try this workaround <https://github.com/tauri-apps/tao/issues/72#issuecomment-975607891>
  /// - **Linux**: Unsupported
  pub fn set_effects<E: Into<Option<crate::utils::config::WindowEffectsConfig>>>(
    &self,
    effects: E,
  ) -> crate::Result<()> {
    self.window.set_effects(effects)
  }

  /// Determines if this window should always be below other windows.
  pub fn set_always_on_bottom(&self, always_on_bottom: bool) -> crate::Result<()> {
    self.window.set_always_on_bottom(always_on_bottom)
  }

  /// Determines if this window should always be on top of other windows.
  pub fn set_always_on_top(&self, always_on_top: bool) -> crate::Result<()> {
    self.window.set_always_on_top(always_on_top)
  }

  /// Sets whether the window should be visible on all workspaces or virtual desktops.
  pub fn set_visible_on_all_workspaces(
    &self,
    visible_on_all_workspaces: bool,
  ) -> crate::Result<()> {
    self
      .window
      .set_visible_on_all_workspaces(visible_on_all_workspaces)
  }

  /// Prevents the window contents from being captured by other apps.
  pub fn set_content_protected(&self, protected: bool) -> crate::Result<()> {
    self.window.set_content_protected(protected)
  }

  /// Resizes this window.
  pub fn set_size<S: Into<Size>>(&self, size: S) -> crate::Result<()> {
    self.window.set_size(size.into())
  }

  /// Sets this window's minimum inner size.
  pub fn set_min_size<S: Into<Size>>(&self, size: Option<S>) -> crate::Result<()> {
    self.window.set_min_size(size.map(|s| s.into()))
  }

  /// Sets this window's maximum inner size.
  pub fn set_max_size<S: Into<Size>>(&self, size: Option<S>) -> crate::Result<()> {
    self.window.set_max_size(size.map(|s| s.into()))
  }

  /// Sets this window's minimum inner width.
  pub fn set_size_constraints(
    &self,
    constriants: tauri_runtime::window::WindowSizeConstraints,
  ) -> crate::Result<()> {
    self.window.set_size_constraints(constriants)
  }

  /// Sets this window's position.
  pub fn set_position<Pos: Into<Position>>(&self, position: Pos) -> crate::Result<()> {
    self.window.set_position(position)
  }

  /// Determines if this window should be fullscreen.
  pub fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()> {
    self.window.set_fullscreen(fullscreen)
  }

  /// Bring the window to front and focus.
  pub fn set_focus(&self) -> crate::Result<()> {
    self.window.set_focus()
  }

  /// Sets this window' icon.
  pub fn set_icon(&self, icon: Image<'_>) -> crate::Result<()> {
    self.window.set_icon(icon)
  }

  /// Sets the window background color.
  ///
  /// ## Platform-specific:
  ///
  /// - **iOS / Android:** Unsupported.
  /// - **macOS**: Not implemented for the webview layer..
  /// - **Windows**:
  ///   - alpha channel is ignored for the window layer.
  ///   - On Windows 7, transparency is not supported and the alpha value will be ignored for the webview layer..
  ///   - On Windows 8 and newer: translucent colors are not supported so any alpha value other than `0` will be replaced by `255` for the webview layer.
  pub fn set_background_color(&self, color: Option<Color>) -> crate::Result<()> {
    self.window.set_background_color(color)?;
    self.webview.set_background_color(color)
  }

  /// Whether to hide the window icon from the taskbar or not.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Unsupported.
  pub fn set_skip_taskbar(&self, skip: bool) -> crate::Result<()> {
    self.window.set_skip_taskbar(skip)
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
    self.window.set_cursor_grab(grab)
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
    self.window.set_cursor_visible(visible)
  }

  /// Modifies the cursor icon of the window.
  pub fn set_cursor_icon(&self, icon: CursorIcon) -> crate::Result<()> {
    self.window.set_cursor_icon(icon)
  }

  /// Changes the position of the cursor in window coordinates.
  pub fn set_cursor_position<Pos: Into<Position>>(&self, position: Pos) -> crate::Result<()> {
    self.window.set_cursor_position(position)
  }

  /// Ignores the window cursor events.
  pub fn set_ignore_cursor_events(&self, ignore: bool) -> crate::Result<()> {
    self.window.set_ignore_cursor_events(ignore)
  }

  /// Starts dragging the window.
  pub fn start_dragging(&self) -> crate::Result<()> {
    self.window.start_dragging()
  }

  /// Sets the taskbar progress state.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / macOS**: Progress bar is app-wide and not specific to this window.
  /// - **Linux**: Only supported desktop environments with `libunity` (e.g. GNOME).
  /// - **iOS / Android:** Unsupported.
  pub fn set_progress_bar(
    &self,
    progress_state: crate::window::ProgressBarState,
  ) -> crate::Result<()> {
    self.window.set_progress_bar(progress_state)
  }

  /// Sets the title bar style. **macOS only**.
  pub fn set_title_bar_style(&self, style: tauri_utils::TitleBarStyle) -> crate::Result<()> {
    self.window.set_title_bar_style(style)
  }

  /// Set the window theme.
  pub fn set_theme(&self, theme: Option<Theme>) -> crate::Result<()> {
    self.window.set_theme(theme)
  }
}

/// Desktop webview setters and actions.
#[cfg(desktop)]
impl<R: Runtime> WebviewWindow<R> {
  /// Opens the dialog to prints the contents of the webview.
  /// Currently only supported on macOS on `wry`.
  /// `window.print()` works on all platforms.
  pub fn print(&self) -> crate::Result<()> {
    self.webview.print()
  }
}

/// Webview APIs.
impl<R: Runtime> WebviewWindow<R> {
  /// Executes a closure, providing it with the webview handle that is specific to the current platform.
  ///
  /// The closure is executed on the main thread.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::Manager;
  ///
  /// fn main() {
  ///   tauri::Builder::default()
  ///     .setup(|app| {
  ///       let main_webview = app.get_webview_window("main").unwrap();
  ///       main_webview.with_webview(|webview| {
  ///         #[cfg(target_os = "linux")]
  ///         {
  ///           // see <https://docs.rs/webkit2gtk/2.0.0/webkit2gtk/struct.WebView.html>
  ///           // and <https://docs.rs/webkit2gtk/2.0.0/webkit2gtk/trait.WebViewExt.html>
  ///           use webkit2gtk::WebViewExt;
  ///           webview.inner().set_zoom_level(4.);
  ///         }
  ///
  ///         #[cfg(windows)]
  ///         unsafe {
  ///           // see <https://docs.rs/webview2-com/0.19.1/webview2_com/Microsoft/Web/WebView2/Win32/struct.ICoreWebView2Controller.html>
  ///           webview.controller().SetZoomFactor(4.).unwrap();
  ///         }
  ///
  ///         #[cfg(target_os = "macos")]
  ///         unsafe {
  ///           let view: &objc2_web_kit::WKWebView = &*webview.inner().cast();
  ///           let controller: &objc2_web_kit::WKUserContentController = &*webview.controller().cast();
  ///           let window: &objc2_app_kit::NSWindow = &*webview.ns_window().cast();
  ///
  ///           view.setPageZoom(4.);
  ///           controller.removeAllUserScripts();
  ///           let bg_color = objc2_app_kit::NSColor::colorWithDeviceRed_green_blue_alpha(0.5, 0.2, 0.4, 1.);
  ///           window.setBackgroundColor(Some(&bg_color));
  ///         }
  ///
  ///         #[cfg(target_os = "android")]
  ///         {
  ///           use jni::objects::JValue;
  ///           webview.jni_handle().exec(|env, _, webview| {
  ///             env.call_method(webview, "zoomBy", "(F)V", &[JValue::Float(4.)]).unwrap();
  ///           })
  ///         }
  ///       });
  ///       Ok(())
  ///   });
  /// }
  /// ```
  #[allow(clippy::needless_doctest_main)] // To avoid a large diff
  #[cfg(feature = "wry")]
  #[cfg_attr(docsrs, doc(feature = "wry"))]
  pub fn with_webview<F: FnOnce(crate::webview::PlatformWebview) + Send + 'static>(
    &self,
    f: F,
  ) -> crate::Result<()> {
    self.webview.with_webview(f)
  }

  /// Returns the current url of the webview.
  pub fn url(&self) -> crate::Result<Url> {
    self.webview.url()
  }

  /// Navigates the webview to the defined url.
  pub fn navigate(&mut self, url: Url) -> crate::Result<()> {
    self.webview.navigate(url)
  }

  /// Handles this window receiving an [`crate::webview::InvokeRequest`].
  pub fn on_message(
    self,
    request: crate::webview::InvokeRequest,
    responder: Box<OwnedInvokeResponder<R>>,
  ) {
    self.webview.on_message(request, responder)
  }

  /// Evaluates JavaScript on this window.
  pub fn eval(&self, js: &str) -> crate::Result<()> {
    self.webview.eval(js)
  }

  /// Opens the developer tools window (Web Inspector).
  /// The devtools is only enabled on debug builds or with the `devtools` feature flag.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Only supported on macOS 10.15+.
  ///   This is a private API on macOS, so you cannot use this if your application will be published on the App Store.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::Manager;
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     #[cfg(debug_assertions)]
  ///     app.get_webview_window("main").unwrap().open_devtools();
  ///     Ok(())
  ///   });
  /// ```
  #[cfg(any(debug_assertions, feature = "devtools"))]
  #[cfg_attr(docsrs, doc(cfg(any(debug_assertions, feature = "devtools"))))]
  pub fn open_devtools(&self) {
    self.webview.open_devtools();
  }

  /// Closes the developer tools window (Web Inspector).
  /// The devtools is only enabled on debug builds or with the `devtools` feature flag.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Only supported on macOS 10.15+.
  ///   This is a private API on macOS, so you cannot use this if your application will be published on the App Store.
  /// - **Windows:** Unsupported.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::Manager;
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     #[cfg(debug_assertions)]
  ///     {
  ///       let webview = app.get_webview_window("main").unwrap();
  ///       webview.open_devtools();
  ///       std::thread::spawn(move || {
  ///         std::thread::sleep(std::time::Duration::from_secs(10));
  ///         webview.close_devtools();
  ///       });
  ///     }
  ///     Ok(())
  ///   });
  /// ```
  #[cfg(any(debug_assertions, feature = "devtools"))]
  #[cfg_attr(docsrs, doc(cfg(any(debug_assertions, feature = "devtools"))))]
  pub fn close_devtools(&self) {
    self.webview.close_devtools();
  }

  /// Checks if the developer tools window (Web Inspector) is opened.
  /// The devtools is only enabled on debug builds or with the `devtools` feature flag.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Only supported on macOS 10.15+.
  ///   This is a private API on macOS, so you cannot use this if your application will be published on the App Store.
  /// - **Windows:** Unsupported.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::Manager;
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     #[cfg(debug_assertions)]
  ///     {
  ///       let webview = app.get_webview_window("main").unwrap();
  ///       if !webview.is_devtools_open() {
  ///         webview.open_devtools();
  ///       }
  ///     }
  ///     Ok(())
  ///   });
  /// ```
  #[cfg(any(debug_assertions, feature = "devtools"))]
  #[cfg_attr(docsrs, doc(cfg(any(debug_assertions, feature = "devtools"))))]
  pub fn is_devtools_open(&self) -> bool {
    self.webview.is_devtools_open()
  }

  /// Set the webview zoom level
  ///
  /// ## Platform-specific:
  ///
  /// - **Android**: Not supported.
  /// - **macOS**: available on macOS 11+ only.
  /// - **iOS**: available on iOS 14+ only.
  pub fn set_zoom(&self, scale_factor: f64) -> crate::Result<()> {
    self.webview.set_zoom(scale_factor)
  }

  /// Clear all browsing data for this webview window.
  pub fn clear_all_browsing_data(&self) -> crate::Result<()> {
    self.webview.clear_all_browsing_data()
  }
}

impl<R: Runtime> Listener<R> for WebviewWindow<R> {
  /// Listen to an event on this webview window.
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::{Manager, Listener};
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let webview_window = app.get_webview_window("main").unwrap();
  ///     webview_window.listen("component-loaded", move |event| {
  ///       println!("window just loaded a component");
  ///     });
  ///
  ///     Ok(())
  ///   });
  /// ```
  fn listen<F>(&self, event: impl Into<String>, handler: F) -> EventId
  where
    F: Fn(Event) + Send + 'static,
  {
    self.manager().listen(
      event.into(),
      EventTarget::WebviewWindow {
        label: self.label().to_string(),
      },
      handler,
    )
  }

  /// Listen to an event on this window webview only once.
  ///
  /// See [`Self::listen`] for more information.
  fn once<F>(&self, event: impl Into<String>, handler: F) -> EventId
  where
    F: FnOnce(Event) + Send + 'static,
  {
    self.manager().once(
      event.into(),
      EventTarget::WebviewWindow {
        label: self.label().to_string(),
      },
      handler,
    )
  }

  /// Unlisten to an event on this webview window.
  ///
  /// # Examples
  /// ```
  /// use tauri::{Manager, Listener};
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let webview_window = app.get_webview_window("main").unwrap();
  ///     let webview_window_ = webview_window.clone();
  ///     let handler = webview_window.listen("component-loaded", move |event| {
  ///       println!("webview_window just loaded a component");
  ///
  ///       // we no longer need to listen to the event
  ///       // we also could have used `webview_window.once` instead
  ///       webview_window_.unlisten(event.id());
  ///     });
  ///
  ///     // stop listening to the event when you do not need it anymore
  ///     webview_window.unlisten(handler);
  ///
  ///     Ok(())
  /// });
  /// ```
  fn unlisten(&self, id: EventId) {
    self.manager().unlisten(id)
  }
}

impl<R: Runtime> Emitter<R> for WebviewWindow<R> {
  /// Emits an event to all [targets](EventTarget).
  ///
  /// # Examples
  /// ```
  /// use tauri::Emitter;
  ///
  /// #[tauri::command]
  /// fn synchronize(window: tauri::WebviewWindow) {
  ///   // emits the synchronized event to all webviews
  ///   window.emit("synchronized", ());
  /// }
  ///   ```
  fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> crate::Result<()> {
    self.manager().emit(event, payload)
  }

  /// Emits an event to all [targets](EventTarget) matching the given target.
  ///
  /// # Examples
  /// ```
  /// use tauri::{Emitter, EventTarget};
  ///
  /// #[tauri::command]
  /// fn download(window: tauri::WebviewWindow) {
  ///   for i in 1..100 {
  ///     std::thread::sleep(std::time::Duration::from_millis(150));
  ///     // emit a download progress event to all listeners
  ///     window.emit_to(EventTarget::any(), "download-progress", i);
  ///     // emit an event to listeners that used App::listen or AppHandle::listen
  ///     window.emit_to(EventTarget::app(), "download-progress", i);
  ///     // emit an event to any webview/window/webviewWindow matching the given label
  ///     window.emit_to("updater", "download-progress", i); // similar to using EventTarget::labeled
  ///     window.emit_to(EventTarget::labeled("updater"), "download-progress", i);
  ///     // emit an event to listeners that used WebviewWindow::listen
  ///     window.emit_to(EventTarget::webview_window("updater"), "download-progress", i);
  ///   }
  /// }
  /// ```
  fn emit_to<I, S>(&self, target: I, event: &str, payload: S) -> crate::Result<()>
  where
    I: Into<EventTarget>,
    S: Serialize + Clone,
  {
    self.manager().emit_to(target, event, payload)
  }

  /// Emits an event to all [targets](EventTarget) based on the given filter.
  ///
  /// # Examples
  /// ```
  /// use tauri::{Emitter, EventTarget};
  ///
  /// #[tauri::command]
  /// fn download(window: tauri::WebviewWindow) {
  ///   for i in 1..100 {
  ///     std::thread::sleep(std::time::Duration::from_millis(150));
  ///     // emit a download progress event to the updater window
  ///     window.emit_filter("download-progress", i, |t| match t {
  ///       EventTarget::WebviewWindow { label } => label == "main",
  ///       _ => false,
  ///     });
  ///   }
  /// }
  ///   ```
  fn emit_filter<S, F>(&self, event: &str, payload: S, filter: F) -> crate::Result<()>
  where
    S: Serialize + Clone,
    F: Fn(&EventTarget) -> bool,
  {
    self.manager().emit_filter(event, payload, filter)
  }
}

impl<R: Runtime> Manager<R> for WebviewWindow<R> {
  fn resources_table(&self) -> MutexGuard<'_, ResourceTable> {
    self
      .webview
      .resources_table
      .lock()
      .expect("poisoned window resources table")
  }
}

impl<R: Runtime> ManagerBase<R> for WebviewWindow<R> {
  fn manager(&self) -> &AppManager<R> {
    self.webview.manager()
  }

  fn manager_owned(&self) -> Arc<AppManager<R>> {
    self.webview.manager_owned()
  }

  fn runtime(&self) -> RuntimeOrDispatch<'_, R> {
    self.webview.runtime()
  }

  fn managed_app_handle(&self) -> &AppHandle<R> {
    self.webview.managed_app_handle()
  }
}
