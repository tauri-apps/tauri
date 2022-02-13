// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
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
//! - **isolation**: Enables the isolation pattern. Enabled by default if the `tauri > pattern > use` config option is set to `isolation` on the `tauri.conf.json` file.
//! - **custom-protocol**: Feature managed by the Tauri CLI. When enabled, Tauri assumes a production environment instead of a development one.
//! - **updater**: Enables the application auto updater. Enabled by default if the `updater` config is defined on the `tauri.conf.json` file.
//! - **devtools**: Enables the developer tools (Web inspector) and [`Window::open_devtools`]. Enabled by default on debug builds.
//! On macOS it uses private APIs, so you can't enable it if your app will be published to the App Store.
//! - **http-api**: Enables the [`api::http`] module.
//! - **reqwest-client**: Uses `reqwest` as HTTP client on the `http` APIs. Improves performance, but increases the bundle size.
//! - **command**: Enables the [`api::process::Command`] APIs.
//! - **dialog**: Enables the [`api::dialog`] module.
//! - **notification**: Enables the [`api::notification`] module.
//! - **cli**: Enables usage of `clap` for CLI argument parsing. Enabled by default if the `cli` config is defined on the `tauri.conf.json` file.
//! - **system-tray**: Enables application system tray API. Enabled by default if the `systemTray` config is defined on the `tauri.conf.json` file.
//! - **macos-private-api**: Enables features only available in **macOS**'s private APIs, currently the `transparent` window functionality and the `fullScreenEnabled` preference setting to `true`. Enabled by default if the `tauri > macosPrivateApi` config flag is set to `true` on the `tauri.conf.json` file.
//! - **window-data-url**: Enables usage of data URLs on the webview.
//!
//! ## Cargo allowlist features
//!
//! The following are a list of [Cargo features](https://doc.rust-lang.org/stable/cargo/reference/manifest.html#the-features-section) that enables commands for Tauri's API package.
//! These features are automatically enabled by the Tauri CLI based on the `allowlist` configuration under `tauri.conf.json`.
//!
//! - **api-all**: Enables all API endpoints.
//!
//! ### Clipboard allowlist
//!
//! - **clipboard-all**: Enables all [Clipboard APIs](https://tauri.studio/en/docs/api/js/modules/clipboard/).
//! - **clipboard-read-text**: Enables the [`readText` API](https://tauri.studio/en/docs/api/js/modules/clipboard/#readtext).
//! - **clipboard-write-text**: Enables the [`writeText` API](https://tauri.studio/en/docs/api/js/modules/clipboard/#writetext).
//!
//! ### Dialog allowlist
//!
//! - **dialog-all**: Enables all [Dialog APIs](https://tauri.studio/en/docs/api/js/modules/dialog).
//! - **dialog-ask**: Enables the [`ask` API](https://tauri.studio/en/docs/api/js/modules/dialog#ask).
//! - **dialog-confirm**: Enables the [`confirm` API](https://tauri.studio/en/docs/api/js/modules/dialog#confirm).
//! - **dialog-message**: Enables the [`message` API](https://tauri.studio/en/docs/api/js/modules/dialog#message).
//! - **dialog-open**: Enables the [`open` API](https://tauri.studio/en/docs/api/js/modules/dialog#open).
//! - **dialog-save**: Enables the [`save` API](https://tauri.studio/en/docs/api/js/modules/dialog#save).
//!
//! ### Filesystem allowlist
//!
//! - **fs-all**: Enables all [Filesystem APIs](https://tauri.studio/en/docs/api/js/modules/fs).
//! - **fs-copy-file**: Enables the [`copyFile` API](https://tauri.studio/en/docs/api/js/modules/fs#copyfile).
//! - **fs-create-dir**: Enables the [`createDir` API](https://tauri.studio/en/docs/api/js/modules/fs#createdir).
//! - **fs-read-dir**: Enables the [`readDir` API](https://tauri.studio/en/docs/api/js/modules/fs#readdir).
//! - **fs-read-file**: Enables the [`readTextFile` API](https://tauri.studio/en/docs/api/js/modules/fs#readtextfile) and the [`readBinaryFile` API](https://tauri.studio/en/docs/api/js/modules/fs#readbinaryfile).
//! - **fs-remove-dir**: Enables the [`removeDir` API](https://tauri.studio/en/docs/api/js/modules/fs#removedir).
//! - **fs-remove-file**: Enables the [`removeFile` API](https://tauri.studio/en/docs/api/js/modules/fs#removefile).
//! - **fs-rename-file**: Enables the [`renameFile` API](https://tauri.studio/en/docs/api/js/modules/fs#renamefile).
//! - **fs-write-file**: Enables the [`writeFile` API](https://tauri.studio/en/docs/api/js/modules/fs#writefile) and the [`writeBinaryFile` API](https://tauri.studio/en/docs/api/js/modules/fs#writebinaryfile).
//!
//! ### Global shortcut allowlist
//!
//! - **global-shortcut-all**: Enables all [GlobalShortcut APIs](https://tauri.studio/en/docs/api/js/modules/globalShortcut).
//!
//! ### HTTP allowlist
//!
//! - **http-all**: Enables all [HTTP APIs](https://tauri.studio/en/docs/api/js/modules/http).
//! - **http-request**: Enables the [`request` APIs](https://tauri.studio/en/docs/api/js/classes/http.client/).
//!
//! ### Notification allowlist
//!
//! - **notification-all**: Enables all [Notification APIs](https://tauri.studio/en/docs/api/js/modules/notification).
//!
//! ### OS allowlist
//!
//! - **os-all**: Enables all [OS APIs](https://tauri.studio/en/docs/api/js/modules/os).
//!
//! ### Path allowlist
//!
//! - **path-all**: Enables all [Path APIs](https://tauri.studio/en/docs/api/js/modules/path).
//!
//! ### Process allowlist
//!
//! - **process-all**: Enables all [Process APIs](https://tauri.studio/en/docs/api/js/modules/process).
//! - **process-exit**: Enables the [`exit` API](https://tauri.studio/en/docs/api/js/modules/process#exit).
//! - **process-relaunch**: Enables the [`relaunch` API](https://tauri.studio/en/docs/api/js/modules/process#relaunch).
//!
//! ### Protocol allowlist
//!
//! - **protocol-all**: Enables all Protocol APIs.
//! - **protocol-asset**: Enables the `asset` custom protocol.
//!
//! ### Shell allowlist
//!
//! - **shell-all**: Enables all [Clipboard APIs](https://tauri.studio/en/docs/api/js/modules/shell).
//! - **shell-execute**: Enables [executing arbitrary programs](https://tauri.studio/en/docs/api/js/classes/shell.Command#constructor).
//! - **shell-sidecar**: Enables [executing a `sidecar` program](https://tauri.studio/en/docs/api/js/classes/shell.Command#sidecar).
//! - **shell-open**: Enables the [`open` API](https://tauri.studio/en/docs/api/js/modules/shell#open).
//!
//! ### Window allowlist
//!
//! - **window-all**: Enables all [Window APIs](https://tauri.studio/en/docs/api/js/modules/window).
//! - **window-create**: Enables the API used to [create new windows](https://tauri.studio/en/docs/api/js/classes/window.webviewwindow/).
//! - **window-center**: Enables the [`center` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#center).
//! - **window-request-user-attention**: Enables the [`requestUserAttention` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#requestuserattention).
//! - **window-set-resizable**: Enables the [`setResizable` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setresizable).
//! - **window-set-title**: Enables the [`setTitle` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#settitle).
//! - **window-maximize**: Enables the [`maximize` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#maximize).
//! - **window-unmaximize**: Enables the [`unmaximize` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#unmaximize).
//! - **window-minimize**: Enables the [`minimize` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#minimize).
//! - **window-unminimize**: Enables the [`unminimize` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#unminimize).
//! - **window-show**: Enables the [`show` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#show).
//! - **window-hide**: Enables the [`hide` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#hide).
//! - **window-close**: Enables the [`close` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#close).
//! - **window-set-decorations**: Enables the [`setDecorations` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setdecorations).
//! - **window-set-always-on-top**: Enables the [`setAlwaysOnTop` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setalwaysontop).
//! - **window-set-size**: Enables the [`setSize` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setsize).
//! - **window-set-min-size**: Enables the [`setMinSize` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setminsize).
//! - **window-set-max-size**: Enables the [`setMaxSize` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setmaxsize).
//! - **window-set-position**: Enables the [`setPosition` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setposition).
//! - **window-set-fullscreen**: Enables the [`setFullscreen` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setfullscreen).
//! - **window-set-focus**: Enables the [`setFocus` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setfocus).
//! - **window-set-icon**: Enables the [`setIcon` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#seticon).
//! - **window-set-skip-taskbar**: Enables the [`setSkipTaskbar` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setskiptaskbar).
//! - **window-start-dragging**: Enables the [`startDragging` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#startdragging).
//! - **window-print**: Enables the [`print` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#print).

