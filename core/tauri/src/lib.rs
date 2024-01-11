// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/tauri/raw/dev/.github/splash.png)](https://tauri.app)
//!
//! Tauri is a framework for building tiny, blazing fast binaries for all major desktop platforms.
//! Developers can integrate any front-end framework that compiles to HTML, JS and CSS for building their user interface.
//! The backend of the application is a rust-sourced binary with an API that the front-end can interact with.
//!
//! # Cargo features
//!
//! The following are a list of [Cargo features](https://doc.rust-lang.org/stable/cargo/reference/manifest.html#the-features-section) that can be enabled or disabled:
//!
//! - **wry** *(enabled by default)*: Enables the [wry](https://github.com/tauri-apps/wry) runtime. Only disable it if you want a custom runtime.
//! - **tracing**: Enables [`tracing`](https://docs.rs/tracing/latest/tracing) for window startup, plugins, `Window::eval`, events, IPC, updater and custom protocol request handlers.
//! - **test**: Enables the [`test`] module exposing unit test helpers.
//! - **objc-exception**: Wrap each msg_send! in a @try/@catch and panics if an exception is caught, preventing Objective-C from unwinding into Rust.
//! - **linux-ipc-protocol**: Use custom protocol for faster IPC on Linux. Requires webkit2gtk v2.40 or above.
//! - **linux-libxdo**: Enables linking to libxdo which enables Cut, Copy, Paste and SelectAll menu items to work on Linux.
//! - **isolation**: Enables the isolation pattern. Enabled by default if the `tauri > pattern > use` config option is set to `isolation` on the `tauri.conf.json` file.
//! - **custom-protocol**: Feature managed by the Tauri CLI. When enabled, Tauri assumes a production environment instead of a development one.
//! - **devtools**: Enables the developer tools (Web inspector) and [`Window::open_devtools`]. Enabled by default on debug builds.
//! On macOS it uses private APIs, so you can't enable it if your app will be published to the App Store.
//! - **native-tls**: Provides TLS support to connect over HTTPS.
//! - **native-tls-vendored**: Compile and statically link to a vendored copy of OpenSSL.
//! - **rustls-tls**: Provides TLS support to connect over HTTPS using rustls.
//! - **process-relaunch-dangerous-allow-symlink-macos**: Allows the [`process::current_binary`] function to allow symlinks on macOS (this is dangerous, see the Security section in the documentation website).
//! - **tray-icon**: Enables application tray icon APIs. Enabled by default if the `trayIcon` config is defined on the `tauri.conf.json` file.
//! - **macos-private-api**: Enables features only available in **macOS**'s private APIs, currently the `transparent` window functionality and the `fullScreenEnabled` preference setting to `true`. Enabled by default if the `tauri > macosPrivateApi` config flag is set to `true` on the `tauri.conf.json` file.
//! - **window-data-url**: Enables usage of data URLs on the webview.
//! - **compression** *(enabled by default): Enables asset compression. You should only disable this if you want faster compile times in release builds - it produces larger binaries.
//! - **config-json5**: Adds support to JSON5 format for `tauri.conf.json`.
//! - **config-toml**: Adds support to TOML format for the configuration `Tauri.toml`.
//! - **icon-ico**: Adds support to set `.ico` window icons. Enables [`Icon::File`] and [`Icon::Raw`] variants.
//! - **icon-png**: Adds support to set `.png` window icons. Enables [`Icon::File`] and [`Icon::Raw`] variants.
//!
//! ## Cargo allowlist features
//!
//! The following are a list of [Cargo features](https://doc.rust-lang.org/stable/cargo/reference/manifest.html#the-features-section) that enables commands for Tauri's API package.
//! These features are automatically enabled by the Tauri CLI based on the `allowlist` configuration under `tauri.conf.json`.
//!
//! ### Protocol allowlist
//!
//! - **protocol-asset**: Enables the `asset` custom protocol.

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]
#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Setups the binding that initializes an iOS plugin.
#[cfg(target_os = "ios")]
#[macro_export]
macro_rules! ios_plugin_binding {
  ($fn_name: ident) => {
    tauri::swift_rs::swift!(fn $fn_name() -> *const ::std::ffi::c_void);
  }
}
#[cfg(target_os = "ios")]
#[doc(hidden)]
pub use cocoa;
#[cfg(target_os = "macos")]
#[doc(hidden)]
pub use embed_plist;
pub use error::{Error, Result};
pub use resources::{Resource, ResourceId, ResourceTable};
#[cfg(target_os = "ios")]
#[doc(hidden)]
pub use swift_rs;
#[cfg(mobile)]
pub use tauri_macros::mobile_entry_point;
pub use tauri_macros::{command, generate_handler};

