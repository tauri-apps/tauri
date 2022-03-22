// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::{WindowBuilder, WindowUrl};

mod commands;

fn main() {
  tauri::Builder::default()
    .on_page_load(|window, _payload| {
      let label = window.label().to_string();
      window.listen("clicked".to_string(), move |_payload| {
        println!("got 'clicked' event on window '{}'", label);
      });
    })
    .invoke_handler(tauri::generate_handler![commands::create_child_window])
    .create_window(
      "main".to_string(),
      WindowUrl::default(),
      |window_builder, webview_attributes| {
        (
          window_builder.title("Main").inner_size(600.0, 400.0),
          webview_attributes,
        )
      },
    )
    .unwrap() // safe to unwrap: window label is valid
    .run(tauri::generate_context!(
      "../../examples/parent-window/src-tauri/tauri.conf.json"
    ))
    .expect("failed to run tauri application");
}
