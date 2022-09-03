// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// This is an example of a tauri app built into a dll
// Calling lib_test1 within the dll will launch the webview

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[no_mangle]
pub extern "C" fn run_tauri() {
  tauri::Builder::default()
    .run(tauri::generate_context!("./tauri.conf.json"))
    .expect("error while running tauri application");
}
