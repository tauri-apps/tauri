#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[cfg(not(feature = "dev-server"))]
pub mod assets;
pub mod config;
pub mod event;
#[cfg(feature = "embedded-server")]
pub mod server;

mod app;
mod endpoints;
#[allow(dead_code)]
mod file_system;
#[allow(dead_code)]
mod salt;
#[cfg(feature = "embedded-server")]
mod tcp;
pub mod extension;

use std::process::Stdio;

use error_chain::error_chain;
use threadpool::ThreadPool;

pub use web_view::Handle;
pub use web_view::WebView;

pub use app::*;
pub use tauri_api as api;

error_chain! {
  foreign_links{
    Api(::tauri_api::Error);
    Json(::serde_json::Error);
    Webview(::web_view::Error);
    Io(::std::io::Error);
  }
  errors{
    Promise(t: String) {
        description("Promise Error")
        display("Promise Error: '{}'", t)
    }
    Command(t: String) {
      description("Command Error")
      display("Command Error: '{}'", t)
    }
  }
}

thread_local!(static POOL: ThreadPool = ThreadPool::new(4));

pub fn spawn<F: FnOnce() -> () + Send + 'static>(task: F) {
  POOL.with(|thread| {
    thread.execute(move || {
      task();
    });
  });
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
        .map_err(|err| crate::ErrorKind::Promise(err.to_string()).into())
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
