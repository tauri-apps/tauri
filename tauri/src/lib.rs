#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[cfg(assets)]
pub mod assets;
pub mod event;
#[cfg(embedded_server)]
pub mod server;
pub mod settings;

#[cfg(cli)]
pub mod cli;

mod app;
mod endpoints;
#[allow(dead_code)]
mod salt;

use std::process::Stdio;

pub use anyhow::Result;
use threadpool::ThreadPool;

pub use web_view::Handle;
use web_view::WebView;

pub use app::*;
pub use tauri_api as api;
thread_local!(static POOL: ThreadPool = ThreadPool::new(4));

pub fn spawn<F: FnOnce() -> () + Send + 'static>(task: F) {
  POOL.with(|thread| {
    thread.execute(move || {
      task();
    });
  });
}

pub fn execute_promise_sync<T: 'static, F: FnOnce() -> crate::Result<String> + Send + 'static>(
  webview: &mut WebView<'_, T>,
  task: F,
  callback: String,
  error: String,
) {
  let handle = webview.handle();
  let callback_string =
    api::rpc::format_callback_result(task().map_err(|err| err.to_string()), callback, error);
  handle
    .dispatch(move |_webview| _webview.eval(callback_string.as_str()))
    .expect("Failed to dispatch promise callback");
}

pub fn execute_promise<T: 'static, F: FnOnce() -> crate::Result<String> + Send + 'static>(
  webview: &mut WebView<'_, T>,
  task: F,
  callback: String,
  error: String,
) {
  let handle = webview.handle();
  POOL.with(|thread| {
    thread.execute(move || {
      let callback_string =
        api::rpc::format_callback_result(task().map_err(|err| err.to_string()), callback, error);
      handle
        .dispatch(move |_webview| _webview.eval(callback_string.as_str()))
        .expect("Failed to dispatch promise callback")
    });
  });
}

pub fn call<T: 'static>(
  webview: &mut WebView<'_, T>,
  command: String,
  args: Vec<String>,
  callback: String,
  error: String,
) {
  execute_promise(
    webview,
    || {
      api::command::get_output(command, args, Stdio::piped())
        .map_err(|err| err)
        .map(|output| format!(r#""{}""#, output))
    },
    callback,
    error,
  );
}

pub fn close_splashscreen<T: 'static>(webview_handle: &Handle<T>) -> crate::Result<()> {
  webview_handle.dispatch(|webview| {
    // send a signal to the runner so it knows that it should redirect to the main app content
    webview.eval(r#"window.external.invoke(JSON.stringify({ cmd: "closeSplashscreen" }))"#)
  })?;
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
      let dummy_task = move || {
        format!("{}-run-dummy-task", task);
        assert!(true);
      };
      // call spawn
      crate::spawn(dummy_task);
    }
  }
}
