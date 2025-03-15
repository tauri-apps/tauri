// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn greet(app: tauri::AppHandle, name: String) -> String {
  use tauri::Manager;
  println!("greet {name}");
  let w = app.get_webview_window("main").unwrap();
  println!("{:?}", w.get_webview_window("main").unwrap().cookies());
  format!("Hello {name}, You have been greeted from Rust!")
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![greet])
    .setup(|app| {
      use tauri::Manager;
      println!("{:?}", app.get_webview_window("main").unwrap().cookies());
      Ok(())
    })
    .run(tauri::generate_context!(
      "../../examples/helloworld/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}
