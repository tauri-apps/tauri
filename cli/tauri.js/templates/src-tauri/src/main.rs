#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

fn main() {
  let context = tauri::generate_tauri_context!();

  tauri::AppBuilder::new().build(context).run();
}
