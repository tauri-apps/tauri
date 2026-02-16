// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Runtime, WebviewWindowBuilder};

#[cfg_attr(feature = "cef", tauri::cef_entry_point)]
fn main() {
  #[cfg(feature = "cef")]
  let builder = tauri::Builder::<tauri::Cef>::default();
  #[cfg(not(feature = "cef"))]
  let builder = tauri::Builder::<tauri::Wry>::new();

  builder
    .setup(|app| {
      WebviewWindowBuilder::new(app, "Third", tauri::WebviewUrl::default())
        .title("Tauri - Third")
        .build()?;

      Ok(())
    })
    .run(generate_context())
    .expect("failed to run tauri application");
}

fn generate_context<R: Runtime>() -> tauri::Context<R> {
  let mut context = tauri::generate_context!("../../examples/multiwindow/tauri.conf.json");
  for cmd in [
    "plugin:event|listen",
    "plugin:event|emit",
    "plugin:event|emit_to",
    "plugin:webview|create_webview_window",
  ] {
    context
      .runtime_authority_mut()
      .__allow_command(cmd.to_string(), tauri_utils::acl::ExecutionContext::Local);
  }
  context
}
