// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fmt::Display,
  io, num,
  path::{self, PathBuf},
};
use thiserror::Error as DeriveError;

/// Errors returned by the bundler.
#[derive(Debug, DeriveError)]
#[non_exhaustive]
pub enum Error {
  /// Error with context. Created by the [`Context`] trait.
  #[error("{0}: {1}")]
  Context(String, Box<Self>),
  /// File system error.
  #[error("{context} {path}: {error}")]
  Fs {
    /// Context of the error.
    context: &'static str,
    /// Path that was accessed.
    path: PathBuf,
    /// Error that occurred.
    error: io::Error,
  },
  /// Child process error.
  #[error("failed to run command {command}: {error}")]
  CommandFailed {
    /// Command that failed.
    command: String,
    /// Error that occurred.
    error: io::Error,
  },
  /// Error running tauri_utils API.
  #[error("{0}")]
  Resource(#[from] tauri_utils::Error),
  /// Bundler error.
  ///
  /// This variant is no longer used as this crate no longer uses anyhow.
  // TODO(v3): remove this variant
  #[error("{0:#}")]
  BundlerError(#[from] anyhow::Error),
  /// I/O error.
  #[error("`{0}`")]
  IoError(#[from] io::Error),
  /// Image error.
  #[error("`{0}`")]
  ImageError(#[from] image::ImageError),
  /// Error walking directory.
  #[error("`{0}`")]
  WalkdirError(#[from] walkdir::Error),
  /// Strip prefix error.
  #[error("`{0}`")]
  StripError(#[from] path::StripPrefixError),
  /// Number parse error.
  #[error("`{0}`")]
  ConvertError(#[from] num::TryFromIntError),
  /// Zip error.
  #[error("`{0}`")]
  ZipError(#[from] zip::result::ZipError),
  /// Hex error.
  #[error("`{0}`")]
  HexError(#[from] hex::FromHexError),
  /// Handlebars template error.
  #[error("`{0}`")]
  HandleBarsError(#[from] handlebars::RenderError),
  /// JSON error.
  #[error("`{0}`")]
  JsonError(#[from] serde_json::error::Error),
  /// Regex error.
  #[cfg(any(target_os = "macos", windows))]
  #[error("`{0}`")]
  RegexError(#[from] regex::Error),
  /// Failed to perform HTTP request.
  #[error("`{0}`")]
  HttpError(#[from] Box<ureq::Error>),
  /// Invalid glob pattern.
  #[cfg(windows)]
  #[error("{0}")]
  GlobPattern(#[from] glob::PatternError),
  /// Failed to use glob pattern.
  #[cfg(windows)]
  #[error("`{0}`")]
  Glob(#[from] glob::GlobError),
  /// Failed to parse the URL
  #[error("`{0}`")]
  UrlParse(#[from] url::ParseError),
  /// Failed to validate downloaded file hash.
  #[error("hash mismatch of downloaded file")]
  HashError,
  /// Failed to parse binary
  #[error("Binary parse error: `{0}`")]
  BinaryParseError(#[from] goblin::error::Error),
  /// Package type is not supported by target platform
  #[error("Wrong package type {0} for platform {1}")]
  InvalidPackageType(String, String),
  /// Bundle type symbol missing in binary
  #[cfg_attr(
    target_os = "linux",
    error("__TAURI_BUNDLE_TYPE variable not found in binary. Make sure tauri crate and tauri-cli are up to date and that symbol stripping is disabled (https://doc.rust-lang.org/cargo/reference/profiles.html#strip)")
  )]
  #[cfg_attr(
    not(target_os = "linux"),
    error("__TAURI_BUNDLE_TYPE variable not found in binary. Make sure tauri crate and tauri-cli are up to date")
  )]
  MissingBundleTypeVar,
  /// Failed to write binary file changed
  #[error("Failed to write binary file changes: `{0}`")]
  BinaryWriteError(String),
  /// Invalid offset while patching binary file
  #[error("Invalid offset while patching binary file")]
  BinaryOffsetOutOfRange,
  /// Unsupported architecture.
  #[error("Architecture Error: `{0}`")]
  ArchError(String),
  /// Couldn't find icons.
  #[error("Could not find Icon paths.  Please make sure they exist in the tauri config JSON file")]
  IconPathError,
  /// Couldn't find background file.
  #[error("Could not find background file. Make sure it exists in the tauri config JSON file and extension is png/jpg/gif")]
  BackgroundPathError,
  /// Error on path util operation.
  #[error("Path Error:`{0}`")]
  PathUtilError(String),
  /// Error on shell script.
  #[error("Shell Scripting Error:`{0}`")]
  ShellScriptError(String),
  /// Generic error.
  #[error("`{0}`")]
  GenericError(String),
  /// No bundled project found for the updater.
  #[error("Unable to find a bundled project for the updater")]
  UnableToFindProject,
  /// String is not UTF-8.
  #[error("string is not UTF-8")]
  Utf8(#[from] std::str::Utf8Error),
  /// Windows SignTool not found.
  #[error("SignTool not found")]
  SignToolNotFound,
  /// Failed to open Windows registry.
  #[error("failed to open registry {0}")]
  OpenRegistry(String),
  /// Failed to get registry value.
  #[error("failed to get {0} value on registry")]
  GetRegistryValue(String),
  /// Failed to enumerate registry keys.
  #[error("failed to enumerate registry keys")]
  FailedToEnumerateRegKeys,
  /// Unsupported OS bitness.
  #[error("unsupported OS bitness")]
  UnsupportedBitness,
  /// Failed to sign application.
  #[error("failed to sign app: {0}")]
  Sign(String),
  /// time error.
  #[cfg(target_os = "macos")]
  #[error("`{0}`")]
  TimeError(#[from] time::error::Error),
  /// Plist error.
  #[cfg(target_os = "macos")]
  #[error(transparent)]
  Plist(#[from] plist::Error),
  /// Rpm error.
  #[cfg(target_os = "linux")]
  #[error("{0}")]
  RpmError(#[from] rpm::Error),
  /// Failed to notarize application.
  #[cfg(target_os = "macos")]
  #[error("failed to notarize app: {0}")]
  AppleNotarization(#[from] NotarizeAuthError),
  /// Failed to codesign application.
  #[cfg(target_os = "macos")]
  #[error("failed codesign application: {0}")]
  AppleCodesign(#[from] Box<tauri_macos_sign::Error>),
  /// Handlebars template error.
  #[error(transparent)]
  Template(#[from] handlebars::TemplateError),
  /// Semver error.
  #[error("`{0}`")]
  SemverError(#[from] semver::Error),
}

#[cfg(target_os = "macos")]
#[allow(clippy::enum_variant_names)]
#[derive(Debug, thiserror::Error)]
pub enum NotarizeAuthError {
  #[error(
    "The team ID is now required for notarization with app-specific password as authentication. Please set the `APPLE_TEAM_ID` environment variable. You can find the team ID in https://developer.apple.com/account#MembershipDetailsCard."
  )]
  MissingTeamId,
  #[error("could not find API key file. Please set the APPLE_API_KEY_PATH environment variables to the path to the {file_name} file")]
  MissingApiKey { file_name: String },
  #[error("no APPLE_ID & APPLE_PASSWORD & APPLE_TEAM_ID or APPLE_API_KEY & APPLE_API_ISSUER & APPLE_API_KEY_PATH environment variables found")]
  MissingCredentials,
}

/// Convenient type alias of Result type.
pub type Result<T> = std::result::Result<T, Error>;

pub trait Context<T> {
  // Required methods
  fn context<C>(self, context: C) -> Result<T>
  where
    C: Display + Send + Sync + 'static;
  fn with_context<C, F>(self, f: F) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
    F: FnOnce() -> C;
}

impl<T> Context<T> for Result<T> {
  fn context<C>(self, context: C) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
  {
    self.map_err(|e| Error::Context(context.to_string(), Box::new(e)))
  }

  fn with_context<C, F>(self, f: F) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
    F: FnOnce() -> C,
  {
    self.map_err(|e| Error::Context(f().to_string(), Box::new(e)))
  }
}

impl<T> Context<T> for Option<T> {
  fn context<C>(self, context: C) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
  {
    self.ok_or_else(|| Error::GenericError(context.to_string()))
  }

  fn with_context<C, F>(self, f: F) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
    F: FnOnce() -> C,
  {
    self.ok_or_else(|| Error::GenericError(f().to_string()))
  }
}

pub trait ErrorExt<T> {
  fn fs_context(self, context: &'static str, path: impl Into<PathBuf>) -> Result<T>;
}

impl<T> ErrorExt<T> for std::result::Result<T, std::io::Error> {
  fn fs_context(self, context: &'static str, path: impl Into<PathBuf>) -> Result<T> {
    self.map_err(|error| Error::Fs {
      context,
      path: path.into(),
      error,
    })
  }
}

#[allow(unused)]
macro_rules! bail {
   ($msg:literal $(,)?) => {
      return Err(crate::Error::GenericError($msg.into()))
   };
    ($err:expr $(,)?) => {
       return Err(crate::Error::GenericError($err))
    };
   ($fmt:expr, $($arg:tt)*) => {
     return Err(crate::Error::GenericError(format!($fmt, $($arg)*)))
   };
}

#[allow(unused)]
pub(crate) use bail;