#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

#[cfg(target_os = "macos")]
#[doc(hidden)]
pub use embed_plist;
/// The Tauri error enum.
pub use error::Error;
#[cfg(shell_scope)]
#[doc(hidden)]
pub use regex;
pub use tauri_macros::{command, generate_handler};

pub mod api;
pub(crate) mod app;
pub mod async_runtime;
pub mod command;
/// The Tauri API endpoints.
mod endpoints;
mod error;
mod event;
mod hooks;
mod manager;
mod pattern;
pub mod plugin;
pub mod window;
pub use tauri_runtime as runtime;
/// The allowlist scopes.
pub mod scope;
pub mod settings;
mod state;
#[cfg(feature = "updater")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "updater")))]
pub mod updater;

pub use tauri_utils as utils;

#[cfg(feature = "wry")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "wry")))]
pub use tauri_runtime_wry::Wry;

/// `Result<T, ::tauri::Error>`
pub type Result<T> = std::result::Result<T, Error>;

/// A task to run on the main thread.
pub type SyncTask = Box<dyn FnOnce() + Send>;

use crate::runtime::window::PendingWindow;
use serde::Serialize;
use std::{collections::HashMap, fmt, sync::Arc};

// Export types likely to be used by the application.
pub use runtime::http;

#[cfg(target_os = "macos")]
#[cfg_attr(doc_cfg, doc(cfg(target_os = "macos")))]
pub use runtime::{menu::NativeImage, ActivationPolicy};

