// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::fmt;

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
  /// Window label must be unique.
  #[error("a window with label `{0}` already exists")]
  WindowLabelAlreadyExists(String),
  /// Embedded asset not found.
  #[error("asset not found: {0}")]
  AssetNotFound(String),
  /// Failed to serialize/deserialize.
  #[error("JSON error: {0}")]
  Json(#[from] serde_json::Error),
  /// IO error.
  #[error("{0}")]
  Io(#[from] std::io::Error),
  /// Failed to load window icon.
  #[error("invalid icon: {0}")]
  InvalidIcon(std::io::Error),
  /// Invalid args when running a command.
  #[error("invalid args `{1}` for command `{0}`: {2}")]
  InvalidArgs(&'static str, &'static str, serde_json::Error),
  /// Encountered an error in the setup hook,
  #[error("error encountered during setup hook: {0}")]
  Setup(SetupError),
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
  /// JNI error.
  #[cfg(target_os = "android")]
  #[error("jni error: {0}")]
  Jni(#[from] jni::errors::Error),
  /// Failed to receive message .
  #[error("failed to receive message")]
  FailedToReceiveMessage,
  /// Menu error.
  #[error("menu error: {0}")]
  #[cfg(desktop)]
  Menu(#[from] muda::Error),
  /// Bad menu icon error.
  #[error(transparent)]
  #[cfg(desktop)]
  BadMenuIcon(#[from] muda::BadIcon),
  /// Tray icon error.
  #[error("tray icon error: {0}")]
  #[cfg(all(desktop, feature = "tray-icon"))]
  #[cfg_attr(docsrs, doc(cfg(all(desktop, feature = "tray-icon"))))]
  Tray(#[from] tray_icon::Error),
  /// Bad tray icon error.
  #[error(transparent)]
  #[cfg(all(desktop, feature = "tray-icon"))]
  #[cfg_attr(docsrs, doc(cfg(all(desktop, feature = "tray-icon"))))]
  BadTrayIcon(#[from] tray_icon::BadIcon),
  /// Path does not have a parent.
  #[error("path does not have a parent")]
  NoParent,
  /// Path does not have an extension.
  #[error("path does not have an extension")]
  NoExtension,
  /// Path does not have a basename.
  #[error("path does not have a basename")]
  NoBasename,
  /// Cannot resolve current directory.
  #[error("failed to read current dir: {0}")]
  CurrentDir(std::io::Error),
  /// Unknown path.
  #[cfg(not(target_os = "android"))]
  #[error("unknown path")]
  UnknownPath,
  /// Failed to invoke mobile plugin.
  #[cfg(target_os = "android")]
  #[error(transparent)]
  PluginInvoke(#[from] crate::plugin::mobile::PluginInvokeError),
  /// window not found.
  #[error("window not found")]
  WindowNotFound,
  /// The resource id is invalid.
  #[error("The resource id {0} is invalid.")]
  BadResourceId(crate::resources::ResourceId),
  /// The anyhow crate error.
  #[error(transparent)]
  Anyhow(#[from] anyhow::Error),
}

/// `Result<T, ::tauri::Error>`
pub type Result<T> = std::result::Result<T, Error>;
