// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// The error types.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
  /// Command error.
  #[error("Command Error: {0}")]
  Command(String),
  /// The path operation error.
  #[error("Path Error: {0}")]
  Path(String),
  /// The path StripPrefixError error.
  #[error("Path Error: {0}")]
  PathPrefix(#[from] std::path::StripPrefixError),
  /// Error showing the dialog.
  #[error("Dialog Error: {0}")]
  Dialog(String),
  /// The dialog operation was cancelled by the user.
  #[error("user cancelled the dialog")]
  DialogCancelled,
  /// The network error.
  #[cfg(all(feature = "http-api", not(feature = "reqwest-client")))]
  #[error("Network Error: {0}")]
  Network(#[from] attohttpc::Error),
  /// The network error.
  #[cfg(feature = "reqwest-client")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "reqwest-client")))]
  #[error("Network Error: {0}")]
  Network(#[from] reqwest::Error),
  /// HTTP method error.
  #[error(transparent)]
  HttpMethod(#[from] http::method::InvalidMethod),
  /// Invalid HTTP header value.
  #[cfg(feature = "reqwest-client")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "reqwest-client")))]
  #[error(transparent)]
  HttpHeaderValue(#[from] http::header::InvalidHeaderValue),
  /// Invalid HTTP header value.
  #[error(transparent)]
  HttpHeader(#[from] http::header::InvalidHeaderName),
  /// Failed to serialize header value as string.
  #[error(transparent)]
  Utf8(#[from] std::string::FromUtf8Error),
  /// HTTP form to must be an object.
  #[error("http form must be an object")]
  InvalidHttpForm,
  /// Semver error.
  #[error(transparent)]
  Semver(#[from] semver::Error),
  /// JSON error.
  #[error(transparent)]
  Json(#[from] serde_json::Error),
  /// Bincode error.
  #[error(transparent)]
  Bincode(#[from] Box<bincode::ErrorKind>),
  /// IO error.
  #[error(transparent)]
  Io(#[from] std::io::Error),
  /// Ignore error.
  #[error("failed to walkdir: {0}")]
  Ignore(#[from] ignore::Error),
  /// ZIP error.
  #[cfg(feature = "fs-extract-api")]
  #[error(transparent)]
  Zip(#[from] zip::result::ZipError),
  /// Extract error.
  #[cfg(feature = "fs-extract-api")]
  #[error("Failed to extract: {0}")]
  Extract(String),
  /// Notification error.
  #[cfg(notification_all)]
  #[error(transparent)]
  Notification(#[from] notify_rust::error::Error),
  /// Url error.
  #[error(transparent)]
  Url(#[from] url::ParseError),
  /// failed to detect the current platform.
  #[error("failed to detect platform: {0}")]
  FailedToDetectPlatform(String),
  /// CLI argument parsing error.
  #[cfg(feature = "cli")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "cli")))]
  #[error("failed to parse CLI arguments: {0}")]
  ParseCliArguments(String),
  /// Shell error.
  #[error("shell error: {0}")]
  Shell(String),
  /// Unknown program name.
  #[error("unknown program name: {0}")]
  UnknownProgramName(String),
  /// HTTP error.
  #[error(transparent)]
  Http(#[from] http::Error),
}

#[cfg(feature = "cli")]
impl From<clap::Error> for Error {
  fn from(error: clap::Error) -> Self {
    Self::ParseCliArguments(error.to_string())
  }
}
