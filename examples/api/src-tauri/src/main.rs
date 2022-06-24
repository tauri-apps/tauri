// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{
  api::dialog::ask, window::WindowBuilder, CustomMenuItem, GlobalShortcutManager, Manager,
  RunEvent, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowEvent, WindowUrl,
};

#[derive(Clone, Serialize)]
struct Reply {
  data: String,
}

fn main() {
  let tray_menu1 = SystemTrayMenu::new()
    .add_item(CustomMenuItem::new("toggle", "Toggle"))
    .add_item(CustomMenuItem::new("new", "New window"))
    .add_item(CustomMenuItem::new("icon_1", "Tray Icon 1"))
    .add_item(CustomMenuItem::new("icon_2", "Tray Icon 2"))
    .add_item(CustomMenuItem::new("switch_menu", "Switch Menu"))
    .add_item(CustomMenuItem::new("exit_app", "Quit"));
  let tray_menu2 = SystemTrayMenu::new()
    .add_item(CustomMenuItem::new("toggle", "Toggle"))
    .add_item(CustomMenuItem::new("new", "New window"))
    .add_item(CustomMenuItem::new("switch_menu", "Switch Menu"))
    .add_item(CustomMenuItem::new("exit_app", "Quit"));
  let is_menu1 = AtomicBool::new(true);

  #[allow(unused_mut)]
  let mut app = tauri::Builder::default()
    .setup(|app| {
      let window = app.get_window("main").unwrap();

      #[cfg(any(windows, target_os = "macos"))]
      let _ = window_shadows::set_shadow(&window, true);
      #[cfg(target_os = "windows")]
      let _ = window_vibrancy::apply_blur(&window, Some((0, 0, 0, 0)));
      #[cfg(target_os = "macos")]
      let _ = window_vibrancy::apply_vibrancy(
        &window,
        window_vibrancy::NSVisualEffectMaterial::HudWindow,
      );

      #[cfg(debug_assertions)]
      window.open_devtools();

      std::thread::spawn(|| {
        let server = match tiny_http::Server::http("localhost:3003") {
          Ok(s) => s,
          Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
          }
        };
        loop {
          if let Ok(mut request) = server.recv() {
            let mut body = Vec::new();
            let _ = request.as_reader().read_to_end(&mut body);
            let response = tiny_http::Response::new(
              tiny_http::StatusCode(200),
              request.headers().to_vec(),
              std::io::Cursor::new(body),
              request.body_length(),
              None,
            );
            let _ = request.respond(response);
          }
        }
      });

      Ok(())
    })
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
    .system_tray(SystemTray::new().with_menu(tray_menu1.clone()))
    .on_system_tray_event(move |app, event| match event {
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
          "new" => {
            WindowBuilder::new(app, "new", WindowUrl::App("index.html".into()))
              .title("Tauri")
              .build()
              .unwrap();
          }
          "icon_1" => {
            #[cfg(target_os = "macos")]
            app.tray_handle().set_icon_as_template(true).unwrap();

            app
              .tray_handle()
              .set_icon(tauri::Icon::Raw(
                include_bytes!("../../../.icons/tray_icon_with_transparency.png").to_vec(),
              ))
              .unwrap();
          }
          "icon_2" => {
            #[cfg(target_os = "macos")]
            app.tray_handle().set_icon_as_template(true).unwrap();

            app
              .tray_handle()
              .set_icon(tauri::Icon::Raw(
                include_bytes!("../../../.icons/icon.ico").to_vec(),
              ))
              .unwrap();
          }
          "switch_menu" => {
            let flag = is_menu1.load(Ordering::Relaxed);
            app
              .tray_handle()
              .set_menu(if flag {
                tray_menu2.clone()
              } else {
                tray_menu1.clone()
              })
              .unwrap();
            is_menu1.store(!flag, Ordering::Relaxed);
          }
          _ => {}
        }
      }
      _ => {}
    })
    .invoke_handler(tauri::generate_handler![
      cmd::log_operation,
      cmd::perform_request,
    ])
    .build(tauri::generate_context!())
    .expect("error while building tauri application");

  #[cfg(target_os = "macos")]
  app.set_activation_policy(tauri::ActivationPolicy::Regular);

  app.run(|app_handle, e| match e {
    // Application is ready (triggered only once)
    RunEvent::Ready => {
      let app_handle = app_handle.clone();
      app_handle
        .global_shortcut_manager()
        .register("CmdOrCtrl+1", move || {
          let app_handle = app_handle.clone();
          let window = app_handle.get_window("main").unwrap();
          window.set_title("New title!").unwrap();
        })
        .unwrap();
    }

    // Triggered when a window is trying to close
    RunEvent::WindowEvent {
      label,
      event: WindowEvent::CloseRequested { api, .. },
      ..
    } => {
      let app_handle = app_handle.clone();
      let window = app_handle.get_window(&label).unwrap();
      // use the exposed close api, and prevent the event loop to close
      api.prevent_close();
      // ask the user if he wants to quit
      ask(
        Some(&window),
        "Tauri API",
        "Are you sure that you want to close this window?",
        move |answer| {
          if answer {
            // .close() cannot be called on the main thread
            std::thread::spawn(move || {
              app_handle.get_window(&label).unwrap().close().unwrap();
            });
          }
        },
      );
    }

    // Keep the event loop running even if all windows are closed
    // This allow us to catch system tray events when there is no window
    RunEvent::ExitRequested { api, .. } => {
      api.prevent_exit();
    }
    _ => {}
  })
}
