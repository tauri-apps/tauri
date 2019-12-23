#![windows_subsystem = "windows"]
mod cmd;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

fn main() {
  tauri::AppBuilder::new()
    .invoke_handler(|_webview, arg| {
      use cmd::Cmd::*;
      let command = serde_json::from_str(arg).unwrap();
      match command {
        // definitions for your custom commands from Cmd here
        MyCustomCommand { argument } => {
          //  your command code
          println!("{}", argument);
        }
      }
    })
    .build()
    .run();
}
