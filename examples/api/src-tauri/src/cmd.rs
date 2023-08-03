// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use tauri::{command, Runtime};

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct RequestBody {
  id: i32,
  name: String,
}

#[command]
pub fn log_operation(event: String, payload: Option<String>) {
  log::info!("{} {:?}", event, payload);
}

#[command]
pub fn perform_request(endpoint: String, body: RequestBody) -> String {
  println!("{} {:?}", endpoint, body);
  "message response".into()
}

#[cfg(not(target_os = "macos"))]
#[command]
pub fn toggle_menu<R: Runtime>(window: tauri::Window<R>) {
  if window.is_menu_visible().unwrap_or_default() {
    let _ = window.hide_menu();
  } else {
    let _ = window.show_menu();
  }
}

#[cfg(target_os = "macos")]
#[command]
pub fn toggle_menu<R: Runtime>(
  app: tauri::AppHandle<R>,
  app_menu: tauri::State<'_, crate::AppMenu<R>>,
) {
  if let Some(menu) = app.remove_menu().unwrap() {
    app_menu.0.lock().unwrap().replace(menu);
  } else {
    app
      .set_menu(app_menu.0.lock().unwrap().clone().expect("no app menu"))
      .unwrap();
  }
}
