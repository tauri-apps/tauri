use std::{
  fs::File,
  io,
  sync::Mutex,
  time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

use crate::{api::file::SafePathBuf, error::into_anyhow, resources::Resource};

pub(crate) fn file_url_to_safe_pathbuf(path: SafePathBuf) -> crate::endpoints::Result<SafePathBuf> {
  if path.as_ref().starts_with("file:") {
    SafePathBuf::new(
      url::Url::parse(&path.display().to_string())?
        .to_file_path()
        .map_err(|_| into_anyhow("Failed to get path from `file:` url"))?,
    )
    .map_err(into_anyhow)
  } else {
    Ok(path)
  }
}

// taken from deno source code: https://github.com/denoland/deno/blob/ffffa2f7c44bd26aec5ae1957e0534487d099f48/runtime/ops/fs.rs#L913
fn to_msec(maybe_time: Result<SystemTime, io::Error>) -> Option<u64> {
  match maybe_time {
    Ok(time) => {
      let msec = time
        .duration_since(UNIX_EPOCH)
        .map(|t| t.as_millis() as u64)
        .unwrap_or_else(|err| err.duration().as_millis() as u64);
      Some(msec)
    }
    Err(_) => None,
  }
}

// taken from deno source code: https://github.com/denoland/deno/blob/ffffa2f7c44bd26aec5ae1957e0534487d099f48/runtime/ops/fs.rs#L926
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
  is_file: bool,
  is_directory: bool,
  is_symlink: bool,
  size: u64,
  // In milliseconds, like JavaScript. Available on both Unix or Windows.
  mtime: Option<u64>,
  atime: Option<u64>,
  birthtime: Option<u64>,
  // Following are only valid under Unix.
  dev: u64,
  ino: u64,
  mode: u32,
  nlink: u64,
  uid: u32,
  gid: u32,
  rdev: u64,
  blksize: u64,
  blocks: u64,
}

// taken from deno source code: https://github.com/denoland/deno/blob/ffffa2f7c44bd26aec5ae1957e0534487d099f48/runtime/ops/fs.rs#L950
#[inline(always)]
pub fn get_stat(metadata: std::fs::Metadata) -> FileInfo {
  // Unix stat member (number types only). 0 if not on unix.
  macro_rules! usm {
    ($member:ident) => {{
      #[cfg(unix)]
      {
        metadata.$member()
      }
      #[cfg(not(unix))]
      {
        0
      }
    }};
  }

  #[cfg(unix)]
  use std::os::unix::fs::MetadataExt;
  FileInfo {
    is_file: metadata.is_file(),
    is_directory: metadata.is_dir(),
    is_symlink: metadata.file_type().is_symlink(),
    size: metadata.len(),
    // In milliseconds, like JavaScript. Available on both Unix or Windows.
    mtime: to_msec(metadata.modified()),
    atime: to_msec(metadata.accessed()),
    birthtime: to_msec(metadata.created()),
    // Following are only valid under Unix.
    dev: usm!(dev),
    ino: usm!(ino),
    mode: usm!(mode),
    nlink: usm!(nlink),
    uid: usm!(uid),
    gid: usm!(gid),
    rdev: usm!(rdev),
    blksize: usm!(blksize),
    blocks: usm!(blocks),
  }
}

pub struct StdFileResource(Mutex<File>);

impl StdFileResource {
  pub fn new(file: File) -> Self {
    Self(Mutex::new(file))
  }

  pub fn with_lock<R, F: FnMut(&File) -> R>(&self, mut f: F) -> R {
    let file = self.0.lock().unwrap();
    f(&file)
  }
}

impl Resource for StdFileResource {}
