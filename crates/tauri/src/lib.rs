// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Tauri is a framework for building tiny, blazing fast binaries for all major desktop platforms.
//! Developers can integrate any front-end framework that compiles to HTML, JS and CSS for building their user interface.
//! The backend of the application is a rust-sourced binary with an API that the front-end can interact with.
//!
//! # Cargo features
//!
//! The following are a list of [Cargo features](https://doc.rust-lang.org/stable/cargo/reference/manifest.html#the-features-section) that can be enabled or disabled:
//!
//! - **wry** *(enabled by default)*: Enables the [wry](https://github.com/tauri-apps/wry) runtime. Only disable it if you want a custom runtime.
//! - **common-controls-v6** *(enabled by default)*: Enables [Common Controls v6](https://learn.microsoft.com/en-us/windows/win32/controls/common-control-versions) support on Windows, mainly for the predefined `about` menu item.
//! - **x11** *(enabled by default)*: Enables X11 support. Disable this if you only target Wayland.
//! - **unstable**: Enables unstable features. Be careful, it might introduce breaking changes in future minor releases.
//! - **tracing**: Enables [`tracing`](https://docs.rs/tracing/latest/tracing) for window startup, plugins, `Window::eval`, events, IPC, updater and custom protocol request handlers.
//! - **test**: Enables the [`mod@test`] module exposing unit test helpers.
//! - **objc-exception**: This feature flag is no-op since 2.3.0.
//! - **linux-libxdo**: Enables linking to libxdo which enables Cut, Copy, Paste and SelectAll menu items to work on Linux.
//! - **isolation**: Enables the isolation pattern. Enabled by default if the `app > security > pattern > use` config option is set to `isolation` on the `tauri.conf.json` file.
//! - **custom-protocol**: Feature managed by the Tauri CLI. When enabled, Tauri assumes a production environment instead of a development one.
//! - **devtools**: Enables the developer tools (Web inspector) and [`window::Window#method.open_devtools`]. Enabled by default on debug builds.
//!   On macOS it uses private APIs, so you can't enable it if your app will be published to the App Store.
//! - **native-tls**: Provides TLS support to connect over HTTPS.
//! - **native-tls-vendored**: Compile and statically link to a vendored copy of OpenSSL.
//! - **rustls-tls**: Provides TLS support to connect over HTTPS using rustls.
//! - **process-relaunch-dangerous-allow-symlink-macos**: Allows the [`process::current_binary`] function to allow symlinks on macOS (this is dangerous, see the Security section in the documentation website).
//! - **tray-icon**: Enables application tray icon APIs. Enabled by default if the `trayIcon` config is defined on the `tauri.conf.json` file.
//! - **macos-private-api**: Enables features only available in **macOS**'s private APIs, currently the `transparent` window functionality and the `fullScreenEnabled` preference setting to `true`. Enabled by default if the `tauri > macosPrivateApi` config flag is set to `true` on the `tauri.conf.json` file.
//! - **webview-data-url**: Enables usage of data URLs on the webview.
//! - **compression** *(enabled by default): Enables asset compression. You should only disable this if you want faster compile times in release builds - it produces larger binaries.
//! - **config-json5**: Adds support to JSON5 format for `tauri.conf.json`.
//! - **config-toml**: Adds support to TOML format for the configuration `Tauri.toml`.
//! - **image-ico**: Adds support to parse `.ico` image, see [`Image`].
//! - **image-png**: Adds support to parse `.png` image, see [`Image`].
//! - **macos-proxy**: Adds support for [`WebviewBuilder::proxy_url`] on macOS. Requires macOS 14+.
//! - **specta**: Add support for [`specta::specta`](https://docs.rs/specta/%5E2.0.0-rc.9/specta/attr.specta.html) with Tauri arguments such as [`State`](crate::State), [`Window`](crate::Window) and [`AppHandle`](crate::AppHandle)
//! - **dynamic-acl** *(enabled by default)*: Enables you to add ACLs at runtime, notably it enables the [`Manager::add_capability`] function.
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
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
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
#[cfg(target_os = "macos")]
#[doc(hidden)]
pub use embed_plist;
pub use error::{Error, Result};
use ipc::RuntimeAuthority;
#[cfg(feature = "dynamic-acl")]
use ipc::RuntimeCapability;
pub use resources::{Resource, ResourceId, ResourceTable};
#[cfg(target_os = "ios")]
#[doc(hidden)]
pub use swift_rs;
pub use tauri_macros::include_image;
#[cfg(mobile)]
pub use tauri_macros::mobile_entry_point;
pub use tauri_macros::{command, generate_handler};

