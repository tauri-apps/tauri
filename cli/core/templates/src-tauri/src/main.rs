#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

fn main() {
  tauri::AppBuilder::default()
    .build(tauri::generate_context!())
    .run();
}
