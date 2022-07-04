// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

fn main() {
  use tauri::{
    api::process::{Command, CommandEvent},
    Manager,
  };

  tauri::Builder::default()
    .setup(move |app| {
      let window = app.get_window("main").unwrap();
      let script_path = app
        .path_resolver()
        .resolve_resource("assets/index.js")
        .unwrap()
        .to_string_lossy()
        .to_string();
      tauri::async_runtime::spawn(async move {
        let (mut rx, _child) = Command::new("node")
          .args(&[script_path])
          .spawn()
          .expect("Failed to spawn node");

        #[allow(clippy::collapsible_match)]
        while let Some(event) = rx.recv().await {
          if let CommandEvent::Stdout(line) = event {
            window
              .emit("message", Some(format!("'{}'", line)))
              .expect("failed to emit event");
          }
        }
      });

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
