// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri API interface.
#![warn(missing_docs)]
// #![feature(const_int_pow)]

/// A module for working with processes.
pub mod dialog;
/// The Dir module is a helper for file system directory management.
pub mod dir;
/// The File API module contains helpers to perform file operations.
pub mod file;
/// The HTTP request API.
pub mod http;
/// The file system path operations API.
pub mod path;
/// The Command API module allows you to manage child processes.
pub mod process;
/// The RPC module includes utilities to send messages to the JS layer of the webview.
pub mod rpc;
/// The shell api.
#[cfg(shell_open)]
pub mod shell;
/// The semver API.
pub mod version;

/// The Tauri config definition.
pub use tauri_utils::config;

/// The CLI args interface.
#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "cli")]
pub use clap;

/// The desktop notifications API module.
#[cfg(notification_all)]
pub mod notification;

pub use tauri_utils::*;

mod error;

/// Tauri API error.
pub use error::Error;
/// Tauri API result type.
pub type Result<T> = std::result::Result<T, Error>;

// Not public API
#[doc(hidden)]
pub mod private {
  pub use once_cell::sync::OnceCell;

  pub trait AsTauriContext {
    fn config() -> &'static crate::api::config::Config;
    fn assets() -> &'static crate::api::assets::EmbeddedAssets;
    fn default_window_icon() -> Option<&'static [u8]>;
    fn package_info() -> crate::api::PackageInfo;
  }
}
