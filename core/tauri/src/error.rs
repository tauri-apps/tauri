// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

/// Runtime errors that can happen inside a Tauri application.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
  /// Runtime error.
  #[error("runtime error: {0}")]
  Runtime(#[from] tauri_runtime::Error),
  /// Failed to create webview.
  #[error("failed to create webview: {0}")]
  CreateWebview(Box<dyn std::error::Error + Send>),
  /// Failed to create window.
  #[error("failed to create window")]
  CreateWindow,
  /// Window label must be unique.
  #[error("a window with label `{0}` already exists")]
  WindowLabelAlreadyExists(String),
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
  FailedToExecuteApi(#[from] crate::api::Error),
  /// IO error.
  #[error("{0}")]
  Io(#[from] std::io::Error),
  /// Failed to decode base64.
  #[cfg(any(fs_write_binary_file, feature = "updater"))]
  #[error("Failed to decode base64 string: {0}")]
  Base64Decode(#[from] base64::DecodeError),
  /// Failed to load window icon.
  #[error("invalid icon: {0}")]
  InvalidIcon(Box<dyn std::error::Error + Send>),
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
  /// Encountered an error in the setup hook,
  #[error("error encountered during setup hook: {0}")]
  Setup(Box<dyn std::error::Error + Send>),
  /// Tauri updater error.
  #[cfg(feature = "updater")]
  #[error("Updater: {0}")]
  TauriUpdater(#[from] crate::updater::Error),
  /// Error initializing plugin.
  #[error("failed to initialize plugin `{0}`: {1}")]
  PluginInitialization(String, String),
  /// `default_path` provided to dialog API doesn't exist.
  #[error("failed to setup dialog: provided default path `{0}` doesn't exist")]
  DialogDefaultPathNotExists(PathBuf),
  /// Encountered an error creating the app system tray,
  #[error("error encountered during tray setup: {0}")]
  SystemTray(Box<dyn std::error::Error + Send>),
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
