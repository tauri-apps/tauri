// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  image::Image,
  menu::{IconMenuItem, MenuId, NativeIcon},
  Manager, Runtime,
};

/// A builder type for [`IconMenuItem`]
pub struct IconMenuItemBuilder<'a> {
  id: Option<MenuId>,
  text: String,
  enabled: bool,
  icon: Option<Image<'a>>,
  native_icon: Option<NativeIcon>,
  accelerator: Option<String>,
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
    }
  }

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

  /// Build the menu item
  pub fn build<R: Runtime, M: Manager<R>>(self, manager: &M) -> crate::Result<IconMenuItem<R>> {
    if self.icon.is_some() {
      if let Some(id) = self.id {
        IconMenuItem::with_id(
          manager,
          id,
          self.text,
          self.enabled,
          self.icon,
          self.accelerator,
        )
      } else {
        IconMenuItem::new(
          manager,
          self.text,
          self.enabled,
          self.icon,
          self.accelerator,
        )
      }
    } else if let Some(id) = self.id {
      IconMenuItem::with_id_and_native_icon(
        manager,
        id,
        self.text,
        self.enabled,
        self.native_icon,
        self.accelerator,
      )
    } else {
      IconMenuItem::with_native_icon(
        manager,
        self.text,
        self.enabled,
        self.native_icon,
        self.accelerator,
      )
    }
  }
}
