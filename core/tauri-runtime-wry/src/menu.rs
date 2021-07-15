// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use tauri_runtime::{
  menu::{
    CustomMenuItem, Menu, MenuEntry, MenuItem, MenuUpdate, Submenu, SystemTrayMenu,
    SystemTrayMenuEntry, SystemTrayMenuItem, TrayHandle,
  },
  window::MenuEvent,
  Icon, SystemTrayEvent,
};
pub use wry::application::{
  event::TrayEvent,
  event_loop::EventLoopProxy,
  menu::{
    ContextMenu as WryContextMenu, CustomMenuItem as WryCustomMenuItem, MenuBar,
    MenuId as WryMenuId, MenuItem as WryMenuItem, MenuItemAttributes as WryMenuItemAttributes,
    MenuType,
  },
  window::WindowId,
};

#[cfg(target_os = "macos")]
use tauri_runtime::menu::NativeImage;
#[cfg(target_os = "macos")]
pub use wry::application::platform::macos::{
  CustomMenuItemExtMacOS, NativeImage as WryNativeImage,
};

#[cfg(feature = "system-tray")]
use crate::{Error, Message, Result, TrayMessage};

#[cfg(feature = "menu")]
use tauri_runtime::menu::MenuHash;

use uuid::Uuid;

use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

#[cfg(feature = "menu")]
pub type MenuEventHandler = Box<dyn Fn(&MenuEvent) + Send>;
#[cfg(feature = "menu")]
pub type MenuEventListeners = Arc<Mutex<HashMap<WindowId, WindowMenuEventListeners>>>;
#[cfg(feature = "menu")]
pub type WindowMenuEventListeners = Arc<Mutex<HashMap<Uuid, MenuEventHandler>>>;

#[cfg(feature = "system-tray")]
pub type SystemTrayEventHandler = Box<dyn Fn(&SystemTrayEvent) + Send>;
#[cfg(feature = "system-tray")]
pub type SystemTrayEventListeners = Arc<Mutex<HashMap<Uuid, SystemTrayEventHandler>>>;
#[cfg(feature = "system-tray")]
pub type SystemTrayItems = Arc<Mutex<HashMap<u16, WryCustomMenuItem>>>;

#[cfg(feature = "system-tray")]
#[derive(Clone)]
pub struct SystemTrayHandle {
  pub(crate) proxy: EventLoopProxy<super::Message>,
}

#[cfg(feature = "system-tray")]
impl TrayHandle for SystemTrayHandle {
  fn set_icon(&self, icon: Icon) -> Result<()> {
    self
      .proxy
      .send_event(Message::Tray(TrayMessage::UpdateIcon(icon)))
      .map_err(|_| Error::FailedToSendMessage)
  }
  fn update_item(&self, id: u16, update: MenuUpdate) -> Result<()> {
    self
      .proxy
      .send_event(Message::Tray(TrayMessage::UpdateItem(id, update)))
      .map_err(|_| Error::FailedToSendMessage)
  }
}

#[cfg(target_os = "macos")]
pub struct NativeImageWrapper(pub WryNativeImage);

