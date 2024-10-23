// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::read;
use tauri::{command, ipc::Response, path::BaseDirectory, AppHandle, Manager, Runtime};

#[command]
fn app_should_close(exit_code: i32) {
  std::process::exit(exit_code);
}

#[command]
async fn read_file<R: Runtime>(app: AppHandle<R>) -> Result<Response, String> {
  let path = app
    .path()
    .resolve(".tauri_3mb.json", BaseDirectory::Home)
    .map_err(|e| e.to_string())?;
  let contents = read(&path).map_err(|e| e.to_string())?;
  Ok(Response::new(contents))
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![app_should_close, read_file])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
