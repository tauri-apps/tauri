//! Tauri is a framework for building tiny, blazing fast binaries for all major desktop platforms.
//! Developers can integrate any front-end framework that compiles to HTML, JS and CSS for building their user interface.
//! The backend of the application is a rust-sourced binary with an API that the front-end can interact with.
//!
//! The user interface in Tauri apps currently leverages Cocoa/WebKit on macOS, gtk-webkit2 on Linux and MSHTML (IE10/11) or Webkit via Edge on Windows.
//! Tauri uses (and contributes to) the MIT licensed project that you can find at [webview](https://github.com/webview/webview).
#![warn(missing_docs, rust_2018_idioms)]

/// The asset management module.
#[cfg(assets)]
pub mod assets;
/// The event system module.
pub mod event;
/// The embedded server helpers.
#[cfg(embedded_server)]
pub mod server;
/// The Tauri-specific settings for your app e.g. notification permission status.
pub mod settings;

/// The CLI args interface.
#[cfg(cli)]
pub mod cli;

/// The webview application entry.
mod app;
/// The Tauri API endpoints.
mod endpoints;
/// The plugin manager module contains helpers to manage runtime plugins.
pub mod plugin;
/// The salt helpers.
mod salt;

pub(crate) mod async_runtime;

/// Alias for a Result with error type anyhow::Error.
pub use anyhow::Result;
pub use app::*;
pub use tauri_api as api;
pub use webview_official::{Webview, WebviewMut};

use std::process::Stdio;

use api::rpc::{format_callback, format_callback_result};
use serde::Serialize;

/// Synchronously executes the given task
/// and evaluates its Result to the JS promise described by the `callback` and `error` function names.
pub fn execute_promise_sync<
  R: Serialize,
  F: futures::Future<Output = Result<R>> + Send + 'static,
>(
  webview: &mut WebviewMut,
  task: F,
  callback: String,
  error: String,
) -> crate::Result<()> {
  async_runtime::block_on(async move {
    let callback_string =
      format_callback_result(task.await.map_err(|err| err.to_string()), callback, error)?;
    webview.dispatch(move |w| w.eval(callback_string.as_str()))?;
    Ok(())
  })
}

/// Asynchronously executes the given task
/// and evaluates its Result to the JS promise described by the `success_callback` and `error_callback` function names.
///
/// If the Result `is_ok()`, the callback will be the `success_callback` function name and the argument will be the Ok value.
/// If the Result `is_err()`, the callback will be the `error_callback` function name and the argument will be the Err value.
pub async fn execute_promise<
  R: Serialize,
  F: futures::Future<Output = Result<R>> + Send + 'static,
>(
  webview: &mut WebviewMut,
  task: F,
  success_callback: String,
  error_callback: String,
) {
  let callback_string = match format_callback_result(
    task.await.map_err(|err| err.to_string()),
    success_callback,
    error_callback.clone(),
  ) {
    Ok(callback_string) => callback_string,
    Err(e) => format_callback(error_callback, e.to_string()),
  };
  webview
    .dispatch(move |webview_ref| webview_ref.eval(callback_string.as_str()))
    .expect("Failed to dispatch promise callback");
}

/// Calls the given command and evaluates its output to the JS promise described by the `callback` and `error` function names.
pub async fn call(
  webview: &mut WebviewMut,
  command: String,
  args: Vec<String>,
  callback: String,
  error: String,
) {
  execute_promise(
    webview,
    async move { api::command::get_output(command, args, Stdio::piped()) },
    callback,
    error,
  )
  .await;
}

/// Closes the splashscreen.
pub fn close_splashscreen(webview: &mut Webview<'_>) -> crate::Result<()> {
  // send a signal to the runner so it knows that it should redirect to the main app content
  webview.eval(r#"window.__TAURI_INVOKE_HANDLER__({ cmd: "closeSplashscreen" })"#);

  Ok(())
}

#[cfg(test)]
mod test {
  use proptest::prelude::*;

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
