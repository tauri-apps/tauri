use thiserror::Error as DeriveError;

use std::{io, num, path};
use {base64, minisign};

#[derive(Debug, DeriveError)]
pub enum Error {
  #[error("{0}")]
  BundlerError(#[from] anyhow::Error),
  #[error("`{0}`")]
  GlobError(#[from] glob::GlobError),
  #[error("`{0}`")]
  GlobPatternError(#[from] glob::PatternError),
  #[error("`{0}`")]
  IoError(#[from] io::Error),
  #[error("`{0}`")]
  ImageError(#[from] image::ImageError),
  #[error("`{0}`")]
  TomlError(#[from] toml::de::Error),
  #[error("`{0}`")]
  WalkdirError(#[from] walkdir::Error),
  #[error("`{0}`")]
  StripError(#[from] path::StripPrefixError),
  #[error("`{0}`")]
  ConvertError(#[from] num::TryFromIntError),
  #[cfg(not(target_os = "linux"))]
  #[error("`{0}`")]
  ZipError(#[from] zip::result::ZipError),
  #[cfg(not(target_os = "linux"))]
  #[error("`{0}`")]
  HexError(#[from] hex::FromHexError),
  #[error("`{0}`")]
  HandleBarsError(#[from] handlebars::RenderError),
  #[error("`{0}`")]
  JsonError(#[from] serde_json::error::Error),
  #[cfg(windows)]
  #[error("`{0}`")]
  RegexError(#[from] regex::Error),
  #[cfg(windows)]
  #[error("`{0}`")]
  HttpError(#[from] attohttpc::Error),
  #[error("hash mismatch of downloaded file")]
  HashError,
  #[error("Architecture Error: `{0}`")]
  ArchError(String),
  #[error(
    "Couldn't get tauri config; please specify the TAURI_CONFIG or TAURI_DIR environment variables"
  )]
  EnvironmentError,
  #[error("Could not find Icon paths.  Please make sure they exist in the tauri config JSON file")]
  IconPathError,
  #[error("Path Error:`{0}`")]
  PathUtilError(String),
  #[error("Shell Scripting Error:`{0}`")]
  ShellScriptError(String),
  #[error("`{0}`")]
  GenericError(String),
  #[error("Unable to create updater package")]
  UpdateBundler,
  #[error("Decode Error: `{0}`")]
  DecodeError(#[from] base64::DecodeError),
  #[error("Utf8Error: `{0}`")]
  Utf8Error(#[from] std::str::Utf8Error),
  #[error("Signing error: `{0}`")]
  MiniSign(#[from] minisign::PError),
}

pub type Result<T> = anyhow::Result<T, Error>;
