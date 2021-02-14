/// The error types.
#[derive(thiserror::Error, Debug)]
pub enum Error {
  /// The extract archive error.
  #[error("Extract Error: {0}")]
  Extract(String),
  /// The Command (spawn process) error.
  #[error("Command Error: {0}")]
  Command(String),
  /// The path operation error.
  #[error("Path Error: {0}")]
  Path(String),
  /// Error showing the dialog.
  #[error("Dialog Error: {0}")]
  Dialog(String),
  /// The dialog operation was cancelled by the user.
  #[error("user cancelled the dialog")]
  DialogCancelled,
  /// CLI config not set.
  #[error("CLI configuration not set on tauri.conf.json")]
  CliNotConfigured,
  /// The HTTP response error.
  #[error("HTTP Response Error: {0}")]
  Response(attohttpc::StatusCode),
  /// The network error.
  #[error("Network Error: {0}")]
  Network(#[from] attohttpc::Error),
  /// HTTP method error.
  #[error("{0}")]
  HttpMethod(#[from] http::method::InvalidMethod),
  /// Invalid HTTO header.
  #[error("{0}")]
  HttpHeader(#[from] attohttpc::header::InvalidHeaderName),
  /// Semver error.
  #[error("{0}")]
  Semver(#[from] semver::SemVerError),
  /// JSON error.
  #[error("{0}")]
  Json(#[from] serde_json::Error),
  /// IO error.
  #[error("{0}")]
  Io(#[from] std::io::Error),
  /// ZIP error.
  #[error("{0}")]
  Zip(#[from] zip::result::ZipError),
  /// Notification error.
  #[error("{0}")]
  Notification(#[from] notify_rust::error::Error),
  /// failed to detect the current platform.
  #[error("failed to detect platform: {0}")]
  FailedToDetectPlatform(String),
  /// CLI argument parsing error.
  #[cfg(feature = "cli")]
  #[error("failed to parse CLI arguments: {0}")]
  ParseCliArguments(#[from] clap::Error),
}

impl From<attohttpc::StatusCode> for Error {
  fn from(error: attohttpc::StatusCode) -> Self {
    Self::Response(error)
  }
}
