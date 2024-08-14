// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use tauri_runtime::{
  menu::{MenuUpdate, SystemTrayMenu, SystemTrayMenuEntry, SystemTrayMenuItem, TrayHandle},
  Icon, SystemTrayEvent,
};
use wry::application::event_loop::EventLoopWindowTarget;
pub use wry::application::{
  event::TrayEvent,
  event_loop::EventLoopProxy,
  menu::{
    ContextMenu as WryContextMenu, CustomMenuItem as WryCustomMenuItem, MenuItem as WryMenuItem,
  },
  system_tray::Icon as WryTrayIcon,
  TrayId as WryTrayId,
};

#[cfg(target_os = "macos")]
pub use wry::application::platform::macos::{
  CustomMenuItemExtMacOS, SystemTrayBuilderExtMacOS, SystemTrayExtMacOS,
};

use wry::application::system_tray::{SystemTray as WrySystemTray, SystemTrayBuilder};

use crate::{send_user_message, Context, Error, Message, Result, TrayId, TrayMessage};

use tauri_runtime::{menu::MenuHash, SystemTray, UserEvent};

use std::{
  cell::RefCell,
  collections::HashMap,
  fmt,
  sync::{Arc, Mutex},
};

pub type GlobalSystemTrayEventHandler = Box<dyn Fn(TrayId, &SystemTrayEvent) + Send>;
pub type GlobalSystemTrayEventListeners = Arc<Mutex<Vec<Arc<GlobalSystemTrayEventHandler>>>>;

pub type SystemTrayEventHandler = Box<dyn Fn(&SystemTrayEvent) + Send>;
pub type SystemTrayEventListeners = Arc<TrayListenersCell>;
pub type SystemTrayItems = Arc<TrayItemsCell>;

#[derive(Debug, Default)]
pub struct TrayItemsCell(pub RefCell<HashMap<u16, WryCustomMenuItem>>);

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for TrayItemsCell {}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for TrayItemsCell {}

#[derive(Default)]
pub struct TrayCell(pub RefCell<Option<WrySystemTray>>);

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for TrayCell {}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for TrayCell {}

#[derive(Default)]
pub struct TrayListenersCell(pub RefCell<Vec<Arc<SystemTrayEventHandler>>>);

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for TrayListenersCell {}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for TrayListenersCell {}

#[derive(Clone, Default)]
pub struct TrayContext {
  pub tray: Arc<TrayCell>,
  pub listeners: SystemTrayEventListeners,
  pub items: SystemTrayItems,
}

impl fmt::Debug for TrayContext {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("TrayContext")
      .field("items", &self.items)
      .finish()
  }
}

#[derive(Clone, Default)]
pub struct SystemTrayManager {
  pub trays: Arc<Mutex<HashMap<TrayId, TrayContext>>>,
  pub global_listeners: GlobalSystemTrayEventListeners,
}

impl fmt::Debug for SystemTrayManager {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("SystemTrayManager")
      .field("trays", &self.trays)
      .finish()
  }
}

/// Wrapper around a [`wry::application::system_tray::Icon`] that can be created from an [`WindowIcon`].
pub struct TrayIcon(pub(crate) WryTrayIcon);

impl TryFrom<Icon> for TrayIcon {
  type Error = Error;
  fn try_from(icon: Icon) -> std::result::Result<Self, Self::Error> {
    WryTrayIcon::from_rgba(icon.rgba, icon.width, icon.height)
      .map(Self)
      .map_err(crate::icon_err)
  }
}

pub fn create_tray<T>(
  id: WryTrayId,
  system_tray: SystemTray,
  event_loop: &EventLoopWindowTarget<T>,
) -> crate::Result<(WrySystemTray, HashMap<u16, WryCustomMenuItem>)> {
  let icon = TrayIcon::try_from(system_tray.icon.expect("tray icon not set"))?;

  let mut items = HashMap::new();

  #[allow(unused_mut)]
  let mut builder = SystemTrayBuilder::new(
    icon.0,
    system_tray
      .menu
      .map(|menu| to_wry_context_menu(&mut items, menu)),
  )
  .with_id(id);

  #[cfg(target_os = "macos")]
  {
    builder = builder
      .with_icon_as_template(system_tray.icon_as_template)
      .with_menu_on_left_click(system_tray.menu_on_left_click);

    if let Some(title) = system_tray.title {
      builder = builder.with_title(&title);
    }
  }

  if let Some(tooltip) = system_tray.tooltip {
    builder = builder.with_tooltip(&tooltip);
  }

  let tray = builder
    .build(event_loop)
    .map_err(|e| Error::SystemTray(Box::new(e)))?;

  Ok((tray, items))
}

