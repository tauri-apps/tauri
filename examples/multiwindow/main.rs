// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::WindowBuilder;

fn main() {
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
    .run(tauri::generate_context!(
      "../../examples/multiwindow/tauri.conf.json"
    ))
    .expect("failed to run tauri application");
}
