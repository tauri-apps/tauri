// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::cli::Args;
use std::{env::current_dir, process::Command};

// the name of the binary to find in $PATH
#[cfg(target_os = "linux")]
const DRIVER_BINARY: &str = "WebKitWebDriver";

#[cfg(target_os = "windows")]
const DRIVER_BINARY: &str = "msedgedriver.exe";

/// Find the native driver binary in the PATH, or exits the process with an error.
pub fn native(args: &Args) -> Command {
  let native_binary = match args.native_driver.as_deref() {
    Some(custom) => {
      if custom.exists() {
        custom.to_owned()
      } else {
        eprintln!(
          "can not find the supplied binary path {}. This is currently required.",
          custom.display()
        );
        match current_dir() {
          Ok(cwd) => eprintln!("current working directory: {}", cwd.display()),
          Err(error) => eprintln!("can not find current working directory: {}", error),
        }
        std::process::exit(1);
      }
    }
    None => match which::which(DRIVER_BINARY) {
      Ok(binary) => binary,
      Err(error) => {
        eprintln!(
          "can not find binary {} in the PATH. This is currently required.\
          You can also pass a custom path with --native-driver",
          DRIVER_BINARY
        );
        eprintln!("{:?}", error);
        std::process::exit(1);
      }
    },
  };

  let mut cmd = Command::new(native_binary);
  cmd.env("TAURI_AUTOMATION", "true"); // 1.x
  cmd.env("TAURI_WEBVIEW_AUTOMATION", "true"); // 2.x
  cmd.arg(format!("--port={}", args.native_port));
  cmd.arg(format!("--host={}", args.native_host));
  cmd
}