pub(crate) mod app;
pub mod async_runtime;
pub mod command;
mod error;
mod event;
pub mod ipc;
mod manager;
mod pattern;
pub mod plugin;
pub(crate) mod protocol;
mod resources;
mod vibrancy;
pub mod window;
use tauri_runtime as runtime;
#[cfg(target_os = "ios")]
mod ios;
#[cfg(desktop)]
pub mod menu;
/// Path APIs.
pub mod path;
pub mod process;
/// The allowlist scopes.
pub mod scope;
mod state;

#[cfg(all(desktop, feature = "tray-icon"))]
#[cfg_attr(docsrs, doc(cfg(all(desktop, feature = "tray-icon"))))]
pub mod tray;
pub use tauri_utils as utils;

pub use http;

/// A Tauri [`Runtime`] wrapper around wry.
#[cfg(feature = "wry")]
#[cfg_attr(docsrs, doc(cfg(feature = "wry")))]
pub type Wry = tauri_runtime_wry::Wry<EventLoopMessage>;
#[cfg(feature = "wry")]
#[cfg_attr(docsrs, doc(cfg(feature = "wry")))]
pub type WryHandle = tauri_runtime_wry::WryHandle<EventLoopMessage>;

#[cfg(all(feature = "wry", target_os = "android"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "wry", target_os = "android"))))]
#[doc(hidden)]
#[macro_export]
macro_rules! android_binding {
  ($domain:ident, $package:ident, $main: ident, $wry: path) => {
    use $wry::{
      android_setup,
      prelude::{JClass, JNIEnv, JString},
    };

    ::tauri::wry::android_binding!($domain, $package, $wry);

    ::tauri::tao::android_binding!(
      $domain,
      $package,
      WryActivity,
      android_setup,
      $main,
      ::tauri::tao
    );

    ::tauri::tao::platform::android::prelude::android_fn!(
      app_tauri,
      plugin,
      PluginManager,
      handlePluginResponse,
      [i32, JString, JString],
    );
    ::tauri::tao::platform::android::prelude::android_fn!(
      app_tauri,
      plugin,
      PluginManager,
      sendChannelData,
      [i64, JString],
    );

    // this function is a glue between PluginManager.kt > handlePluginResponse and Rust
    #[allow(non_snake_case)]
    pub fn handlePluginResponse(
      mut env: JNIEnv,
      _: JClass,
      id: i32,
      success: JString,
      error: JString,
    ) {
      ::tauri::handle_android_plugin_response(&mut env, id, success, error);
    }

    // this function is a glue between PluginManager.kt > sendChannelData and Rust
    #[allow(non_snake_case)]
    pub fn sendChannelData(mut env: JNIEnv, _: JClass, id: i64, data: JString) {
      ::tauri::send_channel_data(&mut env, id, data);
    }
  };
}

#[cfg(all(feature = "wry", target_os = "android"))]
#[doc(hidden)]
pub use plugin::mobile::{handle_android_plugin_response, send_channel_data};
#[cfg(all(feature = "wry", target_os = "android"))]
#[doc(hidden)]
pub use tauri_runtime_wry::{tao, wry};

/// A task to run on the main thread.
pub type SyncTask = Box<dyn FnOnce() + Send>;

use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap,
  fmt::{self, Debug},
  sync::MutexGuard,
};
use utils::acl::resolved::Resolved;

