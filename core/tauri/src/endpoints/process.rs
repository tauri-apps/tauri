// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused_imports)]

use super::InvokeContext;
#[cfg(process_relaunch)]
use crate::Manager;
use crate::Runtime;
use serde::Deserialize;
use tauri_macros::{command_enum, module_command_handler, CommandModule};

/// The API descriptor.
#[command_enum]
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// Relaunch application
  Relaunch,
  /// Close application with provided exit_code
  #[cmd(process_exit, "process > exit")]
  #[serde(rename_all = "camelCase")]
  Exit { exit_code: i32 },
}

impl Cmd {
  #[module_command_handler(process_relaunch)]
  fn relaunch<R: Runtime>(context: InvokeContext<R>) -> super::Result<()> {
    context.window.app_handle().restart();
    Ok(())
  }

  #[cfg(not(process_relaunch))]
  fn relaunch<R: Runtime>(_: InvokeContext<R>) -> super::Result<()> {
    Err(crate::Error::ApiNotAllowlisted("process > relaunch".into()).into_anyhow())
  }

  #[module_command_handler(process_exit)]
  fn exit<R: Runtime>(_context: InvokeContext<R>, exit_code: i32) -> super::Result<()> {
    // would be great if we can have a handler inside tauri
    // who close all window and emit an event that user can catch
    // if they want to process something before closing the app
    std::process::exit(exit_code);
  }
}

#[cfg(test)]
mod tests {
  #[tauri_macros::module_command_test(process_relaunch, "process > relaunch", runtime)]
  #[quickcheck_macros::quickcheck]
  fn relaunch() {}

  #[tauri_macros::module_command_test(process_exit, "process > exit")]
  #[quickcheck_macros::quickcheck]
  fn exit(_exit_code: i32) {}
}
