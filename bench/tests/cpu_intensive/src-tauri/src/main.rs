// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The benchmark can't use this or we can't measure the command time
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// `compute_ms` is the time the web worker took to find the prime numbers.
#[tauri::command]
fn app_completed_successfully(compute_ms: f64) {
  bench_common::set("compute_ms", compute_ms);
  bench_common::finish();
}

fn main() {
  bench_common::init();

  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![app_completed_successfully])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
