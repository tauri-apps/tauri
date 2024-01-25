// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
  io::{BufRead, BufReader},
  process::{Command, Stdio},
};
use tauri::Manager;

fn main() {
  tauri::Builder::default()
    .setup(move |app| {
      let window = app.get_webview_window("main").unwrap();
      let script_path = app
        .path()
        .resolve("assets/index.js", tauri::path::BaseDirectory::Resource)
        .unwrap()
        .to_string_lossy()
        .to_string();
      std::thread::spawn(move || {
        let mut child = Command::new("node")
          .args(&[script_path])
          .stdout(Stdio::piped())
          .spawn()
          .expect("Failed to spawn node");
        let stdout = child.stdout.take().unwrap();
        let mut stdout = BufReader::new(stdout);

        let mut line = String::new();
        loop {
          let n = stdout.read_line(&mut line).unwrap();
          if n == 0 {
            break;
          }

          window
            .emit("message", Some(format!("'{}'", line)))
            .expect("failed to emit event");

          line.clear();
        }
      });

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
