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
  /// The extract archive error.
  #[error("Extract Error: {0}")]
  Extract(String),
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
  #[cfg(not(feature = "reqwest-client"))]
  #[error("Network Error: {0}")]
  Network(#[from] attohttpc::Error),
  /// The network error.
  #[cfg(feature = "reqwest-client")]
  #[error("Network Error: {0}")]
  Network(#[from] reqwest::Error),
  /// HTTP method error.
  #[error("{0}")]
  HttpMethod(#[from] http::method::InvalidMethod),
  /// Invalid HTTP header value.
  #[cfg(feature = "reqwest-client")]
  #[error("{0}")]
  HttpHeaderValue(#[from] http::header::InvalidHeaderValue),
  /// Invalid HTTP header value.
  #[error("{0}")]
  HttpHeader(#[from] http::header::InvalidHeaderName),
  /// Failed to serialize header value as string.
  #[error("failed to convert response header value to string")]
  HttpHeaderToString(#[from] http::header::ToStrError),
  /// HTTP form to must be an object.
  #[error("http form must be an object")]
  InvalidHttpForm,
  /// Semver error.
  #[error("{0}")]
  Semver(#[from] semver::Error),
  /// JSON error.
  #[error("{0}")]
  Json(#[from] serde_json::Error),
  /// Bincode error.
  #[error("{0}")]
  Bincode(#[from] Box<bincode::ErrorKind>),
  /// IO error.
  #[error("{0}")]
  Io(#[from] std::io::Error),
  /// Ignore error.
  #[error("failed to walkdir: {0}")]
  Ignore(#[from] ignore::Error),
  /// ZIP error.
  #[error("{0}")]
  Zip(#[from] zip::result::ZipError),
  /// Notification error.
  #[cfg(notification_all)]
  #[error("{0}")]
  Notification(#[from] notify_rust::error::Error),
  /// failed to detect the current platform.
  #[error("failed to detect platform: {0}")]
  FailedToDetectPlatform(String),
  /// CLI argument parsing error.
  #[cfg(feature = "cli")]
  #[error("failed to parse CLI arguments: {0}")]
  ParseCliArguments(#[from] clap::Error),
  /// Shell error.
  #[error("shell error: {0}")]
  Shell(String),
}
