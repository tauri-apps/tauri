// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{ItemEventHandler, NoHandler};
use crate::{
  Manager, Runtime,
  menu::{MenuEvent, MenuId, MenuItem},
};

/// A builder type for [`MenuItem`]
///
/// The handler type is inferred when using [`Self::handler`].
pub struct MenuItemBuilder<H = NoHandler> {
  id: Option<MenuId>,
  text: String,
  enabled: bool,
  accelerator: Option<String>,
  handler: H,
}

impl MenuItemBuilder {
  /// Create a new menu item builder.
  ///
  /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
  ///   for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn new<S: AsRef<str>>(text: S) -> Self {
    Self {
      id: None,
      text: text.as_ref().to_string(),
      enabled: true,
      accelerator: None,
      handler: NoHandler,
    }
  }

  /// Create a new menu item builder with the specified id.
  ///
  /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
  ///   for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn with_id<I: Into<MenuId>, S: AsRef<str>>(id: I, text: S) -> Self {
    Self {
      id: Some(id.into()),
      text: text.as_ref().to_string(),
      enabled: true,
      accelerator: None,
      handler: NoHandler,
    }
  }
}

impl<H> MenuItemBuilder<H> {
  /// Set the id for this menu item.
  pub fn id<I: Into<MenuId>>(mut self, id: I) -> Self {
    self.id.replace(id.into());
    self
  }

  /// Set the enabled state for this menu item.
  pub fn enabled(mut self, enabled: bool) -> Self {
    self.enabled = enabled;
    self
  }

  /// Set the accelerator for this menu item.
  pub fn accelerator<S: AsRef<str>>(mut self, accelerator: S) -> Self {
    self.accelerator.replace(accelerator.as_ref().to_string());
    self
  }

  /// Set a handler to be called when this item is activated.
  ///
  /// Items with the same ID share a handler registration;
  /// the last handler registered for that ID is used.
  /// Always use a unique ID for each item to ensure its handler is called.
  pub fn handler<R, F>(self, handler: F) -> MenuItemBuilder<F>
  where
    R: Runtime,
    F: Fn(&MenuItem<R>, MenuEvent) + Send + Sync + 'static,
    H: ItemEventHandler<MenuItem<R>>,
  {
    MenuItemBuilder {
      id: self.id,
      text: self.text,
      enabled: self.enabled,
      accelerator: self.accelerator,
      handler,
    }
  }

  /// Build the menu item
  pub fn build<R: Runtime, M: Manager<R>>(self, manager: &M) -> crate::Result<MenuItem<R>>
  where
    H: ItemEventHandler<MenuItem<R>>,
  {
    let i = if let Some(id) = self.id {
      MenuItem::with_id(manager, id, self.text, self.enabled, self.accelerator)?
    } else {
      MenuItem::new(manager, self.text, self.enabled, self.accelerator)?
    };

    if let Some(handler) = self.handler.into_handler() {
      let menu_manager = &manager.manager().menu;
      menu_manager.on_menu_item_event(i.id().clone(), &i.0, move |item, event| {
        handler(&MenuItem(item), event);
      });
    };

    Ok(i)
  }
}
