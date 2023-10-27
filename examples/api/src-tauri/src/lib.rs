// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;
#[cfg(desktop)]
mod tray;

use serde::Serialize;
use tauri::{
  ipc::Channel,
  window::{PageLoadEvent, WindowBuilder},
  App, AppHandle, Manager, RunEvent, Runtime, WindowUrl,
};
use tauri_plugin_sample::{PingRequest, SampleExt};

pub type SetupHook = Box<dyn FnOnce(&mut App) -> Result<(), Box<dyn std::error::Error>> + Send>;
pub type OnEvent = Box<dyn FnMut(&AppHandle, RunEvent)>;

#[derive(Clone, Serialize)]
struct Reply {
  data: String,
}

#[cfg(target_os = "macos")]
pub struct AppMenu<R: Runtime>(pub std::sync::Mutex<Option<tauri::menu::Menu<R>>>);

#[cfg(desktop)]
pub struct PopupMenu<R: Runtime>(tauri::menu::Menu<R>);

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
    .plugin(tauri_plugin_sample::init())
    .setup(move |app| {
      #[cfg(desktop)]
      {
        let handle = app.handle();
        tray::create_tray(&handle)?;
        handle.plugin(tauri_plugin_cli::init())?;
      }

      #[cfg(target_os = "macos")]
      app.manage(AppMenu::<R>(Default::default()));

      #[cfg(desktop)]
      app.manage(PopupMenu(
        tauri::menu::MenuBuilder::new(app)
          .check("check", "Tauri is awesome!")
          .text("text", "Do something")
          .copy()
          .build()?,
      ));

      let mut window_builder = WindowBuilder::new(app, "main", WindowUrl::default());
      #[cfg(desktop)]
      {
        window_builder = window_builder
          .title("Tauri API Validation")
          .inner_size(1000., 800.)
          .min_inner_size(600., 400.)
          .content_protected(true)
          .menu(tauri::menu::Menu::default(&app.handle())?);
      }

      let window = window_builder.build().unwrap();

      #[cfg(debug_assertions)]
      window.open_devtools();

      let value = Some("test".to_string());
      let response = app.sample().ping(PingRequest {
        value: value.clone(),
        on_event: Channel::new(|event| {
          println!("got channel event: {:?}", event);
          Ok(())
        }),
      });
      log::info!("got response: {:?}", response);
      if let Ok(res) = response {
        assert_eq!(res.value, value);
      }

      #[cfg(desktop)]
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
    .on_page_load(|window, payload| {
      if payload.event() == PageLoadEvent::Finished {
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
      }
    });

  #[allow(unused_mut)]
  let mut app = builder
    .invoke_handler(tauri::generate_handler![
      cmd::log_operation,
      cmd::perform_request,
      #[cfg(desktop)]
      cmd::toggle_menu,
      #[cfg(desktop)]
      cmd::popup_context_menu
    ])
    .build(tauri::tauri_build_context!())
    .expect("error while building tauri application");

  #[cfg(target_os = "macos")]
  app.set_activation_policy(tauri::ActivationPolicy::Regular);

  app.run(move |_app_handle, _event| {
    #[cfg(all(desktop, not(test)))]
    if let RunEvent::ExitRequested { api, .. } = &_event {
      // Keep the event loop running even if all windows are closed
      // This allow us to catch tray icon events when there is no window
      api.prevent_exit();
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
