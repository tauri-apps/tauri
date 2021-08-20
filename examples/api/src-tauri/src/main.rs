// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;
mod menu;

#[cfg(target_os = "linux")]
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{
  api::dialog::ask, async_runtime, http::ResponseBuilder, CustomMenuItem, Event,
  GlobalShortcutManager, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowBuilder,
  WindowUrl,
};

#[derive(Serialize)]
struct Reply {
  data: String,
}

#[derive(Serialize, Deserialize)]
struct HttpPost {
  foo: String,
  bar: String,
}

#[derive(Serialize)]
struct HttpReply {
  msg: String,
  request: HttpPost,
}

#[tauri::command]
async fn menu_toggle(window: tauri::Window) {
  window.menu_handle().toggle().unwrap();
}

fn main() {
  #[allow(unused_mut)]
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
    .register_global_uri_scheme_protocol("customprotocol", move |request| {
      // FIXME: would be great to access the app_handle
      // to emit global function or access the state
      if request.method() == "POST" {
        let request: HttpPost = serde_json::from_slice(request.body()).unwrap();
        return ResponseBuilder::new()
          .mimetype("application/json")
          .header("Access-Control-Allow-Origin", "*")
          .status(200)
          .body(serde_json::to_vec(&HttpReply {
            request,
            msg: "Hello from rust!".to_string(),
          })?);
      }

      ResponseBuilder::new()
        .mimetype("text/html")
        .status(404)
        .body(Vec::new())
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
          .add_item(CustomMenuItem::new("icon_2", "Tray Icon 2"))
          .add_item(CustomMenuItem::new("exit_app", "Quit")),
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
          "exit_app" => {
            // exit the app
            app.exit(0);
          }
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

  app.run(|app_handle, e| match e {
    // Application is ready (triggered only once)
    Event::Ready => {
      let app_handle = app_handle.clone();
      // launch a new thread so it doesnt block any channel
      async_runtime::spawn(async move {
        let app_handle = app_handle.clone();
        app_handle
          .global_shortcut_manager()
          .register("CmdOrCtrl+1", move || {
            let app_handle = app_handle.clone();
            let window = app_handle.get_window("main").unwrap();
            window.set_title("New title!").unwrap();
          })
          .unwrap();
      });
    }

    // Triggered when a window is trying to close
    Event::CloseRequested { label, api, .. } => {
      let app_handle = app_handle.clone();
      let window = app_handle.get_window(&label).unwrap();
      // use the exposed close api, and prevent the event loop to close
      api.prevent_close();
      // ask the user if he wants to quit
      // we need to run this on another thread because this is the event loop callback handler
      // and the dialog API needs to communicate with the event loop.
      std::thread::spawn(move || {
        ask(
          Some(&window),
          "Tauri API",
          "Are you sure that you want to close this window?",
          move |answer| {
            if answer {
              app_handle.get_window(&label).unwrap().close().unwrap();
            }
          },
        );
      });
    }

    // Keep the event loop running even if all windows are closed
    // This allow us to catch system tray events when there is no window
    Event::ExitRequested { api, .. } => {
      api.prevent_exit();
    }
    _ => {}
  })
}
