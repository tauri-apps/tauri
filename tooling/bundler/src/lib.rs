// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
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
//!   - Appimage and Debian packages
//! - Windows
//!   - MSI using WiX

pub(crate) trait CommandExt {
  fn pipe(&mut self) -> Result<&mut Self>;
}

impl CommandExt for std::process::Command {
  fn pipe(&mut self) -> Result<&mut Self> {
    self.stdout(os_pipe::dup_stdout()?);
    self.stderr(os_pipe::dup_stderr()?);
    Ok(self)
  }
}

/// The bundle API.
pub mod bundle;
mod error;
pub use bundle::*;
pub use error::{Error, Result};
