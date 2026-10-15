// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  // exposed to the build scripts of crates depending on this one as `DEP_TAURI_RUNTIME_CEF_RUNTIME`,
  // which is how `tauri-build` detects that the application uses the CEF runtime.
  println!("cargo:runtime=cef");
  // The CEF binary distribution `cef-dll-sys` resolved for this build, downloading it when it
  // was missing, passed on to the build scripts of crates depending on this one as
  // `DEP_TAURI_RUNTIME_CEF_CEF_DIR`: where the framework, `libcef` and the resources are.
  println!("cargo:rerun-if-env-changed=DEP_CEF_DLL_WRAPPER_CEF_DIR");
  if let Ok(cef_dir) = std::env::var("DEP_CEF_DLL_WRAPPER_CEF_DIR") {
    println!("cargo:cef_dir={cef_dir}");
  }
}
