//! Tauri is a framework for building tiny, blazing fast binaries for all major desktop platforms.
//! Developers can integrate any front-end framework that compiles to HTML, JS and CSS for building their user interface.
//! The backend of the application is a rust-sourced binary with an API that the front-end can interact with.
//!
//! The user interface in Tauri apps currently leverages Cocoa/WebKit on macOS, gtk-webkit2 on Linux and MSHTML (IE10/11) or Webkit via Edge on Windows.
//! Tauri uses (and contributes to) the MIT licensed project that you can find at [webview](https://github.com/webview/webview).
#![warn(missing_docs, rust_2018_idioms)]

/// The event system module.
pub mod event;
/// The embedded server helpers.
#[cfg(embedded_server)]
pub mod server;
/// The Tauri-specific settings for your app e.g. notification permission status.
pub mod settings;

/// The webview application entry.
mod app;
/// The Tauri API endpoints.
mod endpoints;
/// The plugin manager module contains helpers to manage runtime plugins.
pub mod plugin;
/// The salt helpers.
mod salt;
/// Webview interface.
mod webview;

pub(crate) mod async_runtime;

/// A task to run on the main thread.
pub type SyncTask = Box<dyn FnOnce() + Send>;

/// Alias for a Result with error type anyhow::Error.
pub use anyhow::Result;
pub use app::*;
pub use tauri_api as api;
pub use tauri_macros::FromTauriContext;
pub use webview::{
  ApplicationDispatcherExt, ApplicationExt, Callback, Event, WebviewBuilderExt, WindowBuilderExt,
};

/// The Tauri webview implementations.
pub mod flavors {
  pub use super::webview::wry::WryApplication as Wry;
}

use std::process::Stdio;

use api::rpc::{format_callback, format_callback_result};
use serde::Serialize;

/// Synchronously executes the given task
/// and evaluates its Result to the JS promise described by the `callback` and `error` function names.
pub fn execute_promise_sync<
  D: ApplicationDispatcherExt + 'static,
  R: Serialize,
  F: FnOnce() -> Result<R> + Send + 'static,
>(
  dispatcher: &mut D,
  task: F,
  callback: String,
  error: String,
) {
  let mut dispatcher_ = dispatcher.clone();
  dispatcher.send_event(Event::Run(Box::new(move || {
    let callback_string =
      match format_callback_result(task().map_err(|err| err.to_string()), &callback, &error) {
        Ok(js) => js,
        Err(e) => {
          format_callback_result(Result::<(), String>::Err(e.to_string()), &callback, &error)
            .unwrap()
        }
      };
    dispatcher_.eval(callback_string.as_str());
  })));
}

/// Asynchronously executes the given task
/// and evaluates its Result to the JS promise described by the `success_callback` and `error_callback` function names.
///
/// If the Result `is_ok()`, the callback will be the `success_callback` function name and the argument will be the Ok value.
/// If the Result `is_err()`, the callback will be the `error_callback` function name and the argument will be the Err value.
pub async fn execute_promise<
  D: ApplicationDispatcherExt,
  R: Serialize,
  F: futures::Future<Output = Result<R>> + Send + 'static,
>(
  dispatcher: &mut D,
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
  dispatcher.eval(callback_string.as_str());
}

/// Calls the given command and evaluates its output to the JS promise described by the `callback` and `error` function names.
pub async fn call<D: ApplicationDispatcherExt>(
  dispatcher: &mut D,
  command: String,
  args: Vec<String>,
  callback: String,
  error: String,
) {
  execute_promise(
    dispatcher,
    async move { api::command::get_output(command, args, Stdio::piped()) },
    callback,
    error,
  )
  .await;
}

/// Closes the splashscreen.
pub fn close_splashscreen<D: ApplicationDispatcherExt>(dispatcher: &mut D) -> crate::Result<()> {
  // send a signal to the runner so it knows that it should redirect to the main app content
  dispatcher
    .eval(r#"window.__TAURI_INVOKE_HANDLER__(JSON.stringify({ cmd: "closeSplashscreen" }))"#);

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
