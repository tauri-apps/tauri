// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod cmd;
mod tray;

use serde::Serialize;
use tauri::{
  api::dialog::{ask, message},
  App, GlobalShortcutManager, Manager, RunEvent, Runtime, WindowBuilder, WindowEvent, WindowUrl,
};

#[derive(Clone, Serialize)]
struct Reply {
  data: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  run_app(tauri::Builder::default(), |_app| {})
}

pub fn run_app<R: Runtime, F: FnOnce(&App<R>) + Send + 'static>(
  builder: tauri::Builder<R>,
  setup: F,
) {
  #[allow(unused_mut)]
  let mut builder = builder
    .setup(move |app| {
      tray::create_tray(app)?;

      #[allow(unused_mut)]
      let mut window_builder = WindowBuilder::new(app, "main", WindowUrl::default())
        .user_agent("Tauri API")
        .title("Tauri API Validation")
        .inner_size(1000., 800.)
        .min_inner_size(600., 400.)
        .content_protected(true);

      #[cfg(target_os = "windows")]
      {
        window_builder = window_builder.transparent(true).decorations(false);
      }

      let window = window_builder.build().unwrap();

      #[cfg(target_os = "windows")]
      {
        let _ = window_shadows::set_shadow(&window, true);
      }

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

      setup(app);

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
    });

  #[cfg(target_os = "macos")]
  {
    builder = builder.menu(tauri::Menu::os_default("Tauri API Validation"));
  }

  #[allow(unused_mut)]
  let mut app = builder
    .invoke_handler(tauri::generate_handler![
      cmd::log_operation,
      cmd::perform_request,
    ])
    .build(tauri::tauri_build_context!())
    .expect("error while building tauri application");

  #[cfg(target_os = "macos")]
  app.set_activation_policy(tauri::ActivationPolicy::Regular);

  app.run(move |app_handle, e| {
    match e {
      // Application is ready (triggered only once)
      RunEvent::Ready => {
        let app_handle = app_handle.clone();
        app_handle
          .global_shortcut_manager()
          .register("CmdOrCtrl+1", move || {
            let app_handle = app_handle.clone();
            if let Some(window) = app_handle.get_window("main") {
              message(
                Some(&window),
                "Tauri API",
                "CmdOrCtrl+1 global shortcut triggered",
              );
            }
          })
          .unwrap();
      }

      // Triggered when a window is trying to close
      RunEvent::WindowEvent {
        label,
        event: WindowEvent::CloseRequested { api, .. },
        ..
      } => {
        // for other windows, we handle it in JS
        if label == "main" {
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
      }
      #[cfg(not(test))]
      RunEvent::ExitRequested { api, .. } => {
        // Keep the event loop running even if all windows are closed
        // This allow us to catch system tray events when there is no window
        api.prevent_exit();
      }
      _ => (),
    }
  })
}

#[cfg(test)]
mod tests {
  use tauri::Manager;

  #[test]
  fn run_app() {
    super::run_app(tauri::test::mock_builder(), |app| {
      let window = app.get_window("main").unwrap();
      std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        window.close().unwrap();
      });
    })
  }
}
