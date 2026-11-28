// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Metric reporting shared by the benchmark apps and the benchmark runner.
//!
//! The runner cannot see what happens inside an app, it only knows when the process
//! started and exited, and most of that time is spent booting the webview.
//! So the apps measure themselves: they record how long after being spawned they reached
//! a few milestones (plus whatever else they benchmark) and print it as a single JSON line
//! on stdout, which the runner collects.
//!
//! ***_Internal use only_***

use std::{
  collections::BTreeMap,
  fmt::Write,
  sync::{Mutex, OnceLock},
  time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

/// Set by the runner to the moment it spawned the app, in nanoseconds since the Unix epoch.
pub const SPAWN_TIME_ENV: &str = "TAURI_BENCH_SPAWN_TIME";
/// When set, the app keeps running after reporting its metrics, so the runner can inspect the
/// idle process. The runner is responsible for killing it.
pub const HOLD_ENV: &str = "TAURI_BENCH_HOLD";
/// How many extra windows the hello world app opens after its first one loaded.
pub const WINDOWS_ENV: &str = "TAURI_BENCH_WINDOWS";
/// Marks the stdout line carrying the metrics.
pub const METRICS_PREFIX: &str = "TAURI_BENCH_METRICS ";

enum Value {
  Number(f64),
  Text(String),
}

struct State {
  /// How long the process had been alive when [`init`] was called.
  offset: Duration,
  init: Instant,
  values: Mutex<BTreeMap<String, Value>>,
  reported: Mutex<bool>,
}

static STATE: OnceLock<State> = OnceLock::new();

fn state() -> &'static State {
  STATE.get_or_init(|| {
    let offset = std::env::var(SPAWN_TIME_ENV)
      .ok()
      .and_then(|spawned| spawned.parse::<u64>().ok())
      .and_then(|spawned| {
        SystemTime::now()
          .duration_since(UNIX_EPOCH + Duration::from_nanos(spawned))
          .ok()
      })
      // not started by the runner: measure from here instead
      .unwrap_or_default();

    State {
      offset,
      init: Instant::now(),
      values: Mutex::default(),
      reported: Mutex::new(false),
    }
  })
}

fn insert(name: &str, value: Value, overwrite: bool) {
  let mut values = state().values.lock().unwrap();
  if overwrite || !values.contains_key(name) {
    values.insert(name.to_string(), value);
  }
}

/// Starts the clock. Must be the first thing `main` does.
///
/// Records `startup_main_ms`: the time the OS took to load the binary and its libraries.
pub fn init() {
  mark("startup_main_ms");
}

/// Records the time since the app was spawned, in milliseconds.
/// Only the first call for a given name counts.
pub fn mark(name: &str) {
  let state = state();
  let elapsed = state.offset + state.init.elapsed();
  insert(name, Value::Number(elapsed.as_secs_f64() * 1000.), false);
}

/// Records a measurement taken by the app.
pub fn set(name: &str, value: f64) {
  insert(name, Value::Number(value), true);
}

/// Records a description of the environment (e.g. the webview version).
pub fn info(name: &str, value: impl Into<String>) {
  insert(name, Value::Text(value.into()), true);
}

/// Prints the metrics, once.
pub fn report() {
  let state = state();
  let mut reported = state.reported.lock().unwrap();
  if *reported {
    return;
  }
  *reported = true;

  let values = state.values.lock().unwrap();
  let mut json = String::from("{");
  for (index, (name, value)) in values.iter().enumerate() {
    if index > 0 {
      json.push(',');
    }
    write_json_string(&mut json, name);
    json.push(':');
    match value {
      Value::Number(number) if number.is_finite() => write!(json, "{number}").unwrap(),
      Value::Number(_) => json.push_str("null"),
      Value::Text(text) => write_json_string(&mut json, text),
    }
  }
  json.push('}');

  println!("{METRICS_PREFIX}{json}");
}

/// Prints the metrics and exits, unless the runner asked the app to stay alive.
pub fn finish() {
  report();
  if std::env::var_os(HOLD_ENV).is_none() {
    std::process::exit(0);
  }
}

fn write_json_string(json: &mut String, value: &str) {
  json.push('"');
  for c in value.chars() {
    match c {
      '"' => json.push_str("\\\""),
      '\\' => json.push_str("\\\\"),
      c if c.is_control() => write!(json, "\\u{:04x}", c as u32).unwrap(),
      c => json.push(c),
    }
  }
  json.push('"');
}
