// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tauri utility helpers
#![warn(missing_docs, rust_2018_idioms)]

/// The Assets module allows you to read files that have been bundled by tauri
pub mod assets;
/// Tauri config definition.
pub mod config;
/// Platform helpers
pub mod platform;
/// Process helpers
pub mod process;

/// Result type alias using the crate's error type.
pub type Result<T> = std::result::Result<T, Error>;

/// The error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// Target triple architecture error
  #[error("Unable to determine target-architecture")]
  Architecture,
  /// Target triple OS error
  #[error("Unable to determine target-os")]
  OS,
  /// Target triple environment error
  #[error("Unable to determine target-environment")]
  Environment,
  /// Tried to get resource on an unsupported platform.
  #[error("Unsupported platform for reading resources")]
  UnsupportedPlatform,
  /// Get parent process error
  #[error("Could not get parent process")]
  ParentProcess,
  /// Get parent process PID error
  #[error("Could not get parent PID")]
  ParentPID,
  /// Get child process error
  #[error("Could not get child process")]
  ChildProcess,
  /// IO error.
  #[error("{0}")]
  Io(#[from] std::io::Error),
}
