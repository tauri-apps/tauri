// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::Manager;

#[tauri::command(with_window)]
fn exit(window: tauri::Window) {
  window.close().unwrap();
}

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let window = app.get_window("main").unwrap();
      window.listen("hello".into(), move |_| {
        window_
          .emit(&"reply".to_string(), Some("{ msg: 'TEST' }".to_string()))
          .unwrap();
      });
      window.eval("window.onTauriInit()").unwrap();
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![exit])
    .run(tauri::generate_context!())
    .expect("error encountered while running tauri application");
}
