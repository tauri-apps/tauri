#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[derive(tauri::FromTauriContext)]
#[config_path = "examples/multiwindow/src-tauri/tauri.conf.json"]
struct Context;

use tauri::WebviewBuilderExt;

fn main() {
  tauri::AppBuilder::<Context>::new()
    .setup(|webview_manager| async move {
      if webview_manager.current_window_label() == "Main" {
        webview_manager.listen("clicked", move |_| {
          println!("got 'clicked' event on global channel");
        });
      }
      let current_webview = webview_manager.current_webview().await.unwrap();
      let label = webview_manager.current_window_label().to_string();
      current_webview.listen("clicked", move |_| {
        println!("got 'clicked' event on window '{}'", label)
      });
    })
    .create_webview("Rust".to_string(), tauri::WindowUrl::App, |mut builder| {
      builder = builder.title("Tauri - Rust");
      Ok(builder)
    })
    .unwrap()
    .build()
    .unwrap()
    .run();
}
