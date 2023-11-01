#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_{{ plugin_name_snake_case }}::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
