// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod cmd;
#[cfg(all(desktop, not(test)))]
mod menu_plugin;
#[cfg(all(desktop, not(test)))]
mod tray;

use serde::Serialize;
use tauri::{
  App, Emitter, Listener, WebviewUrl,
  ipc::Channel,
  webview::{PageLoadEvent, WebviewWindowBuilder},
};
#[allow(unused)]
use tauri::{Manager, RunEvent};
use tauri_plugin_sample::{PingRequest, SampleExt};

#[cfg(test)]
type TauriRuntime = tauri::test::MockRuntime;
#[cfg(not(test))]
type TauriRuntime = tauri::DynRuntime;

#[derive(Clone, Serialize)]
struct Reply {
  data: String,
}

#[cfg(target_os = "macos")]
pub struct AppMenu<R: tauri::Runtime>(pub std::sync::Mutex<Option<tauri::menu::Menu<R>>>);

#[cfg(all(desktop, not(test)))]
pub struct PopupMenu<R: tauri::Runtime>(#[allow(dead_code)] tauri::menu::Menu<R>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[cfg_attr(feature = "cef", tauri_runtime_cef::cef_entry_point)]
pub fn run() {
  #[cfg(test)]
  let builder = tauri::test::mock_builder();
  #[cfg(all(not(test), feature = "cef"))]
  let builder = tauri::Builder::default().runtime(tauri_runtime_cef::Cef::default());
  #[cfg(all(not(test), not(feature = "cef")))]
  let builder = tauri::Builder::default().runtime(tauri_runtime_wry::Wry::default());

  run_app(builder, |_app| {});
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
      #[cfg(all(desktop, not(test)))]
      {
        let handle = app.handle();
        tray::create_tray(handle)?;
      }

      #[cfg(all(desktop, not(test)))]
      {
        let handle = app.handle();
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

            // CEF reports the opener's main-frame URL directly from the native
            // popup request, so it can be read without a blocking webview getter.
            #[cfg(feature = "cef")]
            {
              use tauri_runtime_cef::AsCefWindowOpener;
              if let Some(opener) = features.opener().as_cef_window_opener() {
                println!("CEF popup opener source: {:?}", opener.source_url());
              }
            }

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

      #[cfg(all(feature = "cef", not(test)))]
      {
        use tauri_runtime_cef::{FrameEventKind, WebviewWindowBuilderCefExt};

        // Native CEF lifecycle notifications for every frame, child frames included.
        // The handler runs synchronously on CEF's UI thread, so it must return
        // promptly and must not wait on an event loop operation.
        window_builder = window_builder.on_frame_event(|event| match &event.kind {
          FrameEventKind::LoadingStateChanged { is_loading } => println!(
            "CEF browser {} is {}",
            event.browser_id,
            if *is_loading { "loading" } else { "idle" }
          ),
          kind => println!(
            "CEF frame event: browser={} frame={} main={} {kind:?}",
            event.browser_id, event.frame_id, event.is_main
          ),
        });
      }

      let webview = window_builder.build()?;

      #[cfg(debug_assertions)]
      webview.open_devtools();

      #[cfg(all(feature = "cef", not(test)))]
      {
        use tauri_runtime_cef::{DevToolsProtocol, WebviewCefExt};
        // The observer sees the whole browser, the runtime's own requests included,
        // so a real consumer matches `MethodResult` against the IDs it allocated.
        webview
          .on_dev_tools_protocol(|protocol| match protocol {
            DevToolsProtocol::Message(msg) => {
              if let Ok(s) = std::str::from_utf8(&msg) {
                log::info!("DevTools message: {s}");
              } else {
                log::error!("Failed to convert DevTools message to UTF-8");
              }
            }
            DevToolsProtocol::Event { method, params } => {
              log::info!(
                "DevTools event: {method} (params: {})",
                String::from_utf8_lossy(&params)
              );
            }
            DevToolsProtocol::MethodResult {
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
        // The runtime shares the native DevTools request ID space with its callers,
        // so IDs must come from the allocator instead of being hardcoded.
        let message_id = tauri_runtime_cef::allocate_devtools_message_id()
          .expect("native DevTools message identifiers are exhausted");
        let msg = format!(r#"{{"id":{message_id},"method":"Page.enable","params":{{}}}}"#);
        webview
          .send_dev_tools_message(msg.as_bytes())
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
        // Native CEF state, sampled on the CEF UI thread right before the closure
        // runs. The observation is not refreshed after that sample.
        #[cfg(all(feature = "cef", not(test)))]
        {
          use tauri_runtime_cef::WebviewCefExt;

          let _ = webview.with_cef_webview(|cef_webview| {
            let snapshot = cef_webview.snapshot();
            println!(
              "CEF native snapshot: browser={} window={:?} document_admitted={} parent_matches={:?} visible={:?} bounds={:?} dialogs={:?}",
              snapshot.browser_id,
              snapshot.window_label,
              snapshot.document.is_some(),
              snapshot.parent_matches,
              snapshot.visible,
              snapshot.bounds,
              snapshot.dialogs,
            );

            // CEF-owned popups have no Tauri window label and keep their actual opener.
            for popup in cef_webview.popups() {
              println!(
                "  CEF-owned popup: browser={} opened_by_this_browser={}",
                popup.snapshot().browser_id,
                popup.opener().is_some_and(|opener| {
                  opener.is_same_browser(cef_webview.frame_navigation_state())
                }),
              );
            }
          });
        }

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

  #[cfg(all(target_os = "ios", not(test)))]
  let mut counter = 0;
  app.run(move |_app_handle, _event| {
    #[cfg(not(test))]
    match &_event {
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
          .get_webview_window(label)
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
