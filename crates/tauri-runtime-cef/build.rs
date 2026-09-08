// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  // exposed to the build scripts of crates depending on this one as `DEP_TAURI_RUNTIME_CEF_RUNTIME`,
  // which is how `tauri-build` detects that the application uses the CEF runtime.
  println!("cargo:runtime=cef");
}
