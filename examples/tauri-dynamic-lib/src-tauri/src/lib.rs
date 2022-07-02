// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// This is an example of a taui app built into a dll
// Calling lib_test1 within the dll will launch the webview

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[no_mangle]
pub extern "C" fn run_tauri() {
  let context = tauri::generate_context!("./tauri.conf.json");
  tauri::Builder::default()
    .run(context)
    .expect("error while running tauri application");
}
