// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The benchmark can't use this or we can't measure the command time
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Measures the IPC: the frontend drives the commands below in a loop, times them with
//! `performance.now()` and hands the results to [`report`].
//!
//! Timing inside a single process keeps the webview boot, which takes far longer than any of
//! these calls, out of the measurement.

use std::{collections::BTreeMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use tauri::{
  AppHandle, Emitter,
  ipc::{Channel, InvokeBody, InvokeResponseBody, Request, Response},
};

/// Size of the large payloads.
const PAYLOAD_SIZE: usize = 3 * 1024 * 1024;

/// A record of the JSON payload, shaped like what an app would send: a mix of numbers, strings,
/// booleans and arrays.
#[derive(Clone, Serialize, Deserialize)]
struct Record {
  id: u32,
  name: String,
  email: String,
  active: bool,
  score: f64,
  tags: Vec<String>,
  position: [f64; 3],
}

/// Deterministic pseudo random numbers, so every run transfers the same data
/// (and nothing has to be downloaded).
struct Lcg(u64);

impl Lcg {
  fn next(&mut self) -> u32 {
    self.0 = self
      .0
      .wrapping_mul(6364136223846793005)
      .wrapping_add(1442695040888963407);
    (self.0 >> 33) as u32
  }
}

/// Around 3 MB once serialized.
fn records() -> &'static [Record] {
  static RECORDS: OnceLock<Vec<Record>> = OnceLock::new();
  RECORDS.get_or_init(|| {
    let mut random = Lcg(42);
    (0..16_000)
      .map(|id| Record {
        id,
        name: format!("user {} {}", random.next(), random.next()),
        email: format!("user-{}@{}.example.com", random.next(), random.next()),
        active: random.next().is_multiple_of(2),
        score: random.next() as f64 / 1000.,
        tags: (0..4).map(|_| format!("tag-{}", random.next())).collect(),
        position: [
          random.next() as f64 / 7.,
          random.next() as f64 / 11.,
          random.next() as f64 / 13.,
        ],
      })
      .collect()
  })
}

fn bytes() -> &'static [u8] {
  static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
  BYTES.get_or_init(|| {
    let mut random = Lcg(7);
    (0..PAYLOAD_SIZE).map(|_| random.next() as u8).collect()
  })
}

/// The cost of a round trip by itself.
#[tauri::command]
fn noop() {}

/// A large response serialized to JSON.
#[tauri::command]
fn json_out() -> &'static [Record] {
  records()
}

/// A large JSON request, deserialized by the command.
#[tauri::command]
fn json_in(records: Vec<Record>) -> usize {
  records.len()
}

/// A large binary response.
#[tauri::command]
fn raw_out() -> Response {
  Response::new(bytes().to_vec())
}

/// A large binary request.
#[tauri::command]
fn raw_in(request: Request<'_>) -> Result<usize, String> {
  match request.body() {
    InvokeBody::Raw(body) => Ok(body.len()),
    InvokeBody::Json(_) => Err("expected a raw request body".into()),
  }
}

/// Many small messages pushed from Rust through a channel.
#[tauri::command]
fn channel_messages(channel: Channel<u32>, count: u32) -> Result<(), String> {
  for index in 0..count {
    channel.send(index).map_err(|e| e.to_string())?;
  }
  Ok(())
}

/// A large binary payload streamed through a channel in chunks.
#[tauri::command]
fn channel_chunks(
  channel: Channel<InvokeResponseBody>,
  chunk_size: usize,
) -> Result<usize, String> {
  let mut chunks = 0;
  for chunk in bytes().chunks(chunk_size) {
    channel
      .send(InvokeResponseBody::Raw(chunk.to_vec()))
      .map_err(|e| e.to_string())?;
    chunks += 1;
  }
  Ok(chunks)
}

/// Many events emitted from Rust.
#[tauri::command]
fn emit_events(app: AppHandle, count: u32) -> Result<(), String> {
  for index in 0..count {
    app.emit("bench", index).map_err(|e| e.to_string())?;
  }
  Ok(())
}

#[tauri::command]
fn report(metrics: BTreeMap<String, f64>) {
  for (name, value) in metrics {
    bench_common::set(&name, value);
  }
  bench_common::finish();
}

#[tauri::command]
fn fail(message: String) {
  eprintln!("ipc benchmark failed: {message}");
  std::process::exit(1);
}

fn main() {
  bench_common::init();

  // build the payloads before the frontend starts measuring
  records();
  bytes();

  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      noop,
      json_out,
      json_in,
      raw_out,
      raw_in,
      channel_messages,
      channel_chunks,
      emit_events,
      report,
      fail
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
