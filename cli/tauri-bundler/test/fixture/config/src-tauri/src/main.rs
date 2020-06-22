#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

fn main() {
  tauri::AppBuilder::new()
    .invoke_handler(|_webview, arg| Ok(()))
    .build()
    .run();
}
