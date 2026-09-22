// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Windows runtime-handle clone/drop race reproducer for #15408 and #15411.
//!
//! Run on a Windows desktop:
//! `cargo run -p tauri-runtime-wry --example runtime_handle_clone`
//!
//! This intentionally exercises a reported memory-safety bug. Run it in its own
//! process: an affected runtime may abort or terminate with heap corruption.
//! It uses only public safe APIs and keeps Wry alive until every worker joins.
//! No webview, custom protocol, IPC, or shutdown callback is needed to reproduce.
//! A passing run is not a proof that a data race is absent; repeat in fresh
//! processes when comparing an upstream version with a proposed fix.
use std::{
  hint::black_box,
  sync::{Arc, Barrier},
  thread,
};
use tauri_runtime::{Runtime, RuntimeInitArgs};
use tauri_runtime_wry::Wry;
fn main() {
  let runtime = Wry::<()>::new(RuntimeInitArgs::default()).unwrap();
  let handle = runtime.handle();
  let start = Arc::new(Barrier::new(9));
  let workers: Vec<_> = (0..8)
    .map(|_| {
      let handle = handle.clone();
      let start = start.clone();
      thread::spawn(move || {
        start.wait();
        for _ in 0..100_000 {
          let copy = black_box(handle.clone());
          black_box(&copy);
          drop(copy);
        }
      })
    })
    .collect();
  eprintln!("START 8 workers x 100000 clone/drop; runtime remains alive");
  start.wait();
  for worker in workers {
    worker.join().unwrap();
  }
  eprintln!("JOINED workers");
  drop(handle);
  drop(runtime);
  println!("PASS clone/drop and runtime teardown");
}
