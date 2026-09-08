// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{env, path::PathBuf};

// The examples do not run `tauri-build`, so this build script does the minimum `tauri::generate_context!` needs:
// - `OUT_DIR` is only set for crates with a build script, and the macro writes its generated files there;
// - the `withGlobalTauri` option needs the list of global API scripts (from `tauri` and plugins) saved to `OUT_DIR`.
fn main() {
  println!("cargo:rerun-if-changed=build.rs");

  let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is not set"));
  tauri_utils::plugin::save_global_api_scripts_paths(&out_dir, None);
}
