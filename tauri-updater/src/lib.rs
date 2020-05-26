#[macro_use]
pub mod macros;
pub mod http;
pub mod updater;

pub use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Download request failed with status:{0}")]
  Download(attohttpc::StatusCode),
  #[error("Failed to determine parent dir")]
  Updater,
  #[error("Network Error:{0}")]
  Network(String),
  #[error("Config Error:{0} required")]
  Config(String),
}
