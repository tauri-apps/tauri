// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Convenient type alias of Result type for wry.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by wry.
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[cfg(gtk)]
  #[error(transparent)]
  GlibError(#[from] gtk::glib::Error),
  #[cfg(gtk)]
  #[error(transparent)]
  GlibBoolError(#[from] gtk::glib::BoolError),
  #[cfg(gtk)]
  #[error("Fail to fetch security manager")]
  MissingManager,
  #[cfg(gtk)]
  #[error("{0} is not a supported webview parent widget.")]
  UnsupportedParentWidget(String),
  #[error("Failed to initialize the script")]
  InitScriptError,
  #[error("Bad RPC request: {0} ((1))")]
  RpcScriptError(String, String),
  #[error(transparent)]
  NulError(#[from] std::ffi::NulError),
  #[error(transparent)]
  ReceiverError(#[from] std::sync::mpsc::RecvError),
  #[cfg(target_os = "android")]
  #[error(transparent)]
  ReceiverTimeoutError(#[from] crossbeam_channel::RecvTimeoutError),
  #[error(transparent)]
  SenderError(#[from] std::sync::mpsc::SendError<String>),
  #[error("Failed to send the message")]
  MessageSender,
  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),
  #[cfg(target_os = "windows")]
  #[error("WebView2 error: {0}")]
  WebView2Error(webview2_com::Error),
  #[error(transparent)]
  HttpError(#[from] http::Error),
  #[error("Infallible error, something went really wrong: {0}")]
  Infallible(#[from] std::convert::Infallible),
  #[cfg(target_os = "android")]
  #[error(transparent)]
  JniError(#[from] jni::errors::Error),
  #[error("Failed to create proxy endpoint")]
  ProxyEndpointCreationFailed,
  #[error(transparent)]
  WindowHandleError(#[from] raw_window_handle::HandleError),
  #[error("the window handle kind is not supported")]
  UnsupportedWindowHandle,
  #[error(transparent)]
  Utf8Error(#[from] std::str::Utf8Error),
  #[cfg(target_os = "android")]
  #[error(transparent)]
  CrossBeamRecvError(#[from] crossbeam_channel::RecvError),
  #[error("not on the main thread")]
  NotMainThread,
  #[error("Custom protocol task is invalid.")]
  CustomProtocolTaskInvalid,
  #[error("Failed to register URL scheme: {0}, could be due to invalid URL scheme or the scheme is already registered.")]
  UrlSchemeRegisterError(String),
  #[error("Duplicate custom protocol '{0}' registered on the WebViewBuilder")]
  DuplicateCustomProtocol(String),
  #[error("Duplicate custom protocol '{0}' registered on the same web context on Linux")]
  ContextDuplicateCustomProtocol(String),
  #[error(transparent)]
  #[cfg(any(target_os = "macos", target_os = "ios"))]
  UrlParse(#[from] url::ParseError),
  #[cfg(any(target_os = "macos", target_os = "ios"))]
  #[error("data store is currently opened")]
  DataStoreInUse,
  #[cfg(target_os = "android")]
  #[error("Activity not found")]
  ActivityNotFound,
}
