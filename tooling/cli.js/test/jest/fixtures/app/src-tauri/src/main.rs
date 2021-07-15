#[tauri::command(with_window)]
fn exit(window: tauri::Window) {
  window.close().unwrap();
}

fn main() {
  tauri::Builder::default()
    .on_page_load(|window, _| {
      let window_ = window.clone();
      window.listen("hello".into(), move |_| {
        window_
          .emit(&"reply".to_string(), Some("{ msg: 'TEST' }".to_string()))
          .unwrap();
      });
      window.eval("window.onTauriInit()").unwrap();
    })
    .invoke_handler(tauri::generate_handler![exit])
    .run(tauri::generate_context!())
    .expect("error encountered while running tauri application");
}
