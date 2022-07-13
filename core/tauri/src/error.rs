// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fmt, path::PathBuf};

/// A generic boxed error.
#[derive(Debug)]
pub struct SetupError(Box<dyn std::error::Error>);

impl From<Box<dyn std::error::Error>> for SetupError {
  fn from(error: Box<dyn std::error::Error>) -> Self {
    Self(error)
  }
}

// safety: the setup error is only used on the main thread
// and we exit the process immediately.
unsafe impl Send for SetupError {}
unsafe impl Sync for SetupError {}

impl fmt::Display for SetupError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl std::error::Error for SetupError {}

/// Runtime errors that can happen inside a Tauri application.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
  /// Runtime error.
  #[error("runtime error: {0}")]
  Runtime(#[from] tauri_runtime::Error),
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
  InvalidIcon(std::io::Error),
  /// Client with specified ID not found.
  #[error("http client dropped or not initialized")]
  HttpClientNotInitialized,
  /// API not whitelisted on tauri.conf.json
  #[error("'{0}' not in the allowlist (https://tauri.app/docs/api/config#tauri.allowlist)")]
  ApiNotAllowlisted(String),
  /// Invalid args when running a command.
  #[error("invalid args `{1}` for command `{0}`: {2}")]
  InvalidArgs(&'static str, &'static str, serde_json::Error),
  /// Encountered an error in the setup hook,
  #[error("error encountered during setup hook: {0}")]
  Setup(SetupError),
  /// Tauri updater error.
  #[cfg(updater)]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "updater")))]
  #[error("Updater: {0}")]
  TauriUpdater(#[from] crate::updater::Error),
  /// Error initializing plugin.
  #[error("failed to initialize plugin `{0}`: {1}")]
  PluginInitialization(String, String),
  /// A part of the URL is malformed or invalid. This may occur when parsing and combining
  /// user-provided URLs and paths.
  #[error("invalid url: {0}")]
  InvalidUrl(url::ParseError),
  /// Task join error.
  #[error(transparent)]
  JoinError(#[from] tokio::task::JoinError),
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
  /// Invalid glob pattern.
  #[error("invalid glob pattern: {0}")]
  GlobPattern(#[from] glob::PatternError),
  /// Error decoding PNG image.
  #[cfg(feature = "icon-png")]
  #[error("failed to decode PNG: {0}")]
  PngDecode(#[from] png::DecodingError),
  /// The Window's raw handle is invalid for the platform.
  #[error("Unexpected `raw_window_handle` for the current platform")]
  InvalidWindowHandle,
}

pub(crate) fn into_anyhow<T: std::fmt::Display>(err: T) -> anyhow::Error {
  anyhow::anyhow!(err.to_string())
}

impl Error {
  #[allow(dead_code)]
  pub(crate) fn into_anyhow(self) -> anyhow::Error {
    anyhow::anyhow!(self.to_string())
  }
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