#[cfg(target_os = "macos")]
impl From<NativeImage> for NativeImageWrapper {
  fn from(image: NativeImage) -> NativeImageWrapper {
    let wry_image = match image {
      NativeImage::Add => WryNativeImage::Add,
      NativeImage::Advanced => WryNativeImage::Advanced,
      NativeImage::Bluetooth => WryNativeImage::Bluetooth,
      NativeImage::Bookmarks => WryNativeImage::Bookmarks,
      NativeImage::Caution => WryNativeImage::Caution,
      NativeImage::ColorPanel => WryNativeImage::ColorPanel,
      NativeImage::ColumnView => WryNativeImage::ColumnView,
      NativeImage::Computer => WryNativeImage::Computer,
      NativeImage::EnterFullScreen => WryNativeImage::EnterFullScreen,
      NativeImage::Everyone => WryNativeImage::Everyone,
      NativeImage::ExitFullScreen => WryNativeImage::ExitFullScreen,
      NativeImage::FlowView => WryNativeImage::FlowView,
      NativeImage::Folder => WryNativeImage::Folder,
      NativeImage::FolderBurnable => WryNativeImage::FolderBurnable,
      NativeImage::FolderSmart => WryNativeImage::FolderSmart,
      NativeImage::FollowLinkFreestanding => WryNativeImage::FollowLinkFreestanding,
      NativeImage::FontPanel => WryNativeImage::FontPanel,
      NativeImage::GoLeft => WryNativeImage::GoLeft,
      NativeImage::GoRight => WryNativeImage::GoRight,
      NativeImage::Home => WryNativeImage::Home,
      NativeImage::IChatTheater => WryNativeImage::IChatTheater,
      NativeImage::IconView => WryNativeImage::IconView,
      NativeImage::Info => WryNativeImage::Info,
      NativeImage::InvalidDataFreestanding => WryNativeImage::InvalidDataFreestanding,
      NativeImage::LeftFacingTriangle => WryNativeImage::LeftFacingTriangle,
      NativeImage::ListView => WryNativeImage::ListView,
      NativeImage::LockLocked => WryNativeImage::LockLocked,
      NativeImage::LockUnlocked => WryNativeImage::LockUnlocked,
      NativeImage::MenuMixedState => WryNativeImage::MenuMixedState,
      NativeImage::MenuOnState => WryNativeImage::MenuOnState,
      NativeImage::MobileMe => WryNativeImage::MobileMe,
      NativeImage::MultipleDocuments => WryNativeImage::MultipleDocuments,
      NativeImage::Network => WryNativeImage::Network,
      NativeImage::Path => WryNativeImage::Path,
      NativeImage::PreferencesGeneral => WryNativeImage::PreferencesGeneral,
      NativeImage::QuickLook => WryNativeImage::QuickLook,
      NativeImage::RefreshFreestanding => WryNativeImage::RefreshFreestanding,
      NativeImage::Refresh => WryNativeImage::Refresh,
      NativeImage::Remove => WryNativeImage::Remove,
      NativeImage::RevealFreestanding => WryNativeImage::RevealFreestanding,
      NativeImage::RightFacingTriangle => WryNativeImage::RightFacingTriangle,
      NativeImage::Share => WryNativeImage::Share,
      NativeImage::Slideshow => WryNativeImage::Slideshow,
      NativeImage::SmartBadge => WryNativeImage::SmartBadge,
      NativeImage::StatusAvailable => WryNativeImage::StatusAvailable,
      NativeImage::StatusNone => WryNativeImage::StatusNone,
      NativeImage::StatusPartiallyAvailable => WryNativeImage::StatusPartiallyAvailable,
      NativeImage::StatusUnavailable => WryNativeImage::StatusUnavailable,
      NativeImage::StopProgressFreestanding => WryNativeImage::StopProgressFreestanding,
      NativeImage::StopProgress => WryNativeImage::StopProgress,

      NativeImage::TrashEmpty => WryNativeImage::TrashEmpty,
      NativeImage::TrashFull => WryNativeImage::TrashFull,
      NativeImage::User => WryNativeImage::User,
      NativeImage::UserAccounts => WryNativeImage::UserAccounts,
      NativeImage::UserGroup => WryNativeImage::UserGroup,
      NativeImage::UserGuest => WryNativeImage::UserGuest,
    };
    Self(wry_image)
  }
}

pub struct MenuItemAttributesWrapper<'a>(pub WryMenuItemAttributes<'a>);

