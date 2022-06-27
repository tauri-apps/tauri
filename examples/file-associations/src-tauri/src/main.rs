// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::env;
use tauri::{api::dialog::MessageDialogBuilder, Manager};

fn handle_open_files(files: &[String]) {
  MessageDialogBuilder::new("Files open", format!("You opened: {:?}", files)).show(|_| {});
}

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      // macOS
      app.listen_global("open-urls", |f| {
        let urls: Vec<_> = serde_json::from_str::<Vec<String>>(f.payload().unwrap())
          .unwrap()
          .iter()
          .map(|s| url::Url::parse(s).unwrap())
          .collect();

        // filter out non-file:// urls, you may need to handle them by another method
        let file_paths: Vec<_> = urls.iter().filter_map(|url| {
          if url.scheme() == "file" {
            Some(url.path().into())
          } else {
            None
          }
        }).collect();

        handle_open_files(&file_paths);
      });

      // Windows and Linux
      let argv = env::args().collect::<Vec<_>>();
      if argv.len() > 1 {
        // NOTICE: `argv` may include URL protocol (`your-app-protocol://`) or arguments (`--`) if app supports them.
        handle_open_files(&argv[1..]);
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
