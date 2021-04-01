//! Tauri is a framework for building tiny, blazing fast binaries for all major desktop platforms.
//! Developers can integrate any front-end framework that compiles to HTML, JS and CSS for building their user interface.
//! The backend of the application is a rust-sourced binary with an API that the front-end can interact with.
//!
//! The user interface in Tauri apps currently leverages Cocoa/WebKit on macOS, gtk-webkit2 on Linux and MSHTML (IE10/11) or Webkit via Edge on Windows.
//! Tauri uses (and contributes to) the MIT licensed project that you can find at [webview](https://github.com/webview/webview).
#![warn(missing_docs, rust_2018_idioms)]

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

/// The internal runtime between an [`App`] and the webview.
mod runtime;

/// The Tauri error enum.
pub use error::Error;

/// Tauri result type.
pub type Result<T> = std::result::Result<T, Error>;

/// A task to run on the main thread.
pub type SyncTask = Box<dyn FnOnce() + Send>;

pub use app::*;
pub use tauri_api as api;
pub(crate) use tauri_api::private::async_runtime;
pub use tauri_macros::*;

pub use crate::{
  app::{Context, Manager, Tag},
  runtime::{Dispatch, Runtime},
};

/// The Tauri webview implementations.
pub mod flavors {
  pub use super::app::{webview::wry::WryDispatcher, WryApplication as Wry};
}

/// Easy helper function to use the Tauri Context you made during build time.
#[macro_export]
macro_rules! tauri_build_context {
  () => {
    include!(concat!(env!("OUT_DIR"), "/tauri-build-context.rs"))
  };
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
