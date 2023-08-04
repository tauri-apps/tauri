// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use muda::icon::NativeIcon;

use crate::{menu::IconMenuItem, Icon, Manager, Runtime};

/// A builder type for [`IconMenuItem`]
pub struct IconMenuItemBuilder {
  text: String,
  enabled: bool,
  icon: Option<Icon>,
  native_icon: Option<NativeIcon>,
  accelerator: Option<String>,
}

impl IconMenuItemBuilder {
  /// Create a new menu item builder.
  pub fn new<S: AsRef<str>>(text: S) -> Self {
    Self {
      text: text.as_ref().to_string(),
      enabled: true,
      icon: None,
      native_icon: None,
      accelerator: None,
    }
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
  pub fn icon(mut self, icon: Icon) -> Self {
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
  pub fn build<R: Runtime, M: Manager<R>>(self, manager: &M) -> IconMenuItem<R> {
    if self.icon.is_some() {
      IconMenuItem::new(
        manager,
        self.text,
        self.enabled,
        self.icon,
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
