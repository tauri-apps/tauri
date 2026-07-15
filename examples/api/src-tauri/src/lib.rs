// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod cmd;
#[cfg(all(desktop, not(test), not(feature = "cef")))]
mod menu_plugin;
#[cfg(all(desktop, not(test), not(feature = "cef")))]
mod tray;

use serde::Serialize;
use tauri::{
  App, Emitter, Listener, Runtime, WebviewUrl,
  ipc::Channel,
  webview::{PageLoadEvent, WebviewWindowBuilder},
};
#[allow(unused)]
use tauri::{Manager, RunEvent};
use tauri_plugin_sample::{PingRequest, SampleExt};

#[cfg(test)]
type TauriRuntime = tauri::test::MockRuntime;
#[cfg(all(not(test), feature = "cef"))]
type TauriRuntime = tauri::Cef;
#[cfg(all(not(test), not(feature = "cef")))]
type TauriRuntime = tauri::Wry;

#[derive(Clone, Serialize)]
struct Reply {
  data: String,
}

#[cfg(target_os = "macos")]
pub struct AppMenu<R: Runtime>(pub std::sync::Mutex<Option<tauri::menu::Menu<R>>>);

#[cfg(all(desktop, not(test)))]
pub struct PopupMenu<R: Runtime>(#[allow(dead_code)] tauri::menu::Menu<R>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[cfg_attr(feature = "cef", tauri::cef_entry_point)]
pub fn run() {
  run_app(tauri::Builder::<TauriRuntime>::new(), |_app| {});
}

pub fn run_app<F: FnOnce(&App<TauriRuntime>) + Send + 'static>(
  builder: tauri::Builder<TauriRuntime>,
  setup: F,
) {
  let builder = builder
    .plugin(
      tauri_plugin_log::Builder::default()
        .level(log::LevelFilter::Info)
        .build(),
    )
    .plugin(tauri_plugin_sample::init())
    .setup(move |app| {
      #[cfg(all(desktop, not(test), not(feature = "cef")))]
      {
        let handle = app.handle();
        tray::create_tray(handle)?;
        handle.plugin(menu_plugin::init())?;
      }

      #[cfg(target_os = "macos")]
      app.manage(AppMenu::<TauriRuntime>(Default::default()));

      #[cfg(all(desktop, not(test)))]
      app.manage(PopupMenu(
        tauri::menu::MenuBuilder::new(app)
          .check("check", "Tauri is awesome!")
          .text("text", "Do something")
          .copy()
          .build()?,
      ));

      #[allow(unused_mut)]
      let mut window_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .disable_drag_drop_handler()
        .on_document_title_changed(|_window, title| {
          println!("document title changed: {title}");
        })
        .on_address_change(|_webview, url| {
          println!("CEF address changed: {url}");
        });

      #[cfg(all(desktop, not(test)))]
      {
        use std::sync::atomic::{AtomicU64, Ordering};

        let app_ = app.handle().clone();
        let created_window_count = AtomicU64::new(0);

        window_builder = window_builder
          .title("Tauri API Validation")
          .inner_size(1000., 800.)
          .min_inner_size(600., 400.)
          .menu(tauri::menu::Menu::default(app.handle())?)
          .on_new_window(move |url, features| {
            println!("new window requested: {url:?} {features:?}");

            let number = created_window_count.fetch_add(1, Ordering::Relaxed);

            let builder = WebviewWindowBuilder::new(
              &app_,
              format!("new-{number}"),
              tauri::WebviewUrl::External(if cfg!(feature = "cef") {
                url.clone()
              } else {
                "about:blank".parse().unwrap()
              }),
            )
            .window_features(features)
            .on_document_title_changed(|window, title| {
              window.set_title(&title).unwrap();
            })
            .title(url.as_str());

            let window = builder.build().unwrap();
            tauri::webview::NewWindowResponse::Create { window }
          });
      }

      let webview = window_builder.build()?;

      #[cfg(debug_assertions)]
      webview.open_devtools();

      #[cfg(feature = "cef")]
      {
        webview
          .on_dev_tools_protocol(|protocol| match protocol {
            tauri::CefDevToolsProtocol::Message(msg) => {
              if let Ok(s) = std::str::from_utf8(&msg) {
                log::info!("DevTools message: {s}");
              } else {
                log::error!("Failed to convert DevTools message to UTF-8");
              }
            }
            tauri::CefDevToolsProtocol::Event { method, params } => {
              log::info!(
                "DevTools event: {method} (params: {})",
                String::from_utf8_lossy(&params)
              );
            }
            tauri::CefDevToolsProtocol::MethodResult {
              message_id,
              success,
              result,
            } => {
              log::info!(
                "DevTools result: id={message_id} success={success} ({})",
                String::from_utf8_lossy(&result)
              );
            }
          })
          .expect("failed to register DevTools protocol callback");
        let msg = br#"{"id":1,"method":"Page.enable","params":{}}"#;
        webview
          .send_dev_tools_message(msg)
          .expect("failed to send DevTools message");
      }

      let value = Some("test".to_string());
      let response = app.sample().ping(PingRequest {
        value: value.clone(),
        on_event: Channel::new(|event| {
          println!("got channel event: {event:?}");
          Ok(())
        }),
      });
      log::info!("got response: {response:?}");
      if let Ok(res) = response {
        assert_eq!(res.value, value);
      }

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
      cmd::echo,
      cmd::spam,
    ])
    .build(tauri::tauri_build_context!())
    .expect("error while building tauri application");

  #[cfg(target_os = "macos")]
  app.set_activation_policy(tauri::ActivationPolicy::Regular);

  #[cfg(target_os = "ios")]
  let mut counter = 0;
  app.run(move |_app_handle, _event| {
    #[cfg(not(test))]
    match &_event {
      #[cfg(not(feature = "cef"))]
      RunEvent::ExitRequested { api, code, .. } if code.is_none() => {
        // Keep the event loop running even if all windows are closed
        // This allow us to catch tray icon events when there is no window
        // if we manually requested an exit (code is Some(_)) we will let it go through
        api.prevent_exit();
      }
      #[cfg(desktop)]
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
          .get_webview_window(&label)
          .unwrap()
          .destroy()
          .unwrap();
      }
      #[cfg(target_os = "ios")]
      RunEvent::SceneRequested { .. } => {
        counter += 1;
        WebviewWindowBuilder::new(
          _app_handle,
          format!("main-from-scene-{counter}"),
          WebviewUrl::default(),
        )
        .build()
        .unwrap();
      }
      RunEvent::Opened { urls } => {
        println!("opened urls: {:?}", urls);
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
