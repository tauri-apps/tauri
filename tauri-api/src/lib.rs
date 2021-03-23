//! The Tauri API interface.
#![warn(missing_docs, rust_2018_idioms)]

/// The Command API module allows you to manage child processes.
pub mod command;
/// The Dialog API module allows you to show messages and prompt for file paths.
pub mod dialog;
/// The Dir module is a helper for file system directory management.
pub mod dir;
/// The File API module contains helpers to perform file operations.
pub mod file;
/// The HTTP request API.
pub mod http;
/// The file system path operations API.
pub mod path;
/// The RPC module includes utilities to send messages to the JS layer of the webview.
pub mod rpc;
/// The shell api.
pub mod shell;
/// TCP ports access API.
pub mod tcp;
/// The semver API.
pub mod version;

/// The Tauri config definition.
pub use tauri_utils::config;

/// The CLI args interface.
#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
#[macro_use]
extern crate clap;

/// Global shortcuts interface.
#[cfg(feature = "global-shortcut")]
pub mod shortcuts;

/// The desktop notifications API module.
#[cfg(feature = "notification")]
pub mod notification;

pub use tauri_utils::*;

mod error;

/// Tauri API error.
pub use error::Error;
/// Tauri API result type.
pub type Result<T> = std::result::Result<T, Error>;

/// `App` package information.
#[derive(Debug, Clone)]
pub struct PackageInfo {
  /// App name.
  pub name: &'static str,
  /// App version.
  pub version: &'static str,
}

// Not public API
#[doc(hidden)]
pub mod private {
  pub use once_cell::sync::OnceCell;

  pub trait AsTauriContext {
    fn config() -> &'static crate::config::Config;
    fn assets() -> &'static crate::assets::EmbeddedAssets;
    fn default_window_icon() -> Option<&'static [u8]>;
    fn package_info() -> crate::PackageInfo;
  }
}