#[cfg(feature = "system-tray")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
pub use {
  self::app::tray::{SystemTrayEvent, SystemTrayHandle},
  self::runtime::{
    menu::{SystemTrayMenu, SystemTrayMenuItem, SystemTraySubmenu},
    SystemTray,
  },
};
pub use {
  self::app::WindowMenuEvent,
  self::event::{Event, EventHandler},
  self::runtime::menu::{CustomMenuItem, Menu, MenuEntry, MenuItem, Submenu},
  self::window::menu::MenuEvent,
};
pub use {
  self::app::{
    App, AppHandle, AssetResolver, Builder, CloseRequestApi, GlobalWindowEvent, PathResolver,
    RunEvent,
  },
  self::hooks::{
    Invoke, InvokeError, InvokeHandler, InvokeMessage, InvokePayload, InvokeResolver,
    InvokeResponder, InvokeResponse, OnPageLoad, PageLoadPayload, SetupHook,
  },
  self::manager::Asset,
  self::runtime::{
    webview::{WebviewAttributes, WindowBuilder},
    window::{
      dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Pixel, Position, Size},
      WindowEvent,
    },
    ClipboardManager, GlobalShortcutManager, Icon, RunIteration, Runtime, UserAttentionType,
  },
  self::state::{State, StateManager},
  self::utils::{
    assets::Assets,
    config::{Config, WindowUrl},
    Env, PackageInfo,
  },
  self::window::{Monitor, Window},
  scope::*,
};

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

/// User supplied data required inside of a Tauri application.
///
/// # Stability
/// This is the output of the [`generate_context`] macro, and is not considered part of the stable API.
/// Unless you know what you are doing and are prepared for this type to have breaking changes, do not create it yourself.
pub struct Context<A: Assets> {
  pub(crate) config: Config,
  pub(crate) assets: Arc<A>,
  pub(crate) default_window_icon: Option<Vec<u8>>,
  pub(crate) system_tray_icon: Option<Icon>,
  pub(crate) package_info: PackageInfo,
  pub(crate) _info_plist: (),
  pub(crate) pattern: Pattern,
  #[cfg(shell_scope)]
  pub(crate) shell_scope: scope::ShellScopeConfig,
}

