// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{LogicalPosition, LogicalSize, WebviewUrl};

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let width = 800.;
      let height = 600.;

      let window = tauri::window::WindowBuilder::new(app, "main")
        .inner_size(width, height)
        .build()?;

      let _webview1 = window.add_child(
        tauri::webview::WebviewBuilder::new("main1", WebviewUrl::App(Default::default()))
          .auto_resize(),
        LogicalPosition::new(0., 0.),
        LogicalSize::new(width / 2., height / 2.),
      )?;

      let _webview2 = window.add_child(
        tauri::webview::WebviewBuilder::new(
          "main2",
          WebviewUrl::External("https://github.com/tauri-apps/tauri".parse().unwrap()),
        )
        .auto_resize(),
        LogicalPosition::new(width / 2., 0.),
        LogicalSize::new(width / 2., height / 2.),
      )?;

      let _webview3 = window.add_child(
        tauri::webview::WebviewBuilder::new(
          "main3",
          WebviewUrl::External("https://tauri.app".parse().unwrap()),
        )
        .auto_resize(),
        LogicalPosition::new(0., height / 2.),
        LogicalSize::new(width / 2., height / 2.),
      )?;

      let _webview4 = window.add_child(
        tauri::webview::WebviewBuilder::new(
          "main4",
          WebviewUrl::External("https://twitter.com/TauriApps".parse().unwrap()),
        )
        .auto_resize(),
        LogicalPosition::new(width / 2., height / 2.),
        LogicalSize::new(width / 2., height / 2.),
      )?;

      Ok(())
    })
    .run(tauri::generate_context!(
      "../../examples/multiwebview/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}
