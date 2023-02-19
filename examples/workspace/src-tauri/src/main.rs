#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

fn main() {
  core_api::run();
  tauri::Builder::default()
    .run(tauri::tauri_build_context!())
    .expect("error while running tauri application");
}
