//! Tauri is a framework for building tiny, blazing fast binaries for all major desktop platforms.
//! Developers can integrate any front-end framework that compiles to HTML, JS and CSS for building their user interface.
//! The backend of the application is a rust-sourced binary with an API that the front-end can interact with.
//!
//! The user interface in Tauri apps currently leverages Cocoa/WebKit on macOS, gtk-webkit2 on Linux and MSHTML (IE10/11) or Webkit via Edge on Windows.
//! Tauri uses (and contributes to) the MIT licensed project that you can find at [webview](https://github.com/webview/webview).
#![warn(missing_docs, rust_2018_idioms)]

/// The embedded server helpers.
#[cfg(embedded_server)]
pub mod server;
/// The Tauri-specific settings for your app e.g. notification permission status.
pub mod settings;

/// The webview application entry.
mod app;
/// The Tauri API endpoints.
mod endpoints;
mod error;
/// The plugin manager module contains helpers to manage runtime plugins.
pub mod plugin;
/// The salt helpers.
mod salt;

/// The Tauri error enum.
pub use error::Error;
/// Tauri result type.
pub type Result<T> = std::result::Result<T, Error>;

pub(crate) mod async_runtime;

/// A task to run on the main thread.
pub type SyncTask = Box<dyn FnOnce() + Send>;

pub use app::*;
pub use tauri_api as api;
pub use tauri_macros::FromTauriContext;

/// The Tauri webview implementations.
pub mod flavors {
  pub use super::app::WryApplication as Wry;
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
