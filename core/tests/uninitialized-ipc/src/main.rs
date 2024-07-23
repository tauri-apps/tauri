// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{command, generate_context, generate_handler, http::Response, Builder};

const IFRAME: &[u8] = include_bytes!("../iframe.html");

#[command]
fn exit(code: i32) {
  std::process::exit(code)
}

fn main() {
  Builder::default()
    .invoke_handler(generate_handler![exit])
    .register_uri_scheme_protocol("uninitialized", |_app, _request| {
      Ok(Response::new(IFRAME.into()))
    })
    .run(generate_context!())
    .expect("error while running tauri application");
}
