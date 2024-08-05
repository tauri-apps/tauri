// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![warn(missing_docs, rust_2018_idioms)]

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

/// The bundle API.
pub mod bundle;
mod error;
pub use bundle::*;
pub use error::{Error, Result};
