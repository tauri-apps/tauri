// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
  command, webview::PageLoadEvent, window::WindowBuilder, WebviewBuilder, WebviewUrl, Window,
};

#[command]
async fn create_child_window(id: String, window: Window) {
  let builder = WindowBuilder::new(&window, &id)
    .title("Child")
    .inner_size(400.0, 300.0);

  #[cfg(target_os = "macos")]
  let builder = builder.parent_window(window.ns_window().unwrap());
  #[cfg(windows)]
  let builder = builder.parent_window(window.hwnd().unwrap());

  let (_child, _webview) = builder
    .with_webview(WebviewBuilder::new(id, WebviewUrl::default()))
    .unwrap();
}

fn main() {
  tauri::Builder::default()
    .on_page_load(|webview, payload| {
      if payload.event() == PageLoadEvent::Finished {
        let label = webview.label().to_string();
        webview.listen("clicked".to_string(), move |_payload| {
          println!("got 'clicked' event on window '{label}'");
        });
      }
    })
    .invoke_handler(tauri::generate_handler![create_child_window])
    .setup(|app| {
      let (_window, _webview) = WindowBuilder::new(app, "main")
        .title("Main")
        .inner_size(600.0, 400.0)
        .with_webview(WebviewBuilder::new("main", WebviewUrl::default()))?;

      Ok(())
    })
    .run(tauri::generate_context!(
      "../../examples/parent-window/tauri.conf.json"
    ))
    .expect("failed to run tauri application");
}
