// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::WindowBuilder;

fn main() {
  let context = tauri::generate_context!("../../examples/multiwindow/tauri.conf.json");
  tauri::Builder::default()
    .on_page_load(|window, _payload| {
      let label = window.label().to_string();
      window.listen("clicked".to_string(), move |_payload| {
        println!("got 'clicked' event on window '{}'", label);
      });
    })
    .setup(|app| {
      WindowBuilder::new(
        app,
        "Rust".to_string(),
        tauri::WindowUrl::App("index.html".into()),
      )
      .title("Tauri - Rust")
      .build()?;
      Ok(())
    })
    .run(context)
    .expect("failed to run tauri application");
}
