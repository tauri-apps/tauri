// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::WebviewWindowBuilder;

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      WebviewWindowBuilder::new(app, "Third", tauri::WebviewUrl::default())
        .title("Tauri - Third")
        .build()?;

      Ok(())
    })
    .run(generate_context())
    .expect("failed to run tauri application");
}

fn generate_context() -> tauri::Context {
  let mut context = tauri::generate_context!("../../examples/multiwindow/tauri.conf.json");
  for cmd in [
    "plugin:event|listen",
    "plugin:event|emit",
    "plugin:event|emit_to",
    "plugin:webview|create_webview_window",
  ] {
    context
      .runtime_authority_mut()
      .__allow_command(cmd.to_string(), tauri_utils::acl::ExecutionContext::Local);
  }
  context
}
