#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;

fn main() {
  let context = tauri::tauri_build_context!();

  tauri::AppBuilder::<tauri::flavors::Wry>::new()
    .invoke_handler(|_webview, arg| async move {
      use cmd::Cmd::*;
      match serde_json::from_str(&arg) {
        Err(e) => Err(e.into()),
        Ok(command) => {
          match command {
            // definitions for your custom commands from Cmd here
            MyCustomCommand { argument } => {
              //  your command code
              println!("{}", argument);
            }
          }
          Ok(().into())
        }
      }
    })
    .build(context)
    .unwrap()
    .run();
}
