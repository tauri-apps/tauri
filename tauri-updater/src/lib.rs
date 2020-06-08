use std::cmp::min;
use std::env;
use std::io;
use std::path::PathBuf;

#[macro_use]
pub mod macros;
pub mod errors;
pub mod http;
pub mod updater;

use errors::*;

use crate::updater::ReleaseUpdate;

/// Release information
#[derive(Clone, Debug, Default)]
pub struct Release {
  pub version: String,
  pub date: String,
  pub download_url: String,
  pub body: Option<String>,
  pub should_update: bool,
}

impl Release {
  pub fn get_download_url(&self) -> String {
    self.download_url.clone()
  }
}

pub enum CheckStatus {
  UpToDate,
  UpdateAvailable(Release),
}

pub enum InstallStatus {
  Installed,
  Failed,
}

pub enum ProgressStatus {
  Download(u64),
  Extract,
  CopyFiles,
}

pub enum DownloadStatus {
  Downloaded(DownloadedArchive),
  Failed,
}

/// Download things into files
#[derive(Debug)]
pub struct Download {
  url: String,
  headers: reqwest::header::HeaderMap,
}

#[derive(Debug)]
pub struct DownloadedArchive {
  bin_name: String,
  archive_path: PathBuf,
  tmp_dir: tempfile::TempDir,
}

/// Returns a target os
pub fn get_target() -> &'static str {
  if cfg!(target_os = "linux") {
    "linux"
  } else if cfg!(target_os = "macos") {
    "darwin"
  } else if cfg!(target_os = "windows") {
    if env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap() == "64" {
      return "win64";
    }
    "win32"
  } else if cfg!(target_os = "freebsd") {
    "freebsd"
  } else {
    ""
  }
}
