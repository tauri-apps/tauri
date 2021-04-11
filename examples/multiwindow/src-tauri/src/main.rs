// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::Attributes;

fn main() {
  tauri::Builder::default()
    .on_page_load(|window, _payload| {
      let label = window.label().to_string();
      window.listen("clicked".to_string(), move |_payload| {
        println!("got 'clicked' event on window '{}'", label);
      });
    })
    .create_window(
      "Rust".to_string(),
      tauri::WindowUrl::App("index.html".into()),
      |attributes| attributes.title("Tauri - Rust"),
    )
    .run(tauri::generate_context!())
    .expect("failed to run tauri application");
}
