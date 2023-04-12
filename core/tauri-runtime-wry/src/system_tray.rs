// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::fmt;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use tauri_runtime::menu::MenuHash;
pub use tauri_runtime::menu::{Menu, MenuEntry, MenuItem, MenuUpdate, Submenu};
pub use tauri_runtime::system_tray::{
  SystemTrayEvent, SystemTrayMenu, SystemTrayMenuEntry, SystemTrayMenuItem,
};
pub use tauri_runtime::Icon;
use tauri_runtime::UserEvent;
use wry::application::event::Event as WryEvent;
pub use wry::application::event::TrayEvent;
pub use wry::application::event_loop::EventLoopProxy;
use wry::application::event_loop::EventLoopWindowTarget;
pub use wry::application::menu::{
  ContextMenu as WryContextMenu, CustomMenuItem as WryCustomMenuItem, MenuItem as WryMenuItem,
};
#[cfg(target_os = "macos")]
pub use wry::application::platform::macos::{
  CustomMenuItemExtMacOS, SystemTrayBuilderExtMacOS, SystemTrayExtMacOS,
};
pub use wry::application::system_tray::Icon as WrySystemTrayIcon;
use wry::application::system_tray::{
  SystemTray as WrySystemTray, SystemTrayBuilder as WrySystemTrayBuilder,
};
use wry::application::TrayId as WryTrayId;

use crate::wrappers::{PhysicalPositionWrapper, RectWrapper};
use crate::{send_user_message, Context, Error, Message, Result, SystemTrayId};

pub type GlobalSystemTrayEventListener = Box<dyn Fn(SystemTrayId, &SystemTrayEvent) + Send>;
pub type GlobalSystemTrayEventListenerStore = Arc<Mutex<Vec<Arc<GlobalSystemTrayEventListener>>>>;

pub type SystemTrayEventListener = Box<dyn Fn(&SystemTrayEvent) + Send>;
pub type SystemTrayEventListenerStore = Arc<Mutex<Vec<Arc<SystemTrayEventListener>>>>;
pub type SystemTrayItems = Arc<Mutex<HashMap<u16, WryCustomMenuItem>>>;

#[derive(Clone, Default)]
pub struct SystemTrayContext {
  pub tray: Arc<Mutex<Option<WrySystemTray>>>,
  pub listeners_store: SystemTrayEventListenerStore,
  pub tray_menu_items_store: SystemTrayItems,
}

impl fmt::Debug for SystemTrayContext {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("SystemTrayContext")
      .field("tray_menu_items_store", &self.tray_menu_items_store)
      .finish()
  }
}

#[derive(Clone, Default)]
pub struct SystemTrayManager {
  pub trays: Arc<Mutex<HashMap<SystemTrayId, SystemTrayContext>>>,
  pub global_listeners: GlobalSystemTrayEventListenerStore,
}

impl fmt::Debug for SystemTrayManager {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("SystemTrayManager")
      .field("trays", &self.trays)
      .finish()
  }
}

/// Wrapper around a [`wry::application::system_tray::Icon`] that can be created from an [`WindowIcon`].
pub struct SystemTrayIcon(pub(crate) WrySystemTrayIcon);

impl TryFrom<Icon> for SystemTrayIcon {
  type Error = Error;
  fn try_from(icon: Icon) -> std::result::Result<Self, Self::Error> {
    WrySystemTrayIcon::from_rgba(icon.rgba, icon.width, icon.height)
      .map(Self)
      .map_err(crate::icon_err)
  }
}

#[derive(Debug, Clone)]
pub struct SystemTrayHandle<T: UserEvent> {
  pub(crate) context: Context<T>,
  pub(crate) id: SystemTrayId,
  pub(crate) proxy: EventLoopProxy<super::Message<T>>,
}

