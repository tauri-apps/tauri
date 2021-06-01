// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use tauri_runtime::{
  menu::{CustomMenuItem, Menu, MenuItem, SystemTrayMenu, SystemTrayMenuItem},
  window::MenuEvent,
  MenuId, SystemTrayEvent,
};
pub use wry::application::menu::{
  ContextMenu as WryContextMenu, MenuBar, MenuId as WryMenuId, MenuItem as WryMenuItem, MenuType,
};

use uuid::Uuid;

use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

pub type MenuEventHandler = Box<dyn Fn(&MenuEvent) + Send>;
pub type MenuEventListeners = Arc<Mutex<HashMap<Uuid, MenuEventHandler>>>;
pub type SystemTrayEventHandler = Box<dyn Fn(&SystemTrayEvent) + Send>;
pub type SystemTrayEventListeners = Arc<Mutex<HashMap<Uuid, SystemTrayEventHandler>>>;

pub struct MenuItemWrapper(pub WryMenuItem);

impl<I: MenuId> From<MenuItem<I>> for MenuItemWrapper {
  fn from(item: MenuItem<I>) -> Self {
    match item {
      MenuItem::Custom(custom) => Self(WryMenuItem::Custom {
        menu_id: WryMenuId(custom.id_value()),
        text: custom.name,
        enabled: true,
        keyboard_accelerator: None,
        selected: false,
      }),
      MenuItem::About(v) => Self(WryMenuItem::About(v)),
      MenuItem::Hide => Self(WryMenuItem::Hide),
      MenuItem::Services => Self(WryMenuItem::Services),
      MenuItem::HideOthers => Self(WryMenuItem::HideOthers),
      MenuItem::ShowAll => Self(WryMenuItem::ShowAll),
      MenuItem::CloseWindow => Self(WryMenuItem::CloseWindow),
      MenuItem::Quit => Self(WryMenuItem::Quit),
      MenuItem::Copy => Self(WryMenuItem::Copy),
      MenuItem::Cut => Self(WryMenuItem::Cut),
      MenuItem::Undo => Self(WryMenuItem::Undo),
      MenuItem::Redo => Self(WryMenuItem::Redo),
      MenuItem::SelectAll => Self(WryMenuItem::SelectAll),
      MenuItem::Paste => Self(WryMenuItem::Paste),
      MenuItem::EnterFullScreen => Self(WryMenuItem::EnterFullScreen),
      MenuItem::Minimize => Self(WryMenuItem::Minimize),
      MenuItem::Zoom => Self(WryMenuItem::Zoom),
      MenuItem::Separator => Self(WryMenuItem::Separator),
      _ => unimplemented!(),
    }
  }
}

impl<I: MenuId> From<SystemTrayMenuItem<I>> for MenuItemWrapper {
  fn from(item: SystemTrayMenuItem<I>) -> Self {
    match item {
      SystemTrayMenuItem::Custom(custom) => Self(WryMenuItem::Custom {
        menu_id: WryMenuId(custom.id_value()),
        text: custom.name,
        enabled: true,
        keyboard_accelerator: None,
        selected: false,
      }),
      SystemTrayMenuItem::Separator => Self(WryMenuItem::Separator),
      _ => unimplemented!(),
    }
  }
}
