// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![warn(missing_docs, rust_2018_idioms)]
#![allow(
    // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/7422
    clippy::nonstandard_macro_braces,
)]

//! The Tauri bundler is a tool that generates installers or app bundles for executables.
//! It supports auto updating through [tauri](https://docs.rs/tauri).
//!
//! # Platform support
//! - macOS
//!   - DMG and App bundles
//! - Linux
//!   - Appimage and Debian packages
//! - Windows
//!   - MSI using WiX

/// The bundle API.
pub mod bundle;
mod error;
pub use bundle::*;
pub use error::{Error, Result};
