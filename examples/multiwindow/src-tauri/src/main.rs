#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::Attributes;

fn main() {
  tauri::AppBuilder::default()
    .setup(move |webview_manager| {
      if webview_manager.current_window_label() == "Main" {
        webview_manager.listen("clicked", move |_| {
          println!("got 'clicked' event on global channel");
        });
      }
      let current_webview = webview_manager.current_webview().unwrap();
      let label = webview_manager.current_window_label().to_string();
      current_webview.listen("clicked", move |_| {
        println!("got 'clicked' event on window '{}'", label)
      });
    })
    .create_window("Rust".to_string(), tauri::WindowUrl::App, |mut builder| {
      builder = builder.title("Tauri - Rust");
      Ok(builder)
    })
    .unwrap()
    .build(tauri::generate_context!())
    .run();
}
