// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

fn main() {
  tauri_build::try_build(
    tauri_build::Attributes::new()
      .codegen(tauri_build::CodegenContext::new())
      .plugin(
        "app-menu",
        tauri_build::InlinedPlugin::new().commands(&["toggle", "popup"]),
      )
      .app_manifest(
        tauri_build::AppManifest::new().commands(&["log_operation", "perform_request"]),
      ),
  )
  .expect("failed to run tauri-build");
}
