// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{command, window, AppHandle, Manager, WindowUrl};

#[command]
pub fn create_child_window(id: String, app: AppHandle) {
  let main = app.get_window("main").unwrap();

  let child = window::WindowBuilder::new(&app, id, WindowUrl::default())
    .title("Child")
    .inner_size(400.0, 300.0);

  #[cfg(target_os = "macos")]
  let child = child.parent_window(main.ns_window().unwrap());
  #[cfg(target_os = "windows")]
  let child = child.parent_window(main.hwnd().unwrap());

  child.build().unwrap();
}
