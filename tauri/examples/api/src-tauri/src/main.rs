#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;

use serde::Serialize;

#[derive(Serialize)]
struct Reply {
  data: String,
}

#[derive(tauri::FromTauriContext)]
struct Context;

fn main() {
  tauri::AppBuilder::<tauri::flavors::Wry, Context>::new()
    .setup(|webview_manager| async move {
      let dispatcher = webview_manager.current_webview().unwrap();
      let dispatcher_ = dispatcher.clone();
      dispatcher.listen("js-event", move |msg| {
        println!("got js-event with message '{:?}'", msg);
        let reply = Reply {
          data: "something else".to_string(),
        };

        dispatcher_
          .emit("rust-event", Some(reply))
          .expect("failed to emit");
      });
    })
    .invoke_handler(|_webview_manager, arg| async move {
      use cmd::Cmd::*;
      match serde_json::from_str(&arg) {
        Err(e) => Err(e.into()),
        Ok(command) => {
          match command {
            LogOperation { event, payload } => {
              println!("{} {:?}", event, payload);
              Ok(serde_json::Value::Null)
            }
            PerformRequest {
              endpoint,
              body,
            } => {
              println!("{} {:?}", endpoint, body);
              Ok(serde_json::Value::String("message response".to_string()))
            }
          }
        }
      }
    })
    .build()
    .unwrap()
    .run();
}
