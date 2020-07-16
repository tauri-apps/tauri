//! Tauri utility helpers
#![warn(missing_docs, rust_2018_idioms)]

/// Platform helpers
pub mod platform;
/// Process helpers
pub mod process;

pub use anyhow::Result;
use thiserror::Error;

/// The error types.
#[derive(Error, Debug)]
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
  /// Target triple unknown target-os error
  #[error("Unknown target_os")]
  Unknown,
  /// Get parent process error
  #[error("Could not get parent process")]
  ParentProcess,
  /// Get parent process PID error
  #[error("Could not get parent PID")]
  ParentPID,
  /// Get child process error
  #[error("Could not get child process")]
  ChildProcess,
}