impl<A: Assets> fmt::Debug for Context<A> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut d = f.debug_struct("Context");
    d.field("config", &self.config)
      .field("default_window_icon", &self.default_window_icon)
      .field("system_tray_icon", &self.system_tray_icon)
      .field("package_info", &self.package_info)
      .field("pattern", &self.pattern);
    #[cfg(shell_scope)]
    d.field("shell_scope", &self.shell_scope);
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
  pub fn assets(&self) -> Arc<A> {
    self.assets.clone()
  }

  /// A mutable reference to the assets to be served directly by Tauri.
  #[inline(always)]
  pub fn assets_mut(&mut self) -> &mut Arc<A> {
    &mut self.assets
  }

  /// The default window icon Tauri should use when creating windows.
  #[inline(always)]
  pub fn default_window_icon(&self) -> Option<&[u8]> {
    self.default_window_icon.as_deref()
  }

  /// A mutable reference to the default window icon Tauri should use when creating windows.
  #[inline(always)]
  pub fn default_window_icon_mut(&mut self) -> &mut Option<Vec<u8>> {
    &mut self.default_window_icon
  }

  /// The icon to use on the system tray UI.
  #[inline(always)]
  pub fn system_tray_icon(&self) -> Option<&Icon> {
    self.system_tray_icon.as_ref()
  }

  /// A mutable reference to the icon to use on the system tray UI.
  #[inline(always)]
  pub fn system_tray_icon_mut(&mut self) -> &mut Option<Icon> {
    &mut self.system_tray_icon
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

  /// The scoped shell commands, where the `HashMap` key is the name each configuration.
  #[cfg(shell_scope)]
  #[inline(always)]
  pub fn allowed_commands(&self) -> &scope::ShellScopeConfig {
    &self.shell_scope
  }

  /// Create a new [`Context`] from the minimal required items.
  #[inline(always)]
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    config: Config,
    assets: Arc<A>,
    default_window_icon: Option<Vec<u8>>,
    system_tray_icon: Option<Icon>,
    package_info: PackageInfo,
    info_plist: (),
    pattern: Pattern,
    #[cfg(shell_scope)] shell_scope: scope::ShellScopeConfig,
  ) -> Self {
    Self {
      config,
      assets,
      default_window_icon,
      system_tray_icon,
      package_info,
      _info_plist: info_plist,
      pattern,
      #[cfg(shell_scope)]
      shell_scope,
    }
  }
}

// TODO: expand these docs
/// Manages a running application.
pub trait Manager<R: Runtime>: sealed::ManagerBase<R> {
  /// The [`Config`] the manager was created with.
  fn config(&self) -> Arc<Config> {
    self.manager().config()
  }

  /// Emits a event to all windows.
  fn emit_all<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<()> {
    self.manager().emit_filter(event, None, payload, |_| true)
  }

  /// Emits an event to a window with the specified label.
  fn emit_to<S: Serialize + Clone>(&self, label: &str, event: &str, payload: S) -> Result<()> {
    self
      .manager()
      .emit_filter(event, None, payload, |w| label == w.label())
  }

  /// Listen to a global event.
  fn listen_global<F>(&self, event: impl Into<String>, handler: F) -> EventHandler
  where
    F: Fn(Event) + Send + 'static,
  {
    self.manager().listen(event.into(), None, handler)
  }

  /// Listen to a global event only once.
  fn once_global<F>(&self, event: impl Into<String>, handler: F) -> EventHandler
  where
    F: FnOnce(Event) + Send + 'static,
  {
    self.manager().once(event.into(), None, handler)
  }

  /// Trigger a global event.
  fn trigger_global(&self, event: &str, data: Option<String>) {
    self.manager().trigger(event, None, data)
  }

  /// Remove an event listener.
  fn unlisten(&self, handler_id: EventHandler) {
    self.manager().unlisten(handler_id)
  }

  /// Fetch a single window from the manager.
  fn get_window(&self, label: &str) -> Option<Window<R>> {
    self.manager().get_window(label)
  }

  /// Fetch all managed windows.
  fn windows(&self) -> HashMap<String, Window<R>> {
    self.manager().windows()
  }