impl<T: UserEvent> tauri_runtime::system_tray::SystemTrayHandle for SystemTrayHandle<T> {
  fn set_icon(&self, icon: Icon) -> Result<()> {
    self
      .proxy
      .send_event(Message::Tray(self.id, TrayMessage::UpdateIcon(icon)))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_menu(&self, menu: SystemTrayMenu) -> Result<()> {
    self
      .proxy
      .send_event(Message::Tray(self.id, TrayMessage::UpdateMenu(menu)))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn update_item(&self, id: u16, update: MenuUpdate) -> Result<()> {
    self
      .proxy
      .send_event(Message::Tray(self.id, TrayMessage::UpdateItem(id, update)))
      .map_err(|_| Error::FailedToSendMessage)
  }

  #[cfg(target_os = "macos")]
  fn set_icon_as_template(&self, is_template: bool) -> tauri_runtime::Result<()> {
    self
      .proxy
      .send_event(Message::Tray(
        self.id,
        TrayMessage::UpdateIconAsTemplate(is_template),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  #[cfg(target_os = "macos")]
  fn set_title(&self, title: &str) -> tauri_runtime::Result<()> {
    self
      .proxy
      .send_event(Message::Tray(
        self.id,
        TrayMessage::UpdateTitle(title.to_owned()),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_tooltip(&self, tooltip: &str) -> Result<()> {
    self
      .proxy
      .send_event(Message::Tray(
        self.id,
        TrayMessage::UpdateTooltip(tooltip.to_owned()),
      ))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn destroy(&self) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    send_user_message(
      &self.context,
      Message::Tray(self.id, TrayMessage::Destroy(tx)),
    )?;
    rx.recv().unwrap()?;
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

pub type CreateSystemTrayReturn = (
  WrySystemTray,
  Vec<Arc<SystemTrayEventListener>>,
  HashMap<u16, WryCustomMenuItem>,
);

pub fn create_system_tray<T: UserEvent>(
  id: u16,
  mut system_tray: tauri_runtime::system_tray::SystemTray,
  event_loop: &EventLoopWindowTarget<Message<T>>,
) -> crate::Result<CreateSystemTrayReturn> {
  let mut listeners = Vec::new();
  if let Some(l) = system_tray.on_event.take() {
    listeners.push(Arc::new(l));
  }

  let icon = SystemTrayIcon::try_from(system_tray.icon.expect("tray icon not set"))?;

  let mut items = HashMap::new();

  #[allow(unused_mut)]
  let mut builder = WrySystemTrayBuilder::new(
    icon.0,
    system_tray
      .menu
      .map(|menu| to_wry_context_menu(&mut items, menu)),
  )
  .with_id(WryTrayId(id));

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

  Ok((tray, listeners, items))
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
pub enum TrayMessage {
  UpdateItem(u16, MenuUpdate),
  UpdateMenu(SystemTrayMenu),
  UpdateIcon(Icon),
  #[cfg(target_os = "macos")]
  UpdateIconAsTemplate(bool),
  #[cfg(target_os = "macos")]
  UpdateTitle(String),
  UpdateTooltip(String),
  Create(tauri_runtime::system_tray::SystemTray, Sender<Result<()>>),
  Destroy(Sender<Result<()>>),
}

pub fn handle_system_tray_message<T: UserEvent>(
  id: u16,
  message: TrayMessage,
  system_tray_manager: SystemTrayManager,
  event_loop: &EventLoopWindowTarget<Message<T>>,
) {
  let mut trays = system_tray_manager.trays.lock().unwrap();

  if let TrayMessage::Create(tray, tx) = message {
    match create_system_tray(id, tray, event_loop) {
      Ok((tray, listeners, items)) => {
        trays.insert(
          id,
          SystemTrayContext {
            tray: Arc::new(Mutex::new(Some(tray))),
            listeners_store: Arc::new(Mutex::new(listeners)),
            tray_menu_items_store: Arc::new(Mutex::new(items)),
          },
        );

        tx.send(Ok(())).unwrap();
      }

      Err(e) => {
        tx.send(Err(e)).unwrap();
      }
    }
  } else if let Some(tray_context) = trays.get(&id) {
    match message {
      TrayMessage::UpdateItem(menu_id, update) => {
        let mut tray = tray_context.tray_menu_items_store.as_ref().lock().unwrap();
        let item = tray.get_mut(&menu_id).expect("menu item not found");
        match update {
          MenuUpdate::SetEnabled(enabled) => item.set_enabled(enabled),
          MenuUpdate::SetTitle(title) => item.set_title(&title),
          MenuUpdate::SetSelected(selected) => item.set_selected(selected),
          #[cfg(target_os = "macos")]
          MenuUpdate::SetNativeImage(image) => {
            item.set_native_image(NativeImageWrapper::from(image).0)
          }
        }
      }
      TrayMessage::UpdateMenu(menu) => {
        if let Some(tray) = &mut *tray_context.tray.lock().unwrap() {
          let mut items = HashMap::new();
          tray.set_menu(&to_wry_context_menu(&mut items, menu));
          *tray_context.tray_menu_items_store.lock().unwrap() = items;
        }
      }
      TrayMessage::UpdateIcon(icon) => {
        if let Some(tray) = &mut *tray_context.tray.lock().unwrap() {
          if let Ok(icon) = SystemTrayIcon::try_from(icon) {
            tray.set_icon(icon.0);
          }
        }
      }
      #[cfg(target_os = "macos")]
      TrayMessage::UpdateIconAsTemplate(is_template) => {
        if let Some(tray) = &mut *tray_context.tray.lock().unwrap() {
          tray.set_icon_as_template(is_template);
        }
      }
      #[cfg(target_os = "macos")]
      TrayMessage::UpdateTitle(title) => {
        if let Some(tray) = &mut *tray_context.tray.lock().unwrap() {
          tray.set_title(&title);
        }
      }
      TrayMessage::UpdateTooltip(tooltip) => {
        if let Some(tray) = &mut *tray_context.tray.lock().unwrap() {
          tray.set_tooltip(&tooltip);
        }
      }
      TrayMessage::Create(_tray, _tx) => {
        // already handled
      }
      TrayMessage::Destroy(tx) => {
        *tray_context.tray.lock().unwrap() = None;
        tray_context.listeners_store.lock().unwrap().clear();
        tray_context.tray_menu_items_store.lock().unwrap().clear();
        tx.send(Ok(())).unwrap();
      }
    }
  }
}

pub fn handle_system_tray_event<T: UserEvent>(
  event: WryEvent<Message<T>>,
  system_tray_manager: SystemTrayManager,
) {
  let (id, event) = match event {
    WryEvent::MenuEvent { menu_id, .. } => {
      let trays = system_tray_manager.trays.lock().unwrap();
      let id = trays
        .iter()
        .find(|(_, c)| {
          let store = c.tray_menu_items_store.lock().unwrap();
          store.contains_key(&menu_id.0)
        })
        .map(|i| *i.0)
        .unwrap_or_default();

      (id, SystemTrayEvent::MenuItemClick(menu_id.0))
    }
    WryEvent::TrayEvent {
      id,
      bounds,
      event,
      position,
      ..
    } => {
      let (position, bounds) = (
        PhysicalPositionWrapper(position).into(),
        RectWrapper(bounds).into(),
      );
      let event = match event {
        TrayEvent::RightClick => SystemTrayEvent::RightClick { position, bounds },
        TrayEvent::DoubleClick => SystemTrayEvent::DoubleClick { position, bounds },
        // default to left click
        _ => SystemTrayEvent::LeftClick { position, bounds },
      };

      (id.0, event)
    }
    _ => unreachable!(),
  };

  let listeners = {
    let trays = system_tray_manager.trays.lock().unwrap();
    trays
      .get(&id)
      .map(|c| c.listeners_store.lock().unwrap().clone())
  };

  if let Some(listeners) = listeners {
    for handler in listeners.iter() {
      handler(&event);
    }
  }

  let global_listeners = system_tray_manager.global_listeners.lock().unwrap();
  let global_listeners_iter = global_listeners.iter();
  for global_listener in global_listeners_iter {
    global_listener(id, &event);
  }
}
