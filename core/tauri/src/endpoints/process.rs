// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;
#[cfg(process_relaunch)]
use crate::Manager;
use crate::{Runtime, Window};
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
  #[allow(unused_variables)]
  pub fn run<R: Runtime>(self, window: Window<R>) -> crate::Result<InvokeResponse> {
    match self {
      #[cfg(process_relaunch)]
      Self::Relaunch => Ok({
        crate::api::process::restart(&window.state());
        ().into()
      }),
      #[cfg(not(process_relaunch))]
      Self::Relaunch => Err(crate::Error::ApiNotAllowlisted(
        "process > relaunch".to_string(),
      )),

      #[cfg(process_exit)]
      Self::Exit { exit_code } => {
        // would be great if we can have a handler inside tauri
        // who close all window and emit an event that user can catch
        // if they want to process something before closing the app
        std::process::exit(exit_code);
      }
      #[cfg(not(process_exit))]
      Self::Exit { .. } => Err(crate::Error::ApiNotAllowlisted(
        "process > exit".to_string(),
      )),
    }
  }
}