use tauri_utils::assets::AssetsIter;
pub use url::Url;

pub(crate) mod app;
pub mod async_runtime;
mod error;
mod event;
pub mod ipc;
mod manager;
mod pattern;
pub mod plugin;
pub(crate) mod protocol;
mod resources;
mod vibrancy;
pub mod webview;
pub mod window;
use tauri_runtime as runtime;
pub mod image;
#[cfg(target_os = "ios")]
mod ios;
#[cfg(desktop)]
#[cfg_attr(docsrs, doc(cfg(desktop)))]
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
/// A Tauri [`RuntimeHandle`] wrapper around wry.
#[cfg(feature = "wry")]
#[cfg_attr(docsrs, doc(cfg(feature = "wry")))]
pub type WryHandle = tauri_runtime_wry::WryHandle<EventLoopMessage>;

#[cfg(all(feature = "wry", target_os = "android"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "wry", target_os = "android"))))]
#[doc(hidden)]
#[macro_export]
macro_rules! android_binding {
  ($domain:ident, $app_name:ident, $main:ident, $wry:path) => {
    use $wry::{
      android_setup,
      prelude::{JClass, JNIEnv, JString},
    };

    ::tauri::wry::android_binding!($domain, $app_name, $wry);

    ::tauri::tao::android_binding!(
      $domain,
      $app_name,
      WryActivity,
      android_setup,
      $main,
      ::tauri::tao
    );

    // be careful when renaming this, the `Java_app_tauri_plugin_PluginManager_handlePluginResponse` symbol is checked by the CLI
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

use serde::Serialize;
use std::{
  borrow::Cow,
  collections::HashMap,
  fmt::{self, Debug},
  sync::MutexGuard,
};
use utils::assets::{AssetKey, CspHash, EmbeddedAssets};

#[cfg(feature = "wry")]
#[cfg_attr(docsrs, doc(cfg(feature = "wry")))]
pub use tauri_runtime_wry::webview_version;

#[cfg(target_os = "macos")]
#[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
pub use runtime::ActivationPolicy;

pub use self::utils::TitleBarStyle;

use self::event::EventName;
pub use self::event::{Event, EventId, EventTarget};
use self::manager::EmitPayload;
pub use {
  self::app::{
    App, AppHandle, AssetResolver, Builder, CloseRequestApi, ExitRequestApi, RunEvent,
    UriSchemeContext, UriSchemeResponder, WebviewEvent, WindowEvent, RESTART_EXIT_CODE,
  },
  self::manager::Asset,
  self::runtime::{
    dpi::{
      LogicalPosition, LogicalRect, LogicalSize, LogicalUnit, PhysicalPosition, PhysicalRect,
      PhysicalSize, PhysicalUnit, Pixel, PixelUnit, Position, Rect, Size,
    },
    window::{CursorIcon, DragDropEvent, WindowSizeConstraints},
    DeviceEventFilter, UserAttentionType,
  },
  self::state::{State, StateManager},
  self::utils::{
    config::{Config, WebviewUrl},
    Env, PackageInfo, Theme,
  },
  self::webview::{Webview, WebviewWindow, WebviewWindowBuilder},
  self::window::{Monitor, Window},
  scope::*,
};

#[cfg(feature = "unstable")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
pub use {self::webview::WebviewBuilder, self::window::WindowBuilder};

/// The Tauri version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(target_os = "ios")]
#[doc(hidden)]
pub fn log_stdout() {
  #[cfg(target_os = "ios")]
  unsafe {
    crate::ios::log_stdout();
  }
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
/// The webview runtime handle. A wrapper around [`runtime::RuntimeHandle`] with the proper user event type associated.
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

/// Whether we are running in development mode or not.
pub const fn is_dev() -> bool {
  !cfg!(feature = "custom-protocol")
}

/// Represents a container of file assets that are retrievable during runtime.
pub trait Assets<R: Runtime>: Send + Sync + 'static {
  /// Initialize the asset provider.
  fn setup(&self, app: &App<R>) {
    let _ = app;
  }

  /// Get the content of the passed [`AssetKey`].
  fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>>;

  /// Iterator for the assets.
  fn iter(&self) -> Box<tauri_utils::assets::AssetsIter<'_>>;

  /// Gets the hashes for the CSP tag of the HTML on the given path.
  fn csp_hashes(&self, html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_>;
}

impl<R: Runtime> Assets<R> for EmbeddedAssets {
  fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
    EmbeddedAssets::get(self, key)
  }

  fn iter(&self) -> Box<AssetsIter<'_>> {
    EmbeddedAssets::iter(self)
  }

  fn csp_hashes(&self, html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
    EmbeddedAssets::csp_hashes(self, html_path)
  }
}

/// User supplied data required inside of a Tauri application.
///
/// # Stability
/// This is the output of the [`generate_context`] macro, and is not considered part of the stable API.
/// Unless you know what you are doing and are prepared for this type to have breaking changes, do not create it yourself.
#[tauri_macros::default_runtime(Wry, wry)]
pub struct Context<R: Runtime> {
  pub(crate) config: Config,
  #[cfg(dev)]
  pub(crate) config_parent: Option<std::path::PathBuf>,
  /// Asset provider.
  pub assets: Box<dyn Assets<R>>,
  pub(crate) default_window_icon: Option<image::Image<'static>>,
  pub(crate) app_icon: Option<Vec<u8>>,
  #[cfg(all(desktop, feature = "tray-icon"))]
  pub(crate) tray_icon: Option<image::Image<'static>>,
  pub(crate) package_info: PackageInfo,
  pub(crate) pattern: Pattern,
  pub(crate) runtime_authority: RuntimeAuthority,
  pub(crate) plugin_global_api_scripts: Option<&'static [&'static str]>,
}

