// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  path::PathBuf,
  env,
  process::{exit, Command as StdCommand},
};

#[cfg(shell_execute)]
mod command;
#[cfg(shell_execute)]
pub use command::*;

/// Get the current binary
pub fn current_binary() -> Option<PathBuf> {
  let mut current_binary = None;

  // if we are running with an APP Image, we should return the app image path
  #[cfg(target_os = "linux")]
  if let Some(app_image_path) = env::var_os("APPIMAGE") {
    current_binary = Some(PathBuf::from(app_image_path));
  }

  // if we didn't extracted binary in previous step,
  // let use the current_exe from current environment
  if current_binary.is_none() {
    if let Ok(current_process) = env::current_exe() {
      current_binary = Some(current_process);
    }
  }

  current_binary
}

/// Restart the process.
pub fn restart() {
  if let Some(path) = current_binary() {
    StdCommand::new(path)
      .spawn()
      .expect("application failed to start");
  }

  exit(0);
}
