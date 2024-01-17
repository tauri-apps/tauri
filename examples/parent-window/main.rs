// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
  command,
  window::{PageLoadEvent, WindowBuilder},
  Window, WindowUrl,
};

#[command]
async fn create_child_window(id: String, window: Window) {
  let _child = WindowBuilder::new(&window, id, WindowUrl::default())
    .title("Child")
    .inner_size(400.0, 300.0)
    .parent(&window)
    .unwrap()
    .build()
    .unwrap();
}

fn main() {
  tauri::Builder::default()
    .on_page_load(|window, payload| {
      if payload.event() == PageLoadEvent::Finished {
        let label = window.label().to_string();
        window.listen("clicked".to_string(), move |_payload| {
          println!("got 'clicked' event on window '{label}'");
        });
      }
    })
    .invoke_handler(tauri::generate_handler![create_child_window])
    .setup(|app| {
      WindowBuilder::new(app, "main".to_string(), WindowUrl::default())
        .title("Main")
        .inner_size(600.0, 400.0)
        .build()?;
      Ok(())
    })
    .run(tauri::generate_context!(
      "../../examples/parent-window/tauri.conf.json"
    ))
    .expect("failed to run tauri application");
}
