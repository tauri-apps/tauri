// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

fn main() {
  let mut codegen = tauri_build::CodegenContext::new();

  if std::env::var("DEBUG") == Ok("true".to_string()) {
    codegen = codegen.dev();
  }

  tauri_build::try_build(tauri_build::Attributes::new().codegen(codegen).plugin(
    "app-menu",
    tauri_build::InlinedPlugin::new().commands(&["toggle", "popup"]),
  ))
  .expect("failed to run tauri-build");
}
