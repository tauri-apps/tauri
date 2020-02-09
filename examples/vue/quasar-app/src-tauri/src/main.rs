#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;

use serde::Serialize;

use std::io::BufRead;

fn main() {
  tauri::AppBuilder::new()
    .setup(|webview, _| {
      let handle1 = webview.handle();
      std::thread::spawn(move || {
        let resource_dir =
          tauri::api::platform::resource_dir().expect("failed to get resource dir");
        let node_package_path = resource_dir.join("resources/packaged-node.js");
        let stdout = std::process::Command::new("node")
          .args(vec![node_package_path])
          .stdout(std::process::Stdio::piped())
          .spawn()
          .expect("Failed to spawn packaged node")
          .stdout
          .expect("Failed to get packaged node stdout");

        let reader = std::io::BufReader::new(stdout);

        reader
          .lines()
          .filter_map(|line| line.ok())
          .for_each(|line| {
            tauri::event::emit(&handle1, String::from("node"), format!("'{}'", line))
          });
      });

      let handle2 = webview.handle();
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
