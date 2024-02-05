// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

fn main() {
  let mut codegen = tauri_build::CodegenContext::new();
  if !cfg!(feature = "custom-protocol") {
    codegen = codegen.dev();
  }

  tauri_build::try_build(tauri_build::Attributes::new().codegen(codegen).plugin(
    "app-menu",
    tauri_build::InlinedPlugin::new().commands(&["toggle", "popup"]).permissions_path_pattern("./permissions/app-menu/*"),
  ))
  .expect("failed to run tauri-build");
}
