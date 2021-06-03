// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use crate::{
  runtime::{
    menu::{MenuUpdate, SystemTrayMenu, SystemTrayMenuEntry, TrayHandle},
    window::dpi::{PhysicalPosition, PhysicalSize},
    Icon, MenuId, Runtime, SystemTray,
  },
  Params,
};

use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

pub(crate) fn get_menu_ids<I: MenuId>(map: &mut HashMap<u32, I>, menu: &SystemTrayMenu<I>) {
  for item in &menu.items {
    match item {
      SystemTrayMenuEntry::CustomItem(c) => {
        map.insert(c.id_value(), c.id.clone());
      }
      SystemTrayMenuEntry::Submenu(s) => get_menu_ids(map, &s.inner),
      _ => {}
    }
  }
}

/// System tray event.
#[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
#[non_exhaustive]
pub enum SystemTrayEvent<I: MenuId> {
  /// Tray context menu item was clicked.
  #[non_exhaustive]
  MenuItemClick {
    /// The id of the menu item.
    id: I,
  },
  /// Tray icon received a left click.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Unsupported
  #[non_exhaustive]
  LeftClick {
    /// The position of the tray icon.
    position: PhysicalPosition<f64>,
    /// The size of the tray icon.
    size: PhysicalSize<f64>,
  },
  /// Tray icon received a right click.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Unsupported
  /// - **macOS:** `Ctrl` + `Left click` fire this event.
  #[non_exhaustive]
  RightClick {
    /// The position of the tray icon.
    position: PhysicalPosition<f64>,
    /// The size of the tray icon.
    size: PhysicalSize<f64>,
  },
  /// Fired when a menu item receive a `Double click`
  ///
  /// ## Platform-specific
  ///
  /// - **macOS / Linux:** Unsupported
  ///
  #[non_exhaustive]
  DoubleClick {
    /// The position of the tray icon.
    position: PhysicalPosition<f64>,
    /// The size of the tray icon.
    size: PhysicalSize<f64>,
  },
}

crate::manager::default_args! {
  /// A handle to a system tray. Allows updating the context menu items.
  pub struct SystemTrayHandle<P: Params> {
    pub(crate) ids: Arc<HashMap<u32, P::SystemTrayMenuId>>,
    pub(crate) inner: Arc<Mutex<<P::Runtime as Runtime>::TrayHandler>>,
  }
}

impl<P: Params> Clone for SystemTrayHandle<P> {
  fn clone(&self) -> Self {
    Self {
      ids: self.ids.clone(),
      inner: self.inner.clone(),
    }
  }
}

crate::manager::default_args! {
  /// A handle to a system tray menu item.
  pub struct SystemTrayMenuItemHandle<P: Params> {
    id: u32,
    tray_handler: Arc<Mutex<<P::Runtime as Runtime>::TrayHandler>>,
  }
}

impl<P: Params> Clone for SystemTrayMenuItemHandle<P> {
  fn clone(&self) -> Self {
    Self {
      id: self.id,
      tray_handler: self.tray_handler.clone(),
    }
  }
}

impl<P: Params> SystemTrayHandle<P> {
  pub fn get_item(&self, id: &P::SystemTrayMenuId) -> SystemTrayMenuItemHandle<P> {
    for (raw, item_id) in self.ids.iter() {
      if item_id == id {
        return SystemTrayMenuItemHandle {
          id: *raw,
          tray_handler: self.inner.clone(),
        };
      }
    }
    panic!("item id not found")
  }

  /// Updates the tray icon. Must be a [`Icon::File`] on Linux and a [`Icon::Raw`] on Windows and macOS.
  pub fn set_icon(&self, icon: Icon) -> crate::Result<()> {
    self
      .inner
      .lock()
      .unwrap()
      .set_icon(icon)
      .map_err(Into::into)
  }
}

impl<P: Params> SystemTrayMenuItemHandle<P> {
  /// Modifies the enabled state of the menu item.
  pub fn set_enabled(&self, enabled: bool) -> crate::Result<()> {
    self
      .tray_handler
      .lock()
      .unwrap()
      .update_item(self.id, MenuUpdate::SetEnabled(enabled))
      .map_err(Into::into)
  }

  /// Modifies the title (label) of the menu item.
  pub fn set_title<S: Into<String>>(&self, title: S) -> crate::Result<()> {
    self
      .tray_handler
      .lock()
      .unwrap()
      .update_item(self.id, MenuUpdate::SetTitle(title.into()))
      .map_err(Into::into)
  }

  /// Modifies the selected state of the menu item.
  pub fn set_selected(&self, selected: bool) -> crate::Result<()> {
    self
      .tray_handler
      .lock()
      .unwrap()
      .update_item(self.id, MenuUpdate::SetSelected(selected))
      .map_err(Into::into)
  }
}
