// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let handle = app.handle();
      tauri::async_runtime::spawn(async move {
        match handle.updater().check().await {
          Ok(update) => {
            if let Err(e) = update.download_and_install().await {
              println!("{}", e);
              std::process::exit(1);
            }
            std::process::exit(0);
          }
          Err(e) => {
            println!("{}", e);
            std::process::exit(1);
          }
        }
      });
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
