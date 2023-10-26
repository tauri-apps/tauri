// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Runtime, WindowBuilder};

fn window_plugin<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
  tauri::plugin::Builder::new("my-window-plugin")
    .on_navigation(|window, _url| {
      let label = window.label().to_string();
      window.listen("clicked".to_string(), move |_payload| {
        println!("got 'clicked' event on window '{label}'");
      });
      true
    })
    .build()
}

fn main() {
  tauri::Builder::default()
    .plugin(window_plugin())
    .setup(|app| {
      #[allow(unused_mut)]
      let mut builder = WindowBuilder::new(
        app,
        "Rust".to_string(),
        tauri::WindowUrl::App("index.html".into()),
      );
      #[cfg(target_os = "macos")]
      {
        builder = builder.tabbing_identifier("Rust");
      }
      let _window = builder.title("Tauri - Rust").build()?;

      Ok(())
    })
    .run(tauri::generate_context!(
      "../../examples/multiwindow/tauri.conf.json"
    ))
    .expect("failed to run tauri application");
}
