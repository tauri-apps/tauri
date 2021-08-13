// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
#![allow(
    // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/7422
    clippy::nonstandard_macro_braces,
)]

mod cmd;
mod menu;

#[cfg(target_os = "linux")]
use std::path::PathBuf;

use serde::Serialize;
use tauri::{
  CustomMenuItem, Event, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowBuilder,
  WindowUrl,
};

#[derive(Serialize)]
struct Reply {
  data: String,
}

#[tauri::command]
async fn menu_toggle(window: tauri::Window) {
  window.menu_handle().toggle().unwrap();
}

fn main() {
  let mut app = tauri::Builder::default()
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
    .system_tray(
      SystemTray::new().with_menu(
        SystemTrayMenu::new()
          .add_item(CustomMenuItem::new("toggle", "Toggle"))
          .add_item(CustomMenuItem::new("new", "New window"))
          .add_item(CustomMenuItem::new("icon_1", "Tray Icon 1"))
          .add_item(CustomMenuItem::new("icon_2", "Tray Icon 2")),
      ),
    )
    .on_system_tray_event(|app, event| match event {
      SystemTrayEvent::LeftClick {
        position: _,
        size: _,
        ..
      } => {
        let window = app.get_window("main").unwrap();
        window.show().unwrap();
        window.set_focus().unwrap();
      }
      SystemTrayEvent::MenuItemClick { id, .. } => {
        let item_handle = app.tray_handle().get_item(&id);
        match id.as_str() {
          "toggle" => {
            let window = app.get_window("main").unwrap();
            let new_title = if window.is_visible().unwrap() {
              window.hide().unwrap();
              "Show"
            } else {
              window.show().unwrap();
              "Hide"
            };
            item_handle.set_title(new_title).unwrap();
          }
          "new" => app
            .create_window(
              "new",
              WindowUrl::App("index.html".into()),
              |window_builder, webview_attributes| {
                (window_builder.title("Tauri"), webview_attributes)
              },
            )
            .unwrap(),
          #[cfg(target_os = "macos")]
          "icon_1" => {
            app.tray_handle().set_icon_as_template(true).unwrap();

            app
              .tray_handle()
              .set_icon(tauri::Icon::Raw(
                include_bytes!("../../../.icons/tray_icon_with_transparency.png").to_vec(),
              ))
              .unwrap();
          }
          #[cfg(target_os = "macos")]
          "icon_2" => {
            app.tray_handle().set_icon_as_template(true).unwrap();

            app
              .tray_handle()
              .set_icon(tauri::Icon::Raw(
                include_bytes!("../../../.icons/tray_icon.png").to_vec(),
              ))
              .unwrap();
          }
          #[cfg(target_os = "linux")]
          "icon_1" => app
            .tray_handle()
            .set_icon(tauri::Icon::File(PathBuf::from(
              "../../../.icons/tray_icon_with_transparency.png",
            )))
            .unwrap(),
          #[cfg(target_os = "linux")]
          "icon_2" => app
            .tray_handle()
            .set_icon(tauri::Icon::File(PathBuf::from(
              "../../../.icons/tray_icon.png",
            )))
            .unwrap(),
          #[cfg(target_os = "windows")]
          "icon_1" => app
            .tray_handle()
            .set_icon(tauri::Icon::Raw(
              include_bytes!("../../../.icons/tray_icon_with_transparency.ico").to_vec(),
            ))
            .unwrap(),
          #[cfg(target_os = "windows")]
          "icon_2" => app
            .tray_handle()
            .set_icon(tauri::Icon::Raw(
              include_bytes!("../../../.icons/icon.ico").to_vec(),
            ))
            .unwrap(),
          _ => {}
        }
      }
      _ => {}
    })
    .invoke_handler(tauri::generate_handler![
      cmd::log_operation,
      cmd::perform_request,
      menu_toggle,
    ])
    .build(tauri::generate_context!())
    .expect("error while building tauri application");

  #[cfg(target_os = "macos")]
  app.set_activation_policy(tauri::ActivationPolicy::Regular);

  app.run(|app_handle, e| {
    if let Event::CloseRequested { label, api, .. } = e {
      api.prevent_close();
      let window = app_handle.get_window(&label).unwrap();
      window.emit("close-requested", ()).unwrap();
    }
  })
}
