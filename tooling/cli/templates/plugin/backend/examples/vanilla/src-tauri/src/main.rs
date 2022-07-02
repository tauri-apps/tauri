#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

fn main() {
  let context = tauri::generate_context!();
  tauri::Builder::default()
    .plugin(tauri_plugin_{{ plugin_name_snake_case }}::init())
    .run(context)
    .expect("error while running tauri application");
}
