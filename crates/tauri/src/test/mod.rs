// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Utilities for unit testing on Tauri applications.
//!
//! # Stability
//!
//! This module is unstable.
//!
//! # Examples
//!
//! ```rust
//! use tauri::test::{mock_builder, mock_context, noop_assets};
//!
//! #[tauri::command]
//! fn ping() -> &'static str {
//!     "pong"
//! }
//!
//! fn create_app<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::App<R> {
//!     builder
//!         .invoke_handler(tauri::generate_handler![ping])
//!         // remove the string argument to use your app's config file
//!         .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
//!         .expect("failed to build app")
//! }
//!
//! fn main() {
//!     // Use `tauri::Builder::default()` to use the default runtime rather than the `MockRuntime`;
//!     // let app = create_app(tauri::Builder::default());
//!     let app = create_app(mock_builder());
//!     let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
//!
//!     // run the `ping` command and assert it returns `pong`
//!     let res = tauri::test::get_ipc_response(
//!         &webview,
//!         tauri::webview::InvokeRequest {
//!             cmd: "ping".into(),
//!             callback: tauri::ipc::CallbackFn(0),
//!             error: tauri::ipc::CallbackFn(1),
//!             // alternatively use "tauri://localhost"
//!             url: "http://tauri.localhost".parse().unwrap(),
//!             body: tauri::ipc::InvokeBody::default(),
//!             headers: Default::default(),
//!             invoke_key: tauri::test::INVOKE_KEY.to_string(),
//!         },
//!     ).map(|b| b.deserialize::<String>().unwrap());
//! }
//! ```

#![allow(unused_variables)]

mod mock_runtime;
pub use mock_runtime::*;
use serde::Serialize;
use serialize_to_javascript::DefaultTemplate;

use std::{borrow::Cow, collections::HashMap, fmt::Debug};

use crate::{
  ipc::{InvokeError, InvokeResponse, InvokeResponseBody},
  webview::InvokeRequest,
  App, Assets, Builder, Context, Pattern, Runtime, Webview,
};
use tauri_utils::{
  acl::resolved::Resolved,
  assets::{AssetKey, AssetsIter, CspHash},
  config::{AppConfig, Config},
};

/// The invoke key used for tests.
pub const INVOKE_KEY: &str = "__invoke-key__";

/// An empty [`Assets`] implementation.
pub struct NoopAsset {
  assets: HashMap<String, Vec<u8>>,
  csp_hashes: Vec<CspHash<'static>>,
}

impl<R: Runtime> Assets<R> for NoopAsset {
  fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
    None
  }

  fn iter(&self) -> Box<AssetsIter<'_>> {
    Box::new(
      self
        .assets
        .iter()
        .map(|(k, b)| (Cow::Borrowed(k.as_str()), Cow::Borrowed(b.as_slice()))),
    )
  }

  fn csp_hashes(&self, html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
    Box::new(self.csp_hashes.iter().copied())
  }
}

/// Creates a new empty [`Assets`] implementation.
pub fn noop_assets() -> NoopAsset {
  NoopAsset {
    assets: Default::default(),
    csp_hashes: Default::default(),
  }
}

/// Creates a new [`crate::Context`] for testing.
pub fn mock_context<R: Runtime, A: Assets<R>>(assets: A) -> crate::Context<R> {
  Context {
    config: Config {
      schema: None,
      product_name: Default::default(),
      main_binary_name: Default::default(),
      version: Default::default(),
      identifier: Default::default(),
      app: AppConfig {
        with_global_tauri: Default::default(),
        windows: Vec::new(),
        security: Default::default(),
        tray_icon: None,
        macos_private_api: false,
        enable_gtk_app_id: false,
      },
      bundle: Default::default(),
      build: Default::default(),
      plugins: Default::default(),
    },
    assets: Box::new(assets),
    default_window_icon: None,
    app_icon: None,
    #[cfg(all(desktop, feature = "tray-icon"))]
    tray_icon: None,
    package_info: crate::PackageInfo {
      name: "test".into(),
      version: "0.1.0".parse().unwrap(),
      authors: "Tauri",
      description: "Tauri test",
      crate_name: "test",
    },
    pattern: Pattern::Brownfield,
    runtime_authority: crate::runtime_authority!(Default::default(), Resolved::default()),
    plugin_global_api_scripts: None,

    #[cfg(dev)]
    config_parent: None,
  }
}

/// Creates a new [`Builder`] using the [`MockRuntime`].
///
/// To use a dummy [`Context`], see [`mock_app`].
///
/// # Examples
///
/// ```rust
/// #[cfg(test)]
/// fn do_something() {
///   let app = tauri::test::mock_builder()
///     // remove the string argument to use your app's config file
///     .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
///     .unwrap();
/// }
/// ```
pub fn mock_builder() -> Builder<MockRuntime> {
  let mut builder = Builder::<MockRuntime>::new().enable_macos_default_menu(false);

  builder.invoke_initialization_script = crate::app::InvokeInitializationScript {
    process_ipc_message_fn: crate::manager::webview::PROCESS_IPC_MESSAGE_FN,
    os_name: std::env::consts::OS,
    fetch_channel_data_command: crate::ipc::channel::FETCH_CHANNEL_DATA_COMMAND,
    invoke_key: INVOKE_KEY,
  }
  .render_default(&Default::default())
  .unwrap()
  .into_string();

  builder.invoke_key = INVOKE_KEY.to_string();

  builder
}

