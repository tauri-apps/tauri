// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{menu::CheckMenuItem, Manager, Runtime};

/// A builder type for [`CheckMenuItem`]
pub struct CheckMenuItemBuilder {
  text: String,
  enabled: bool,
  checked: bool,
  accelerator: Option<String>,
}

impl Default for CheckMenuItemBuilder {
  fn default() -> Self {
    Self::new("")
  }
}

impl CheckMenuItemBuilder {
  /// Create a new menu item builder.
  pub fn new<S: AsRef<str>>(text: S) -> Self {
    Self {
      text: text.as_ref().to_string(),
      enabled: true,
      checked: true,
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

  /// Set the checked state for this menu item.
  pub fn checked(mut self, checked: bool) -> Self {
    self.checked = checked;
    self
  }

  /// Set the accelerator for this menu item.
  pub fn accelerator<S: AsRef<str>>(mut self, accelerator: S) -> Self {
    self.accelerator.replace(accelerator.as_ref().to_string());
    self
  }

  /// Build the menu item
  pub fn build<R: Runtime, M: Manager<R>>(self, manager: &M) -> CheckMenuItem<R> {
    CheckMenuItem::new(
      manager,
      self.text,
      self.enabled,
      self.checked,
      self.accelerator,
    )
  }
}
