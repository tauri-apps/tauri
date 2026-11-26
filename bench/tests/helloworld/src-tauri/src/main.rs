// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The benchmark can't use this or we can't measure the command time
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::webview::PageLoadEvent;

#[tauri::command]
fn app_loaded_successfully() {
  bench_common::mark("startup_first_invoke_ms");
  if let Ok(version) = tauri::webview_version() {
    bench_common::info("webview_version", version);
  }
  bench_common::finish();
}

fn main() {
  bench_common::init();

  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![app_loaded_successfully])
    // the windows defined in the configuration are created right before the setup hook
    .setup(|_app| {
      bench_common::mark("startup_window_created_ms");
      Ok(())
    })
    .on_page_load(|_webview, payload| {
      if payload.event() == PageLoadEvent::Started {
        bench_common::mark("startup_page_load_started_ms");
      }
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
