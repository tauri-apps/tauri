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
  ipc::{InvokeError, InvokeResponse, InvokeResponseBody, RuntimeAuthority},
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
    runtime_authority: RuntimeAuthority::new(Default::default(), Resolved::default()),
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
///             url: "http://tauri.localhost".parse().unwrap(),
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
///             url: "http://tauri.localhost".parse().unwrap(),
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
  use std::time::Duration;

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
}