#[cfg(feature = "wry")]
#[cfg_attr(docsrs, doc(cfg(feature = "wry")))]
pub use tauri_runtime_wry::webview_version;

#[cfg(target_os = "macos")]
#[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
pub use runtime::ActivationPolicy;

#[cfg(target_os = "macos")]
pub use self::utils::TitleBarStyle;

pub use self::event::{Event, EventId};
pub use {
  self::app::{App, AppHandle, AssetResolver, Builder, CloseRequestApi, RunEvent, WindowEvent},
  self::manager::Asset,
  self::runtime::{
    webview::WebviewAttributes,
    window::{
      dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Pixel, Position, Size},
      CursorIcon, FileDropEvent,
    },
    DeviceEventFilter, RunIteration, UserAttentionType,
  },
  self::state::{State, StateManager},
  self::utils::{
    assets::Assets,
    config::{Config, WindowUrl},
    Env, PackageInfo, Theme,
  },
  self::window::{Monitor, Window, WindowBuilder},
  scope::*,
};

/// The Tauri version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(target_os = "ios")]
#[doc(hidden)]
pub fn log_stdout() {
  use std::{
    ffi::CString,
    fs::File,
    io::{BufRead, BufReader},
    os::unix::prelude::*,
    thread,
  };

  let mut logpipe: [RawFd; 2] = Default::default();
  unsafe {
    libc::pipe(logpipe.as_mut_ptr());
    libc::dup2(logpipe[1], libc::STDOUT_FILENO);
    libc::dup2(logpipe[1], libc::STDERR_FILENO);
  }
  thread::spawn(move || unsafe {
    let file = File::from_raw_fd(logpipe[0]);
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    loop {
      buffer.clear();
      if let Ok(len) = reader.read_line(&mut buffer) {
        if len == 0 {
          break;
        } else if let Ok(msg) = CString::new(buffer.as_bytes())
          .map_err(|_| ())
          .and_then(|c| c.into_string().map_err(|_| ()))
        {
          log::info!("{}", msg);
        }
      }
    }
  });
}

/// The user event type.
#[derive(Debug, Clone)]
pub enum EventLoopMessage {
  /// An event from a menu item, could be on the window menu bar, application menu bar (on macOS) or tray icon menu.
  #[cfg(desktop)]
  MenuEvent(menu::MenuEvent),
  /// An event from a menu item, could be on the window menu bar, application menu bar (on macOS) or tray icon menu.
  #[cfg(all(desktop, feature = "tray-icon"))]
  #[cfg_attr(docsrs, doc(cfg(all(desktop, feature = "tray-icon"))))]
  TrayIconEvent(tray::TrayIconEvent),
}

/// The webview runtime interface. A wrapper around [`runtime::Runtime`] with the proper user event type associated.
pub trait Runtime: runtime::Runtime<EventLoopMessage> {}
pub trait RuntimeHandle: runtime::RuntimeHandle<EventLoopMessage> {}

impl<W: runtime::Runtime<EventLoopMessage>> Runtime for W {}
impl<R: runtime::RuntimeHandle<EventLoopMessage>> RuntimeHandle for R {}

/// Reads the config file at compile time and generates a [`Context`] based on its content.
///
/// The default config file path is a `tauri.conf.json` file inside the Cargo manifest directory of
/// the crate being built.
///
/// # Custom Config Path
///
/// You may pass a string literal to this macro to specify a custom path for the Tauri config file.
/// If the path is relative, it will be search for relative to the Cargo manifest of the compiling
/// crate.
///
/// # Note
///
/// This macro should not be called if you are using [`tauri-build`] to generate the context from
/// inside your build script as it will just cause excess computations that will be discarded. Use
/// either the [`tauri-build`] method or this macro - not both.
///
/// [`tauri-build`]: https://docs.rs/tauri-build
pub use tauri_macros::generate_context;

/// Include a [`Context`] that was generated by [`tauri-build`] inside your build script.
///
/// You should either use [`tauri-build`] and this macro to include the compile time generated code,
/// or [`generate_context!`]. Do not use both at the same time, as they generate the same code and
/// will cause excess computations that will be discarded.
///
/// [`tauri-build`]: https://docs.rs/tauri-build
#[macro_export]
macro_rules! tauri_build_context {
  () => {
    include!(concat!(env!("OUT_DIR"), "/tauri-build-context.rs"))
  };
}

