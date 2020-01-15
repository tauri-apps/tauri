#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::io::BufRead;

fn main() {
  tauri::AppBuilder::new()
    .setup(|_webview| {
      let handle1 = _webview.handle();
      std::thread::spawn(move || {
        let stdout = tauri::api::command::spawn_relative_command(
          tauri::api::command::binary_command("packaged-node".to_string()).expect("failed to get binary command"),
          Vec::new(),
          std::process::Stdio::piped(),
        )
          .expect("Failed to spawn packaged node")
          .stdout.expect("Failed to get packaged node stdout");
        let reader = std::io::BufReader::new(stdout);

        reader
          .lines()
          .filter_map(|line| line.ok())
          .for_each(|line| {
            tauri::event::emit(&handle1, String::from("node"), format!("'{}'", line))
          });
      });

      let handle2 = _webview.handle();
      tauri::event::listen(String::from("hello"), move |msg| {
        #[derive(Serialize)]
        pub struct Reply {
          pub msg: String,
          pub rep: String,
        }

        let reply = Reply {
          msg: format!("{}", msg).to_string(),
          rep: "something else".to_string(),
        };

        tauri::event::emit(
          &handle2,
          String::from("reply"),
          serde_json::to_string(&reply).unwrap(),
        );

        println!("Message from emit:hello => {}", msg);
      });
    })
    .build()
    .run();
}
