// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Compile-time and runtime types for Tauri plugins.
#[cfg(feature = "build")]
pub use build::*;

#[cfg(feature = "build")]
mod build {
  use std::{
    env::vars_os,
    fs,
    path::{Path, PathBuf},
  };

  const GLOBAL_API_SCRIPT_PATH_KEY: &str = "GLOBAL_API_SCRIPT_PATH";
  /// Known file name of the file that contains an array with the path of all API scripts defined with [`define_global_api_script_path`].
  pub const GLOBAL_API_SCRIPT_FILE_LIST_PATH: &str = "__global-api-script.js";

  /// Defines the path to the global API script using Cargo instructions.
  pub fn define_global_api_script_path(path: PathBuf) {
    println!(
      "cargo:{GLOBAL_API_SCRIPT_PATH_KEY}={}",
      path
        .canonicalize()
        .expect("failed to canonicalize global API script path")
        .display()
    )
  }

  /// Collects the path of all the global API scripts defined with [`define_global_api_script_path`]
  /// and saves them to the out dir with filename [`GLOBAL_API_SCRIPT_FILE_LIST_PATH`].
  pub fn save_global_api_scripts_paths(out_dir: &Path) {
    let mut scripts = Vec::new();

    for (key, value) in vars_os() {
      let key = key.to_string_lossy();

      if key.starts_with("DEP_") && key.ends_with(GLOBAL_API_SCRIPT_PATH_KEY) {
        let script_path = PathBuf::from(value);
        scripts.push(script_path);
      }
    }

    fs::write(
      out_dir.join(GLOBAL_API_SCRIPT_FILE_LIST_PATH),
      serde_json::to_string(&scripts).expect("failed to serialize global API script paths"),
    )
    .expect("failed to write global API script");
  }

  /// Read global api scripts from [`GLOBAL_API_SCRIPT_FILE_LIST_PATH`]
  pub fn read_global_api_scripts(out_dir: &Path) -> Option<Vec<String>> {
    let global_scripts_path = out_dir.join(GLOBAL_API_SCRIPT_FILE_LIST_PATH);
    if !global_scripts_path.exists() {
      return None;
    }

    let global_scripts_str = fs::read_to_string(global_scripts_path)
      .expect("failed to read plugin global API script paths");
    let global_scripts = serde_json::from_str::<Vec<PathBuf>>(&global_scripts_str)
      .expect("failed to parse plugin global API script paths");

    Some(
      global_scripts
        .into_iter()
        .map(|p| {
          fs::read_to_string(&p).unwrap_or_else(|e| {
            panic!(
              "failed to read plugin global API script {}: {e}",
              p.display()
            )
          })
        })
        .collect(),
    )
  }
}
