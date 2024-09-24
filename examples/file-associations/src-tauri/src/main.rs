// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn handle_file_associations(app: AppHandle, files: Vec<PathBuf>) {
  // -- Scope handling start --

  // You can remove this block if you only want to know about the paths, but not actually "use" them in the frontend.

  // This requires the `fs` tauri plugin and is required to make the plugin's frontend work:
  // use tauri_plugin_fs::FsExt;
  // let fs_scope = app.fs_scope();

  // This is for the `asset:` protocol to work:
  let asset_protocol_scope = app.asset_protocol_scope();

  for file in &files {
    // This requires the `fs` plugin:
    // let _ = fs_scope.allow_file(file);

    // This is for the `asset:` protocol:
    let _ = asset_protocol_scope.allow_file(file);
  }

  // -- Scope handling end --

  let files = files
    .into_iter()
    .map(|f| {
      let file = f.to_string_lossy().replace("\\", "\\\\"); // escape backslash
      format!("\"{file}\"",) // wrap in quotes for JS array
    })
    .collect::<Vec<_>>()
    .join(",");

  tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
    .initialization_script(&format!("window.openedFiles = [{files}]"))
    .build()
    .unwrap();
}

fn main() {
  tauri::Builder::default()
    .setup(|#[allow(unused_variables)] app| {
      #[cfg(any(windows, target_os = "linux"))]
      {
        let mut files = Vec::new();

        // NOTICE: `args` may include URL protocol (`your-app-protocol://`)
        // or arguments (`--`) if your app supports them.
        // files may aslo be passed as `file://path/to/file`
        for maybe_file in std::env::args().skip(1) {
          // skip flags like -f or --flag
          if maybe_file.starts_with("-") {
            continue;
          }

          // handle `file://` path urls and skip other urls
          if let Ok(url) = url::Url::parse(&maybe_file) {
            if let Ok(path) = url.to_file_path() {
              files.push(path);
            }
          } else {
            files.push(PathBuf::from(maybe_file))
          }
        }

        handle_file_associations(app.handle().clone(), files);
      }

      Ok(())
    })
    .build(tauri::generate_context!())
    .expect("error while running tauri application")
    .run(
      #[allow(unused_variables)]
      |app, event| {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        if let tauri::RunEvent::Opened { urls } = event {
          let files = urls
            .into_iter()
            .filter_map(|url| url.to_file_path().ok())
            .collect::<Vec<_>>();

          handle_file_associations(app.clone(), files);
        }
      },
    );
}
