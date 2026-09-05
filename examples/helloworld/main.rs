// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello {name}, You have been greeted from Rust!")
}

#[cfg_attr(feature = "cef", tauri_runtime_cef::cef_entry_point)]
fn main() {
  #[cfg(not(feature = "cef"))]
  let builder = tauri::Builder::default().runtime(tauri_runtime_wry::Wry);
  #[cfg(feature = "cef")]
  let builder = tauri::Builder::default().runtime(tauri_runtime_cef::Cef::default());
  builder
    .invoke_handler(tauri::generate_handler![greet])
    .run(tauri::generate_context!(
      "../../examples/helloworld/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}
