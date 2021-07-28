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
    return match self {
      Self::Platform => Ok(std::env::consts::OS.into()),
      Self::Version => Ok(os_info::get().version().to_string().into()),
      Self::Type => {
        #[cfg(target_os = "linux")]
        return Ok("Linux".into());
        #[cfg(target_os = "windows")]
        return Ok("Windows_NT".into());
        #[cfg(target_os = "macos")]
        return Ok("Darwing".into());
      }
      Self::Arch => Ok(std::env::consts::ARCH.into()),
      Self::Tempdir => Ok(std::env::temp_dir().into()),
    };
    #[cfg(not(os_all))]
    Err(crate::Error::ApiNotAllowlisted("os".into()))
  }
}
