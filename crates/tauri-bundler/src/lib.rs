// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri bundler is a tool that generates installers or app bundles for executables.
//! It supports auto updating through [tauri](https://docs.rs/tauri).
//!
//! # Platform support
//! - macOS
//!   - DMG and App bundles
//! - Linux
//!   - Appimage, Debian and RPM packages
//! - Windows
//!   - MSI using WiX

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]
#![warn(missing_docs, rust_2018_idioms)]

/// The bundle API.
pub mod bundle;
mod error;
mod utils;
pub use bundle::*;
pub use error::{Error, Result};
