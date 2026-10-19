// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[cfg(mobile)]
  #[error(transparent)]
  PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
  #[error(transparent)]
  Tauri(#[from] tauri::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
