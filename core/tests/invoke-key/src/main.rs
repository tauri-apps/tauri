// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{command, generate_context, generate_handler, Builder};

mod subscriber;

#[command]
fn error_if_called() {
  std::process::exit(1)
}

fn main() {
  tracing::subscriber::set_global_default(subscriber::InvokeKeyErrorSubscriber)
    .expect("unable to set tracing global subscriber");

  Builder::default()
    .invoke_handler(generate_handler![error_if_called])
    .run(generate_context!())
    .expect("error while running tauri application");
}