pub use pattern::Pattern;

/// A icon definition.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Icon {
  /// Icon from file path.
  #[cfg(any(feature = "icon-ico", feature = "icon-png"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "icon-ico", feature = "icon-png"))))]
  File(std::path::PathBuf),
  /// Icon from raw RGBA bytes. Width and height is parsed at runtime.
  #[cfg(any(feature = "icon-ico", feature = "icon-png"))]
  #[cfg_attr(docsrs, doc(cfg(any(feature = "icon-ico", feature = "icon-png"))))]
  Raw(Vec<u8>),
  /// Icon from raw RGBA bytes.
  Rgba {
    /// RGBA bytes of the icon image.
    rgba: Vec<u8>,
    /// Icon width.
    width: u32,
    /// Icon height.
    height: u32,
  },
}

impl TryFrom<Icon> for runtime::Icon {
  type Error = Error;

  fn try_from(icon: Icon) -> Result<Self> {
    #[allow(irrefutable_let_patterns)]
    if let Icon::Rgba {
      rgba,
      width,
      height,
    } = icon
    {
      Ok(Self {
        rgba,
        width,
        height,
      })
    } else {
      #[cfg(not(any(feature = "icon-ico", feature = "icon-png")))]
      panic!("unexpected Icon variant");
      #[cfg(any(feature = "icon-ico", feature = "icon-png"))]
      {
        let bytes = match icon {
          Icon::File(p) => std::fs::read(p)?,
          Icon::Raw(r) => r,
          Icon::Rgba { .. } => unreachable!(),
        };
        let extension = infer::get(&bytes)
          .expect("could not determine icon extension")
          .extension();
        match extension {
        #[cfg(feature = "icon-ico")]
        "ico" => {
          let icon_dir = ico::IconDir::read(std::io::Cursor::new(bytes))?;
          let entry = &icon_dir.entries()[0];
          Ok(Self {
            rgba: entry.decode()?.rgba_data().to_vec(),
            width: entry.width(),
            height: entry.height(),
          })
        }
        #[cfg(feature = "icon-png")]
        "png" => {
          let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
          let mut reader = decoder.read_info()?;
          let mut buffer = Vec::new();
          while let Ok(Some(row)) = reader.next_row() {
            buffer.extend(row.data());
          }
          Ok(Self {
            rgba: buffer,
            width: reader.info().width,
            height: reader.info().height,
          })
        }
        _ => panic!(
          "image `{extension}` extension not supported; please file a Tauri feature request. `png` or `ico` icons are supported with the `icon-png` and `icon-ico` feature flags"
        ),
      }
      }
    }
  }
}

/// User supplied data required inside of a Tauri application.
///
/// # Stability
/// This is the output of the [`generate_context`] macro, and is not considered part of the stable API.
/// Unless you know what you are doing and are prepared for this type to have breaking changes, do not create it yourself.
pub struct Context<A: Assets> {
  pub(crate) config: Config,
  pub(crate) assets: Box<A>,
  pub(crate) default_window_icon: Option<Icon>,
  pub(crate) app_icon: Option<Vec<u8>>,
  #[cfg(all(desktop, feature = "tray-icon"))]
  pub(crate) tray_icon: Option<Icon>,
  pub(crate) package_info: PackageInfo,
  pub(crate) _info_plist: (),
  pub(crate) pattern: Pattern,
  pub(crate) resolved_acl: Resolved,
}

impl<A: Assets> fmt::Debug for Context<A> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut d = f.debug_struct("Context");
    d.field("config", &self.config)
      .field("default_window_icon", &self.default_window_icon)
      .field("app_icon", &self.app_icon)
      .field("package_info", &self.package_info)
      .field("pattern", &self.pattern);

    #[cfg(all(desktop, feature = "tray-icon"))]
    d.field("tray_icon", &self.tray_icon);

    d.finish()
  }
}

