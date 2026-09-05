#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .runtime(tauri_runtime_wry::Wry)
    .plugin(tauri_plugin_{{ plugin_name_snake_case }}::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