/// Temporary struct that overrides the Debug formatting for the `app_icon` field.
///
/// It reduces the output size compared to the default, as that would format the binary
/// data as a slice of numbers `[65, 66, 67]`. This instead shows the length of the Vec.
///
/// For example: `Some([u8; 493])`
pub(crate) struct DebugAppIcon<'a>(&'a Option<Vec<u8>>);

impl std::fmt::Debug for DebugAppIcon<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.0 {
      Option::None => f.write_str("None"),
      Option::Some(icon) => f
        .debug_tuple("Some")
        .field(&format_args!("[u8; {}]", icon.len()))
        .finish(),
    }
  }
}

impl<R: Runtime> fmt::Debug for Context<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut d = f.debug_struct("Context");
    d.field("config", &self.config)
      .field("default_window_icon", &self.default_window_icon)
      .field("app_icon", &DebugAppIcon(&self.app_icon))
      .field("package_info", &self.package_info)
      .field("pattern", &self.pattern)
      .field("plugin_global_api_scripts", &self.plugin_global_api_scripts);

    #[cfg(all(desktop, feature = "tray-icon"))]
    d.field("tray_icon", &self.tray_icon);

    d.finish()
  }
}

impl<R: Runtime> Context<R> {
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
  pub fn assets(&self) -> &dyn Assets<R> {
    self.assets.as_ref()
  }

