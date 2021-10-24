// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to child processes management.

use crate::Env;

use std::{
  env,
  path::PathBuf,
  process::{exit, Command as StdCommand},
};

#[cfg(feature = "command")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "command")))]
mod command;
#[cfg(feature = "command")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "command")))]
pub use command::*;

/// Gets the current binary.
#[allow(unused_variables)]
pub fn current_binary(env: &Env) -> Option<PathBuf> {
  let mut current_binary = None;

  // if we are running with an APP Image, we should return the app image path
  #[cfg(target_os = "linux")]
  if let Some(app_image_path) = &env.appimage {
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

/// Restarts the process.
pub fn restart(env: &Env) {
  if let Some(path) = current_binary(env) {
    StdCommand::new(path)
      .spawn()
      .expect("application failed to start");
  }

  exit(0);
}