impl<'a> From<&'a CustomMenuItem> for MenuItemAttributesWrapper<'a> {
  fn from(item: &'a CustomMenuItem) -> Self {
    let mut attributes = WryMenuItemAttributes::new(&item.title)
      .with_enabled(item.enabled)
      .with_selected(item.selected)
      .with_id(WryMenuId(item.id));
    if let Some(accelerator) = item.keyboard_accelerator.as_ref() {
      attributes = attributes.with_accelerators(&accelerator.parse().expect("invalid accelerator"));
    }
    Self(attributes)
  }
}

pub struct MenuItemWrapper(pub WryMenuItem);

impl From<MenuItem> for MenuItemWrapper {
  fn from(item: MenuItem) -> Self {
    match item {
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

impl From<SystemTrayMenuItem> for MenuItemWrapper {
  fn from(item: SystemTrayMenuItem) -> Self {
    match item {
      SystemTrayMenuItem::Separator => Self(WryMenuItem::Separator),
      _ => unimplemented!(),
    }
  }
}

#[cfg(feature = "menu")]
pub fn convert_menu_id(mut new_menu: Menu, menu: Menu) -> Menu {
  for item in menu.items {
    match item {
      MenuEntry::CustomItem(c) => {
        let mut item = CustomMenuItem::new(c.id_str, c.title);
        #[cfg(target_os = "macos")]
        if let Some(native_image) = c.native_image {
          item = item.native_image(native_image);
        }
        if let Some(accelerator) = c.keyboard_accelerator {
          item = item.accelerator(accelerator);
        }
        if !c.enabled {
          item = item.disabled();
        }
        if c.selected {
          item = item.selected();
        }
        new_menu = new_menu.add_item(item);
      }
      MenuEntry::NativeItem(i) => {
        new_menu = new_menu.add_native_item(i);
      }
      MenuEntry::Submenu(submenu) => {
        let new_submenu = convert_menu_id(Menu::new(), submenu.inner);
        new_menu = new_menu.add_submenu(Submenu::new(submenu.title, new_submenu));
      }
    }
  }
  new_menu
}

#[cfg(feature = "menu")]
pub fn to_wry_menu(
  custom_menu_items: &mut HashMap<MenuHash, WryCustomMenuItem>,
  menu: Menu,
) -> MenuBar {
  let mut wry_menu = MenuBar::new();
  for item in menu.items {
    match item {
      MenuEntry::CustomItem(c) => {
        let mut attributes = MenuItemAttributesWrapper::from(&c).0;
        attributes = attributes.with_id(WryMenuId(c.id));
        #[allow(unused_mut)]
        let mut item = wry_menu.add_item(attributes);
        #[cfg(target_os = "macos")]
        if let Some(native_image) = c.native_image {
          item.set_native_image(NativeImageWrapper::from(native_image).0);
        }
        custom_menu_items.insert(c.id, item);
      }
      MenuEntry::NativeItem(i) => {
        wry_menu.add_native_item(MenuItemWrapper::from(i).0);
      }
      MenuEntry::Submenu(submenu) => {
        wry_menu.add_submenu(
          &submenu.title,
          submenu.enabled,
          to_wry_menu(custom_menu_items, submenu.inner),
        );
      }
    }
  }
  wry_menu
}

#[cfg(feature = "system-tray")]
pub fn to_wry_context_menu(
  custom_menu_items: &mut HashMap<MenuHash, WryCustomMenuItem>,
  menu: SystemTrayMenu,
) -> WryContextMenu {
  let mut tray_menu = WryContextMenu::new();
  for item in menu.items {
    match item {
      SystemTrayMenuEntry::CustomItem(c) => {
        #[allow(unused_mut)]
        let mut item = tray_menu.add_item(MenuItemAttributesWrapper::from(&c).0);
        #[cfg(target_os = "macos")]
        if let Some(native_image) = c.native_image {
          item.set_native_image(NativeImageWrapper::from(native_image).0);
        }
        custom_menu_items.insert(c.id, item);
      }
      SystemTrayMenuEntry::NativeItem(i) => {
        tray_menu.add_native_item(MenuItemWrapper::from(i).0);
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
