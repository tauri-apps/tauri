#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

pub mod command;
pub mod dialog;
pub mod dir;
pub mod file;
pub mod http;
pub mod path;
pub mod rpc;
pub mod tcp;
pub mod version;

pub use tauri_utils::*;

pub use anyhow::Result;
use thiserror::Error;

#[cfg(feature = "shortcut")]
pub mod shortcut;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Extract Error:{0}")]
  Extract(String),
  #[error("Command Error:{0}")]
  Command(String),
  #[error("File Error:{0}")]
  File(String),
  #[error("Path Error:{0}")]
  Path(String),
  #[error("Dialog Error:{0}")]
  Dialog(String),
  #[error("Network Error:{0}")]
  Network(attohttpc::StatusCode),
}
