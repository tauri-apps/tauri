// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use tauri::command;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct RequestBody {
  id: i32,
  name: String,
}

#[command]
pub fn log_operation(event: String, payload: Option<String>) {
  log::info!("{} {:?}", event, payload);
}

#[derive(serde::Serialize)]
pub struct R {
  x: String,
}

#[command]
pub fn perform_request(endpoint: String, body: RequestBody) -> R {
  println!("{} {:?}", endpoint, body);
  R {
    x: "message response".into(),
  }
}
