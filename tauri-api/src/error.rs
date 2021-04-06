/// The error types.
#[derive(thiserror::Error, Debug)]
pub enum Error {
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
  #[error("Network Error: {0}")]
  Network(#[from] reqwest::Error),
  /// HTTP method error.
  #[error("{0}")]
  HttpMethod(#[from] http::method::InvalidMethod),
  /// Invalid HTTO header.
  #[error("{0}")]
  HttpHeader(#[from] reqwest::header::InvalidHeaderName),
  /// Failed to serialize header value as string.
  #[error("failed to convert response header value to string")]
  HttpHeaderToString(#[from] reqwest::header::ToStrError),
  /// HTTP form to must be an object.
  #[error("http form must be an object")]
  InvalidHttpForm,
  /// Semver error.
  #[error("{0}")]
  Semver(#[from] semver::SemVerError),
  /// JSON error.
  #[error("{0}")]
  Json(#[from] serde_json::Error),
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
  #[cfg(feature = "notification")]
  #[error("{0}")]
  Notification(#[from] notify_rust::error::Error),
  /// failed to detect the current platform.
  #[error("failed to detect platform: {0}")]
  FailedToDetectPlatform(String),
  /// CLI argument parsing error.
  #[cfg(feature = "cli")]
  #[error("failed to parse CLI arguments: {0}")]
  ParseCliArguments(#[from] clap::Error),
  /// Shortcut error.
  #[cfg(feature = "global-shortcut")]
  #[error("shortcut error: {0}")]
  Shortcut(#[from] tauri_hotkey::Error),
  /// Shell error.
  #[error("shell error: {0}")]
  Shell(String),
}
