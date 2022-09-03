// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod app_paths;
pub mod config;
pub mod framework;
pub mod template;
pub mod updater_signature;

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

pub fn command_env(debug: bool) -> HashMap<String, String> {
  let mut map = HashMap::new();

  map.insert("TAURI_PLATFORM".into(), std::env::consts::OS.into());
  map.insert("TAURI_ARCH".into(), std::env::consts::ARCH.into());
  map.insert("TAURI_FAMILY".into(), std::env::consts::FAMILY.into());
  map.insert(
    "TAURI_PLATFORM_VERSION".into(),
    os_info::get().version().to_string(),
  );

  #[cfg(target_os = "linux")]
  map.insert("TAURI_PLATFORM_TYPE".into(), "Linux".into());
  #[cfg(target_os = "windows")]
  map.insert("TAURI_PLATFORM_TYPE".into(), "Windows_NT".into());
  #[cfg(target_os = "macos")]
  map.insert("TAURI_PLATFORM_TYPE".into(), "Darwin".into());

  if debug {
    map.insert("TAURI_DEBUG".into(), "true".to_string());
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
