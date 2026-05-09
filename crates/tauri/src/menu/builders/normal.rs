// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{menu::MenuId, menu::MenuItem, Manager, Runtime};

/// A builder type for [`MenuItem`]
pub struct MenuItemBuilder {
  id: Option<MenuId>,
  text: String,
  enabled: bool,
  accelerator: Option<String>,
  sf_symbol: Option<String>,
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
      sf_symbol: None,
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
      sf_symbol: None,
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

  /// Set this menu item's icon to an SF Symbol by name (macOS 11+).
  ///
  /// SF Symbols are template images that automatically adapt to the system
  /// light/dark appearance.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux**: Unsupported (no-op).
  pub fn sf_symbol<S: AsRef<str>>(mut self, symbol: S) -> Self {
    self.sf_symbol.replace(symbol.as_ref().to_string());
    self
  }

  /// Build the menu item
  pub fn build<R: Runtime, M: Manager<R>>(self, manager: &M) -> crate::Result<MenuItem<R>> {
    let item = if let Some(id) = self.id {
      MenuItem::with_id(manager, id, self.text, self.enabled, self.accelerator)?
    } else {
      MenuItem::new(manager, self.text, self.enabled, self.accelerator)?
    };
    if let Some(symbol) = self.sf_symbol {
      item.set_sf_symbol(Some(symbol))?;
    }
    Ok(item)
  }
}
