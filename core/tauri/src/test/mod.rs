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
//! #[tauri::command]
//! fn my_cmd() {}
//!
//! fn create_app<R: tauri::Runtime>(mut builder: tauri::Builder<R>) -> tauri::App<R> {
//!   builder
//!     .setup(|app| {
//!       // do something
//!       Ok(())
//!     })
//!     .invoke_handler(tauri::generate_handler![my_cmd])
//!     // remove the string argument on your app
//!     .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
//!     .expect("failed to build app")
//! }
//!
//! fn main() {
//!   // Use `tauri::Builder::default()` to use the default runtime rather than the `MockRuntime`;
//!   // let app = create_app(tauri::Builder::default());
//!   let app = create_app(tauri::test::mock_builder());
//!   // app.run(|_handle, _event| {});
//! }
//!
//! //#[cfg(test)]
//! mod tests {
//!   use tauri::Manager;
//!   //#[cfg(test)]
//!   fn something() {
//!     let app = super::create_app(tauri::test::mock_builder());
//!     let window = app.get_window("main").unwrap();
//!     // do something with the app and window
//!     // in this case we'll run the my_cmd command with no arguments
//!     tauri::test::assert_ipc_response(
//!       &window,
//!       tauri::InvokePayload {
//!         cmd: "my_cmd".into(),
//!         tauri_module: None,
//!         callback: tauri::api::ipc::CallbackFn(0),
//!         error: tauri::api::ipc::CallbackFn(1),
//!         inner: serde_json::Value::Null,
//!       },
//!       Ok(())
//!     );
//!   }
//! }
//! ```

#![allow(unused_variables)]

mod mock_runtime;
pub use mock_runtime::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value as JsonValue;

use std::{
  borrow::Cow,
  collections::HashMap,
  fmt::Debug,
  hash::{Hash, Hasher},
  sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
  },
};

use crate::hooks::window_invoke_responder;
#[cfg(shell_scope)]
use crate::ShellScopeConfig;
use crate::{api::ipc::CallbackFn, App, Builder, Context, InvokePayload, Manager, Pattern, Window};
use tauri_utils::{
  assets::{AssetKey, Assets, CspHash},
  config::{CliConfig, Config, PatternKind, TauriConfig},
};

/// A key for an [`Ipc`] call.
#[derive(Eq, PartialEq)]
pub struct IpcKey {
  /// callback
  pub callback: CallbackFn,
  /// error callback
  pub error: CallbackFn,
}

impl Hash for IpcKey {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.callback.0.hash(state);
    self.error.0.hash(state);
  }
}

/// Structure to retrieve result of a Tauri command
pub struct Ipc(Mutex<HashMap<IpcKey, Sender<std::result::Result<JsonValue, JsonValue>>>>);

impl Ipc {
  /// Insert an `Ipc` which will be used by the Tauri backend to send back the result of a Tauri
  /// command
  pub fn insert_ipc(&self, key: IpcKey, tx: Sender<std::result::Result<JsonValue, JsonValue>>) {
    self.0.lock().unwrap().insert(key, tx);
  }

