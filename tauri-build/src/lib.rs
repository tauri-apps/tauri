#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate lazy_static;

mod api;
pub mod config;
pub mod event;

#[cfg(feature = "embedded-server")]
pub mod server;

#[allow(dead_code)]
mod file_system;
#[allow(dead_code)]
mod salt;

#[cfg(feature = "embedded-server")]
mod tcp;

#[cfg(not(feature = "dev-server"))]
pub mod assets;

mod app;

use std::process::Stdio;

use threadpool::ThreadPool;

pub use app::*;
use web_view::*;

thread_local!(static POOL: ThreadPool = ThreadPool::new(4));

pub fn spawn<F: FnOnce() -> () + Send + 'static>(what: F) {
  POOL.with(|thread| {
    thread.execute(move || {
      what();
    });
  });
}

pub fn execute_promise<T: 'static, F: FnOnce() -> Result<String, String> + Send + 'static>(
  webview: &mut WebView<'_, T>,
  what: F,
  callback: String,
  error: String,
) {
  let handle = webview.handle();
  POOL.with(|thread| {
    thread.execute(move || {
      let callback_string = tauri::rpc::format_callback_result(what(), callback, error);
      handle
        .dispatch(move |_webview| _webview.eval(callback_string.as_str()))
        .unwrap()
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
      tauri::command::get_output(command, args, Stdio::piped())
        .map_err(|err| format!("`{}`", err))
        .map(|output| format!("`{}`", output))
    },
    callback,
    error,
  );
}