/// Creates a new [`App`] for testing using the [`mock_context`] with a [`noop_assets`].
pub fn mock_app() -> App<MockRuntime> {
  mock_builder().build(mock_context(noop_assets())).unwrap()
}

/// Executes the given IPC message and assert the response matches the expected value.
///
/// # Examples
///
/// ```rust
/// use tauri::test::{mock_builder, mock_context, noop_assets};
///
/// #[tauri::command]
/// fn ping() -> &'static str {
///     "pong"
/// }
///
/// fn create_app<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::App<R> {
///     builder
///         .invoke_handler(tauri::generate_handler![ping])
///         // remove the string argument to use your app's config file
///         .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
///         .expect("failed to build app")
/// }
///
/// fn main() {
///     let app = create_app(mock_builder());
///     let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
///
///     // run the `ping` command and assert it returns `pong`
///     tauri::test::assert_ipc_response(
///         &webview,
///         tauri::webview::InvokeRequest {
///             cmd: "ping".into(),
///             callback: tauri::ipc::CallbackFn(0),
///             error: tauri::ipc::CallbackFn(1),
///             url: if cfg!(any(windows, target_os = "android")) {
///                 "http://tauri.localhost"
///             } else {
///                 "tauri://localhost"
///             }.parse().unwrap(),
///             body: tauri::ipc::InvokeBody::default(),
///             headers: Default::default(),
///             invoke_key: tauri::test::INVOKE_KEY.to_string(),
///         },
///       Ok("pong")
///     );
/// }
/// ```
pub fn assert_ipc_response<
  T: Serialize + Debug + Send + Sync + 'static,
  W: AsRef<Webview<MockRuntime>>,
>(
  webview: &W,
  request: InvokeRequest,
  expected: Result<T, T>,
) {
  let response =
    get_ipc_response(webview, request).map(|b| b.deserialize::<serde_json::Value>().unwrap());
  assert_eq!(
    response,
    expected
      .map(|e| serde_json::to_value(e).unwrap())
      .map_err(|e| serde_json::to_value(e).unwrap())
  );
}

#[allow(clippy::needless_doctest_main)]
/// Executes the given IPC message and get the return value.
///
/// # Examples
///
/// ```rust
/// use tauri::test::{mock_builder, mock_context, noop_assets};
///
/// #[tauri::command]
/// fn ping() -> &'static str {
///     "pong"
/// }
///
/// fn create_app<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::App<R> {
///     builder
///         .invoke_handler(tauri::generate_handler![ping])
///         // remove the string argument to use your app's config file
///         .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
///         .expect("failed to build app")
/// }
///
/// fn main() {
///     let app = create_app(mock_builder());
///     let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
///
///     // run the `ping` command and assert it returns `pong`
///     let res = tauri::test::get_ipc_response(
///         &webview,
///         tauri::webview::InvokeRequest {
///             cmd: "ping".into(),
///             callback: tauri::ipc::CallbackFn(0),
///             error: tauri::ipc::CallbackFn(1),
///             url: if cfg!(any(windows, target_os = "android")) {
///                 "http://tauri.localhost"
///             } else {
///                 "tauri://localhost"
///             }.parse().unwrap(),
///             body: tauri::ipc::InvokeBody::default(),
///             headers: Default::default(),
///             invoke_key: tauri::test::INVOKE_KEY.to_string(),
///         },
///     );
///     assert!(res.is_ok());
///     assert_eq!(res.unwrap().deserialize::<String>().unwrap(), String::from("pong"));
/// }
///```
pub fn get_ipc_response<W: AsRef<Webview<MockRuntime>>>(
  webview: &W,
  request: InvokeRequest,
) -> Result<InvokeResponseBody, serde_json::Value> {
  let (tx, rx) = std::sync::mpsc::sync_channel(1);
  webview.as_ref().clone().on_message(
    request,
    Box::new(move |_window, _cmd, response, _callback, _error| {
      tx.send(response).unwrap();
    }),
  );

  let res = rx.recv().expect("Failed to receive result from command");
  match res {
    InvokeResponse::Ok(b) => Ok(b),
    InvokeResponse::Err(InvokeError(v)) => Err(v),
  }
}

#[cfg(test)]
mod tests {
  use std::{
    sync::{
      atomic::{AtomicBool, Ordering},
      Arc, Mutex,
    },
    time::Duration,
  };

  use super::mock_app;

  #[test]
  fn run_app() {
    let app = mock_app();

    let w = crate::WebviewWindowBuilder::new(&app, "main", Default::default())
      .build()
      .unwrap();

    std::thread::spawn(move || {
      std::thread::sleep(Duration::from_secs(1));
      w.close().unwrap();
    });

    app.run(|_app, event| {
      println!("{event:?}");
    });
  }

