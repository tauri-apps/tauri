// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The benchmark can't use this or we can't measure the command time
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
  sync::{
    OnceLock,
    atomic::{AtomicUsize, Ordering},
  },
  time::Instant,
};

use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder, webview::PageLoadEvent};

/// How many extra windows the runner asked for.
fn extra_windows() -> usize {
  std::env::var(bench_common::WINDOWS_ENV)
    .ok()
    .and_then(|windows| windows.parse().ok())
    .unwrap_or(0)
}

/// Called by every window once its page loaded.
#[tauri::command]
fn app_loaded_successfully(app: AppHandle) {
  static LOADED: AtomicUsize = AtomicUsize::new(0);
  static OPENING: OnceLock<Instant> = OnceLock::new();

  let loaded = LOADED.fetch_add(1, Ordering::SeqCst) + 1;
  let extra_windows = extra_windows();

  if loaded == 1 {
    bench_common::mark("startup_first_invoke_ms");
    if let Ok(version) = tauri::webview_version() {
      bench_common::info("webview_version", version);
    }

    if extra_windows == 0 {
      bench_common::finish();
      return;
    }

    // a window can't be created from a synchronous command, which blocks the main thread
    std::thread::spawn(move || {
      OPENING.get_or_init(Instant::now);
      for index in 0..extra_windows {
        WebviewWindowBuilder::new(&app, format!("extra-{index}"), WebviewUrl::default())
          .build()
          .expect("failed to create window");
      }
    });
  } else if loaded == extra_windows + 1 {
    let elapsed = OPENING.get().map(Instant::elapsed).unwrap_or_default();
    bench_common::set(
      "open_window_ms",
      elapsed.as_secs_f64() * 1000. / extra_windows as f64,
    );
    bench_common::finish();
  }
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
