// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use tauri_runtime::{
  menu::{
    Menu, MenuEntry, MenuItem, MenuUpdate, Submenu, SystemTrayMenu, SystemTrayMenuEntry,
    SystemTrayMenuItem, TrayHandle,
  },
  Icon, SystemTrayEvent,
};
pub use wry::application::{
  event::TrayEvent,
  event_loop::EventLoopProxy,
  menu::{
    ContextMenu as WryContextMenu, CustomMenuItem as WryCustomMenuItem, MenuItem as WryMenuItem,
  },
};

#[cfg(target_os = "macos")]
pub use wry::application::platform::macos::CustomMenuItemExtMacOS;

use crate::{Error, Message, Result, TrayMessage};

use tauri_runtime::menu::MenuHash;

use uuid::Uuid;

use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

pub type SystemTrayEventHandler = Box<dyn Fn(&SystemTrayEvent) + Send>;
pub type SystemTrayEventListeners = Arc<Mutex<HashMap<Uuid, SystemTrayEventHandler>>>;
pub type SystemTrayItems = Arc<Mutex<HashMap<u16, WryCustomMenuItem>>>;

#[derive(Debug, Clone)]
pub struct SystemTrayHandle {
  pub(crate) proxy: EventLoopProxy<super::Message>,
}

impl TrayHandle for SystemTrayHandle {
  fn set_icon(&self, icon: Icon) -> Result<()> {
    self
      .proxy
      .send_event(Message::Tray(TrayMessage::UpdateIcon(icon)))
      .map_err(|_| Error::FailedToSendMessage)
  }
  fn set_menu(&self, menu: SystemTrayMenu) -> Result<()> {
    self
      .proxy
      .send_event(Message::Tray(TrayMessage::UpdateMenu(menu)))
      .map_err(|_| Error::FailedToSendMessage)
  }
  fn update_item(&self, id: u16, update: MenuUpdate) -> Result<()> {
    self
      .proxy
      .send_event(Message::Tray(TrayMessage::UpdateItem(id, update)))
      .map_err(|_| Error::FailedToSendMessage)
  }
  #[cfg(target_os = "macos")]
  fn set_icon_as_template(&self, is_template: bool) -> tauri_runtime::Result<()> {
    self
      .proxy
      .send_event(Message::Tray(TrayMessage::UpdateIconAsTemplate(
        is_template,
      )))
      .map_err(|_| Error::FailedToSendMessage)
  }
}

impl From<SystemTrayMenuItem> for crate::MenuItemWrapper {
  fn from(item: SystemTrayMenuItem) -> Self {
    match item {
      SystemTrayMenuItem::Separator => Self(WryMenuItem::Separator),
      _ => unimplemented!(),
    }
  }
}

pub fn to_wry_context_menu(
  custom_menu_items: &mut HashMap<MenuHash, WryCustomMenuItem>,
  menu: SystemTrayMenu,
) -> WryContextMenu {
  let mut tray_menu = WryContextMenu::new();
  for item in menu.items {
    match item {
      SystemTrayMenuEntry::CustomItem(c) => {
        #[allow(unused_mut)]
        let mut item = tray_menu.add_item(crate::MenuItemAttributesWrapper::from(&c).0);
        #[cfg(target_os = "macos")]
        if let Some(native_image) = c.native_image {
          item.set_native_image(crate::NativeImageWrapper::from(native_image).0);
        }
        custom_menu_items.insert(c.id, item);
      }
      SystemTrayMenuEntry::NativeItem(i) => {
        tray_menu.add_native_item(crate::MenuItemWrapper::from(i).0);
      }
      SystemTrayMenuEntry::Submenu(submenu) => {
        tray_menu.add_submenu(
          &submenu.title,
          submenu.enabled,
          to_wry_context_menu(custom_menu_items, submenu.inner),
        );
      }
    }
  }
  tray_menu
}
