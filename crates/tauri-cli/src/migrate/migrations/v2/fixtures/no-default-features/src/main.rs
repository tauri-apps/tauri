use tauri::{Builder, Manager};

fn main() {
  let app = tauri::Builder::default().build(tauri::generate_context!()).unwrap();
  let _webview = tauri::webview_version();
  let _ = app.run_on_main_thread(|| {});
  app.run(|_, _| {});
  let _unused = Builder::<tauri::Wry>::new();
}
