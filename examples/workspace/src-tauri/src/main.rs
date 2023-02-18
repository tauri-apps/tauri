#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  core_api::run();
  tauri::Builder::default()
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
