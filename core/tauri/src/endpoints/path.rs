// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;
use crate::{api::path::BaseDirectory, Config, PackageInfo};
use serde::Deserialize;

#[cfg(path_all)]
use crate::api::path::resolve_path;
#[cfg(path_all)]
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  ResolvePath {
    path: String,
    directory: Option<BaseDirectory>,
  },
  Resolve {
    paths: Vec<String>,
  },
  Normalize {
    path: String,
  },
  Join {
    paths: Vec<String>,
  },
  Dirname {
    path: String,
  },
  Extname {
    path: String,
  },
  Basename {
    path: String,
    ext: Option<String>,
  },
}

impl Cmd {
  #[allow(unused_variables)]
  pub fn run(
    self,
    config: Arc<Config>,
    package_info: &PackageInfo,
  ) -> crate::Result<InvokeResponse> {
    #[cfg(path_all)]
    return match self {
      Cmd::ResolvePath { directory, path } => {
        resolve_path_handler(&config, package_info, path, directory).map(Into::into)
      }
      Cmd::Resolve { paths } => resolve(paths).map(Into::into),
      Cmd::Normalize { path } => normalize(path).map(Into::into),
      Cmd::Join { paths } => join(paths).map(Into::into),
      Cmd::Dirname { path } => dirname(path).map(Into::into),
      Cmd::Extname { path } => extname(path).map(Into::into),
      Cmd::Basename { path, ext } => basename(path, ext).map(Into::into),
    };
    #[cfg(not(path_all))]
    Err(crate::Error::ApiNotAllowlisted("path".into()))
  }
}

#[cfg(path_all)]
pub fn resolve_path_handler(
  config: &Config,
  package_info: &PackageInfo,
  path: String,
  directory: Option<BaseDirectory>,
) -> crate::Result<PathBuf> {
  resolve_path(config, package_info, path, directory).map_err(Into::into)
}

#[cfg(path_all)]
pub fn resolve(paths: Vec<String>) -> crate::Result<String> {
  // start with the current directory
  let mut resolved_path = PathBuf::new().join(".");

  for path in paths {
    let path_buf = PathBuf::from(path);

    // if we encounter an absolute path, we use it as the starting path for next iteration
    if path_buf.is_absolute() {
      resolved_path = path_buf;
    } else {
      resolved_path = resolved_path.join(&path_buf);
    }
  }

  normalize(resolved_path.to_string_lossy().to_string())
}

#[cfg(path_all)]
pub fn normalize(path: String) -> crate::Result<String> {
  let path = std::fs::canonicalize(path)?;
  let path = path.to_string_lossy().to_string();

  // remove `\\\\?\\` on windows, UNC path
  #[cfg(target_os = "windows")]
  let path = path.replace("\\\\?\\", "");

  Ok(path)
}

#[cfg(path_all)]
pub fn join(paths: Vec<String>) -> crate::Result<String> {
  let mut joined_path = PathBuf::new();
  for path in paths {
    joined_path = joined_path.join(path);
  }
  normalize(joined_path.to_string_lossy().to_string())
}

#[cfg(path_all)]
pub fn dirname(path: String) -> crate::Result<String> {
  match Path::new(&path).parent() {
    Some(path) => Ok(path.to_string_lossy().to_string()),
    None => Err(crate::Error::FailedToExecuteApi(crate::api::Error::Path(
      "Couldn't get the parent directory".into(),
    ))),
  }
}

#[cfg(path_all)]
pub fn extname(path: String) -> crate::Result<String> {
  match Path::new(&path)
    .extension()
    .and_then(std::ffi::OsStr::to_str)
  {
    Some(path) => Ok(path.to_string()),
    None => Err(crate::Error::FailedToExecuteApi(crate::api::Error::Path(
      "Couldn't get the extension of the file".into(),
    ))),
  }
}

#[cfg(path_all)]
pub fn basename(path: String, ext: Option<String>) -> crate::Result<String> {
  match Path::new(&path)
    .file_name()
    .and_then(std::ffi::OsStr::to_str)
  {
    Some(path) => Ok(if let Some(ext) = ext {
      path.replace(ext.as_str(), "")
    } else {
      path.to_string()
    }),
    None => Err(crate::Error::FailedToExecuteApi(crate::api::Error::Path(
      "Couldn't get the basename".into(),
    ))),
  }
}
