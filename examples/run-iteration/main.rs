// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  let mut app = tauri::Builder::default()
    .build(tauri::generate_context!(
      "../../examples/helloworld/tauri.conf.json"
    ))
    .expect("error while building tauri application");

  app.run_iteration(|_app, _event| {});
  app.cleanup_before_exit();
}
