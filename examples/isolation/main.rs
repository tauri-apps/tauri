// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn ping() {
  println!("ping: {:?}", std::time::Instant::now());
}

#[cfg_attr(feature = "cef", tauri_runtime_cef::cef_entry_point)]
fn main() {
  #[cfg(feature = "cef")]
  let builder = tauri::Builder::default().runtime(tauri_runtime_cef::Cef::default());
  #[cfg(not(feature = "cef"))]
  let builder = tauri::Builder::default().runtime(tauri_runtime_wry::Wry);

  builder
    .invoke_handler(tauri::generate_handler![ping])
    .run(tauri::generate_context!(
      "../../examples/isolation/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}
