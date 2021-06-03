// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  runtime::{menu::MenuUpdate, Dispatch, MenuId, Runtime},
  Params,
};

use std::collections::HashMap;

/// The window menu event.
#[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
#[derive(Debug, Clone)]
pub struct MenuEvent<I: MenuId> {
  pub(crate) menu_item_id: I,
}

#[cfg(feature = "menu")]
impl<I: MenuId> MenuEvent<I> {
  /// The menu item id.
  pub fn menu_item_id(&self) -> &I {
    &self.menu_item_id
  }
}

crate::manager::default_args! {
  /// A handle to a system tray. Allows updating the context menu items.
  pub struct MenuHandle<P: Params> {
    pub(crate) ids: HashMap<u32, P::MenuId>,
    pub(crate) dispatcher: <P::Runtime as Runtime>::Dispatcher,
  }
}

impl<P: Params> Clone for MenuHandle<P> {
  fn clone(&self) -> Self {
    Self {
      ids: self.ids.clone(),
      dispatcher: self.dispatcher.clone(),
    }
  }
}

crate::manager::default_args! {
  /// A handle to a system tray menu item.
  pub struct MenuItemHandle<P: Params> {
    id: u32,
    dispatcher: <P::Runtime as Runtime>::Dispatcher,
  }
}

impl<P: Params> Clone for MenuItemHandle<P> {
  fn clone(&self) -> Self {
    Self {
      id: self.id,
      dispatcher: self.dispatcher.clone(),
    }
  }
}

impl<P: Params> MenuHandle<P> {
  pub fn get_item(&self, id: &P::MenuId) -> MenuItemHandle<P> {
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
}

impl<P: Params> MenuItemHandle<P> {
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
}
