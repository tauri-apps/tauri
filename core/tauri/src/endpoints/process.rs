// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeContext;
#[cfg(process_relaunch)]
use crate::Manager;
use crate::Runtime;
use serde::Deserialize;
use tauri_macros::{module_command_handler, CommandModule};

/// The API descriptor.
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// Relaunch application
  Relaunch,
  /// Close application with provided exit_code
  #[serde(rename_all = "camelCase")]
  Exit { exit_code: i32 },
}

impl Cmd {
  #[module_command_handler(process_relaunch, "process > relaunch")]
  fn relaunch<R: Runtime>(context: InvokeContext<R>) -> crate::Result<()> {
    crate::api::process::restart(&context.window.state());
    Ok(())
  }

  #[module_command_handler(process_exit, "process > exit")]
  fn exit<R: Runtime>(_context: InvokeContext<R>, exit_code: i32) -> crate::Result<()> {
    // would be great if we can have a handler inside tauri
    // who close all window and emit an event that user can catch
    // if they want to process something before closing the app
    std::process::exit(exit_code);
  }
}

#[cfg(test)]
mod tests {
  #[tauri_macros::module_command_test(process_relaunch, "process > relaunch")]
  #[quickcheck_macros::quickcheck]
  fn relaunch() {}

  #[tauri_macros::module_command_test(process_exit, "process > exit")]
  #[quickcheck_macros::quickcheck]
  fn exit(_exit_code: i32) {}
}
