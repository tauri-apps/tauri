// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{ItemEventHandler, NoHandler};
use crate::{
  Manager, Runtime,
  image::Image,
  menu::{IconMenuItem, MenuEvent, MenuId, NativeIcon},
};

/// A builder type for [`IconMenuItem`]
///
/// The handler type is inferred when using [`Self::handler`].
pub struct IconMenuItemBuilder<'a, H = NoHandler> {
  id: Option<MenuId>,
  text: String,
  enabled: bool,
  icon: Option<Image<'a>>,
  native_icon: Option<NativeIcon>,
  accelerator: Option<String>,
  handler: H,
}

impl<'a> IconMenuItemBuilder<'a> {
  /// Create a new menu item builder.
  ///
  /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
  ///   for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn new<S: AsRef<str>>(text: S) -> Self {
    Self {
      id: None,
      text: text.as_ref().to_string(),
      enabled: true,
      icon: None,
      native_icon: None,
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
      icon: None,
      native_icon: None,
      accelerator: None,
      handler: NoHandler,
    }
  }
}

impl<'a, H> IconMenuItemBuilder<'a, H> {
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

  /// Set the icon for this menu item.
  ///
  /// **Note:** This method conflicts with [`Self::native_icon`]
  /// so calling one of them, will reset the other.
  pub fn icon(mut self, icon: Image<'a>) -> Self {
    self.icon.replace(icon);
    self.native_icon = None;
    self
  }

  /// Set the icon for this menu item.
  ///
  /// **Note:** This method conflicts with [`Self::icon`]
  /// so calling one of them, will reset the other.
  pub fn native_icon(mut self, icon: NativeIcon) -> Self {
    self.native_icon.replace(icon);
    self.icon = None;
    self
  }

  /// Set a handler to be called when this item is activated.
  ///
  /// The handler receives a temporary reference to the item and does not keep it alive.
  /// It is removed when the last item reference is dropped, including references held
  /// by menus and submenus. Items with the same ID share a handler registration;
  /// the last handler registered for that ID is used.
  pub fn handler<R, F>(self, handler: F) -> IconMenuItemBuilder<'a, F>
  where
    R: Runtime,
    F: Fn(&IconMenuItem<R>, MenuEvent) + Send + Sync + 'static,
    H: ItemEventHandler<IconMenuItem<R>>,
  {
    IconMenuItemBuilder {
      id: self.id,
      text: self.text,
      enabled: self.enabled,
      icon: self.icon,
      native_icon: self.native_icon,
      accelerator: self.accelerator,
      handler,
    }
  }

  /// Build the menu item
  pub fn build<R: Runtime, M: Manager<R>>(self, manager: &M) -> crate::Result<IconMenuItem<R>>
  where
    H: ItemEventHandler<IconMenuItem<R>>,
  {
    let i = if self.icon.is_some() {
      if let Some(id) = self.id {
        IconMenuItem::with_id(
          manager,
          id,
          self.text,
          self.enabled,
          self.icon,
          self.accelerator,
        )?
      } else {
        IconMenuItem::new(
          manager,
          self.text,
          self.enabled,
          self.icon,
          self.accelerator,
        )?
      }
    } else if let Some(id) = self.id {
      IconMenuItem::with_id_and_native_icon(
        manager,
        id,
        self.text,
        self.enabled,
        self.native_icon,
        self.accelerator,
      )?
    } else {
      IconMenuItem::with_native_icon(
        manager,
        self.text,
        self.enabled,
        self.native_icon,
        self.accelerator,
      )?
    };

    if let Some(handler) = self.handler.into_handler() {
      let menu_manager = &manager.manager().menu;
      menu_manager.on_menu_item_event(i.id().clone(), &i.0, move |item, event| {
        handler(&IconMenuItem(item), event);
      });
    };

    Ok(i)
  }
}
