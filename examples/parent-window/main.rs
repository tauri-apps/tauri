// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{webview::PageLoadEvent, WebviewUrl, WebviewWindowBuilder};
use tauri_utils::acl::ExecutionContext;

fn main() {
  let mut context = tauri::generate_context!("../../examples/parent-window/tauri.conf.json");
  for cmd in [
    "plugin:event|listen",
    "plugin:webview|create_webview_window",
    "plugin:window|internal_on_mousemove",
    "plugin:window|internal_on_mousedown",
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
      let _webview = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("Main")
        .inner_size(600.0, 400.0)
        .build()?;

      Ok(())
    })
    .run(context)
    .expect("failed to run tauri application");
}
