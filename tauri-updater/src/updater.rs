use reqwest::{self, header};
use std::cmp::min;
use std::env;
use std::fs;
use std::io;
use tauri_api::{file::Extract, file::Move};

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
  errors::*, CheckStatus, DownloadStatus, DownloadedArchive, InstallStatus, ProgressStatus, Release,
};

/// Updates to a specified or latest release
pub trait ReleaseUpdate {
  /// Current version of binary being updated
  fn current_version(&self) -> String;

  /// Target platform the update is being performed for
  fn target(&self) -> String;

  /// Where is located current App to update -- extract path will automatically generated based on the target
  fn executable_path(&self) -> PathBuf;

  /// Where we need to extract the archive content
  fn extract_path(&self) -> PathBuf;

  // Should we update?
  fn status(&self) -> CheckStatus;

  // Get the release details
  fn release_details(&self) -> Release;

  fn send_progress(&self, status: ProgressStatus);

  fn download(&self) -> Result<DownloadStatus> {
    // send event that we start the download process at 0%
    self.send_progress(ProgressStatus::Download(0));

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
    let tmp_archive = fs::File::create(&tmp_archive_path)?;

    // prepare our download
    use io::BufRead;
    use std::io::Write;

    // set our headers
    let mut headers = header::HeaderMap::new();
    headers.insert(header::ACCEPT, "application/octet-stream".parse().unwrap());

    if !headers.contains_key(header::USER_AGENT) {
      headers.insert(
        header::USER_AGENT,
        "tauri/updater".parse().expect("invalid user-agent"),
      );
    }

    set_ssl_vars!();
    let resp = reqwest::blocking::Client::new()
      .get(&url)
      .headers(headers)
      .send()?;
    let size = resp
      .headers()
      .get(reqwest::header::CONTENT_LENGTH)
      .map(|val| {
        val
          .to_str()
          .map(|s| s.parse::<u64>().unwrap_or(0))
          .unwrap_or(0)
      })
      .unwrap_or(0);
    if !resp.status().is_success() {
      bail!(
        Error::Update,
        "Download request failed with status: {:?}",
        resp.status()
      )
    }

    let mut src = io::BufReader::new(resp);
    let mut downloaded = 0;
    let mut dest = &tmp_archive;

    loop {
      let n = {
        let buf = src.fill_buf()?;
        dest.write_all(&buf)?;
        buf.len()
      };
      if n == 0 {
        break;
      }
      src.consume(n);
      // calc the progress
      downloaded = min(downloaded + n as u64, size);
      // send progress to our listener in percent
      self.send_progress(ProgressStatus::Download((downloaded * 100) / size));
    }

    Ok(DownloadStatus::Downloaded(DownloadedArchive {
      archive_path: tmp_archive_path,
      tmp_dir,
      bin_name,
    }))
  }

  fn install(&self, archive: DownloadedArchive) -> Result<InstallStatus> {
    // send event that we start the extracting
    self.send_progress(ProgressStatus::Extract);

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
    self.send_progress(ProgressStatus::CopyFiles);
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