impl<A: Assets> Context<A> {
  /// The config the application was prepared with.
  #[inline(always)]
  pub fn config(&self) -> &Config {
    &self.config
  }

  /// A mutable reference to the config the application was prepared with.
  #[inline(always)]
  pub fn config_mut(&mut self) -> &mut Config {
    &mut self.config
  }

  /// The assets to be served directly by Tauri.
  #[inline(always)]
  pub fn assets(&self) -> &A {
    &self.assets
  }

  /// A mutable reference to the assets to be served directly by Tauri.
  #[inline(always)]
  pub fn assets_mut(&mut self) -> &mut A {
    &mut self.assets
  }

  /// The default window icon Tauri should use when creating windows.
  #[inline(always)]
  pub fn default_window_icon(&self) -> Option<&Icon> {
    self.default_window_icon.as_ref()
  }

  /// A mutable reference to the default window icon Tauri should use when creating windows.
  #[inline(always)]
  pub fn default_window_icon_mut(&mut self) -> &mut Option<Icon> {
    &mut self.default_window_icon
  }

  /// The icon to use on the system tray UI.
  #[cfg(all(desktop, feature = "tray-icon"))]
  #[cfg_attr(docsrs, doc(cfg(all(desktop, feature = "tray-icon"))))]
  #[inline(always)]
  pub fn tray_icon(&self) -> Option<&Icon> {
    self.tray_icon.as_ref()
  }

  /// A mutable reference to the icon to use on the tray icon.
  #[cfg(all(desktop, feature = "tray-icon"))]
  #[cfg_attr(docsrs, doc(cfg(all(desktop, feature = "tray-icon"))))]
  #[inline(always)]
  pub fn tray_icon_mut(&mut self) -> &mut Option<Icon> {
    &mut self.tray_icon
  }

  /// Package information.
  #[inline(always)]
  pub fn package_info(&self) -> &PackageInfo {
    &self.package_info
  }

  /// A mutable reference to the package information.
  #[inline(always)]
  pub fn package_info_mut(&mut self) -> &mut PackageInfo {
    &mut self.package_info
  }

  /// The application pattern.
  #[inline(always)]
  pub fn pattern(&self) -> &Pattern {
    &self.pattern
  }

  /// Create a new [`Context`] from the minimal required items.
  #[inline(always)]
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    config: Config,
    assets: Box<A>,
    default_window_icon: Option<Icon>,
    app_icon: Option<Vec<u8>>,
    package_info: PackageInfo,
    info_plist: (),
    pattern: Pattern,
    resolved_acl: Resolved,
  ) -> Self {
    Self {
      config,
      assets,
      default_window_icon,
      app_icon,
      #[cfg(all(desktop, feature = "tray-icon"))]
      tray_icon: None,
      package_info,
      _info_plist: info_plist,
      pattern,
      resolved_acl,
    }
  }

  /// Sets the app tray icon.
  #[cfg(all(desktop, feature = "tray-icon"))]
  #[cfg_attr(docsrs, doc(cfg(all(desktop, feature = "tray-icon"))))]
  #[inline(always)]
  pub fn set_tray_icon(&mut self, icon: Icon) {
    self.tray_icon.replace(icon);
  }

  /// Sets the app shell scope.
  #[cfg(shell_scope)]
  #[inline(always)]
  pub fn set_shell_scope(&mut self, scope: scope::ShellScopeConfig) {
    self.shell_scope = scope;
  }
}

// TODO: expand these docs
/// Manages a running application.
pub trait Manager<R: Runtime>: sealed::ManagerBase<R> {
  /// The application handle associated with this manager.
  fn app_handle(&self) -> &AppHandle<R> {
    self.managed_app_handle()
  }

  /// The [`Config`] the manager was created with.
  fn config(&self) -> &Config {
    self.manager().config()
  }

  /// The [`PackageInfo`] the manager was created with.
  fn package_info(&self) -> &PackageInfo {
    self.manager().package_info()
  }

