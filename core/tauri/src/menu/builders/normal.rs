// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{menu::MenuItem, Manager, Runtime};

/// A builder type for [`MenuItem`]
pub struct MenuItemBuilder {
  text: String,
  enabled: bool,
  accelerator: Option<String>,
}

impl Default for MenuItemBuilder {
  fn default() -> Self {
    Self::new("")
  }
}

impl MenuItemBuilder {
  /// Create a new menu item builder.
  pub fn new<S: AsRef<str>>(text: S) -> Self {
    Self {
      text: text.as_ref().to_string(),
      enabled: true,
      accelerator: None,
    }
  }

  /// Set the text for this menu item.
  pub fn text<S: AsRef<str>>(mut self, text: S) -> Self {
    self.text = text.as_ref().to_string();
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

  /// Build the menu item
  pub fn build<R: Runtime, M: Manager<R>>(self, manager: &M) -> MenuItem<R> {
    MenuItem::new(manager, self.text, self.enabled, self.accelerator)
  }
}