  /// Replace the [`Assets`] implementation and returns the previous value so you can use it as a fallback if desired.
  #[inline(always)]
  pub fn set_assets(&mut self, assets: Box<dyn Assets<R>>) -> Box<dyn Assets<R>> {
    std::mem::replace(&mut self.assets, assets)
  }

  /// The default window icon Tauri should use when creating windows.
  #[inline(always)]
  pub fn default_window_icon(&self) -> Option<&image::Image<'_>> {
    self.default_window_icon.as_ref()
  }

  /// Set the default window icon Tauri should use when creating windows.
  #[inline(always)]
  pub fn set_default_window_icon(&mut self, icon: Option<image::Image<'static>>) {
    self.default_window_icon = icon;
  }

  /// The icon to use on the tray icon.
  #[cfg(all(desktop, feature = "tray-icon"))]
  #[cfg_attr(docsrs, doc(cfg(all(desktop, feature = "tray-icon"))))]
  #[inline(always)]
  pub fn tray_icon(&self) -> Option<&image::Image<'_>> {
    self.tray_icon.as_ref()
  }

  /// Set the icon to use on the tray icon.
  #[cfg(all(desktop, feature = "tray-icon"))]
  #[cfg_attr(docsrs, doc(cfg(all(desktop, feature = "tray-icon"))))]
  #[inline(always)]
  pub fn set_tray_icon(&mut self, icon: Option<image::Image<'static>>) {
    self.tray_icon = icon;
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

  /// A mutable reference to the resolved ACL.
  ///
  /// # Stability
  ///
  /// This API is unstable.
  #[doc(hidden)]
  #[inline(always)]
  pub fn runtime_authority_mut(&mut self) -> &mut RuntimeAuthority {
    &mut self.runtime_authority
  }

  /// Create a new [`Context`] from the minimal required items.
  #[inline(always)]
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    config: Config,
    assets: Box<dyn Assets<R>>,
    default_window_icon: Option<image::Image<'static>>,
    app_icon: Option<Vec<u8>>,
    package_info: PackageInfo,
    pattern: Pattern,
    runtime_authority: RuntimeAuthority,
    plugin_global_api_scripts: Option<&'static [&'static str]>,
  ) -> Self {
    Self {
      config,
      #[cfg(dev)]
      config_parent: None,
      assets,
      default_window_icon,
      app_icon,
      #[cfg(all(desktop, feature = "tray-icon"))]
      tray_icon: None,
      package_info,
      pattern,
      runtime_authority,
      plugin_global_api_scripts,
    }
  }

  #[cfg(dev)]
  #[doc(hidden)]
  pub fn with_config_parent(&mut self, config_parent: impl AsRef<std::path::Path>) {
    self
      .config_parent
      .replace(config_parent.as_ref().to_owned());
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

  /// Fetch a single window from the manager.
  #[cfg(feature = "unstable")]
  #[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
  fn get_window(&self, label: &str) -> Option<Window<R>> {
    self.manager().get_window(label)
  }

  /// Fetch the focused window. Returns `None` if there is not any focused window.
  #[cfg(feature = "unstable")]
  #[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
  fn get_focused_window(&self) -> Option<Window<R>> {
    self.manager().get_focused_window()
  }

  /// Fetch all managed windows.
  #[cfg(feature = "unstable")]
  #[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
  fn windows(&self) -> HashMap<String, Window<R>> {
    self.manager().windows()
  }

  /// Fetch a single webview from the manager.
  #[cfg(feature = "unstable")]
  #[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
  fn get_webview(&self, label: &str) -> Option<Webview<R>> {
    self.manager().get_webview(label)
  }

  /// Fetch all managed webviews.
  #[cfg(feature = "unstable")]
  #[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
  fn webviews(&self) -> HashMap<String, Webview<R>> {
    self.manager().webviews()
  }

  /// Fetch a single webview window from the manager.
  fn get_webview_window(&self, label: &str) -> Option<WebviewWindow<R>> {
    self.manager().get_webview(label).and_then(|webview| {
      let window = webview.window();
      if window.is_webview_window() {
        Some(WebviewWindow { window, webview })
      } else {
        None
      }
    })
  }

  /// Fetch all managed webview windows.
  fn webview_windows(&self) -> HashMap<String, WebviewWindow<R>> {
    self
      .manager()
      .webviews()
      .into_iter()
      .filter_map(|(label, webview)| {
        let window = webview.window();
        if window.is_webview_window() {
          Some((label, WebviewWindow { window, webview }))
        } else {
          None
        }
      })
      .collect::<HashMap<_, _>>()
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

  /// Removes the state managed by the application for T. Returns the state if it was actually removed.
  ///
  /// <div class="warning">
  ///
  /// This method is *UNSAFE* and calling it will cause previously obtained references through
  /// [Manager::state] and [State::inner] to become dangling references.
  ///
  /// It is currently deprecated and may be removed in the future.
  ///
  /// If you really want to unmanage a state, use [std::sync::Mutex] and [Option::take] to wrap the state instead.
  ///
  /// See [tauri-apps/tauri#12721] for more information.
  ///
  /// [tauri-apps/tauri#12721]: https://github.com/tauri-apps/tauri/issues/12721
  ///
  /// </div>
  #[deprecated(
    since = "2.3.0",
    note = "This method is unsafe, since it can cause dangling references."
  )]
  fn unmanage<T>(&self) -> Option<T>
  where
    T: Send + Sync + 'static,
  {
    // The caller decides to break the safety here, then OK, just let it go.
    unsafe { self.manager().state().unmanage() }
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
    self.manager().state.try_get().unwrap_or_else(|| {
      panic!(
        "state() called before manage() for {}",
        std::any::type_name::<T>()
      )
    })
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

  /// Get a reference to the resources table of this manager.
  fn resources_table(&self) -> MutexGuard<'_, ResourceTable>;

  /// Gets the managed [`Env`].
  fn env(&self) -> Env {
    self.state::<Env>().inner().clone()
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

  /// Adds a capability to the app.
  ///
  /// Note that by default every capability file in the `src-tauri/capabilities` folder
  /// are automatically enabled unless specific capabilities are configured in [`tauri.conf.json > app > security > capabilities`],
  /// so you should use a different director for the runtime-added capabilities or use [tauri_build::Attributes::capabilities_path_pattern].
  ///
  /// # Examples
  /// ```
  /// use tauri::Manager;
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     #[cfg(feature = "beta")]
  ///     app.add_capability(include_str!("../capabilities/beta/cap.json"));
  ///
  ///     #[cfg(feature = "stable")]
  ///     app.add_capability(include_str!("../capabilities/stable/cap.json"));
  ///     Ok(())
  ///   });
  /// ```
  ///
  /// The above example assumes the following directory layout:
  /// ```md
  /// ├── capabilities
  /// │   ├── app (default capabilities used by any app flavor)
  /// |   |   |-- cap.json
  /// │   ├── beta (capabilities only added to a `beta` flavor)
  /// |   |   |-- cap.json
  /// │   ├── stable (capabilities only added to a `stable` flavor)
  /// |       |-- cap.json
  /// ```
  ///
  /// For this layout to be properly parsed by Tauri, we need to change the build script to
  ///
  /// ```skip
  /// // only pick up capabilities in the capabilities/app folder by default
  /// let attributes = tauri_build::Attributes::new().capabilities_path_pattern("./capabilities/app/*.json");
  /// tauri_build::try_build(attributes).unwrap();
  /// ```
  ///
  /// [`tauri.conf.json > app > security > capabilities`]: https://tauri.app/reference/config/#capabilities
  /// [tauri_build::Attributes::capabilities_path_pattern]: https://docs.rs/tauri-build/2/tauri_build/struct.Attributes.html#method.capabilities_path_pattern
  #[cfg(feature = "dynamic-acl")]
  fn add_capability(&self, capability: impl RuntimeCapability) -> Result<()> {
    self
      .manager()
      .runtime_authority
      .lock()
      .unwrap()
      .add_capability(capability)
  }
}

/// Listen to events.
pub trait Listener<R: Runtime>: sealed::ManagerBase<R> {
  /// Listen to an emitted event on this manager.
  ///
  /// # Examples
  /// ```
  /// use tauri::{Manager, Listener, Emitter};
  ///
  /// #[tauri::command]
  /// fn synchronize(window: tauri::Window) {
  ///   // emits the synchronized event to all windows
  ///   window.emit("synchronized", ());
  /// }
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     app.listen("synchronized", |event| {
  ///       println!("app is in sync");
  ///     });
  ///     Ok(())
  ///   })
  ///   .invoke_handler(tauri::generate_handler![synchronize]);
  /// ```
  /// # Panics
  /// Will panic if `event` contains characters other than alphanumeric, `-`, `/`, `:` and `_`
  fn listen<F>(&self, event: impl Into<String>, handler: F) -> EventId
  where
    F: Fn(Event) + Send + 'static;

  /// Listen to an event on this manager only once.
  ///
  /// See [`Self::listen`] for more information.
  /// # Panics
  /// Will panic if `event` contains characters other than alphanumeric, `-`, `/`, `:` and `_`
  fn once<F>(&self, event: impl Into<String>, handler: F) -> EventId
  where
    F: FnOnce(Event) + Send + 'static;

  /// Remove an event listener.
  ///
  /// # Examples
  /// ```
  /// use tauri::{Manager, Listener};
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let handle = app.handle().clone();
  ///     let handler = app.listen_any("ready", move |event| {
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
  fn unlisten(&self, id: EventId);

  /// Listen to an emitted event to any [target](EventTarget).
  ///
  /// # Examples
  /// ```
  /// use tauri::{Manager, Emitter, Listener};
  ///
  /// #[tauri::command]
  /// fn synchronize(window: tauri::Window) {
  ///   // emits the synchronized event to all windows
  ///   window.emit("synchronized", ());
  /// }
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     app.listen_any("synchronized", |event| {
  ///       println!("app is in sync");
  ///     });
  ///     Ok(())
  ///   })
  ///   .invoke_handler(tauri::generate_handler![synchronize]);
  /// ```
  /// # Panics
  /// Will panic if `event` contains characters other than alphanumeric, `-`, `/`, `:` and `_`
  fn listen_any<F>(&self, event: impl Into<String>, handler: F) -> EventId
  where
    F: Fn(Event) + Send + 'static,
  {
    let event = EventName::new(event.into()).unwrap();
    self.manager().listen(event, EventTarget::Any, handler)
  }

  /// Listens once to an emitted event to any [target](EventTarget) .
  ///
  /// See [`Self::listen_any`] for more information.
  /// # Panics
  /// Will panic if `event` contains characters other than alphanumeric, `-`, `/`, `:` and `_`
  fn once_any<F>(&self, event: impl Into<String>, handler: F) -> EventId
  where
    F: FnOnce(Event) + Send + 'static,
  {
    let event = EventName::new(event.into()).unwrap();
    self.manager().once(event, EventTarget::Any, handler)
  }
}

/// Emit events.
pub trait Emitter<R: Runtime>: sealed::ManagerBase<R> {
  /// Emits an event to all [targets](EventTarget).
  ///
  /// # Examples
  /// ```
  /// use tauri::Emitter;
  ///
  /// #[tauri::command]
  /// fn synchronize(app: tauri::AppHandle) {
  ///   // emits the synchronized event to all webviews
  ///   app.emit("synchronized", ());
  /// }
  /// ```
  fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<()> {
    let event = EventName::new(event)?;
    let payload = EmitPayload::Serialize(&payload);
    self.manager().emit(event, payload)
  }

  /// Similar to [`Emitter::emit`] but the payload is json serialized.
  fn emit_str(&self, event: &str, payload: String) -> Result<()> {
    let event = EventName::new(event)?;
    let payload = EmitPayload::<()>::Str(payload);
    self.manager().emit(event, payload)
  }

  /// Emits an event to all [targets](EventTarget) matching the given target.
  ///
  /// # Examples
  /// ```
  /// use tauri::{Emitter, EventTarget};
  ///
  /// #[tauri::command]
  /// fn download(app: tauri::AppHandle) {
  ///   for i in 1..100 {
  ///     std::thread::sleep(std::time::Duration::from_millis(150));
  ///     // emit a download progress event to all listeners
  ///     app.emit_to(EventTarget::any(), "download-progress", i);
  ///     // emit an event to listeners that used App::listen or AppHandle::listen
  ///     app.emit_to(EventTarget::app(), "download-progress", i);
  ///     // emit an event to any webview/window/webviewWindow matching the given label
  ///     app.emit_to("updater", "download-progress", i); // similar to using EventTarget::labeled
  ///     app.emit_to(EventTarget::labeled("updater"), "download-progress", i);
  ///     // emit an event to listeners that used WebviewWindow::listen
  ///     app.emit_to(EventTarget::webview_window("updater"), "download-progress", i);
  ///   }
  /// }
  /// ```
  fn emit_to<I, S>(&self, target: I, event: &str, payload: S) -> Result<()>
  where
    I: Into<EventTarget>,
    S: Serialize + Clone,
  {
    let event = EventName::new(event)?;
    let payload = EmitPayload::Serialize(&payload);
    self.manager().emit_to(target.into(), event, payload)
  }

  /// Similar to [`Emitter::emit_to`] but the payload is json serialized.
  fn emit_str_to<I>(&self, target: I, event: &str, payload: String) -> Result<()>
  where
    I: Into<EventTarget>,
  {
    let event = EventName::new(event)?;
    let payload = EmitPayload::<()>::Str(payload);
    self.manager().emit_to(target.into(), event, payload)
  }

  /// Emits an event to all [targets](EventTarget) based on the given filter.
  ///
  /// # Examples
  /// ```
  /// use tauri::{Emitter, EventTarget};
  ///
  /// #[tauri::command]
  /// fn download(app: tauri::AppHandle) {
  ///   for i in 1..100 {
  ///     std::thread::sleep(std::time::Duration::from_millis(150));
  ///     // emit a download progress event to the updater window
  ///     app.emit_filter("download-progress", i, |t| match t {
  ///       EventTarget::WebviewWindow { label } => label == "main",
  ///       _ => false,
  ///     });
  ///   }
  /// }
  /// ```
  fn emit_filter<S, F>(&self, event: &str, payload: S, filter: F) -> Result<()>
  where
    S: Serialize + Clone,
    F: Fn(&EventTarget) -> bool,
  {
    let event = EventName::new(event)?;
    let payload = EmitPayload::Serialize(&payload);
    self.manager().emit_filter(event, payload, filter)
  }

  /// Similar to [`Emitter::emit_filter`] but the payload is json serialized.
  fn emit_str_filter<F>(&self, event: &str, payload: String, filter: F) -> Result<()>
  where
    F: Fn(&EventTarget) -> bool,
  {
    let event = EventName::new(event)?;
    let payload = EmitPayload::<()>::Str(payload);
    self.manager().emit_filter(event, payload, filter)
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
    Dispatch(R::WindowDispatcher),
  }

  /// Managed handle to the application runtime.
  pub trait ManagerBase<R: Runtime> {
    fn manager(&self) -> &AppManager<R>;
    fn manager_owned(&self) -> Arc<AppManager<R>>;
    fn runtime(&self) -> RuntimeOrDispatch<'_, R>;
    fn managed_app_handle(&self) -> &AppHandle<R>;
  }
}

struct UnsafeSend<T>(T);
unsafe impl<T> Send for UnsafeSend<T> {}

impl<T> UnsafeSend<T> {
  fn take(self) -> T {
    self.0
  }
}

#[allow(unused)]
macro_rules! run_main_thread {
  ($handle:ident, $ex:expr) => {{
    use std::sync::mpsc::channel;
    let (tx, rx) = channel();
    let task = move || {
      let f = $ex;
      let _ = tx.send(f());
    };
    $handle
      .run_on_main_thread(task)
      .and_then(|_| rx.recv().map_err(|_| crate::Error::FailedToReceiveMessage))
  }};
}

#[allow(unused)]
pub(crate) use run_main_thread;

#[cfg(any(test, feature = "test"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test")))]
pub mod test;

#[cfg(feature = "specta")]
const _: () = {
  use specta::{datatype::DataType, function::FunctionArg, TypeMap};

  impl<T: Send + Sync + 'static> FunctionArg for crate::State<'_, T> {
    fn to_datatype(_: &mut TypeMap) -> Option<DataType> {
      None
    }
  }

  impl<R: crate::Runtime> FunctionArg for crate::AppHandle<R> {
    fn to_datatype(_: &mut TypeMap) -> Option<DataType> {
      None
    }
  }

  impl<R: crate::Runtime> FunctionArg for crate::Window<R> {
    fn to_datatype(_: &mut TypeMap) -> Option<DataType> {
      None
    }
  }

  impl<R: crate::Runtime> FunctionArg for crate::Webview<R> {
    fn to_datatype(_: &mut TypeMap) -> Option<DataType> {
      None
    }
  }

  impl<R: crate::Runtime> FunctionArg for crate::WebviewWindow<R> {
    fn to_datatype(_: &mut TypeMap) -> Option<DataType> {
      None
    }
  }
};

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
          "Feature {checked_feature} was checked in the alias build step but it does not exist in crates/tauri/Cargo.toml"
        );
      }
    }
  }
}

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
        let _ = format!("{task}-run-dummy-task");
      };
      // call spawn
      crate::async_runtime::spawn(dummy_task);
    }
  }
}

