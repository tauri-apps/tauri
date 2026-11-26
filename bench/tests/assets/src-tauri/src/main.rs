// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The benchmark can't use this or we can't measure the command time
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! The hello world app serves a single tiny HTML file. This one has a frontend the size of a
//! real application (see the build script), to measure the asset protocol and the
//! decompression of the embedded assets.

use std::collections::BTreeMap;

use tauri::webview::PageLoadEvent;

/// Called once every script ran and every asset was fetched.
#[tauri::command]
fn assets_loaded(metrics: BTreeMap<String, f64>) {
  bench_common::mark("startup_assets_loaded_ms");
  for (name, value) in metrics {
    bench_common::set(&name, value);
  }
  bench_common::finish();
}

#[tauri::command]
fn fail(message: String) {
  eprintln!("assets benchmark failed: {message}");
  std::process::exit(1);
}

fn main() {
  bench_common::init();

  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![assets_loaded, fail])
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
