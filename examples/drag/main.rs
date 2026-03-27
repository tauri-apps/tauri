// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  tauri::Builder::default()
    .run(generate_context())
    .expect("error while running tauri application");
}

fn generate_context<R: tauri::Runtime>() -> tauri::Context<R> {
  let mut context = tauri::generate_context!("../../examples/drag/tauri.conf.json");
  for cmd in [
    "plugin:window|start_dragging",
    "plugin:window|internal_toggle_maximize",
  ] {
    context
      .runtime_authority_mut()
      .__allow_command(cmd.to_string(), tauri_utils::acl::ExecutionContext::Local);
  }
  context
}
