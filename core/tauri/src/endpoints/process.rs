// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::process::exit;

use super::InvokeResponse;
use crate::api::process::restart;
use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// Relaunch application
  Relaunch,
  /// Close application with provided exit_code
  #[serde(rename_all = "camelCase")]
  Exit { exit_code: i32 },
}

impl Cmd {
  pub fn run(self) -> crate::Result<InvokeResponse> {
    match self {
      Self::Relaunch => Ok({
        restart();
        ().into()
      }),
      Self::Exit { exit_code } => {
        // would be great if we can have a handler inside tauri
        // who close all window and emit an event that user can catch
        // if they want to process something before closing the app
        exit(exit_code);
      }
    }
  }
}