  /// Add `state` to the state managed by the application.
  /// See [`crate::Builder#manage`] for instructions.
  fn manage<T>(&self, state: T)
  where
    T: Send + Sync + 'static,
  {
    self.manager().state().set(state);
  }

  /// Gets the managed state for the type `T`. Panics if the type is not managed.
  fn state<T>(&self) -> State<'_, T>
  where
    T: Send + Sync + 'static,
  {
    self
      .manager()
      .inner
      .state
      .try_get()
      .expect("state() called before manage() for given type")
  }

  /// Tries to get the managed state for the type `T`. Returns `None` if the type is not managed.
  fn try_state<T>(&self) -> Option<State<'_, T>>
  where
    T: Send + Sync + 'static,
  {
    self.manager().inner.state.try_get()
  }

  /// Gets the managed [`Env`].
  fn env(&self) -> Env {
    self.state::<Env>().inner().clone()
  }

  /// Gets the scope for the filesystem APIs.
  fn fs_scope(&self) -> FsScope {
    self.state::<Scopes>().inner().fs.clone()
  }

  /// Gets the scope for the asset protocol.
  #[cfg(protocol_asset)]
  fn asset_protocol_scope(&self) -> FsScope {
    self.state::<Scopes>().inner().asset_protocol.clone()
  }

  /// Gets the scope for the shell execute APIs.
  #[cfg(shell_scope)]
  fn shell_scope(&self) -> ShellScope {
    self.state::<Scopes>().inner().shell.clone()
  }
}

/// Prevent implementation details from leaking out of the [`Manager`] trait.
pub(crate) mod sealed {
  use crate::{app::AppHandle, manager::WindowManager};
  use tauri_runtime::{Runtime, RuntimeHandle};

  /// A running [`Runtime`] or a dispatcher to it.
  pub enum RuntimeOrDispatch<'r, R: Runtime> {
    /// Reference to the running [`Runtime`].
    Runtime(&'r R),

    /// Handle to the running [`Runtime`].
    RuntimeHandle(R::Handle),

    /// A dispatcher to the running [`Runtime`].
    Dispatch(R::Dispatcher),
  }

  #[derive(Clone, serde::Serialize)]
  struct WindowCreatedEvent {
    label: String,
  }

  /// Managed handle to the application runtime.
  pub trait ManagerBase<R: Runtime> {
    /// The manager behind the [`Managed`] item.
    fn manager(&self) -> &WindowManager<R>;

    fn runtime(&self) -> RuntimeOrDispatch<'_, R>;
    fn app_handle(&self) -> AppHandle<R>;

    /// Creates a new [`Window`] on the [`Runtime`] and attaches it to the [`Manager`].
    fn create_new_window(
      &self,
      pending: crate::PendingWindow<R>,
    ) -> crate::Result<crate::Window<R>> {
      use crate::runtime::Dispatch;
      let labels = self.manager().labels().into_iter().collect::<Vec<_>>();
      let pending = self
        .manager()
        .prepare_window(self.app_handle(), pending, &labels)?;
      let window = match self.runtime() {
        RuntimeOrDispatch::Runtime(runtime) => runtime.create_window(pending),
        RuntimeOrDispatch::RuntimeHandle(handle) => handle.create_window(pending),
        RuntimeOrDispatch::Dispatch(mut dispatcher) => dispatcher.create_window(pending),
      }
      .map(|window| self.manager().attach_window(self.app_handle(), window))?;

      self.manager().emit_filter(
        "tauri://window-created",
        None,
        Some(WindowCreatedEvent {
          label: window.label().into(),
        }),
        |w| w != &window,
      )?;

      Ok(window)
    }
  }
}

/// Utilities for unit testing on Tauri applications.
#[cfg(test)]
pub mod test;

#[cfg(test)]
mod test_utils {
  use proptest::prelude::*;

  pub fn assert_send<T: Send>() {}
  pub fn assert_sync<T: Sync>() {}

  proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    // check to see if spawn executes a function.
    fn check_spawn_task(task in "[a-z]+") {
      // create dummy task function
      let dummy_task = async move {
        format!("{}-run-dummy-task", task);
      };
      // call spawn
      crate::async_runtime::spawn(dummy_task);
    }
  }
}
