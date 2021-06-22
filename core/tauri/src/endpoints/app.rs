// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;
use crate::api::PackageInfo;
use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
#[allow(clippy::enum_variant_names)]
pub enum Cmd {
  /// Get Application Version
  GetAppVersion,
  /// Get Application Name
  GetAppName,
  /// Get Tauri Version
  GetTauriVersion,
}

impl Cmd {
  pub fn run(self, package_info: PackageInfo) -> crate::Result<InvokeResponse> {
    match self {
      Self::GetAppVersion => Ok(package_info.version.into()),
      Self::GetAppName => Ok(package_info.name.into()),
      Self::GetTauriVersion => Ok(env!("CARGO_PKG_VERSION").into()),
    }
  }
}
