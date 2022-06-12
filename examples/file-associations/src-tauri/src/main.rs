// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::env;

use tauri::{api::dialog::MessageDialogBuilder, Manager};

fn handle_open_file(file: &str) {
  MessageDialogBuilder::new("File open", format!("You opened: {}", file))
    .show(|_| {});
}

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      // macOS
      app.listen_global("open-file", |f| {
        handle_open_file(f.payload().unwrap());
      });

      // Windows and Linux
      let argv = env::args().collect::<Vec<_>>();
      if argv.len() > 1 {
        for file in argv[1..].iter() {
          handle_open_file(file);
        }
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
