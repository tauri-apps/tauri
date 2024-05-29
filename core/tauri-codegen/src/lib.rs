// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/tauri/raw/dev/.github/splash.png)](https://tauri.app)
//!
//! - Embed, hash, and compress assets, including icons for the app as well as the tray icon.
//! - Parse `tauri.conf.json` at compile time and generate the Config struct.

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

pub use self::context::{context_codegen, ContextData};
use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};
pub use tauri_utils::config::{parse::ConfigError, Config};
use tauri_utils::platform::Target;

mod context;
pub mod embedded_assets;
#[doc(hidden)]
pub mod vendor;

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

  #[error(transparent)]
  Json(#[from] serde_json::Error),

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

  // this should be impossible because of the use of `current_dir()` above, but handle it anyways
  let parent = path
    .parent()
    .map(ToOwned::to_owned)
    .ok_or_else(|| CodegenConfigError::Parent(path.into_owned()))?;

  let target = std::env::var("TAURI_ENV_TARGET_TRIPLE")
    .as_deref()
    .map(Target::from_triple)
    .unwrap_or_else(|_| Target::current());

  // in the future we may want to find a way to not need the TAURI_CONFIG env var so that
  // it is impossible for the content of two separate configs to get mixed up. The chances are
  // already unlikely unless the developer goes out of their way to run the cli on a different
  // project than the target crate.
  let mut config = serde_json::from_value(tauri_utils::config::parse::read_from(
    target,
    parent.clone(),
  )?)?;

  if let Ok(env) = std::env::var("TAURI_CONFIG") {
    let merge_config: serde_json::Value =
      serde_json::from_str(&env).map_err(CodegenConfigError::FormatInline)?;
    json_patch::merge(&mut config, &merge_config);
  }

  // Set working directory to where `tauri.config.json` is, so that relative paths in it are parsed correctly.
  let old_cwd = std::env::current_dir().map_err(CodegenConfigError::CurrentDir)?;
  std::env::set_current_dir(parent.clone()).map_err(CodegenConfigError::CurrentDir)?;

  let config = serde_json::from_value(config)?;

  // Reset working directory.
  std::env::set_current_dir(old_cwd).map_err(CodegenConfigError::CurrentDir)?;

  Ok((config, parent))
}
