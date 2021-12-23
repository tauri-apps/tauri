// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{CustomMenuItem, Menu, MenuItem};

pub fn get_menu() -> Menu {
  #[allow(unused_mut)]
  let mut disable_item =
    CustomMenuItem::new("disable-menu", "Disable menu").accelerator("CmdOrControl+D");
  #[allow(unused_mut)]
  let mut test_item = CustomMenuItem::new("test", "Test").accelerator("CmdOrControl+T");
  #[cfg(target_os = "macos")]
  {
    disable_item = disable_item.native_image(tauri::NativeImage::MenuOnState);
    test_item = test_item.native_image(tauri::NativeImage::Add);
  }

  // create a submenu
  let my_app_menu = [
    MenuItem::Copy,
    MenuItem::new_submenu("Sub menu", [disable_item.into()]),
  ];

  let test_menu = [
    CustomMenuItem::new("selected/disabled", "Selected and disabled").into(),
    MenuItem::Separator,
    test_item.into(),
  ];

  // add all our childs to the menu (order is how they'll appear)
  Menu::with_items([
    MenuItem::new_submenu("My app", my_app_menu),
    MenuItem::new_submenu("Other menu", test_menu),
  ])
}