  #[test]
  fn window_getters_reflect_setters() {
    let app = mock_app();
    let window = crate::WebviewWindowBuilder::new(&app, "main", Default::default())
      .build()
      .unwrap();

    window.set_title("Hello Tauri").unwrap();
    assert_eq!(window.title().unwrap(), "Hello Tauri");

    window
      .set_position(crate::PhysicalPosition::new(50, 60))
      .unwrap();
    let position = window.outer_position().unwrap();
    assert_eq!((position.x, position.y), (50, 60));

    window
      .set_size(crate::PhysicalSize::new(400u32, 300u32))
      .unwrap();
    let size = window.inner_size().unwrap();
    assert_eq!((size.width, size.height), (400, 300));

    window.set_fullscreen(true).unwrap();
    assert!(window.is_fullscreen().unwrap());
    assert_ne!(window.inner_size().unwrap().width, 400);
    window.set_fullscreen(false).unwrap();
    assert_eq!(window.inner_size().unwrap().width, 400);

    window.maximize().unwrap();
    assert!(window.is_maximized().unwrap());
    window.unmaximize().unwrap();
    assert!(!window.is_maximized().unwrap());
    assert_eq!(window.inner_size().unwrap().width, 400);

    window.hide().unwrap();
    assert!(!window.is_visible().unwrap());
    window.show().unwrap();
    assert!(window.is_visible().unwrap());

    window.set_resizable(false).unwrap();
    assert!(!window.is_resizable().unwrap());
    // the maximize button is disabled when the window is not resizable
    assert!(!window.is_maximizable().unwrap());
  }

  #[test]
  fn webview_records_evaluated_scripts() {
    let app = mock_app();
    let webview_window = crate::WebviewWindowBuilder::new(&app, "main", Default::default())
      .build()
      .unwrap();

    webview_window.eval("console.log('first')").unwrap();
    webview_window.eval("console.log('second')").unwrap();

    let dispatcher = &webview_window.webview.webview.dispatcher;
    assert_eq!(
      dispatcher.last_evaluated_script().unwrap(),
      "console.log('second')"
    );
    assert!(dispatcher
      .evaluated_scripts()
      .contains(&"console.log('first')".into()));
  }

  #[test]
  fn window_events_dispatch_synchronously() {
    let app = mock_app();
    let window = crate::WebviewWindowBuilder::new(&app, "main", Default::default())
      .build()
      .unwrap();

    let events = Arc::new(Mutex::new(Vec::new()));
    let events_ = events.clone();
    window.on_window_event(move |event| {
      let name = match event {
        crate::WindowEvent::Resized(_) => "resized",
        crate::WindowEvent::Moved(_) => "moved",
        crate::WindowEvent::CloseRequested { .. } => "close-requested",
        crate::WindowEvent::Destroyed => "destroyed",
        crate::WindowEvent::Focused(_) => "focused",
        crate::WindowEvent::ThemeChanged(_) => "theme-changed",
        _ => "other",
      };
      events_.lock().unwrap().push(name);
    });

    window
      .set_size(crate::LogicalSize::new(600.0, 500.0))
      .unwrap();
    window
      .set_position(crate::LogicalPosition::new(10.0, 10.0))
      .unwrap();
    window.close().unwrap();

    assert_eq!(
      *events.lock().unwrap(),
      vec!["resized", "moved", "close-requested", "destroyed"]
    );
  }

  #[test]
  fn close_can_be_prevented() {
    let app = mock_app();
    let window = crate::WebviewWindowBuilder::new(&app, "main", Default::default())
      .build()
      .unwrap();

    let destroyed = Arc::new(AtomicBool::new(false));
    let destroyed_ = destroyed.clone();
    window.on_window_event(move |event| match event {
      crate::WindowEvent::CloseRequested { api, .. } => api.prevent_close(),
      crate::WindowEvent::Destroyed => destroyed_.store(true, Ordering::Relaxed),
      _ => {}
    });

    window.close().unwrap();

    assert!(!destroyed.load(Ordering::Relaxed));
    // the window is still alive and functional
    assert!(window.is_visible().unwrap());
  }

  #[test]
  fn app_theme_applies_to_windows() {
    let app = mock_app();
    let window = crate::WebviewWindowBuilder::new(&app, "main", Default::default())
      .build()
      .unwrap();

    assert_eq!(window.theme().unwrap(), crate::Theme::Light);
    app.set_theme(Some(crate::Theme::Dark));
    assert_eq!(window.theme().unwrap(), crate::Theme::Dark);
  }

  #[test]
  fn exit_code_is_propagated() {
    let app = mock_app();
    let handle = app.handle().clone();

    std::thread::spawn(move || {
      std::thread::sleep(Duration::from_millis(50));
      handle.exit(5);
    });

    let code = app.run_return(|_app, _event| {});
    assert_eq!(code, 5);
  }
}
