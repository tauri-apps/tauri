// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_plugin_dialog::{DialogExt, MessageDialogBuilder};

#[tauri::command]
async fn greet(window: tauri::Window, name: String) -> bool {
  MessageDialogBuilder::new(
    window.dialog().to_owned(),
    "Tauri Example",
    format!("Hello {name}"),
  )
  .parent(&window)
  .blocking_show()
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![greet])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
