// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fs,
  path::{Path, PathBuf},
};

use crate::Settings;

mod linuxdeploy;
mod sharun_cef;

// TODO: Consider auto fallback to linuxdeploy on unsupported systems.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  if settings.bundle_settings().cef_path.is_some() {
    sharun_cef::bundle_project(settings)
  } else {
    linuxdeploy::bundle_project(settings)
  }
}

fn write_and_make_executable(path: &Path, data: Vec<u8>) -> std::io::Result<()> {
  use std::os::unix::fs::PermissionsExt;

  fs::write(path, data)?;
  fs::set_permissions(path, fs::Permissions::from_mode(0o770))?;

  Ok(())
}
