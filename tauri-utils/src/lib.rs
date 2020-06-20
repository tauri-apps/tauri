pub mod platform;
pub mod process;

pub use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Unable to determine target-architecture")]
  Architecture,
  #[error("Unable to determine target-os")]
  OS,
  #[error("Unable to determine target-environment")]
  Environment,
  #[error("Unknown target_os")]
  Unknown,
  #[error("Could not get parent process")]
  ParentProcess,
  #[error("Could not get parent PID")]
  ParentPID,
  #[error("Could not get child process")]
  ChildProcess,
}
