#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate lazy_static;

pub mod config;
mod endpoints;
pub mod event;

#[cfg(feature = "embedded-server")]
pub mod server;

#[allow(dead_code)]
mod file_system;
#[allow(dead_code)]
mod salt;

#[cfg(feature = "embedded-server")]
mod tcp;

mod app;
#[cfg(not(feature = "dev-server"))]
pub mod assets;

use std::process::Stdio;

use threadpool::ThreadPool;

pub use app::*;
use web_view::WebView;

pub use tauri_api as api;

// Result alias
type TauriResult<T> = Result<T, Box<dyn std::error::Error>>;

thread_local!(static POOL: ThreadPool = ThreadPool::new(4));

pub fn spawn<F: FnOnce() -> () + Send + 'static>(task: F) {
  POOL.with(|thread| {
    thread.execute(move || {
      task();
    });
  });
}

pub fn execute_promise<T: 'static, F: FnOnce() -> Result<String, String> + Send + 'static>(
  webview: &mut WebView<'_, T>,
  task: F,
  callback: String,
  error: String,
) {
  let handle = webview.handle();
  POOL.with(|thread| {
    thread.execute(move || {
      let callback_string = api::rpc::format_callback_result(task(), callback, error);
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
        .map_err(|err| format!("`{}`", err))
        .map(|output| format!("`{}`", output))
    },
    callback,
    error,
  );
}

#[cfg(test)]
mod test {
  use proptest::prelude::*;

  proptest! {
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
