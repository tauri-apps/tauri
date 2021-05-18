// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;
mod menu;

use serde::Serialize;
use tauri::{CustomMenuItem, Manager, SystemTrayMenuItem, WindowUrl};

#[derive(Serialize)]
struct Reply {
  data: String,
}

fn main() {
  tauri::Builder::default()
    .on_page_load(|window, _| {
      let window_ = window.clone();
      window.listen("js-event", move |event| {
        println!("got js-event with message '{:?}'", event.payload());
        let reply = Reply {
          data: "something else".to_string(),
        };

        window_
          .emit("rust-event", Some(reply))
          .expect("failed to emit");
      });
    })
    .menu(menu::get_menu())
    .on_menu_event(|event| {
      println!("{:?}", event.menu_item_id());
    })
    .system_tray(vec![
      SystemTrayMenuItem::Custom(CustomMenuItem::new("toggle".into(), "Toggle")),
      SystemTrayMenuItem::Custom(CustomMenuItem::new("new".into(), "New window")),
    ])
    .on_system_tray_event(|app, event| {
      match event.menu_item_id().as_str() {
        "toggle" => {
          let window = app.get_window("main").unwrap();
          // TODO: window.is_visible API
          window.hide().unwrap();
        }
        "new" => app
          .create_window(
            "new".into(),
            WindowUrl::App("index.html".into()),
            |window_builder, webview_attributes| (window_builder, webview_attributes),
          )
          .unwrap(),
        _ => {}
      }
    })
    .invoke_handler(tauri::generate_handler![
      cmd::log_operation,
      cmd::perform_request
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
