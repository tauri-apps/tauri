// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::{env, sync::Mutex};
use tauri::Manager;

struct OpenedUrls(Mutex<Option<Vec<url::Url>>>);

fn main() {
  tauri::Builder::default()
    .manage(OpenedUrls(Default::default()))
    .setup(|app| {
      #[cfg(any(windows, target_os = "linux"))]
      {
        // NOTICE: `args` may include URL protocol (`your-app-protocol://`) or arguments (`--`) if app supports them.
        let mut urls = Vec::new();
        for arg in env::args().skip(1) {
          if let Ok(url) = url::Url::parse(&arg) {
            urls.push(url);
          }
        }

        if !urls.is_empty() {
          app.state::<OpenedUrls>().0.lock().unwrap().replace(urls);
        }
      }

      let opened_urls = if let Some(urls) = &*app.state::<OpenedUrls>().0.lock().unwrap() {
        urls
          .iter()
          .map(|u| u.as_str().replace("\\", "\\\\"))
          .collect::<Vec<_>>()
          .join(", ")
      } else {
        "".into()
      };

      tauri::WebviewWindowBuilder::new(app, "main", Default::default())
        .initialization_script(&format!("window.openedUrls = `{opened_urls}`"))
        .initialization_script(&format!("console.log(`{opened_urls}`)"))
        .build()
        .unwrap();

      Ok(())
    })
    .build(tauri::generate_context!())
    .expect("error while running tauri application")
    .run(
      #[allow(unused_variables)]
      |app, event| {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        if let tauri::RunEvent::Opened { urls } = event {
          if let Some(w) = app.get_webview_window("main") {
            let urls = urls
              .iter()
              .map(|u| u.as_str())
              .collect::<Vec<_>>()
              .join(",");
            let _ = w.eval(&format!("window.onFileOpen(`{urls}`)"));
          }

          app.state::<OpenedUrls>().0.lock().unwrap().replace(urls);
        }
      },
    );
}