  /// Listen to an event emitted on any window.
  ///
  /// # Examples
  /// ```
  /// use tauri::Manager;
  ///
  /// #[tauri::command]
  /// fn synchronize(window: tauri::Window) {
  ///   // emits the synchronized event to all windows
  ///   window.emit("synchronized", ());
  /// }
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     app.listen_global("synchronized", |event| {
  ///       println!("app is in sync");
  ///     });
  ///     Ok(())
  ///   })
  ///   .invoke_handler(tauri::generate_handler![synchronize]);
  /// ```
  fn listen_global<F>(&self, event: impl Into<String>, handler: F) -> EventId
  where
    F: Fn(Event) + Send + 'static,
  {
    self.manager().listen(event.into(), None, handler)
  }

  /// Remove an event listener.
  ///
  /// # Examples
  /// ```
  /// use tauri::Manager;
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let handle = app.handle().clone();
  ///     let handler = app.listen_global("ready", move |event| {
  ///       println!("app is ready");
  ///
  ///       // we no longer need to listen to the event
  ///       // we also could have used `app.once_global` instead
  ///       handle.unlisten(event.id());
  ///     });
  ///
  ///     // stop listening to the event when you do not need it anymore
  ///     app.unlisten(handler);
  ///
  ///
  ///     Ok(())
  ///   });
  /// ```
  fn unlisten(&self, id: EventId) {
    self.manager().unlisten(id)
  }

  /// Listen to a global event only once.
  ///
  /// See [`Self::listen_global`] for more information.
  fn once_global<F>(&self, event: impl Into<String>, handler: F)
  where
    F: FnOnce(Event) + Send + 'static,
  {
    self.manager().once(event.into(), None, handler)
  }

  /// Emits an event to all windows.
  ///
  /// If using [`Window`] to emit the event, it will be used as the source.
  ///
  /// # Examples
  /// ```
  /// use tauri::Manager;
  ///
  /// #[tauri::command]
  /// fn synchronize(app: tauri::AppHandle) {
  ///   // emits the synchronized event to all windows
  ///   app.emit("synchronized", ());
  /// }
  /// ```
  #[cfg_attr(
    feature = "tracing",
    tracing::instrument("app::emit", skip(self, payload))
  )]
  fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<()> {
    self.manager().emit(event, None, payload)
  }

