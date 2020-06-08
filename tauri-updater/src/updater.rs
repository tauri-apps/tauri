use reqwest::{self, header};
use std::env;
use std::fs;

use tauri_api::{file::Extract, file::Move};

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
  errors::*, CheckStatus, Download, DownloadStatus, DownloadedArchive, InstallStatus, Release,
};

/// Updates to a specified or latest release
pub trait ReleaseUpdate {
  /// Current version of binary being updated
  fn current_version(&self) -> String;

  /// Target platform the update is being performed for
  fn target(&self) -> String;

  /// API Url
  fn url(&self) -> String;

  /// Where is located current App to update -- extract path will automatically generated based on the target
  fn executable_path(&self) -> PathBuf;

  /// Where we need to extract the archive content
  fn extract_path(&self) -> PathBuf;

  // Should we update?
  fn status(&self) -> CheckStatus;

  // Get the release details
  fn release_details(&self) -> Release;

  fn download(&self) -> Result<DownloadStatus> {
    // get OS
    let target = self.target();
    // get release extracted in check()
    let release = self.release_details();
    // download url for selected release
    let url = release.get_download_url();
    // extract path
    let extract_path = self.extract_path();
    // tmp dir
    let tmp_dir_parent = if cfg!(windows) {
      env::var_os("TEMP").map(PathBuf::from)
    } else {
      extract_path.parent().map(PathBuf::from)
    }
    .ok_or_else(|| Error::Update("Failed to determine parent dir".into()))?;

    // used for temp file name
    // if we cant extract app name, we use unix epoch duration
    let bin_name = std::env::current_exe()
      .ok()
      .and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
      .and_then(|s| s.into_string().ok())
      .unwrap_or(
        SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .unwrap()
          .subsec_nanos()
          .to_string(),
      );

    // tmp dir for extraction
    let tmp_dir = tempfile::Builder::new()
      .prefix(&format!("{}_download", bin_name))
      .tempdir_in(tmp_dir_parent)?;

    let tmp_archive_path = tmp_dir.path().join(detect_archive_in_url(&url, &target));
    let mut tmp_archive = fs::File::create(&tmp_archive_path)?;

    // prepare our download
    let mut download = Download::from_url(&url);

    // set our headers
    let mut headers = header::HeaderMap::new();
    headers.insert(header::ACCEPT, "application/octet-stream".parse().unwrap());
    download.set_headers(headers);

    // download the file
    download.download_to(&mut tmp_archive)?;

    Ok(DownloadStatus::Downloaded(DownloadedArchive {
      archive_path: tmp_archive_path,
      tmp_dir,
      bin_name,
    }))
  }

  fn install(&self, archive: DownloadedArchive) -> Result<InstallStatus> {
    // extract using tauri api  inside a tmp path
    let extract_process =
      Extract::from_source(&archive.archive_path).extract_into(&archive.tmp_dir.path());
    match extract_process {
      Ok(_) => (),
      Err(err) => bail!(Error::Update, "Extract failed with status: {:?}", err),
    };

    let tmp_file = archive
      .tmp_dir
      .path()
      .join(&format!("__{}_backup", archive.bin_name));

    // move into the final position
    let move_process = Move::from_source(&archive.tmp_dir.path())
      .replace_using_temp(&tmp_file)
      .to_dest(&self.extract_path());

    match move_process {
      Ok(_) => Ok(InstallStatus::Installed),
      Err(err) => bail!(Error::Update, "Move failed with status: {:?}", err),
    }
  }
}

// Return the archive type to save on disk
fn detect_archive_in_url(path: &str, target: &str) -> String {
  path
    .split('/')
    .next_back()
    .unwrap_or(&archive_name_by_os(target))
    .to_string()
}

// Fallback archive name by os
// The main objective is to provide the right extension based on the target
// if we cant extract the archive type in the url we'll fallback to this value
fn archive_name_by_os(target: &str) -> String {
  let archive_name = match target {
    "darwin" | "linux" => "update.tar.gz",
    _ => "update.zip",
  };
  archive_name.to_string()
}
