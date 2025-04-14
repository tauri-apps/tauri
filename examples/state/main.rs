// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use tauri::State;

struct Counter(Mutex<usize>);

#[tauri::command]
fn increment(counter: State<'_, Counter>) -> usize {
  let mut c = counter.0.lock().unwrap();
  *c += 1;
  *c
}

#[tauri::command]
fn decrement(counter: State<'_, Counter>) -> usize {
  let mut c = counter.0.lock().unwrap();
  *c -= 1;
  *c
}

#[tauri::command]
fn reset(counter: State<'_, Counter>) -> usize {
  let mut c = counter.0.lock().unwrap();
  *c = 0;
  *c
}

#[tauri::command]
fn get(counter: State<'_, Counter>) -> usize {
  *counter.0.lock().unwrap()
}

fn main() {
  tauri::Builder::default()
    .manage(Counter(Mutex::new(0)))
    .invoke_handler(tauri::generate_handler![increment, decrement, reset, get])
    .run(tauri::generate_context!(
      "../../examples/state/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}
