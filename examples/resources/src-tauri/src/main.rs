// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, window::Color};

#[tauri::command]
fn read_to_string(path: &str) -> String {
  std::fs::read_to_string(path).unwrap_or_default()
}

#[cfg_attr(feature = "cef", tauri::cef_entry_point)]
fn main() {
  #[cfg(feature = "cef")]
  let builder = tauri::Builder::<tauri::Cef>::default();
  #[cfg(not(feature = "cef"))]
  let builder = tauri::Builder::<tauri::Wry>::new();

  builder
    .setup(move |app| {
      let window = tauri::WindowBuilder::new(app, "main")
        .background_color(Color(255, 0, 0, 255)) // red
        .build()?;

      let size = window.inner_size()?;

      let webview_builder = tauri::WebviewBuilder::new("main", tauri::WebviewUrl::default())
        .background_color(Color(0, 0, 255, 255)); // blue

      let webview1 = window.add_child(
        webview_builder,
        tauri::LogicalPosition::new(0, 0),
        tauri::LogicalSize::new(size.width / 2, size.height),
      )?;

      let webview_builder =
        tauri::WebviewBuilder::new("main2", tauri::WebviewUrl::default()).transparent(true);

      let webview2 = window.add_child(
        webview_builder,
        tauri::LogicalPosition::new(size.width / 2, 0),
        tauri::LogicalSize::new(size.width / 2, size.height),
      )?;

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![read_to_string])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
