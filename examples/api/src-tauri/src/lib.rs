// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod cmd;
#[cfg(desktop)]
mod menu_plugin;
#[cfg(desktop)]
mod tray;

use serde::Serialize;
use tauri::{
  ipc::Channel,
  webview::{PageLoadEvent, WebviewWindowBuilder},
  App, Emitter, Listener, Runtime, WebviewUrl,
};
#[allow(unused)]
use tauri::{Manager, RunEvent};
use tauri_plugin_sample::{PingRequest, SampleExt};

#[derive(Clone, Serialize)]
struct Reply {
  data: String,
}

#[cfg(target_os = "macos")]
pub struct AppMenu<R: Runtime>(pub std::sync::Mutex<Option<tauri::menu::Menu<R>>>);

#[cfg(all(desktop, not(test)))]
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
    .plugin(
      tauri_plugin_log::Builder::default()
        .level(log::LevelFilter::Info)
        .build(),
    )
    .plugin(tauri_plugin_sample::init())
    .setup(move |app| {
      #[cfg(all(desktop, not(test)))]
      {
        let handle = app.handle();
        tray::create_tray(handle)?;
        handle.plugin(menu_plugin::init())?;
      }

      #[cfg(target_os = "macos")]
      app.manage(AppMenu::<R>(Default::default()));

      #[cfg(all(desktop, not(test)))]
      app.manage(PopupMenu(
        tauri::menu::MenuBuilder::new(app)
          .check("check", "Tauri is awesome!")
          .text("text", "Do something")
          .copy()
          .build()?,
      ));

      let mut window_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default());

      #[cfg(all(desktop, not(test)))]
      {
        window_builder = window_builder
          .title("Tauri API Validation")
          .inner_size(1000., 800.)
          .min_inner_size(600., 400.)
          .menu(tauri::menu::Menu::default(app.handle())?);
      }

      let webview = window_builder.build()?;

      #[cfg(debug_assertions)]
      webview.open_devtools();

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
    .on_page_load(|webview, payload| {
      if payload.event() == PageLoadEvent::Finished {
        let webview_ = webview.clone();
        webview.listen("js-event", move |event| {
          println!("got js-event with message '{:?}'", event.payload());
          let reply = Reply {
            data: "something else".to_string(),
          };

          webview_
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
      cmd::echo
    ])
    .build(tauri::tauri_build_context!())
    .expect("error while building tauri application");

  #[cfg(target_os = "macos")]
  app.set_activation_policy(tauri::ActivationPolicy::Regular);

  app.run(move |_app_handle, _event| {
    #[cfg(all(desktop, not(test)))]
    match &_event {
      RunEvent::ExitRequested { api, code, .. } => {
        // Keep the event loop running even if all windows are closed
        // This allow us to catch tray icon events when there is no window
        // if we manually requested an exit (code is Some(_)) we will let it go through
        if code.is_none() {
          api.prevent_exit();
        }
      }
      RunEvent::WindowEvent {
        event: tauri::WindowEvent::CloseRequested { api, .. },
        label,
        ..
      } => {
        println!("closing window...");
        // run the window destroy manually just for fun :)
        // usually you'd show a dialog here to ask for confirmation or whatever
        api.prevent_close();
        _app_handle
          .get_webview_window(label)
          .unwrap()
          .destroy()
          .unwrap();
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
      let window = app.get_webview_window("main").unwrap();
      std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        window.close().unwrap();
      });
    })
  }
}
