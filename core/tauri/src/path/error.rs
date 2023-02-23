// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{ser::Serializer, Serialize};

/// Path result.
pub type Result<T> = std::result::Result<T, Error>;

/// Path error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
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
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}
