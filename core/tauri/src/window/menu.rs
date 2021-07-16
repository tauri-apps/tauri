// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::runtime::{
  menu::{MenuHash, MenuId, MenuIdRef, MenuUpdate},
  Dispatch, Runtime,
};

use tauri_macros::default_runtime;

use std::collections::HashMap;

/// The window menu event.
#[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
#[derive(Debug, Clone)]
pub struct MenuEvent {
  pub(crate) menu_item_id: MenuId,
}

#[cfg(feature = "menu")]
impl MenuEvent {
  /// The menu item id.
  pub fn menu_item_id(&self) -> MenuIdRef<'_> {
    &self.menu_item_id
  }
}

/// A handle to a system tray. Allows updating the context menu items.
#[default_runtime(crate::Wry, wry)]
pub struct MenuHandle<R: Runtime> {
  pub(crate) ids: HashMap<MenuHash, MenuId>,
  pub(crate) dispatcher: R::Dispatcher,
}

impl<R: Runtime> Clone for MenuHandle<R> {
  fn clone(&self) -> Self {
    Self {
      ids: self.ids.clone(),
      dispatcher: self.dispatcher.clone(),
    }
  }
}

/// A handle to a system tray menu item.
#[default_runtime(crate::Wry, wry)]
pub struct MenuItemHandle<R: Runtime> {
  id: u16,
  dispatcher: R::Dispatcher,
}

impl<R: Runtime> Clone for MenuItemHandle<R> {
  fn clone(&self) -> Self {
    Self {
      id: self.id,
      dispatcher: self.dispatcher.clone(),
    }
  }
}

impl<R: Runtime> MenuHandle<R> {
  /// Gets a handle to the menu item that has the specified `id`.
  pub fn get_item(&self, id: MenuIdRef<'_>) -> MenuItemHandle<R> {
    for (raw, item_id) in self.ids.iter() {
      if item_id == id {
        return MenuItemHandle {
          id: *raw,
          dispatcher: self.dispatcher.clone(),
        };
      }
    }
    panic!("item id not found")
  }

  /// Shows the menu.
  pub fn show(&self) -> crate::Result<()> {
    self.dispatcher.show_menu().map_err(Into::into)
  }

  /// Hides the menu.
  pub fn hide(&self) -> crate::Result<()> {
    self.dispatcher.hide_menu().map_err(Into::into)
  }

  /// Whether the menu is visible or not.
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn is_visible(&self) -> crate::Result<bool> {
    self.dispatcher.is_menu_visible().map_err(Into::into)
  }

  /// Toggles the menu visibility.
  ///
  /// # Panics
  ///
  /// Panics if the app is not running yet, usually when called on the [`setup`](crate::Builder#method.setup) closure.
  /// You can spawn a task to use the API using the [`async_runtime`](crate::async_runtime) to prevent the panic.
  pub fn toggle(&self) -> crate::Result<()> {
    if self.is_visible()? {
      self.hide()
    } else {
      self.show()
    }
  }
}

impl<R: Runtime> MenuItemHandle<R> {
  /// Modifies the enabled state of the menu item.
  pub fn set_enabled(&self, enabled: bool) -> crate::Result<()> {
    self
      .dispatcher
      .update_menu_item(self.id, MenuUpdate::SetEnabled(enabled))
      .map_err(Into::into)
  }

  /// Modifies the title (label) of the menu item.
  pub fn set_title<S: Into<String>>(&self, title: S) -> crate::Result<()> {
    self
      .dispatcher
      .update_menu_item(self.id, MenuUpdate::SetTitle(title.into()))
      .map_err(Into::into)
  }

  /// Modifies the selected state of the menu item.
  pub fn set_selected(&self, selected: bool) -> crate::Result<()> {
    self
      .dispatcher
      .update_menu_item(self.id, MenuUpdate::SetSelected(selected))
      .map_err(Into::into)
  }

  #[cfg(target_os = "macos")]
  #[cfg_attr(doc_cfg, doc(cfg(target_os = "macos")))]
  pub fn set_native_image(&self, image: crate::NativeImage) -> crate::Result<()> {
    self
      .dispatcher
      .update_menu_item(self.id, MenuUpdate::SetNativeImage(image))
      .map_err(Into::into)
  }
}
