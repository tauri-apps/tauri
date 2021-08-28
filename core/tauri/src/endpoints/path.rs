// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;
use crate::{api::path::BaseDirectory, Config, PackageInfo};
use serde::Deserialize;
#[cfg(path_all)]
use std::path::{Component, Path, PathBuf, MAIN_SEPARATOR};
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
  IsAbsolute {
    path: String,
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
      Cmd::IsAbsolute { path } => Ok(Path::new(&path).is_absolute()).map(Into::into),
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
  crate::api::path::resolve_path(config, package_info, path, directory).map_err(Into::into)
}

#[cfg(path_all)]
fn resolve(paths: Vec<String>) -> crate::Result<String> {
  // Start with current directory path because users might pass empty or vec!["."]
  // then start adding paths from the vector one by one using path.push()
  // so if an absolute path is encountered in the iteration, we use it as the current full path
  // examples:
  // 1. vec!["."] or vec![] will be equal to std::env::current_dir()
  // 2. vec!["/foo/bar", "/tmp/file", "baz"] will be equal to PathBuf::from("/tmp/file/baz")
  let mut path = std::env::current_dir()?;
  for p in paths {
    path.push(p);
  }
  Ok(normalize_path(&path).to_string_lossy().to_string())
}

#[cfg(path_all)]
fn join(paths: Vec<String>) -> crate::Result<String> {
  let path = PathBuf::from(
    paths
      .iter()
      .map(|p| {
        // Add MAIN_SEPARATOR if this is not the first element in the vector
        // and if it doesn't already have a spearator.
        // Doing this to ensure that the vector elements are separated in
        // the resulting string so path.components() can work correctly when called
        //  in normalize_path_no_absolute() later
        if !p.starts_with("/") && !p.starts_with("\\") && p != &paths[0] {
          let mut tmp = String::from(MAIN_SEPARATOR);
          tmp.push_str(p);
          tmp
        } else {
          p.to_string()
        }
      })
      .collect::<String>(),
  );

  let p = normalize_path_no_absolute(&path)
    .to_string_lossy()
    .to_string();
  Ok(if p.is_empty() { ".".into() } else { p })
}

#[cfg(path_all)]
fn normalize(path: String) -> crate::Result<String> {
  let mut p = normalize_path_no_absolute(Path::new(&path))
    .to_string_lossy()
    .to_string();
  Ok(if p.is_empty() {
    // Nodejs will return ".." if we used normalize("..")
    // and will return "." if we used normalize("") or normalize(".")
    String::from(if path == ".." { path } else { ".".into() })
  } else {
    // If the path passed to this function contains a trailing separator,
    // we make sure to perserve it. That's how NodeJS works
    if (path.ends_with("/") || path.ends_with("\\")) && (!p.ends_with("/") || !p.ends_with("\\")) {
      p.push(MAIN_SEPARATOR);
    }
    p
  })
}

#[cfg(path_all)]
fn dirname(path: String) -> crate::Result<String> {
  match Path::new(&path).parent() {
    Some(p) => Ok(p.to_string_lossy().to_string()),
    None => Err(crate::Error::FailedToExecuteApi(crate::api::Error::Path(
      "Couldn't get the parent directory".into(),
    ))),
  }
}

#[cfg(path_all)]
fn extname(path: String) -> crate::Result<String> {
  match Path::new(&path)
    .extension()
    .and_then(std::ffi::OsStr::to_str)
  {
    Some(p) => Ok(p.to_string()),
    None => Err(crate::Error::FailedToExecuteApi(crate::api::Error::Path(
      "Couldn't get the extension of the file".into(),
    ))),
  }
}

#[cfg(path_all)]
fn basename(path: String, ext: Option<String>) -> crate::Result<String> {
  match Path::new(&path)
    .file_name()
    .and_then(std::ffi::OsStr::to_str)
  {
    Some(p) => Ok(if let Some(ext) = ext {
      p.replace(ext.as_str(), "")
    } else {
      p.to_string()
    }),
    None => Err(crate::Error::FailedToExecuteApi(crate::api::Error::Path(
      "Couldn't get the basename".into(),
    ))),
  }
}

/// Resolve ".." and "." if there is any , this snippet is taken from cargo's paths util
/// https://github.com/rust-lang/cargo/blob/46fa867ff7043e3a0545bf3def7be904e1497afd/crates/cargo-util/src/paths.rs#L73-L106
#[cfg(path_all)]
fn normalize_path(path: &Path) -> PathBuf {
  let mut components = path.components().peekable();
  let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
    components.next();
    PathBuf::from(c.as_os_str())
  } else {
    PathBuf::new()
  };

  for component in components {
    match component {
      Component::Prefix(..) => unreachable!(),
      Component::RootDir => {
        ret.push(component.as_os_str());
      }
      Component::CurDir => {}
      Component::ParentDir => {
        ret.pop();
      }
      Component::Normal(c) => {
        ret.push(c);
      }
    }
  }
  ret
}

/// Resolve ".." and "." if there is any , this snippet is taken from cargo's paths util but
/// slightly modified to not resolve absolute paths
/// https://github.com/rust-lang/cargo/blob/46fa867ff7043e3a0545bf3def7be904e1497afd/crates/cargo-util/src/paths.rs#L73-L106
#[cfg(path_all)]
fn normalize_path_no_absolute(path: &Path) -> PathBuf {
  let mut components = path.components().peekable();
  let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
    components.next();
    PathBuf::from(c.as_os_str())
  } else {
    PathBuf::new()
  };

  for component in components {
    match component {
      Component::Prefix(..) => unreachable!(),
      Component::RootDir => {
        ret.push(component.as_os_str());
      }
      Component::CurDir => {}
      Component::ParentDir => {
        ret.pop();
      }
      Component::Normal(c) => {
        // Using PathBuf::push here will replace the whole path if an absolute path is encountered
        // which is not the intended behavior, so instead of that, convert the current resolved path
        // to a string and do simple string concatenation with the current component then convert it
        // back to a PathBuf
        let mut p = ret.to_string_lossy().to_string();
        // Don't add the separator if the resolved path is empty,
        // otherwise we are gonna have unwanted leading separator
        if !p.is_empty() && !p.ends_with("/") && !p.ends_with("\\") {
          p.push(MAIN_SEPARATOR);
        }
        if let Some(c) = c.to_str() {
          p.push_str(c);
        }
        ret = PathBuf::from(p);
      }
    }
  }
  ret
}
