pub use tempfile::TempDir;

use reqwest::header;
use std::cmp::min;
use std::env;
use std::io;

#[macro_use]
pub mod macros;
pub mod errors;
pub mod http;
pub mod updater;

use errors::*;

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

/// Download things into files
#[derive(Debug)]
pub struct Download {
  url: String,
  headers: reqwest::header::HeaderMap,
}

impl Download {
  /// Specify download url
  pub fn from_url(url: &str) -> Self {
    Self {
      url: url.to_owned(),
      headers: reqwest::header::HeaderMap::new(),
    }
  }

  /// Set the download request headers
  pub fn set_headers(&mut self, headers: reqwest::header::HeaderMap) -> &mut Self {
    self.headers = headers;
    self
  }

  pub fn download_to<T: io::Write>(&self, mut dest: T) -> Result<()> {
    use io::BufRead;
    let mut headers = self.headers.clone();
    if !headers.contains_key(header::USER_AGENT) {
      headers.insert(
        header::USER_AGENT,
        "tauri/updater".parse().expect("invalid user-agent"),
      );
    }

    set_ssl_vars!();
    let resp = reqwest::blocking::Client::new()
      .get(&self.url)
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
      downloaded = min(downloaded + n as u64, size);
    }

    Ok(())
  }
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
