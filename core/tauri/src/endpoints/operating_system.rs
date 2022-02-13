// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeContext;
use crate::Runtime;
use serde::Deserialize;
use std::path::PathBuf;
use tauri_macros::{module_command_handler, CommandModule};

/// The API descriptor.
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  Platform,
  Version,
  OsType,
  Arch,
  Tempdir,
}

impl Cmd {
  #[module_command_handler(os_all, "os > all")]
  fn platform<R: Runtime>(_context: InvokeContext<R>) -> crate::Result<&'static str> {
    Ok(os_platform())
  }

  #[module_command_handler(os_all, "os > all")]
  fn version<R: Runtime>(_context: InvokeContext<R>) -> crate::Result<String> {
    Ok(os_info::get().version().to_string())
  }

  #[module_command_handler(os_all, "os > all")]
  fn os_type<R: Runtime>(_context: InvokeContext<R>) -> crate::Result<&'static str> {
    Ok(os_type())
  }

  #[module_command_handler(os_all, "os > all")]
  fn arch<R: Runtime>(_context: InvokeContext<R>) -> crate::Result<&'static str> {
    Ok(std::env::consts::ARCH)
  }

  #[module_command_handler(os_all, "os > all")]
  fn tempdir<R: Runtime>(_context: InvokeContext<R>) -> crate::Result<PathBuf> {
    Ok(std::env::temp_dir())
  }
}

#[cfg(os_all)]
fn os_type() -> &'static str {
  #[cfg(target_os = "linux")]
  return "Linux";
  #[cfg(target_os = "windows")]
  return "Windows_NT";
  #[cfg(target_os = "macos")]
  return "Darwin";
}
#[cfg(os_all)]
fn os_platform() -> &'static str {
  match std::env::consts::OS {
    "windows" => "win32",
    "macos" => "darwin",
    _ => std::env::consts::OS,
  }
}

#[cfg(test)]
mod tests {
  #[tauri_macros::module_command_test(os_all, "os > all")]
  #[quickcheck_macros::quickcheck]
  fn platform() {}

  #[tauri_macros::module_command_test(os_all, "os > all")]
  #[quickcheck_macros::quickcheck]
  fn version() {}

  #[tauri_macros::module_command_test(os_all, "os > all")]
  #[quickcheck_macros::quickcheck]
  fn os_type() {}

  #[tauri_macros::module_command_test(os_all, "os > all")]
  #[quickcheck_macros::quickcheck]
  fn arch() {}

  #[tauri_macros::module_command_test(os_all, "os > all")]
  #[quickcheck_macros::quickcheck]
  fn tempdir() {}
}
