// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

#[tauri::command]
fn read_to_string(path: &str) -> String {
  std::fs::read_to_string(path).unwrap_or_default()
}

fn main() {
  tauri::Builder::default()
    .setup(move |app| {
      let path = app
        .path()
        .resolve("assets/index.js", tauri::path::BaseDirectory::Resource)
        .unwrap();

      let content = std::fs::read_to_string(&path).unwrap();

      println!("Resource `assets/index.js` path: {}", path.display());
      println!("Resource `assets/index.js` content:\n{}\n", content);

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![read_to_string])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
