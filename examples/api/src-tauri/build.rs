// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_build::WindowsAttributes;

fn main() {
  tauri_build::try_build(
    tauri_build::Attributes::new()
      .codegen(tauri_build::CodegenContext::new())
      // link the app manifest into tests too to prevent `STATUS_ENTRYPOINT_NOT_FOUND` errors
      .windows_attributes(WindowsAttributes::new().link_resource_into_all_targets(true))
      .plugin(
        "app-menu",
        tauri_build::InlinedPlugin::new().commands(&["toggle", "popup"]),
      )
      .app_manifest(tauri_build::AppManifest::new().commands(&[
        "log_operation",
        "perform_request",
        "echo",
        "spam",
      ])),
  )
  .expect("failed to run tauri-build");
}
