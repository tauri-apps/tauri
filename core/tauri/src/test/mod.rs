// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
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
//!     let app = create_app(mock_builder());
//!     let window = tauri::WindowBuilder::new(&app, "main", Default::default())
//!         .build()
//!         .unwrap();
//!
//!     // run the `ping` command and assert it returns `pong`
//!     let res = tauri::test::get_ipc_response::<String>(
//!         &window,
//!         tauri::window::InvokeRequest {
//!             cmd: "ping".into(),
//!             callback: tauri::ipc::CallbackFn(0),
//!             error: tauri::ipc::CallbackFn(1),
//!             body: tauri::ipc::InvokeBody::default(),
//!             headers: Default::default(),
//!         },
//!     );
//! }
//! ```

#![allow(unused_variables)]

mod mock_runtime;
pub use mock_runtime::*;
use serde::{de::DeserializeOwned, Serialize};

use std::{borrow::Cow, fmt::Debug};

use crate::{
  ipc::{InvokeBody, InvokeError, InvokeResponse},
  window::InvokeRequest,
  App, Builder, Context, Pattern, Window,
};
use tauri_utils::{
  assets::{AssetKey, Assets, CspHash},
  config::{Config, PatternKind, TauriConfig},
};

/// An empty [`Assets`] implementation.
pub struct NoopAsset {
  csp_hashes: Vec<CspHash<'static>>,
}

impl Assets for NoopAsset {
  fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
    None
  }

  fn csp_hashes(&self, html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
    Box::new(self.csp_hashes.iter().copied())
  }
}

/// Creates a new empty [`Assets`] implementation.
pub fn noop_assets() -> NoopAsset {
  NoopAsset {
    csp_hashes: Default::default(),
  }
}

/// Creates a new [`crate::Context`] for testing.
pub fn mock_context<A: Assets>(assets: A) -> crate::Context<A> {
  Context {
    config: Config {
      schema: None,
      package: Default::default(),
      tauri: TauriConfig {
        pattern: PatternKind::Brownfield,
        windows: Vec::new(),
        bundle: Default::default(),
        security: Default::default(),
        tray_icon: None,
        macos_private_api: false,
      },
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
    _info_plist: (),
    pattern: Pattern::Brownfield(std::marker::PhantomData),
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
  Builder::<MockRuntime>::new().enable_macos_default_menu(false)
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
///     let window = tauri::WindowBuilder::new(&app, "main", Default::default())
///         .build()
///         .unwrap();
///
///     // run the `ping` command and assert it returns `pong`
///     tauri::test::assert_ipc_response(
///         &window,
///         tauri::window::InvokeRequest {
///             cmd: "ping".into(),
///             callback: tauri::ipc::CallbackFn(0),
///             error: tauri::ipc::CallbackFn(1),
///             body: tauri::ipc::InvokeBody::default(),
///             headers: Default::default(),
///         },
///       Ok("pong")
///     );
/// }
/// ```
pub fn assert_ipc_response<T: Serialize + Debug + Send + Sync + 'static>(
  window: &Window<MockRuntime>,
  request: InvokeRequest,
  expected: Result<T, T>,
) {
  let response = get_ipc_response::<serde_json::Value>(window, request);
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
///     let window = tauri::WindowBuilder::new(&app, "main", Default::default())
///         .build()
///         .unwrap();
///
///     // run the `ping` command and assert it returns `pong`
///     let res = tauri::test::get_ipc_response::<String>(
///         &window,
///         tauri::window::InvokeRequest {
///             cmd: "ping".into(),
///             callback: tauri::ipc::CallbackFn(0),
///             error: tauri::ipc::CallbackFn(1),
///             body: tauri::ipc::InvokeBody::default(),
///             headers: Default::default(),
///         },
///     );
///     assert!(res.is_ok());
///     assert_eq!(res.unwrap(), String::from("pong"));
/// }
///```
pub fn get_ipc_response<T>(
  window: &Window<MockRuntime>,
  request: InvokeRequest,
) -> Result<T, serde_json::Value>
where
  T: DeserializeOwned + Debug,
{
  let (tx, rx) = std::sync::mpsc::sync_channel(1);
  window.clone().on_message(
    request,
    Box::new(move |_window, _cmd, response, _callback, _error| {
      tx.send(response).unwrap();
    }),
  );

  let res = rx.recv().expect("Failed to receive result from command");
  match res {
    InvokeResponse::Ok(InvokeBody::Json(v)) => Ok(serde_json::from_value(v).unwrap()),
    InvokeResponse::Ok(InvokeBody::Raw(v)) => Ok(serde_json::from_slice(&v).unwrap()),
    InvokeResponse::Err(InvokeError(v)) => Err(v),
  }
}

/// Executes the given IPC message and get the return value as a raw buffer.
///
/// # Examples
///
/// ```rust
/// use tauri::ipc::Response;
/// use tauri::test::{get_ipc_response_raw, mock_builder, mock_context, noop_assets};
///
/// #[tauri::command]
/// fn ping() -> Response {
///     Response::new("pong".as_bytes().to_vec())
/// }
///
/// fn create_app<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::App<R> {
///     builder
///         .invoke_handler(tauri::generate_handler![ping])
///         // remove the string argument on your app
///         .build(mock_context(noop_assets()))
///         .expect("failed to build app")
/// }
///
/// fn main() {
///     let app = create_app(mock_builder());
///     let window = tauri::WindowBuilder::new(&app, "main", Default::default())
///         .build()
///         .unwrap();
///
///     // run the `ping` command and assert it returns `pong`
///     let res = get_ipc_response_raw(
///         &window,
///         tauri::window::InvokeRequest {
///             cmd: "ping".into(),
///             callback: tauri::ipc::CallbackFn(0),
///             error: tauri::ipc::CallbackFn(1),
///             body: tauri::ipc::InvokeBody::default(),
///             headers: Default::default(),
///         },
///     );
///     assert!(res.is_ok());
///     assert_eq!(res.unwrap(), "pong".as_bytes().to_vec());
/// }
///```
pub fn get_ipc_response_raw(
  window: &Window<MockRuntime>,
  request: InvokeRequest,
) -> Result<Vec<u8>, String> {
  let (tx, rx) = std::sync::mpsc::sync_channel(1);
  window.clone().on_message(
    request,
    Box::new(move |_window, _cmd, response, _callback, _error| {
      tx.send(response).unwrap();
    }),
  );

  let res = rx.recv().expect("Failed to receive result from command");
  match res {
    InvokeResponse::Ok(InvokeBody::Json(v)) => {
      Ok(serde_json::to_string(&v).unwrap().as_bytes().to_vec())
    }
    InvokeResponse::Ok(InvokeBody::Raw(v)) => Ok(v),
    InvokeResponse::Err(InvokeError(v)) => Err(v.to_string()),
  }
}

#[cfg(test)]
mod tests {
  use crate::WindowBuilder;
  use std::time::Duration;

  use super::mock_app;

  #[test]
  fn run_app() {
    let app = mock_app();

    let w = WindowBuilder::new(&app, "main", Default::default())
      .build()
      .unwrap();

    std::thread::spawn(move || {
      std::thread::sleep(Duration::from_secs(1));
      w.close().unwrap();
    });

    app.run(|_app, event| {
      println!("{:?}", event);
    });
  }
}
