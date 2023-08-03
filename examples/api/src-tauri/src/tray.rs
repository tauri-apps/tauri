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
  let toggle_i = MenuItem::new(app, "Toggle", true, None);
  let new_window_i = MenuItem::new(app, "New window", true, None);
  let icon_i_1 = MenuItem::new(app, "Icon 1", true, None);
  let icon_i_2 = MenuItem::new(app, "Icon 2", true, None);
  #[cfg(target_os = "macos")]
  let set_title_i = MenuItem::new(app, "Set Title", true, None);
  let switch_i = MenuItem::new(app, "Switch Menu", true, None);
  let quit_i = MenuItem::new(app, "Quit", true, None);
  let remove_tray_i = MenuItem::new(app, "Remove Tray icon", true, None);
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

  const TRAY_ID: u32 = 21937;

  let _ = TrayIconBuilder::with_id(TRAY_ID)
    .tooltip("Tauri")
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&menu1)
    .on_menu_event(move |app, event| match event.id {
      i if i == quit_i.id() => {
        app.exit(0);
      }
      i if i == remove_tray_i.id() => {
        app.remove_tray_by_id(TRAY_ID);
      }
      i if i == toggle_i.id() => {
        if let Some(window) = app.get_window("main") {
          let new_title = if window.is_visible().unwrap_or_default() {
            let _ = window.hide();
            "Show"
          } else {
            let _ = window.show();
            "Hide"
          };
          toggle_i.set_text(new_title).unwrap();
        }
      }
      i if i == new_window_i.id() => {
        let _ = WindowBuilder::new(app, "new", WindowUrl::App("index.html".into()))
          .title("Tauri")
          .build();
      }
      #[cfg(target_os = "macos")]
      i if i == set_title_i.id() => {
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
          let _ = tray.set_title(Some("Tauri"));
        }
      }
      i if i == icon_i_1.id() || i == icon_i_2.id() => {
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
          let _ = tray.set_icon(Some(tauri::Icon::Raw(if i == icon_i_1.id() {
            include_bytes!("../../../.icons/icon.ico").to_vec()
          } else {
            include_bytes!("../../../.icons/tray_icon_with_transparency.png").to_vec()
          })));
        }
      }
      i if i == switch_i.id() => {
        let flag = is_menu1.load(Ordering::Relaxed);
        let (menu, tooltip) = if flag {
          (menu2.clone(), "Menu 2")
        } else {
          (menu1.clone(), "Tauri")
        };
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
          let _ = tray.set_menu(Some(menu));
          let _ = tray.set_tooltip(Some(tooltip));
        }
        is_menu1.store(!flag, Ordering::Relaxed);
      }

      _ => {}
    })
    .on_tray_event(|tray, event| {
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
