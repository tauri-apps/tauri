// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(all(desktop, not(test)))]

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
  include_image,
  menu::{Menu, MenuItem},
  tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
  Manager, Runtime, WebviewUrl,
};

pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
  let toggle_i = MenuItem::with_id(app, "toggle", "Toggle", true, None::<&str>)?;
  let new_window_i = MenuItem::with_id(app, "new-window", "New window", true, None::<&str>)?;
  let icon_i_1 = MenuItem::with_id(app, "icon-1", "Icon 1", true, None::<&str>)?;
  let icon_i_2 = MenuItem::with_id(app, "icon-2", "Icon 2", true, None::<&str>)?;
  #[cfg(target_os = "macos")]
  let set_title_i = MenuItem::with_id(app, "set-title", "Set Title", true, None::<&str>)?;
  let switch_i = MenuItem::with_id(app, "switch-menu", "Switch Menu", true, None::<&str>)?;
  let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
  let remove_tray_i =
    MenuItem::with_id(app, "remove-tray", "Remove Tray icon", true, None::<&str>)?;
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
    .show_menu_on_left_click(false)
    .on_menu_event(move |app, event| match event.id.as_ref() {
      "quit" => {
        app.exit(0);
      }
      "remove-tray" => {
        app.remove_tray_by_id("tray-1");
      }
      "toggle" => {
        if let Some(window) = app.get_webview_window("main") {
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
        let _webview =
          tauri::WebviewWindowBuilder::new(app, "new", WebviewUrl::App("index.html".into()))
            .title("Tauri")
            .build()
            .unwrap();
      }
      #[cfg(target_os = "macos")]
      "set-title" => {
        if let Some(tray) = app.tray_by_id("tray-1") {
          let _ = tray.set_title(Some("Tauri"));
        }
      }
      i @ "icon-1" | i @ "icon-2" => {
        if let Some(tray) = app.tray_by_id("tray-1") {
          let icon = if i == "icon-1" {
            include_image!("../../.icons/icon.ico")
          } else {
            include_image!("../../.icons/tray_icon_with_transparency.png")
          };
          let _ = tray.set_icon(Some(icon));
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
      if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      } = event
      {
        let app = tray.app_handle();
        if let Some(window) = app.get_webview_window("main") {
          let _ = window.show();
          let _ = window.set_focus();
        }
      }
    })
    .build(app);

  Ok(())
}
