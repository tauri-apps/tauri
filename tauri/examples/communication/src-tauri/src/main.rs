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
  let context = tauri::tauri_build_context!();

  tauri::AppBuilder::<tauri::flavors::Wry>::new()
    .setup(|webview_manager| async move {
      let current_webview = webview_manager.current_webview().unwrap().clone();
      let current_webview_ = current_webview.clone();
      current_webview.listen(String::from("js-event"), move |msg| {
        println!("got js-event with message '{:?}'", msg);
        let reply = Reply {
          data: "something else".to_string(),
        };

        current_webview_
          .emit(String::from("rust-event"), Some(reply))
          .expect("failed to emit");
      });
    })
    .invoke_handler(|webview_manager, arg: String| async move {
      use cmd::Cmd::*;
      match serde_json::from_str(&arg) {
        Err(e) => Err(e.into()),
        Ok(command) => {
          match command {
            LogOperation { event, payload } => {
              println!("{} {:?}", event, payload);
            }
            PerformRequest {
              endpoint,
              body,
              callback,
              error,
            } => {
              // tauri::execute_promise is a helper for APIs that uses the tauri.promisified JS function
              // so you can easily communicate between JS and Rust with promises
              tauri::execute_promise(
                &webview_manager,
                async move {
                  println!("{} {:?}", endpoint, body);
                  // perform an async operation here
                  // if the returned value is Ok, the promise will be resolved with its value
                  // if the returned value is Err, the promise will be rejected with its value
                  // the value is a string that will be eval'd
                  Ok("{ key: 'response', value: [{ id: 3 }] }".to_string())
                },
                callback,
                error,
              )
              .await
            }
          }
          Ok(())
        }
      }
    })
    .build(context)
    .run();
}
