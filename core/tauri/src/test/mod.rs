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

#[cfg(shell_scope)]
use std::collections::HashMap;
use std::{borrow::Cow, sync::Arc};

#[cfg(shell_scope)]
use crate::ShellScopeConfig;
use crate::{App, Builder, Context, InvokePayload, Pattern, Window};
use tauri_utils::{
  assets::{AssetKey, Assets, CspHash},
  config::{CliConfig, Config, PatternKind, TauriConfig},
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
  Builder::<MockRuntime>::new()
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
pub fn assert_ipc_response<T: Serialize>(
  window: &Window<MockRuntime>,
  payload: InvokePayload,
  expected: Result<T, T>,
) {
  let callback = payload.callback;
  let error = payload.error;
  window.clone().on_message(payload).unwrap();

  let mut num_tries = 0;
  let evaluated_script = loop {
    std::thread::sleep(std::time::Duration::from_millis(50));
    let evaluated_script = window.dispatcher().last_evaluated_script();
    if let Some(s) = evaluated_script {
      break s;
    }
    num_tries += 1;
    if num_tries == 20 {
      panic!("Response script not evaluated");
    }
  };
  let (expected_response, fn_name) = match expected {
    Ok(payload) => (payload, callback),
    Err(payload) => (payload, error),
  };
  let expected = format!(
    "window[\"_{}\"]({})",
    fn_name.0,
    crate::api::ipc::serialize_js(&expected_response).unwrap()
  );

  println!("Last evaluated script:");
  println!("{evaluated_script}");
  println!("Expected:");
  println!("{expected}");
  assert!(evaluated_script.contains(&expected));
}

#[cfg(test)]
pub(crate) fn mock_invoke_context() -> crate::endpoints::InvokeContext<MockRuntime> {
  use crate::Manager;
  let app = mock_app();
  crate::endpoints::InvokeContext {
    window: app.get_window("main").unwrap(),
    config: app.config(),
    package_info: app.package_info().clone(),
  }
}
