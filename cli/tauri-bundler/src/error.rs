use thiserror::Error as DeriveError;

use {
  glob, handlebars, hex, image, serde_json, std::io, std::num, std::path, target_build_utils, term,
  toml, walkdir,
};

#[cfg(windows)]
use {attohttpc, regex};

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
  TargetError(#[from] target_build_utils::Error),
  #[error("`{0}`")]
  TermError(#[from] term::Error),
  #[error("`{0}`")]
  TomlError(#[from] toml::de::Error),
  #[error("`{0}`")]
  WalkdirError(#[from] walkdir::Error),
  #[error("`{0}`")]
  StripError(#[from] path::StripPrefixError),
  #[error("`{0}`")]
  ConvertError(#[from] num::TryFromIntError),
  #[error("`{0}`")]
  ZipError(#[from] zip::result::ZipError),
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
  #[error("Error running Candle.exe")]
  CandleError,
  #[error("Error running Light.exe")]
  LightError,
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
}

pub type Result<T> = anyhow::Result<T, Error>;
