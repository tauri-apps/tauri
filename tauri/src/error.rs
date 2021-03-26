/// The plugin error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
  /// Failed to create webview.
  #[error("failed to create webview")]
  CreateWebview,
  /// Failed to create window.
  #[error("failed to create window")]
  CreateWindow,
  /// Can't access webview dispatcher because the webview was closed or not found.
  #[error("webview not found: invalid label or it was closed")]
  WebviewNotFound,
  /// Failed to send message to webview.
  #[error("failed to send message to the webview")]
  FailedToSendMessage,
  /// Embedded asset not found.
  #[error("asset not found: {0}")]
  AssetNotFound(String),
  /// Failed to serialize/deserialize.
  #[error("JSON error: {0}")]
  Json(serde_json::Error),
  /// Unknown API type.
  #[error("unknown API: {0:?}")]
  UnknownApi(Option<serde_json::Error>),
  /// Failed to execute tauri API.
  #[error("failed to execute API: {0}")]
  FailedToExecuteApi(#[from] tauri_api::Error),
  /// IO error.
  #[error("{0}")]
  Io(#[from] std::io::Error),
  /// Failed to decode base64.
  #[error("Failed to decode base64 string: {0}")]
  Base64Decode(#[from] base64::DecodeError),
  /// Failed to load window icon.
  #[error("invalid icon: {0}")]
  InvalidIcon(String),
  /// Client with specified ID not found.
  #[error("http client dropped or not initialized")]
  HttpClientNotInitialized,
  /// API not enabled by Tauri.
  #[error("{0}")]
  ApiNotEnabled(String),
  /// API not whitelisted on tauri.conf.json
  #[error("'{0}' not on the allowlist (https://tauri.studio/docs/api/config#tauri.allowlist)")]
  ApiNotAllowlisted(String),
  /// Invalid args when running a command.
  #[error("invalid args for command `{0}`: {1}")]
  InvalidArgs(&'static str, serde_json::Error),
  /// Tauri updater error.
  #[error("Updater: {0}")]
  TauriUpdater(#[from] tauri_updater::Error),
}

impl From<serde_json::Error> for Error {
  fn from(error: serde_json::Error) -> Self {
    if error.to_string().contains("unknown variant") {
      Self::UnknownApi(Some(error))
    } else {
      Self::Json(error)
    }
  }
}
