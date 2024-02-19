// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{webview::PageLoadEvent, WebviewWindowBuilder};
use tauri_utils::acl::ExecutionContext;

fn main() {
  let mut context = tauri::generate_context!("../../examples/multiwindow/tauri.conf.json");
  for cmd in [
    "plugin:event|listen",
    "plugin:event|emit",
    "plugin:event|emit_to",
  ] {
    context
      .runtime_authority_mut()
      .__allow_command(cmd.to_string(), ExecutionContext::Local);
  }

  tauri::Builder::default()
    .on_page_load(|webview, payload| {
      if payload.event() == PageLoadEvent::Finished {
        let label = webview.label().to_string();
        webview.listen("clicked".to_string(), move |_payload| {
          println!("got 'clicked' event on window '{label}'");
        });
      }
    })
    .setup(|app| {
      #[allow(unused_mut)]
      let mut builder =
        WebviewWindowBuilder::new(app, "Rust", tauri::WebviewUrl::App("index.html".into()));
      #[cfg(target_os = "macos")]
      {
        builder = builder.tabbing_identifier("Rust");
      }
      let _webview = builder.title("Tauri - Rust").build()?;

      Ok(())
    })
    .run(context)
    .expect("failed to run tauri application");
}
