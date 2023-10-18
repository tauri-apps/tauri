// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
  menu::{Menu, MenuItem},
  tray::{ClickType, TrayIconBuilder},
  Manager, Runtime, WindowBuilder, WindowUrl,
};

pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
  let toggle_i = MenuItem::with_id(app, "toggle", "Toggle", true, None);
  let new_window_i = MenuItem::with_id(app, "new-window", "New window", true, None);
  let icon_i_1 = MenuItem::with_id(app, "icon-1", "Icon 1", true, None);
  let icon_i_2 = MenuItem::with_id(app, "icon-2", "Icon 2", true, None);
  #[cfg(target_os = "macos")]
  let set_title_i = MenuItem::with_id(app, "set-title", "Set Title", true, None);
  let switch_i = MenuItem::with_id(app, "switch-menu", "Switch Menu", true, None);
  let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None);
  let remove_tray_i = MenuItem::with_id(app, "remove-tray", "Remove Tray icon", true, None);
  let menu1 = Menu::with_items(
    app,
    &[
      &toggle_i,
      &new_window_i,
      &icon_i_1,
      &icon_i_2,
      #[cfg(target_os = "macos")]
      &set_title_i,
      &switch_i,
      &quit_i,
      &remove_tray_i,
    ],
  )?;
  let menu2 = Menu::with_items(
    app,
    &[&toggle_i, &new_window_i, &switch_i, &quit_i, &remove_tray_i],
  )?;

  let is_menu1 = AtomicBool::new(true);

  let _ = TrayIconBuilder::with_id("tray-1")
    .tooltip("Tauri")
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&menu1)
    .menu_on_left_click(false)
    .on_menu_event(move |app, event| match event.id.as_ref() {
      "quit" => {
        app.exit(0);
      }
      "remove-tray" => {
        app.remove_tray_by_id("tray-1");
      }
      "toggle" => {
        if let Some(window) = app.get_window("main") {
          let new_title = if window.is_visible().unwrap_or_default() {
            let _ = window.hide();
            "Show"
          } else {
            let _ = window.show();
            let _ = window.set_focus();
            "Hide"
          };
          toggle_i.set_text(new_title).unwrap();
        }
      }
      "new-window" => {
        let _ = WindowBuilder::new(app, "new", WindowUrl::App("index.html".into()))
          .title("Tauri")
          .build();
      }
      #[cfg(target_os = "macos")]
      "set-title" => {
        if let Some(tray) = app.tray_by_id("tray-1") {
          let _ = tray.set_title(Some("Tauri"));
        }
      }
      i @ "icon-1" | i @ "icon-2" => {
        if let Some(tray) = app.tray_by_id("tray-1") {
          let _ = tray.set_icon(Some(tauri::Icon::Raw(if i == "icon-1" {
            include_bytes!("../../../.icons/icon.ico").to_vec()
          } else {
            include_bytes!("../../../.icons/tray_icon_with_transparency.png").to_vec()
          })));
        }
      }
      "switch-menu" => {
        let flag = is_menu1.load(Ordering::Relaxed);
        let (menu, tooltip) = if flag {
          (menu2.clone(), "Menu 2")
        } else {
          (menu1.clone(), "Tauri")
        };
        if let Some(tray) = app.tray_by_id("tray-1") {
          let _ = tray.set_menu(Some(menu));
          let _ = tray.set_tooltip(Some(tooltip));
        }
        is_menu1.store(!flag, Ordering::Relaxed);
      }

      _ => {}
    })
    .on_tray_icon_event(|tray, event| {
      if event.click_type == ClickType::Left {
        let app = tray.app_handle();
        if let Some(window) = app.get_window("main") {
          let _ = window.show();
          let _ = window.set_focus();
        }
      }
    })
    .build(app);

  Ok(())
}
