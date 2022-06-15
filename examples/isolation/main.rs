// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

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
  let context = tauri::generate_context!("../../examples/isolation/tauri.conf.json");
  tauri::Builder::default()
    .menu(tauri::Menu::window_default(&context.package_info().name))
    .invoke_handler(tauri::generate_handler![ping])
    .run(context)
    .expect("error while running tauri application");
}
