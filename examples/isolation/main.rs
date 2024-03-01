// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Instant;

#[tauri::command]
fn ping() {
  dbg!(format!("ping: {:?}", Instant::now()));
}

#[cfg(not(feature = "isolation"))]
fn main() {
  compile_error!("Feature `isolation` is required to run this example");
}

#[cfg(feature = "isolation")]
fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![ping])
    .run(tauri::generate_context!(
      "../../examples/isolation/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}
