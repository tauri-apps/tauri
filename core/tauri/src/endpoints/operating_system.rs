// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused_imports)]

use super::InvokeContext;
use crate::Runtime;
use serde::Deserialize;
use std::path::PathBuf;
use tauri_macros::{command_enum, module_command_handler, CommandModule};

/// The API descriptor.
#[command_enum]
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  Platform,
  Version,
  Arch,
  Tempdir,
  Hostname,
}

#[cfg(os_all)]
impl Cmd {
  fn platform<R: Runtime>(_context: InvokeContext<R>) -> super::Result<&'static str> {
    Ok(std::env::consts::OS)
  }

  fn version<R: Runtime>(_context: InvokeContext<R>) -> super::Result<String> {
    Ok(os_info::get().version().to_string())
  }

  fn arch<R: Runtime>(_context: InvokeContext<R>) -> super::Result<&'static str> {
    Ok(std::env::consts::ARCH)
  }

  fn tempdir<R: Runtime>(_context: InvokeContext<R>) -> super::Result<PathBuf> {
    Ok(std::env::temp_dir())
  }

  fn hostname<R: Runtime>(_context: InvokeContext<R>) -> super::Result<String> {
    Ok(gethostname::gethostname().to_string_lossy().to_string())
  }
}

#[cfg(not(os_all))]
impl Cmd {
  fn platform<R: Runtime>(_context: InvokeContext<R>) -> super::Result<&'static str> {
    Err(crate::Error::ApiNotAllowlisted("os > all".into()).into_anyhow())
  }

  fn version<R: Runtime>(_context: InvokeContext<R>) -> super::Result<String> {
    Err(crate::Error::ApiNotAllowlisted("os > all".into()).into_anyhow())
  }

  fn arch<R: Runtime>(_context: InvokeContext<R>) -> super::Result<&'static str> {
    Err(crate::Error::ApiNotAllowlisted("os > all".into()).into_anyhow())
  }

  fn tempdir<R: Runtime>(_context: InvokeContext<R>) -> super::Result<PathBuf> {
    Err(crate::Error::ApiNotAllowlisted("os > all".into()).into_anyhow())
  }

  fn hostname<R: Runtime>(_context: InvokeContext<R>) -> super::Result<PathBuf> {
    Err(crate::Error::ApiNotAllowlisted("os > all".into()).into_anyhow())
  }
}

#[cfg(test)]
mod tests {
  #[tauri_macros::module_command_test(os_all, "os > all", runtime)]
  #[quickcheck_macros::quickcheck]
  fn platform() {}

  #[tauri_macros::module_command_test(os_all, "os > all", runtime)]
  #[quickcheck_macros::quickcheck]
  fn version() {}

  #[tauri_macros::module_command_test(os_all, "os > all", runtime)]
  #[quickcheck_macros::quickcheck]
  fn arch() {}

  #[tauri_macros::module_command_test(os_all, "os > all", runtime)]
  #[quickcheck_macros::quickcheck]
  fn tempdir() {}

  #[tauri_macros::module_command_test(os_all, "os > all", runtime)]
  #[quickcheck_macros::quickcheck]
  fn hostname() {}
}
