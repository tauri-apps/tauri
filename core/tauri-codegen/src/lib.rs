// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use self::context::{context_codegen, ContextData};
use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};
pub use tauri_utils::config::{parse::ConfigError, Config};

mod context;
pub mod embedded_assets;

/// Represents all the errors that can happen while reading the config during codegen.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CodegenConfigError {
  #[error("unable to access current working directory: {0}")]
  CurrentDir(std::io::Error),

  // this error should be "impossible" because we use std::env::current_dir() - cover it anyways
  #[error("Tauri config file has no parent, this shouldn't be possible. file an issue on https://github.com/tauri-apps/tauri - target {0}")]
  Parent(PathBuf),

  #[error("unable to parse inline JSON TAURI_CONFIG env var: {0}")]
  FormatInline(serde_json::Error),

  #[error("{0}")]
  ConfigError(#[from] ConfigError),
}

/// Get the [`Config`] from the `TAURI_CONFIG` environmental variable, or read from the passed path.
///
/// If the passed path is relative, it should be relative to the current working directory of the
/// compiling crate.
pub fn get_config(path: &Path) -> Result<(Config, PathBuf), CodegenConfigError> {
  let path = if path.is_relative() {
    let cwd = std::env::current_dir().map_err(CodegenConfigError::CurrentDir)?;
    Cow::Owned(cwd.join(path))
  } else {
    Cow::Borrowed(path)
  };

  // in the future we may want to find a way to not need the TAURI_CONFIG env var so that
  // it is impossible for the content of two separate configs to get mixed up. The chances are
  // already unlikely unless the developer goes out of their way to run the cli on a different
  // project than the target crate.
  let config = if let Ok(env) = std::env::var("TAURI_CONFIG") {
    serde_json::from_str(&env).map_err(CodegenConfigError::FormatInline)?
  } else {
    tauri_utils::config::parse(path.to_path_buf())?
  };

  // this should be impossible because of the use of `current_dir()` above, but handle it anyways
  let parent = path
    .parent()
    .map(ToOwned::to_owned)
    .ok_or_else(|| CodegenConfigError::Parent(path.into_owned()))?;

  Ok((config, parent))
}
