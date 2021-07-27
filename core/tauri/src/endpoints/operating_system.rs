// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;
use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  Platform,
  Version,
  Type,
  Arch,
  Tempdir,
}

impl Cmd {
  #[allow(unused_variables)]
  pub fn run(self) -> crate::Result<InvokeResponse> {
    #[cfg(os_all)]
    return Ok(match self {
      Self::Platform => std::env::consts::OS.into(),
      Self::Version => os_info::get().version().to_string().into(),
      #[cfg(target_os = "linux")]
      Self::Type => "Linux".into(),
      #[cfg(target_os = "windows")]
      Self::Type => "Windows_NT".into(),
      #[cfg(target_os = "macos")]
      Self::Type => "Darwin".into(),
      Self::Arch => std::env::consts::ARCH.into(),
      Self::Tempdir => std::env::temp_dir().into(),
    });
    #[cfg(not(os_all))]
    Err(crate::Error::ApiNotAllowlisted("os".into()))
  }
}
