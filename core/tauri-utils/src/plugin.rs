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
    path::{Path, PathBuf},
  };

  const GLOBAL_API_SCRIPT_PATH_KEY: &str = "GLOBAL_API_SCRIPT_PATH";
  /// Known file name of the script that contains all API scripts defined with [`define_global_api_script_path`].
  pub const COLLECTED_GLOBAL_API_SCRIPT_PATH: &str = "__global-api-script.js";

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

  /// Loads all the global API scripts defined with [`define_global_api_script_path`]
  /// and saves them to the out dir with filename [`COLLECTED_GLOBAL_API_SCRIPT_PATH`].
  pub fn load_global_api_scripts(out_dir: &Path) {
    let mut merged_script = String::new();

    for (key, value) in vars_os() {
      let key = key.to_string_lossy();

      if key.starts_with("DEP_") && key.ends_with(GLOBAL_API_SCRIPT_PATH_KEY) {
        let script_path = PathBuf::from(value);
        let script = std::fs::read_to_string(&script_path).unwrap_or_else(|e| {
          panic!(
            "failed to read global script path at {}: {e}",
            script_path.display()
          )
        });

        merged_script.push_str(";(function () {\n");
        merged_script.push_str(&script);
        merged_script.push_str("})()");
      }
    }

    std::fs::write(
      out_dir.join(COLLECTED_GLOBAL_API_SCRIPT_PATH),
      merged_script,
    )
    .expect("failed to write global API script");
  }
}
