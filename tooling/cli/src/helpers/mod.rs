// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub mod app_paths;
pub mod config;
pub mod framework;
pub mod template;
pub mod updater_signature;
pub mod web_dev_server;

use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

pub fn command_env(debug: bool) -> HashMap<&'static str, String> {
  let mut map = HashMap::new();

  map.insert(
    "TAURI_PLATFORM_VERSION",
    os_info::get().version().to_string(),
  );

  if debug {
    map.insert("TAURI_DEBUG", "true".into());
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