  /// Emits an event to the window with the specified label.
  ///
  /// If using [`Window`] to emit the event, it will be used as the source.
  ///
  /// # Examples
  /// ```
  /// use tauri::Manager;
  ///
  /// #[tauri::command]
  /// fn download(app: tauri::AppHandle) {
  ///   for i in 1..100 {
  ///     std::thread::sleep(std::time::Duration::from_millis(150));
  ///     // emit a download progress event to the updater window
  ///     app.emit_to("updater", "download-progress", i);
  ///   }
  /// }
  /// ```
  #[cfg_attr(
    feature = "tracing",
    tracing::instrument("app::emit::to", skip(self, payload))
  )]
  fn emit_to<S: Serialize + Clone>(&self, label: &str, event: &str, payload: S) -> Result<()> {
    self
      .manager()
      .emit_filter(event, None, payload, |w| label == w.label())
  }

  /// Emits an event to specific windows based on a filter.
  ///
  /// If using [`Window`] to emit the event, it will be used as the source.
  ///
  /// # Examples
  /// ```
  /// use tauri::Manager;
  ///
  /// #[tauri::command]
  /// fn download(app: tauri::AppHandle) {
  ///   for i in 1..100 {
  ///     std::thread::sleep(std::time::Duration::from_millis(150));
  ///     // emit a download progress event to the updater window
  ///     app.emit_filter("download-progress", i, |w| w.label() == "main" );
  ///   }
  /// }
  /// ```
  #[cfg_attr(
    feature = "tracing",
    tracing::instrument("app::emit::filter", skip(self, payload, filter))
  )]
  fn emit_filter<S, F>(&self, event: &str, payload: S, filter: F) -> Result<()>
  where
    S: Serialize + Clone,
    F: Fn(&Window<R>) -> bool,
  {
    self.manager().emit_filter(event, None, payload, filter)
  }

  /// Fetch a single window from the manager.
  fn get_window(&self, label: &str) -> Option<Window<R>> {
    self.manager().get_window(label)
  }
  /// Fetch the focused window. Returns `None` if there is not any focused window.
  fn get_focused_window(&self) -> Option<Window<R>> {
    self.manager().get_focused_window()
  }

  /// Fetch all managed windows.
  fn windows(&self) -> HashMap<String, Window<R>> {
    self.manager().windows()
  }

  /// Add `state` to the state managed by the application.
  ///
  /// If the state for the `T` type has previously been set, the state is unchanged and false is returned. Otherwise true is returned.
  ///
  /// Managed state can be retrieved by any command handler via the
  /// [`State`] guard. In particular, if a value of type `T`
  /// is managed by Tauri, adding `State<T>` to the list of arguments in a
  /// command handler instructs Tauri to retrieve the managed value.
  /// Additionally, [`state`](Self#method.state) can be used to retrieve the value manually.
  ///
  /// # Mutability
  ///
  /// Since the managed state is global and must be [`Send`] + [`Sync`], mutations can only happen through interior mutability:
  ///
  /// ```rust,no_run
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
  /// ```rust,no_run
  /// use tauri::{Manager, State};
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
  ///   .setup(|app| {
  ///     app.manage(MyInt(0));
  ///     app.manage(MyString("tauri".into()));
  ///     // `MyInt` is already managed, so `manage()` returns false
  ///     assert!(!app.manage(MyInt(1)));
  ///     // read the `MyInt` managed state with the turbofish syntax
  ///     let int = app.state::<MyInt>();
  ///     assert_eq!(int.0, 0);
  ///     // read the `MyString` managed state with the `State` guard
  ///     let val: State<MyString> = app.state();
  ///     assert_eq!(val.0, "tauri");
  ///     Ok(())
  ///   })
  ///   .invoke_handler(tauri::generate_handler![int_command, string_command])
  ///   // on an actual app, remove the string argument
  ///   .run(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
  ///   .expect("error while running tauri application");
  /// ```
  fn manage<T>(&self, state: T) -> bool
  where
    T: Send + Sync + 'static,
  {
    self.manager().state().set(state)
  }

  /// Retrieves the managed state for the type `T`.
  ///
  /// # Panics
  ///
  /// Panics if the state for the type `T` has not been previously [managed](Self::manage).
  /// Use [try_state](Self::try_state) for a non-panicking version.
  fn state<T>(&self) -> State<'_, T>
  where
    T: Send + Sync + 'static,
  {
    self
      .manager()
      .state
      .try_get()
      .expect("state() called before manage() for given type")
  }

  /// Attempts to retrieve the managed state for the type `T`.
  ///
  /// Returns `Some` if the state has previously been [managed](Self::manage). Otherwise returns `None`.
  fn try_state<T>(&self) -> Option<State<'_, T>>
  where
    T: Send + Sync + 'static,
  {
    self.manager().state.try_get()
  }

  /// Get a reference to the resources table.
  fn resources_table(&self) -> MutexGuard<'_, ResourceTable> {
    self.manager().resources_table()
  }

  /// Gets the managed [`Env`].
  fn env(&self) -> Env {
    self.state::<Env>().inner().clone()
  }

  /// Gets the scope for the IPC.
  fn ipc_scope(&self) -> scope::ipc::Scope {
    self.state::<Scopes>().inner().ipc.clone()
  }

  /// Gets the scope for the asset protocol.
  #[cfg(feature = "protocol-asset")]
  fn asset_protocol_scope(&self) -> scope::fs::Scope {
    self.state::<Scopes>().inner().asset_protocol.clone()
  }

  /// The path resolver.
  fn path(&self) -> &crate::path::PathResolver<R> {
    self.state::<crate::path::PathResolver<R>>().inner()
  }
}

/// Prevent implementation details from leaking out of the [`Manager`] trait.
pub(crate) mod sealed {
  use super::Runtime;
  use crate::{app::AppHandle, manager::AppManager};
  use std::sync::Arc;

