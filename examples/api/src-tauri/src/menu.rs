// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};

pub fn get_menu() -> Menu<String> {
  // create a submenu
  let my_sub_menu =
    Menu::new().add_item(CustomMenuItem::new("disable-menu".into(), "Disable menu"));

  let my_app_menu = Menu::new()
    .add_native_item(MenuItem::Copy)
    .add_submenu(Submenu::new("Sub menu", my_sub_menu));

  let test_menu = Menu::new()
    .add_item(CustomMenuItem::new(
      "selected/disabled".into(),
      "Selected and disabled",
    ))
    .add_native_item(MenuItem::Separator)
    .add_item(CustomMenuItem::new("test".into(), "Test"));

  // add all our childs to the menu (order is how they'll appear)
  Menu::new()
    .add_submenu(Submenu::new("My app", my_app_menu))
    .add_submenu(Submenu::new("Other menu", test_menu))
}
