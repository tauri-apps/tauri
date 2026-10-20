// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Locates the external tools used when bundling macOS artifacts on Linux.
//!
//! The tools are never downloaded while bundling. They can be supplied through
//! an explicit environment variable, the bundler tools directory, the Tauri
//! cache, or `PATH`.

use crate::Settings;

use std::{env, path::PathBuf};

pub fn resolve(settings: &Settings, name: &str, env_var: &str) -> crate::Result<PathBuf> {
  if let Some(path) = env::var_os(env_var) {
    let path = PathBuf::from(path);
    if path.is_file() {
      return Ok(path);
    }
    return Err(crate::Error::GenericError(format!(
      "{env_var} points to {}, but the file does not exist",
      path.display()
    )));
  }

  find(settings, name).ok_or_else(|| {
    crate::Error::GenericError(format!(
      "could not find `{name}`. Install it, set {env_var}, or place it in the bundler tools cache"
    ))
  })
}

fn find(settings: &Settings, name: &str) -> Option<PathBuf> {
  let directories = [
    settings
      .local_tools_directory()
      .map(|directory| directory.join(".tauri")),
    dirs::cache_dir().map(|directory| directory.join("tauri")),
  ];

  for directory in directories.into_iter().flatten() {
    for candidate in [directory.join(name), directory.join("bin").join(name)] {
      if candidate.is_file() {
        return Some(candidate);
      }
    }
  }

  which::which(name).ok()
}
