// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri API interface.

#[cfg(all(desktop, feature = "dialog"))]
#[cfg_attr(doc_cfg, doc(cfg(all(desktop, feature = "dialog"))))]
pub mod dialog;
pub mod dir;
pub mod file;
#[cfg(feature = "http-api")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "http-api")))]
pub mod http;
pub mod ipc;
#[cfg(feature = "os-api")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "os-api")))]
pub mod os;
pub mod path;
pub mod process;
#[cfg(feature = "shell-open-api")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "shell-open-api")))]
pub mod shell;
pub mod version;

#[cfg(feature = "cli")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "cli")))]
pub mod cli;

#[cfg(feature = "cli")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "cli")))]
pub use clap;

#[cfg(all(desktop, any(feature = "notification", feature = "dox")))]
#[cfg_attr(doc_cfg, doc(cfg(all(desktop, feature = "notification"))))]
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
