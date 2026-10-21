mod plugin;
mod tray;

use tauri::{AppHandle, Manager, Wry};

#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello, {name}! You've been greeted from Rust!")
}

fn main_window(app: &AppHandle<Wry>) -> tauri::WebviewWindow<Wry> {
  app.get_webview_window("main").unwrap()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(plugin::init())
    .setup(|app| {
      tray::create(app.handle())?;
      let window = main_window(app.handle());
      log::info!("{} webview {:?}", window.label(), tauri::webview_version());
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![greet])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
