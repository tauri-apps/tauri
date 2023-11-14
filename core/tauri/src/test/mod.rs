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
//!   let app = create_app(tauri::Builder::default());
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

#[derive(Eq, PartialEq)]
struct IpcKey {
  callback: CallbackFn,
  error: CallbackFn,
}

impl Hash for IpcKey {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.callback.0.hash(state);
    self.error.0.hash(state);
  }
}

struct Ipc(Mutex<HashMap<IpcKey, Sender<std::result::Result<JsonValue, JsonValue>>>>);

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
    let mut ipc_ = ipc.0.lock().unwrap();
    if let Some(tx) = ipc_.remove(&IpcKey { callback, error }) {
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
/// fn main() {
///   let app = create_app(tauri::Builder::default());
///   // app.run(|_handle, _event| {});}
/// }
///
/// //#[cfg(test)]
/// mod tests {
///   use tauri::Manager;
///
///   //#[cfg(test)]
///   fn something() {
///     let app = super::create_app(tauri::test::mock_builder());
///     let window = app.get_window("main").unwrap();
///
///     // run the `ping` command and assert it returns `pong`
///     tauri::test::assert_ipc_response(
///       &window,
///       tauri::InvokePayload {
///         cmd: "ping".into(),
///         tauri_module: None,
///         callback: tauri::api::ipc::CallbackFn(0),
///         error: tauri::api::ipc::CallbackFn(1),
///         inner: serde_json::Value::Null,
///       },
///       // the expected response is a success with the "pong" payload
///       // we could also use Err("error message") here to ensure the command failed
///       Ok("pong")
///     );
///   }
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

use crate::RunEvent;
/// Invoke a Tauri command given an `InvokePayload`.
///
/// The function closes the application after receiving command result.
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
///fn main() {
///     let app = super::create_app(tauri::test::mock_builder());
///     let payload = create_invoke_payload("ping".into(), CommandArgs::new())
///     invoke_tauri_cmd(app, payload);
///}
/// ```
pub fn invoke_tauri_cmd(app: App<MockRuntime>, payload: InvokePayload) {
  let w = app.get_window("main").expect("Could not get main window");

  let (tx, rx) = std::sync::mpsc::channel();

  std::thread::spawn(move || {
    let callback = payload.callback;
    let error = payload.error;
    let ipc = w.state::<Ipc>();
    ipc.0.lock().unwrap().insert(IpcKey { callback, error }, tx);
    w.clone().on_message(payload).unwrap();
  });

  app.run(move |app_handle, event| {
    match event {
      // We have received a message that all windows were closed
      RunEvent::ExitRequested { .. } => {}
      RunEvent::Exit => {}
      RunEvent::Ready => {
        let res = rx.recv().expect("Failed to receive result from command");
        app_handle.exit(0);
        // w.close().expect("Failed to close the fuzz window");
      }
      event => {}
    }
  });
}

/// Helper function to create a Tauri `InvokePayload`.
pub fn create_invoke_payload(cmd_name: String, command_args: CommandArgs) -> InvokePayload {
  let mut json_map = serde_json::map::Map::new();
  for (k, v) in command_args.inner {
    json_map.insert(k, v.into());
  }

  InvokePayload {
    cmd: cmd_name,
    tauri_module: None,
    callback: CallbackFn(0),
    error: CallbackFn(1),
    inner: serde_json::Value::Object(json_map),
  }
}

/// A wrapper around HashMap to facilitate `InvokePayload` creation.
pub struct CommandArgs {
  /// Inner type
  pub inner: HashMap<String, serde_json::Value>,
}

impl CommandArgs {
  /// Create a `CommandArgs`.
  pub fn new() -> CommandArgs {
    CommandArgs {
      inner: HashMap::new(),
    }
  }

  /// Insert a key, value pair that will be converted into the correct json type.
  pub fn insert(
    &mut self,
    key: impl Into<String>,
    value: impl Into<serde_json::Value>,
  ) -> Option<serde_json::Value> {
    self.inner.insert(key.into(), value.into())
  }
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