#[derive(Debug, Clone)]
pub struct SystemTrayHandle<T: UserEvent> {
  pub(crate) context: Context<T>,
  pub(crate) id: TrayId,
  pub(crate) proxy: EventLoopProxy<super::Message<T>>,
  pub(crate) pending: PendingSystemTray,
}

impl<T: UserEvent> TrayHandle for SystemTrayHandle<T> {
  fn set_icon(&self, icon: Icon) -> Result<()> {
    if let Some(pending) = &mut *self.pending.0.borrow_mut() {
      pending.icon.replace(icon);
      Ok(())
    } else {
      self
        .proxy
        .send_event(Message::Tray(self.id, TrayMessage::UpdateIcon(icon)))
        .map_err(|_| Error::FailedToSendMessage)
    }
  }

  fn set_menu(&self, menu: SystemTrayMenu) -> Result<()> {
    if let Some(pending) = &mut *self.pending.0.borrow_mut() {
      pending.menu.replace(menu);
      Ok(())
    } else {
      self
        .proxy
        .send_event(Message::Tray(self.id, TrayMessage::UpdateMenu(menu)))
        .map_err(|_| Error::FailedToSendMessage)
    }
  }

  fn update_item(&self, id: u16, update: MenuUpdate) -> Result<()> {
    if let Some(_pending) = &mut *self.pending.0.borrow_mut() {
      // do nothing
      Ok(())
    } else {
      self
        .proxy
        .send_event(Message::Tray(self.id, TrayMessage::UpdateItem(id, update)))
        .map_err(|_| Error::FailedToSendMessage)
    }
  }

  #[cfg(target_os = "macos")]
  fn set_icon_as_template(&self, is_template: bool) -> tauri_runtime::Result<()> {
    if let Some(pending) = &mut *self.pending.0.borrow_mut() {
      pending.icon_as_template = is_template;
      Ok(())
    } else {
      self
        .proxy
        .send_event(Message::Tray(
          self.id,
          TrayMessage::UpdateIconAsTemplate(is_template),
        ))
        .map_err(|_| Error::FailedToSendMessage)
    }
  }

  #[cfg(target_os = "macos")]
  fn set_title(&self, title: &str) -> tauri_runtime::Result<()> {
    if let Some(pending) = &mut *self.pending.0.borrow_mut() {
      pending.title.replace(title.to_string());
      Ok(())
    } else {
      self
        .proxy
        .send_event(Message::Tray(
          self.id,
          TrayMessage::UpdateTitle(title.to_owned()),
        ))
        .map_err(|_| Error::FailedToSendMessage)
    }
  }

  fn set_tooltip(&self, tooltip: &str) -> Result<()> {
    if let Some(pending) = &mut *self.pending.0.borrow_mut() {
      pending.tooltip.replace(tooltip.to_string());
      Ok(())
    } else {
      self
        .proxy
        .send_event(Message::Tray(
          self.id,
          TrayMessage::UpdateTooltip(tooltip.to_owned()),
        ))
        .map_err(|_| Error::FailedToSendMessage)
    }
  }

  fn destroy(&self) -> Result<()> {
    if self.pending.0.borrow_mut().take().is_none() {
      let (tx, rx) = std::sync::mpsc::channel();
      send_user_message(
        &self.context,
        Message::Tray(self.id, TrayMessage::Destroy(tx)),
      )?;
      rx.recv().unwrap()?;
    }
    Ok(())
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

#[derive(Debug, Clone)]
pub struct PendingSystemTray(pub Arc<RefCell<Option<SystemTray>>>);

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for PendingSystemTray {}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for PendingSystemTray {}
