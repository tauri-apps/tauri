// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_plugin_updater::UpdaterExt;

fn main() {
  eprintln!("running tauri v2 app...");
  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_updater::Builder::new().build())
    .setup(|app| {
      let handle = app.handle().clone();
      println!("current version: {}", app.package_info().version);
      tauri::async_runtime::spawn(async move {
        match handle.updater().unwrap().check().await {
          Ok(Some(update)) => {
            println!("got update {}", update.version);

            if let Err(e) = update
              .download_and_install(
                |chunk, _content_length| {
                  println!("downloaded {chunk} bytes");
                },
                || {
                  println!("finished downloading");
                },
              )
              .await
            {
              println!("{e}");
              std::process::exit(1);
            } else {
              std::process::exit(0);
            }
          }
          Ok(None) => {
            println!("update not found");
            std::process::exit(2)
          }
          Err(e) => {
            println!("{e}");
            std::process::exit(1);
          }
        }
      });
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
