// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// The result type of Tauri API module.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type of Tauri API module.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
  /// JSON error.
  #[error(transparent)]
  Json(#[from] serde_json::Error),
}
