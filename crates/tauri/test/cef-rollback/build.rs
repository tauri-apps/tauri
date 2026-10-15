// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

fn main() {
  // The CEF binary distribution the application links, as `cef-dll-sys` resolved it and
  // `tauri-runtime-cef` passes it on. `tests/rollback.rs` lays the application out as the
  // macOS bundle CEF runs from, and needs the framework from there to do it.
  println!("cargo:rerun-if-env-changed=DEP_TAURI_RUNTIME_CEF_CEF_DIR");
  if let Ok(cef_dir) = std::env::var("DEP_TAURI_RUNTIME_CEF_CEF_DIR") {
    println!("cargo:rustc-env=CEF_ROLLBACK_CEF_DIR={cef_dir}");
  }
  tauri_build::build()
}
