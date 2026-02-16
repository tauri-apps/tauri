// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn ping() {
  println!("ping: {:?}", std::time::Instant::now());
}

#[cfg_attr(feature = "cef", tauri::cef_entry_point)]
fn main() {
  #[cfg(feature = "cef")]
  let builder = tauri::Builder::<tauri::Cef>::default();
  #[cfg(not(feature = "cef"))]
  let builder = tauri::Builder::<tauri::Wry>::new();

  builder
    .invoke_handler(tauri::generate_handler![ping])
    .run(tauri::generate_context!(
      "../../examples/isolation/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}
