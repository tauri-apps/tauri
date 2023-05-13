// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// The error types.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
  /// Semver error.
  #[error(transparent)]
  Semver(#[from] semver::Error),
  /// JSON error.
  #[error(transparent)]
  Json(#[from] serde_json::Error),
  /// IO error.
  #[error(transparent)]
  Io(#[from] std::io::Error),
}