  /// A running [`Runtime`] or a dispatcher to it.
  pub enum RuntimeOrDispatch<'r, R: Runtime> {
    /// Reference to the running [`Runtime`].
    Runtime(&'r R),

    /// Handle to the running [`Runtime`].
    RuntimeHandle(R::Handle),

    /// A dispatcher to the running [`Runtime`].
    Dispatch(R::Dispatcher),
  }

  /// Managed handle to the application runtime.
  pub trait ManagerBase<R: Runtime> {
    fn manager(&self) -> &AppManager<R>;
    fn manager_owned(&self) -> Arc<AppManager<R>>;
    fn runtime(&self) -> RuntimeOrDispatch<'_, R>;
    fn managed_app_handle(&self) -> &AppHandle<R>;
  }
}

#[cfg(any(test, feature = "test"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test")))]
pub mod test;

#[cfg(test)]
mod tests {
  use cargo_toml::Manifest;
  use std::{env::var, fs::read_to_string, path::PathBuf, sync::OnceLock};

  static MANIFEST: OnceLock<Manifest> = OnceLock::new();
  const CHECKED_FEATURES: &str = include_str!(concat!(env!("OUT_DIR"), "/checked_features"));

  fn get_manifest() -> &'static Manifest {
    MANIFEST.get_or_init(|| {
      let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap());
      Manifest::from_path(manifest_dir.join("Cargo.toml")).expect("failed to parse Cargo manifest")
    })
  }

  #[test]
  fn features_are_documented() {
    let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap());
    let lib_code = read_to_string(manifest_dir.join("src/lib.rs")).expect("failed to read lib.rs");

    for f in get_manifest().features.keys() {
      if !(f.starts_with("__") || f == "default" || lib_code.contains(&format!("*{f}**"))) {
        panic!("Feature {f} is not documented");
      }
    }
  }

  #[test]
  fn aliased_features_exist() {
    let checked_features = CHECKED_FEATURES.split(',');
    let manifest = get_manifest();
    for checked_feature in checked_features {
      if !manifest.features.iter().any(|(f, _)| f == checked_feature) {
        panic!(
          "Feature {checked_feature} was checked in the alias build step but it does not exist in core/tauri/Cargo.toml"
        );
      }
    }
  }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum IconDto {
  #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
  File(std::path::PathBuf),
  #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
  Raw(Vec<u8>),
  Rgba {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
  },
}

impl From<IconDto> for Icon {
  fn from(icon: IconDto) -> Self {
    match icon {
      #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
      IconDto::File(path) => Self::File(path),
      #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
      IconDto::Raw(raw) => Self::Raw(raw),
      IconDto::Rgba {
        rgba,
        width,
        height,
      } => Self::Rgba {
        rgba,
        width,
        height,
      },
    }
  }
}

#[allow(unused)]
macro_rules! run_main_thread {
  ($self:ident, $ex:expr) => {{
    use std::sync::mpsc::channel;
    let (tx, rx) = channel();
    let self_ = $self.clone();
    let task = move || {
      let f = $ex;
      let _ = tx.send(f(self_));
    };
    $self.app_handle.run_on_main_thread(Box::new(task))?;
    rx.recv().map_err(|_| crate::Error::FailedToReceiveMessage)
  }};
}

#[allow(unused)]
pub(crate) use run_main_thread;

#[cfg(test)]
mod test_utils {
  use proptest::prelude::*;

  pub fn assert_send<T: Send>() {}
  pub fn assert_sync<T: Sync>() {}

  #[allow(dead_code)]
  pub fn assert_not_allowlist_error<T>(res: anyhow::Result<T>) {
    if let Err(e) = res {
      assert!(!e.to_string().contains("not on the allowlist"));
    }
  }

  proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    // check to see if spawn executes a function.
    fn check_spawn_task(task in "[a-z]+") {
      // create dummy task function
      let dummy_task = async move {
        format!("{task}-run-dummy-task");
      };
      // call spawn
      crate::async_runtime::spawn(dummy_task);
    }
  }
}
