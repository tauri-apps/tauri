// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

fn main() {
  let context = tauri::generate_context!("../../examples/navigation/tauri.conf.json");
  tauri::Builder::default()
    .run(context)
    .expect("error while running tauri application");
}
