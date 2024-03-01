// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
fn connect(connection: State<'_, Connection>) {
  *connection.0.lock().unwrap() = Some(Client {});
}

#[tauri::command]
fn disconnect(connection: State<'_, Connection>) {
  // drop the connection
  *connection.0.lock().unwrap() = None;
}

#[tauri::command]
fn connection_send(connection: State<'_, Connection>) {
  connection
    .0
    .lock()
    .unwrap()
    .as_ref()
    .expect("connection not initialize; use the `connect` command first")
    .send();
}

#[tauri::command]
fn increment_counter(counter: State<'_, Counter>) -> usize {
  counter.0.fetch_add(1, Ordering::Relaxed) + 1
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
    .manage(Connection(Default::default()))
    .invoke_handler(tauri::generate_handler![
      increment_counter,
      db_insert,
      db_read,
      connect,
      disconnect,
      connection_send
    ])
    .run(tauri::generate_context!(
      "../../examples/state/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}
