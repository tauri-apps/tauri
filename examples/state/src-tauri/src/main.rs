// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
#![allow(
    // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/7422
    clippy::nonstandard_macro_braces,
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

struct Client;

impl Client {
  fn send(&self) {}
}

#[derive(Default)]
struct Connection(Mutex<Option<Client>>);

#[tauri::command]
fn connect(connection: State<Connection>) {
  *connection.0.lock().unwrap() = Some(Client {});
}

#[tauri::command]
fn disconnect(connection: State<Connection>) {
  // drop the connection
  *connection.0.lock().unwrap() = None;
}

#[tauri::command]
fn connection_send(connection: State<Connection>) {
  connection
    .0
    .lock()
    .unwrap()
    .as_ref()
    .expect("connection not initialize; use the `connect` command first")
    .send();
}

#[tauri::command]
fn increment_counter(counter: State<Counter>) -> usize {
  counter.0.fetch_add(1, Ordering::Relaxed) + 1
}

#[tauri::command]
fn db_insert(key: String, value: String, db: State<Database>) {
  db.0.lock().unwrap().insert(key, value);
}

#[tauri::command]
fn db_read(key: String, db: State<Database>) -> Option<String> {
  db.0.lock().unwrap().get(&key).cloned()
}

fn main() {
  tauri::Builder::default()
    .manage(Counter(AtomicUsize::new(0)))
    .manage(Database(Default::default()))
    .manage(Connection(Default::default()))
    .invoke_handler(tauri::generate_handler![
      increment_counter,
      db_insert,
      db_read,
      connect,
      disconnect,
      connection_send
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
