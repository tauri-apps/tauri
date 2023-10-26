// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[tauri::command(with_window)]
fn exit(window: tauri::Window) {
  window.close().unwrap();
}

fn main() {
  tauri::Builder::default()
    .on_page_load(|window, payload| {
      if payload.event() == tauri::window::PageLoadEvent::Finished {
        let window_ = window.clone();
        window.listen("hello".into(), move |_| {
          window_
            .emit(&"reply".to_string(), Some("{ msg: 'TEST' }".to_string()))
            .unwrap();
        });
        window.eval("window.onTauriInit()").unwrap();
      }
    })
    .invoke_handler(tauri::generate_handler![exit])
    .run(tauri::generate_context!())
    .expect("error encountered while running tauri application");
}
