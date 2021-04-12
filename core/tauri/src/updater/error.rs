// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use thiserror::Error;

/// All errors that can occur while running the updater.
#[derive(Debug, Error)]
pub enum Error {
  /// IO Errors.
  #[error("`{0}`")]
  Io(#[from] std::io::Error),
  /// Reqwest Errors.
  #[error("Request error: {0}")]
  Reqwest(#[from] reqwest::Error),
  /// Semver Errors.
  #[error("Unable to compare version: {0}")]
  Semver(#[from] semver::SemVerError),
  /// JSON (Serde) Errors.
  #[error("JSON error: {0}")]
  SerdeJson(#[from] serde_json::Error),
  /// Minisign is used for signature validation.
  #[error("Verify signature error: {0}")]
  Minisign(#[from] minisign_verify::Error),
  /// Error with Minisign base64 decoding.
  #[error("Signature decoding error: {0}")]
  Base64(#[from] base64::DecodeError),
  /// UTF8 Errors in signature.
  #[error("Signature encoding error: {0}")]
  Utf8(#[from] std::str::Utf8Error),
  /// Tauri utils, mainly extract and file move.
  #[error("Tauri API error: {0}")]
  TauriApi(#[from] crate::api::Error),
  /// Network error.
  #[error("Network error: {0}")]
  Network(String),
  /// Metadata (JSON) error.
  #[error("Remote JSON error: {0}")]
  RemoteMetadata(String),
  /// Error building updater.
  #[error("Unable to prepare the updater: {0}")]
  Builder(String),
  /// Updater is not supported for current operating system or platform.
  #[error("Unsuported operating system or platform")]
  UnsupportedPlatform,
  /// Public key found in `tauri.conf.json` but no signature announced remotely.
  #[error("Signature not available but public key provided, skipping update")]
  PubkeyButNoSignature,
  /// Triggered when there is NO error and the two versions are equals.
  /// On client side, it's important to catch this error.
  #[error("No updates available")]
  UpToDate,
}

pub type Result<T = ()> = std::result::Result<T, Error>;
