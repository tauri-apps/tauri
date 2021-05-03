// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
  },
};

use tauri::State;

struct Counter(AtomicUsize);

#[derive(Default)]
struct Database(Arc<Mutex<HashMap<String, String>>>);

#[tauri::command]
fn increment_counter(counter: State<'_, Counter>) -> usize {
  let count = counter.0.fetch_add(1, Ordering::Relaxed) + 1;
  count
}

#[tauri::command]
fn db_insert(key: String, value: String, db: State<'_, Database>) {
  db.0.lock().unwrap().insert(key, value);
}

#[tauri::command]
fn db_read(key: String, db: State<'_, Database>) -> Option<String> {
  db.0.lock().unwrap().get(&key).cloned()
}

fn main() {
  tauri::Builder::default()
    .manage(Counter(AtomicUsize::new(0)))
    .manage(Database(Default::default()))
    .invoke_handler(tauri::generate_handler![
      increment_counter,
      db_insert,
      db_read
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
