use std::process::exit;

use super::InvokeResponse;
use crate::api::{app::restart_application, PackageInfo};
use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// Get Application Version
  GetAppVersion,
  /// Get Application Name
  GetAppName,
  /// Get Tauri Version
  GetTauriVersion,
  /// Relaunch application
  Relaunch,
  /// Close application with provided exit_code
  #[serde(rename_all = "camelCase")]
  Exit { exit_code: i32 },
}

impl Cmd {
  pub fn run(self, package_info: PackageInfo) -> crate::Result<InvokeResponse> {
    match self {
      Self::GetAppVersion => {
        return Ok(package_info.version.into());
      }
      Self::GetAppName => {
        return Ok(package_info.name.into());
      }
      Self::GetTauriVersion => {
        return Ok(env!("CARGO_PKG_VERSION").into());
      }
      Self::Relaunch => {
        return Ok(restart_application(None).into());
      }
      Self::Exit { exit_code } => {
        // would be great if we can have a handler inside tauri
        // who close all window and emit an event that user can catch
        // if they want to process something before closing the app
        exit(exit_code);
      }
    }
  }
}
