// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod cmd;
#[cfg(desktop)]
mod menu_plugin;
#[cfg(desktop)]
mod tray;

#[cfg(target_env = "ohos")]
mod ohos_log {
  pub fn init() {
    // 直接使用 hilog crate 初始化
    hilog::Builder::new()
      .set_tag("tauritest")
      .filter_level(log::LevelFilter::Trace)
      .init();
  }
}

use serde::Serialize;
#[cfg(not(target_env = "ohos"))]
use tauri::ipc::Channel;
use tauri::{
  webview::{PageLoadEvent, WebviewWindowBuilder},
  App, Emitter, Listener, Runtime, WebviewUrl,
};
#[allow(unused)]
use tauri::{Manager, RunEvent};
#[cfg(not(target_env = "ohos"))]
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
  #[cfg(target_env = "ohos")]
  std::panic::set_hook(Box::new(|info| {
    let msg = format!("PANIC: {info}\n");
    let _ = std::fs::write("/data/storage/el2/base/cache/panic.log", &msg);
    eprintln!("{msg}");
  }));

  run_app(tauri::Builder::default(), |_app| {})
}

pub fn run_app<R: Runtime, F: FnOnce(&App<R>) + Send + 'static>(
  builder: tauri::Builder<R>,
  setup: F,
) {
  #[cfg(not(target_env = "ohos"))]
  let builder = builder
    .plugin(tauri_plugin_sample::init())
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_http::init())
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_autostart::init(
      tauri_plugin_autostart::MacosLauncher::LaunchAgent,
      None,
    ));

  #[cfg(target_env = "ohos")]
  let builder = builder
    .plugin(
      tauri_plugin_log::Builder::default()
        .level(log::LevelFilter::Info)
        .clear_targets()
        .target(tauri_plugin_log::Target::new(
          tauri_plugin_log::TargetKind::Stdout,
        ))
        .build(),
    )
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_os::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_process::init());

  #[cfg(target_env = "ohos")]
  {
    // tauri_plugin_log 已初始化 log facade，不再手动调用 ohos_log::init()
    log::info!("OHOS log initialized via tauri_plugin_log");
  };

  #[allow(unused_mut)]
  let mut builder = builder
    // 1. Test custom URI scheme protocol
    .register_uri_scheme_protocol("myapp", |_ctx, request| {
      log::info!("Custom scheme request: {:?}", request.uri());

      // Return a simple response
      let body = r#"
        <!DOCTYPE html>
        <html>
        <body>
          <h1>✅ Custom Scheme Works!</h1>
          <p>Requested: <span id="url"></span></p>
          <script>document.getElementById('url').textContent = location.href;</script>
        </body>
        </html>
      "#.as_bytes().to_vec();

      tauri::http::Response::builder()
        .header("Content-Type", "text/html")
        .status(200)
        .body(body)
        .unwrap()
    })
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

      let mut window_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .initialization_script("document.addEventListener('DOMContentLoaded', () => { document.title = '✅ INIT SCRIPT WORKED!'; });")
        .on_document_title_changed(|_window, title| {
          log::info!("document title changed: {title}");
        })
        // 2. Test navigation intercept (shouldOverrideUrlLoading)
        .on_navigation(|url| {
          log::info!("Navigation intercepted: {url}");
          // Don't block navigation for our test
          true
        })
        // 3. Test web resource request intercept (onLoadIntercept)
        .on_web_resource_request(|request, response| {
          log::info!("Resource request: {:?}", request.uri());
          // Add a custom header to test
          response.headers_mut().insert("X-Tauri-Test", tauri::http::HeaderValue::from_static("intercepted"));
        })
        // 4. Test download intercept
        .on_download(|_webview, event| {
          log::info!("on_download event received");
          match event {
            tauri::webview::DownloadEvent::Requested { url, destination } => {
              log::info!("Download requested: {}", url);

              // 打印默认保存路径
              log::info!("Default destination: {:?}", destination);

              // 可以在这里修改保存路径
              // *destination = "/custom/path".into();
            }
            tauri::webview::DownloadEvent::Finished { url, path, success } => {
              log::info!("Download finished: {}, success: {}, path: {:?}", url, success, path);
            }
            _ => {
              log::info!("Other download event");
            }
          }
          true // 允许下载
        });

      #[cfg(all(desktop, not(test)))]
      {
        let app_ = app.handle().clone();
        let mut created_window_count = std::sync::atomic::AtomicUsize::new(0);

        window_builder = window_builder
          .title("Tauri API Validation")
          .inner_size(1000., 800.)
          .min_inner_size(600., 400.)
          .menu(tauri::menu::Menu::default(app.handle())?)
          .on_new_window(move |url, features| {
            log::info!("new window requested: {url:?} {features:?}");

            let number = created_window_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            let builder = WebviewWindowBuilder::new(
              &app_,
              format!("new-{number}"),
              tauri::WebviewUrl::External("about:blank".parse().unwrap()),
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

      // Test eval functionality
      log::info!("Testing eval functionality...");
      webview.eval("document.title = '✅ Rust eval works!'")?;
      webview.eval_with_callback("document.title", |title| {
        log::info!("Window title from JS: {}", title);
      })?;
      webview.eval(r#"
        const div = document.createElement('div');
        div.style.cssText = 'position:fixed;top:20px;left:20px;background:green;color:white;padding:20px;font-size:24px;z-index:9999;';
        div.textContent = '✅ Rust eval is working!';
        document.body.appendChild(div);
      "#)?;

      #[cfg(not(target_env = "ohos"))]
      {
        let value = Some("test".to_string());
        let response = app.sample().ping(PingRequest {
          value: value.clone(),
          on_event: Channel::new(|event| {
            log::info!("got channel event: {event:?}");
            Ok(())
          }),
        });
        log::info!("got response: {:?}", response);
        // when #[cfg(desktop)], Rust will detect pattern as irrefutable
        #[allow(irrefutable_let_patterns)]
        if let Ok(res) = response {
          assert_eq!(res.value, value);
        }
      }

      #[cfg(target_env = "ohos")]
      {
        log::info!("OHOS platform initialized successfully"); // No logger initialized on OHOS yet
      }

      #[cfg(desktop)]
      std::thread::spawn(|| {
        let server = match tiny_http::Server::http("localhost:3003") {
          Ok(s) => s,
          Err(e) => {
            log::error!("{e}");
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
      match payload.event() {
        PageLoadEvent::Started => {
          log::info!("Page Begin: {}", payload.url());
        }
        PageLoadEvent::Finished => {
          log::info!("Page End: {}", payload.url());
        }
      }

      if payload.event() == PageLoadEvent::Finished {
        let webview_ = webview.clone();
        webview.listen("js-event", move |event| {
          log::info!("got js-event with message '{:?}'", event.payload());
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
      cmd::write_test_report,
      cmd::console_log,
      cmd::flush_console_log,
      cmd::clear_console_log,
      cmd::test_eval,
      cmd::test_navigate,
      cmd::test_reload,
      cmd::create_isolated_window,
      cmd::dummy_command,
      cmd::create_window_with_custom_ua,
      cmd::create_window_no_throttle,
      cmd::create_transparent_window,
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
      // Keep the event loop running even if all windows are closed
      // This allow us to catch tray icon events when there is no window
      // if we manually requested an exit (code is Some(_)) we will let it go through
      #[cfg(desktop)]
      RunEvent::ExitRequested { api, code, .. } if code.is_none() => {
        api.prevent_exit();
      }
      #[cfg(desktop)]
      RunEvent::WindowEvent {
        event: tauri::WindowEvent::CloseRequested { api, .. },
        label,
        ..
      } => {
        log::info!("closing window...");
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
      #[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
      RunEvent::Opened { urls } => {
        log::info!("opened urls: {:?}", urls);
      }
      _ => (),
    }
  });
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
