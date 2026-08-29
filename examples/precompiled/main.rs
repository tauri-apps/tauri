// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env::current_dir;

#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello {name}, You have been greeted from Rust!")
}

fn main() {
  let current_dir = current_dir().expect("failed to get current directory");

  let context = tauri::generate_context!("../../examples/precompiled/Tauri.toml");
  let context = context
    .load_runtime_context(&current_dir.join("examples/precompiled/src-tauri"), None)
    .expect("failed to load runtime context");

  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![greet])
    .run(context)
    .expect("error while running tauri application");
}
