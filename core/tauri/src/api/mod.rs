// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri API interface.
#![warn(missing_docs)]
// #![feature(const_int_pow)]

pub mod dialog;
pub mod dir;
pub mod file;
pub mod http;
pub mod ipc;
pub mod path;
pub mod process;
#[cfg(shell_open)]
pub mod shell;
pub mod version;

#[cfg(feature = "cli")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "cli")))]
pub mod cli;

#[cfg(feature = "cli")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "cli")))]
pub use clap;

#[cfg(notification_all)]
pub mod notification;

mod error;

/// The error type of Tauri API module.
pub use error::Error;
/// The result type of Tauri API module.
pub type Result<T> = std::result::Result<T, Error>;

// Not public API
#[doc(hidden)]
pub mod private {
  pub use once_cell::sync::OnceCell;

  pub trait AsTauriContext {
    fn config() -> &'static crate::Config;
    fn assets() -> &'static crate::utils::assets::EmbeddedAssets;
    fn default_window_icon() -> Option<&'static [u8]>;
    fn package_info() -> crate::PackageInfo;
  }
}
