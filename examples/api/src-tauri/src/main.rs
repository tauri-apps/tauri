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

fn main() {
  tauri::AppBuilder::default()
    .setup(|webview_manager| async move {
      let dispatcher = webview_manager.current_webview().unwrap();
      let dispatcher_ = dispatcher.clone();
      dispatcher.listen("js-event", move |event| {
        println!("got js-event with message '{:?}'", event.payload());
        let reply = Reply {
          data: "something else".to_string(),
        };

        dispatcher_
          .emit("rust-event", Some(reply))
          .expect("failed to emit");
      });
    })
    .invoke_handler(tauri::generate_handler![
      cmd::log_operation,
      cmd::perform_request
    ])
    .build(tauri::generate_context!())
    .run();
}
