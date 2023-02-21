// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Instant;

#[tauri::command]
fn ping() {
  dbg!(format!("ping: {:?}", Instant::now()));
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![ping])
    .run(tauri::tauri_build_context!())
    .expect("error while running tauri application");
}
