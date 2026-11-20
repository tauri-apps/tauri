// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
  Manager, Runtime, WebviewUrl, include_image,
  menu::{Menu, MenuItemBuilder},
  tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
  let toggle_i = MenuItemBuilder::with_id("toggle", "Toggle")
    .handler(|item, _event| {
      if let Some(window) = item.app_handle().get_webview_window("main") {
        let new_title = if window.is_visible().unwrap_or_default() {
          let _ = window.hide();
          "Show"
        } else {
          let _ = window.show();
          let _ = window.set_focus();
          "Hide"
        };
        item.set_text(new_title).unwrap();
      }
    })
    .build(app)?;

  let new_window_i = MenuItemBuilder::with_id("new-window", "New window")
    .handler(|item, _event| {
      let _webview = tauri::WebviewWindowBuilder::new(
        item.app_handle(),
        "new",
        WebviewUrl::App("index.html".into()),
      )
      .title("Tauri")
      .build()
      .unwrap();
    })
    .build(app)?;

  let icon_i_1 = MenuItemBuilder::with_id("icon-1", "Icon 1")
    .handler(|item, _event| {
      if let Some(tray) = item.app_handle().tray_by_id("tray-1") {
        let _ = tray.set_icon(Some(include_image!("../../.icons/icon.ico")));
      }
    })
    .build(app)?;

  let icon_i_2 = MenuItemBuilder::with_id("icon-2", "Icon 2")
    .handler(|item, _event| {
      if let Some(tray) = item.app_handle().tray_by_id("tray-1") {
        let _ = tray.set_icon(Some(include_image!(
          "../../.icons/tray_icon_with_transparency.png"
        )));
      }
    })
    .build(app)?;

  #[cfg(target_os = "macos")]
  let set_title_i = MenuItemBuilder::with_id("set-title", "Set Title")
    .handler(|item, _event| {
      if let Some(tray) = item.app_handle().tray_by_id("tray-1") {
        let _ = tray.set_title(Some("Tauri"));
      }
    })
    .build(app)?;

  let menu1 = Menu::new(app)?;
  let menu2 = Menu::new(app)?;

  let switch_i = MenuItemBuilder::with_id("switch-menu", "Switch Menu")
    .handler({
      let menu1 = menu1.clone();
      let menu2 = menu2.clone();
      let is_menu1 = AtomicBool::new(true);

      move |item, _event| {
        let flag = is_menu1.load(Ordering::Relaxed);
        let (menu, tooltip) = if flag {
          (menu2.clone(), "Menu 2")
        } else {
          (menu1.clone(), "Tauri")
        };
        if let Some(tray) = item.app_handle().tray_by_id("tray-1") {
          let _ = tray.set_menu(Some(menu));
          let _ = tray.set_tooltip(Some(tooltip));
        }
        is_menu1.store(!flag, Ordering::Relaxed);
      }
    })
    .build(app)?;

  let quit_i = MenuItemBuilder::with_id("quit", "Quit")
    .handler(|item, _event| item.app_handle().exit(0))
    .build(app)?;

  let remove_tray_i = MenuItemBuilder::with_id("remove-tray", "Remove Tray icon")
    .handler(|item, _event| {
      item.app_handle().remove_tray_by_id("tray-1");
    })
    .build(app)?;

  menu1.append_items(&[
    &toggle_i,
    &new_window_i,
    &icon_i_1,
    &icon_i_2,
    #[cfg(target_os = "macos")]
    &set_title_i,
    &switch_i,
    &quit_i,
    &remove_tray_i,
  ])?;
  menu2.append_items(&[&toggle_i, &new_window_i, &switch_i, &quit_i, &remove_tray_i])?;

  let _tray = TrayIconBuilder::with_id("tray-1")
    .tooltip("Tauri")
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&menu1)
    .show_menu_on_left_click(false)
    .on_tray_icon_event(|tray, event| {
      if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      } = event
      {
        let app = tray.app_handle();
        if let Some(window) = app.get_webview_window("main") {
          let _ = window.unminimize();
          let _ = window.show();
          let _ = window.set_focus();
        }
      }
    })
    .build(app)?;

  Ok(())
}