/// Simple dependency-free string encoder using [Z85].
mod z85 {
  const TABLE: &[u8; 85] =
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

  /// Encode bytes with [Z85].
  ///
  /// # Panics
  ///
  /// Will panic if the input bytes are not a multiple of 4.
  pub fn encode(bytes: &[u8]) -> String {
    assert_eq!(bytes.len() % 4, 0);

    let mut buf = String::with_capacity(bytes.len() * 5 / 4);
    for chunk in bytes.chunks_exact(4) {
      let mut chars = [0u8; 5];
      let mut chunk = u32::from_be_bytes(chunk.try_into().unwrap()) as usize;
      for byte in chars.iter_mut().rev() {
        *byte = TABLE[chunk % 85];
        chunk /= 85;
      }

      buf.push_str(std::str::from_utf8(&chars).unwrap());
    }

    buf
  }

  #[cfg(test)]
  mod tests {
    #[test]
    fn encode() {
      assert_eq!(
        super::encode(&[0x86, 0x4F, 0xD2, 0x6F, 0xB5, 0x59, 0xF7, 0x5B]),
        "HelloWorld"
      );
    }
  }
}

/// Generate a random 128-bit [Z85] encoded [`String`].
///
/// [Z85]: https://rfc.zeromq.org/spec/32/
pub(crate) fn generate_invoke_key() -> Result<String> {
  let mut bytes = [0u8; 16];
  getrandom::fill(&mut bytes)?;
  Ok(z85::encode(&bytes))
}
