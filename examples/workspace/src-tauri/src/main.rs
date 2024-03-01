// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  core_api::run();
  tauri::Builder::default()
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