  /// Remove an `Ipc` from the hashmap
  pub fn remove_ipc(
    &self,
    key: &IpcKey,
  ) -> Option<Sender<std::result::Result<JsonValue, JsonValue>>> {
    self.0.lock().unwrap().remove(key)
  }
}

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
        windows: vec![Default::default()],
        cli: Some(CliConfig {
          description: None,
          long_description: None,
          before_help: None,
          after_help: None,
          args: None,
          subcommands: None,
        }),
        bundle: Default::default(),
        allowlist: Default::default(),
        security: Default::default(),
        updater: Default::default(),
        system_tray: None,
        macos_private_api: false,
      },
      build: Default::default(),
      plugins: Default::default(),
    },
    assets: Arc::new(assets),
    default_window_icon: None,
    app_icon: None,
    system_tray_icon: None,
    package_info: crate::PackageInfo {
      name: "test".into(),
      version: "0.1.0".parse().unwrap(),
      authors: "Tauri",
      description: "Tauri test",
    },
    _info_plist: (),
    pattern: Pattern::Brownfield(std::marker::PhantomData),
    #[cfg(shell_scope)]
    shell_scope: ShellScopeConfig {
      open: None,
      scopes: HashMap::new(),
    },
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
  let mut builder = Builder::<MockRuntime>::new().manage(Ipc(Default::default()));

  builder.invoke_responder = Arc::new(|window, response, callback, error| {
    let window_ = window.clone();
    let ipc = window_.state::<Ipc>();
    if let Some(tx) = ipc.remove_ipc(&IpcKey { callback, error }) {
      tx.send(response.into_result()).unwrap();
    } else {
      window_invoke_responder(window, response, callback, error)
    }
  });

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
/// #[tauri::command]
/// fn ping() -> &'static str {
///   "pong"
/// }
///
/// fn create_app<R: tauri::Runtime>(mut builder: tauri::Builder<R>) -> tauri::App<R> {
///   builder
///     .invoke_handler(tauri::generate_handler![ping])
///     // remove the string argument on your app
///     .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
///     .expect("failed to build app")
/// }
///
/// use tauri::Manager;
/// use tauri::test::mock_builder;
/// fn main() {
///   // app createion with a `MockRuntime`
///   let app = create_app(mock_builder());
///   let window = app.get_window("main").unwrap();
///
///   // run the `ping` command and assert it returns `pong`
///   tauri::test::assert_ipc_response(
///     &window,
///     tauri::InvokePayload {
///       cmd: "ping".into(),
///       tauri_module: None,
///       callback: tauri::api::ipc::CallbackFn(0),
///       error: tauri::api::ipc::CallbackFn(1),
///       inner: serde_json::Value::Null,
///     },
///     // the expected response is a success with the "pong" payload
///     // we could also use Err("error message") here to ensure the command failed
///     Ok("pong")
///   );
/// }
/// ```
pub fn assert_ipc_response<T: Serialize + Debug>(
  window: &Window<MockRuntime>,
  payload: InvokePayload,
  expected: Result<T, T>,
) {
  let callback = payload.callback;
  let error = payload.error;
  let ipc = window.state::<Ipc>();
  let (tx, rx) = channel();
  ipc.0.lock().unwrap().insert(IpcKey { callback, error }, tx);
  window.clone().on_message(payload).unwrap();

  assert_eq!(
    rx.recv().unwrap(),
    expected
      .map(|e| serde_json::to_value(e).unwrap())
      .map_err(|e| serde_json::to_value(e).unwrap())
  );
}

/// The application processes the command and stops.
///
/// # Examples
///
/// ```rust
///
/// #[tauri::command]
/// fn ping() -> &'static str {
///   "pong"
/// }
///
/// fn create_app<R: tauri::Runtime>(mut builder: tauri::Builder<R>) -> tauri::App<R> {
///   builder
///     .invoke_handler(tauri::generate_handler![ping])
///     // remove the string argument on your app
///     .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
///     .expect("failed to build app")
/// }
///
/// use tauri::test::*;
/// use tauri::Manager;
/// let app = create_app(mock_builder());
/// let window = app.get_window("main").unwrap();
///
/// // run the `ping` command and assert it returns `pong`
/// let res = tauri::test::get_ipc_response::<String>(
///   &window,
///   tauri::InvokePayload {
///     cmd: "ping".into(),
///     tauri_module: None,
///     callback: tauri::api::ipc::CallbackFn(0),
///     error: tauri::api::ipc::CallbackFn(1),
///     inner: serde_json::Value::Null,
///   });
/// assert_eq!(res, Ok("pong".into()))
/// ```
pub fn get_ipc_response<T: DeserializeOwned + Debug>(
  window: &Window<MockRuntime>,
  payload: InvokePayload,
) -> Result<T, T> {
  let callback = payload.callback;
  let error = payload.error;
  let ipc = window.state::<Ipc>();
  let (tx, rx) = channel();
  ipc.0.lock().unwrap().insert(IpcKey { callback, error }, tx);
  window.clone().on_message(payload).unwrap();

  let res: Result<JsonValue, JsonValue> = rx.recv().expect("Failed to receive result from command");
  res
    .map(|v| serde_json::from_value(v).unwrap())
    .map_err(|e| serde_json::from_value(e).unwrap())
}

#[cfg(test)]
pub(crate) fn mock_invoke_context() -> crate::endpoints::InvokeContext<MockRuntime> {
  let app = mock_app();
  crate::endpoints::InvokeContext {
    window: app.get_window("main").unwrap(),
    config: app.config(),
    package_info: app.package_info().clone(),
  }
}

#[cfg(test)]
mod tests {
  use crate::Manager;
  use std::time::Duration;

  use super::mock_app;

  #[test]
  fn run_app() {
    let app = mock_app();
    let w = app.get_window("main").unwrap();
    std::thread::spawn(move || {
      std::thread::sleep(Duration::from_secs(1));
      w.close().unwrap();
    });

    app.run(|_app, event| {
      println!("{:?}", event);
    });
  }
}
