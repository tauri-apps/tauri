// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeContext;
use crate::Runtime;
use serde::Deserialize;
use tauri_macros::{command_enum, module_command_handler, CommandModule};

/// The API descriptor.
#[command_enum]
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
#[allow(clippy::enum_variant_names)]
pub enum Cmd {
  /// Get Application Version
  GetAppVersion,
  /// Get Application Name
  GetAppName,
  /// Get Tauri Version
  GetTauriVersion,
  /// Shows the application on macOS.
  #[cmd(app_show, "app > show")]
  Show,
  /// Hides the application on macOS.
  #[cmd(app_hide, "app > hide")]
  Hide,
}

impl Cmd {
  fn get_app_version<R: Runtime>(context: InvokeContext<R>) -> super::Result<String> {
    Ok(context.package_info.version.to_string())
  }

  fn get_app_name<R: Runtime>(context: InvokeContext<R>) -> super::Result<String> {
    Ok(context.package_info.name)
  }

  fn get_tauri_version<R: Runtime>(_context: InvokeContext<R>) -> super::Result<&'static str> {
    Ok(env!("CARGO_PKG_VERSION"))
  }

  #[module_command_handler(app_show)]
  #[allow(unused_variables)]
  fn show<R: Runtime>(context: InvokeContext<R>) -> super::Result<()> {
    #[cfg(target_os = "macos")]
    context.window.app_handle.show()?;
    Ok(())
  }

  #[module_command_handler(app_hide)]
  #[allow(unused_variables)]
  fn hide<R: Runtime>(context: InvokeContext<R>) -> super::Result<()> {
    #[cfg(target_os = "macos")]
    context.window.app_handle.hide()?;
    Ok(())
  }
}
