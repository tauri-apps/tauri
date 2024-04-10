// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod app_paths;
pub mod cargo;
pub mod config;
pub mod flock;
pub mod framework;
pub mod npm;
pub mod prompts;
pub mod template;
pub mod updater_signature;

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  process::Command,
};

pub fn command_env(debug: bool) -> HashMap<&'static str, String> {
  let mut map = HashMap::new();

  map.insert(
    "TAURI_ENV_PLATFORM_VERSION",
    os_info::get().version().to_string(),
  );

  if debug {
    map.insert("TAURI_ENV_DEBUG", "true".into());
  }

  map
}

pub fn resolve_tauri_path<P: AsRef<Path>>(path: P, crate_name: &str) -> PathBuf {
  let path = path.as_ref();
  if path.is_absolute() {
    path.join(crate_name)
  } else {
    PathBuf::from("..").join(path).join(crate_name)
  }
}

pub fn cross_command(bin: &str) -> Command {
  #[cfg(target_os = "windows")]
  let cmd = {
    let mut cmd = Command::new("cmd");
    cmd.arg("/c").arg(bin);
    cmd
  };
  #[cfg(not(target_os = "windows"))]
  let cmd = Command::new(bin);
  cmd
}
