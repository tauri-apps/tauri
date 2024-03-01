// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
  let mut app = tauri::Builder::default()
    .build(tauri::generate_context!(
      "../../examples/run-iteration/tauri.conf.json"
    ))
    .expect("error while building tauri application");

  loop {
    app.run_iteration(|_app, _event| {
      //println!("{:?}", _event);
    });

    if app.webview_windows().is_empty() {
      app.cleanup_before_exit();
      break;
    }
  }
}
