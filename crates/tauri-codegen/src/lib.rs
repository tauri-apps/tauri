// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! - Embed, hash, and compress assets, including icons for the app as well as the tray icon.
//! - Parse `tauri.conf.json` at compile time and generate the Config struct.

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]

pub use self::context::{context_codegen, ContextData};
use crate::embedded_assets::{ensure_out_dir, EmbeddedAssetsError};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use std::{
  borrow::Cow,
  fmt::{self, Write},
  path::{Path, PathBuf},
};
pub use tauri_utils::config::{parse::ConfigError, Config};
use tauri_utils::platform::Target;
use tauri_utils::write_if_changed;

mod context;
pub mod embedded_assets;
pub mod image;
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
  let mut config =
    serde_json::from_value(tauri_utils::config::parse::read_from(target, parent.clone())?.0)?;

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

/// Create a blake3 checksum of the passed bytes.
fn checksum(bytes: &[u8]) -> Result<String, fmt::Error> {
  let mut hasher = vendor::blake3_reference::Hasher::default();
  hasher.update(bytes);

  let mut bytes = [0u8; 32];
  hasher.finalize(&mut bytes);

  let mut hex = String::with_capacity(2 * bytes.len());
  for b in bytes {
    write!(hex, "{b:02x}")?;
  }
  Ok(hex)
}

/// Cache the data to `$OUT_DIR`, only if it does not already exist.
///
/// Due to using a checksum as the filename, an existing file should be the exact same content
/// as the data being checked.
struct Cached {
  checksum: String,
}

impl TryFrom<String> for Cached {
  type Error = EmbeddedAssetsError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::try_from(Vec::from(value))
  }
}

impl TryFrom<Vec<u8>> for Cached {
  type Error = EmbeddedAssetsError;

  fn try_from(content: Vec<u8>) -> Result<Self, Self::Error> {
    let checksum = checksum(content.as_ref()).map_err(EmbeddedAssetsError::Hex)?;
    let path = ensure_out_dir()?.join(&checksum);

    write_if_changed(&path, &content)
      .map(|_| Self { checksum })
      .map_err(|error| EmbeddedAssetsError::AssetWrite { path, error })
  }
}

impl ToTokens for Cached {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let path = &self.checksum;
    tokens.append_all(quote!(::std::concat!(::std::env!("OUT_DIR"), "/", #path)))
  }
}
