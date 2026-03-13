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
      let window = tauri::WebviewWindowBuilder::new(app, "main", Default::default())
        .title("normal")
        .build()?;
      let window = tauri::WebviewWindowBuilder::new(app, "main1", Default::default())
        .background_color(Color(0, 255, 0, 255))
        .title("normal with background color")
        .build()?;
      let window = tauri::WebviewWindowBuilder::new(app, "main2", Default::default())
        .decorations(false)
        .title("decorationless")
        .build()?;
      let window = tauri::WebviewWindowBuilder::new(app, "main3", Default::default())
        .decorations(false)
        .background_color(Color(255, 0, 0, 255))
        .title("decorationless with background color")
        .build()?;
      let window = tauri::WebviewWindowBuilder::new(app, "main4", Default::default())
        .decorations(false)
        .transparent(true)
        .title("decorationless transparent")
        .build()?;

      let window = tauri::WindowBuilder::new(app, "main5")
        .title("multiwebview")
        .build()?;
      let wv1 = tauri::WebviewBuilder::new("main6", Default::default());
      window.add_child(
        wv1,
        tauri::LogicalPosition::new(0, 0),
        tauri::LogicalSize::new(400, 400),
      )?;

      let window = tauri::WindowBuilder::new(app, "main7")
        .decorations(false)
        .title("multiwebview decorationless")
        .build()?;

      let wv1 = tauri::WebviewBuilder::new("main8", Default::default());
      window.add_child(
        wv1,
        tauri::LogicalPosition::new(0, 0),
        tauri::LogicalSize::new(400, 400),
      )?;

      let wv1 = tauri::WebviewBuilder::new("main9", Default::default())
        .background_color(Color(255, 0, 0, 255));
      window.add_child(
        wv1,
        tauri::LogicalPosition::new(400, 0),
        tauri::LogicalSize::new(400, 400),
      )?;

      let wv1 = tauri::WebviewBuilder::new("main10", Default::default()).transparent(true);
      window.add_child(
        wv1,
        tauri::LogicalPosition::new(0, 400),
        tauri::LogicalSize::new(400, 400),
      )?;

      let wv1 = tauri::WebviewBuilder::new("main11", Default::default())
        .background_color(Color(0, 0, 255, 255));
      window.add_child(
        wv1,
        tauri::LogicalPosition::new(400, 400),
        tauri::LogicalSize::new(400, 400),
      )?;

      let window = tauri::WindowBuilder::new(app, "main12")
        .decorations(false)
        .transparent(true)
        .title("multiwebview transparent")
        .build()?;

      let wv1 = tauri::WebviewBuilder::new("main13", Default::default())
        .background_color(Color(255, 0, 0, 255));
      window.add_child(
        wv1,
        tauri::LogicalPosition::new(0, 0),
        tauri::LogicalSize::new(400, 400),
      )?;

      let wv1 = tauri::WebviewBuilder::new("main14", Default::default());
      window.add_child(
        wv1,
        tauri::LogicalPosition::new(400, 400),
        tauri::LogicalSize::new(400, 400),
      )?;

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![read_to_string])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
