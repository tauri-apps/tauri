// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::env;
use tauri::api::dialog::MessageDialogBuilder;

fn handle_open_files(files: &[String]) {
  MessageDialogBuilder::new(
    "Files open",
    format!(
      "You opened: {:?}",
      files
        .iter()
        .map(|f| percent_encoding::percent_decode(f.as_bytes())
          .decode_utf8_lossy()
          .into_owned())
        .collect::<Vec<String>>()
    ),
  )
  .show(|_| {});
}

fn main() {
  tauri::Builder::default()
    .setup(|_app| {
      #[cfg(any(windows, target_os = "linux"))]
      {
        // Windows and Linux
        let argv = env::args().collect::<Vec<_>>();
        if argv.len() > 1 {
          // NOTICE: `argv` may include URL protocol (`your-app-protocol://`) or arguments (`--`) if app supports them.
          handle_open_files(&argv[1..]);
        }
      }
      Ok(())
    })
    .build(tauri::generate_context!())
    .expect("error while running tauri application")
    .run(|_app, event| {
      #[cfg(target_os = "macos")]
      if let tauri::RunEvent::OpenURLs(urls) = event {
        // filter out non-file:// urls, you may need to handle them by another method
        let file_paths: Vec<_> = urls
          .iter()
          .filter_map(|url| {
            if url.scheme() == "file" {
              Some(url.path().into())
            } else {
              None
            }
          })
          .collect();

        handle_open_files(&file_paths);
      }
    });
}
