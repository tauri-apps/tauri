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
  #[cfg(feature = "updater")]
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
  #[error("invalid args `{1}` for command `{0}`: {2}")]
  InvalidArgs(&'static str, &'static str, serde_json::Error),
  /// Encountered an error in the setup hook,
  #[error("error encountered during setup hook: {0}")]
  Setup(Box<dyn std::error::Error + Send>),
  /// Tauri updater error.
  #[cfg(feature = "updater")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "updater")))]
  #[error("Updater: {0}")]
  TauriUpdater(#[from] crate::updater::Error),
  /// Error initializing plugin.
  #[error("failed to initialize plugin `{0}`: {1}")]
  PluginInitialization(String, String),
  /// Encountered an error creating the app system tray,
  #[error("error encountered during tray setup: {0}")]
  SystemTray(Box<dyn std::error::Error + Send>),
  /// A part of the URL is malformed or invalid. This may occur when parsing and combining
  /// user-provided URLs and paths.
  #[error("invalid url: {0}")]
  InvalidUrl(url::ParseError),
  /// Task join error.
  #[error(transparent)]
  JoinError(Box<dyn std::error::Error + Send>),
  /// Path not allowed by the scope.
  #[error("path not allowed on the configured scope: {0}")]
  PathNotAllowed(PathBuf),
  /// The user did not allow sending notifications.
  #[error("sending notification was not allowed by the user")]
  NotificationNotAllowed,
  /// URL not allowed by the scope.
  #[error("url not allowed on the configured scope: {0}")]
  UrlNotAllowed(url::Url),
  /// Sidecar not allowed by the configuration.
  #[error("sidecar not configured under `tauri.conf.json > tauri > bundle > externalBin`: {0}")]
  SidecarNotAllowed(PathBuf),
  /// Sidecar was not found by the configuration.
  #[cfg(shell_scope)]
  #[error("sidecar configuration found, but unable to create a path to it: {0}")]
  SidecarNotFound(#[from] Box<crate::ShellScopeError>),
  /// Program not allowed by the scope.
  #[error("program not allowed on the configured shell scope: {0}")]
  ProgramNotAllowed(PathBuf),
  /// An error happened inside the isolation pattern.
  #[cfg(feature = "isolation")]
  #[error("isolation pattern error: {0}")]
  IsolationPattern(#[from] tauri_utils::pattern::isolation::Error),
  /// An invalid window URL was provided. Includes details about the error.
  #[error("invalid window url: {0}")]
  InvalidWindowUrl(&'static str),
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
