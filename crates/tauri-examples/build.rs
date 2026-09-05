// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// `tauri::generate_context!` writes its generated files to `OUT_DIR`, which is only set for crates with a build script.
fn main() {
  println!("cargo:rerun-if-changed=build.rs");
}
