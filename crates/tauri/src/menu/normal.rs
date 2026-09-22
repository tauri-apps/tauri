// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use crate::menu::MenuItemInner;
use crate::{Manager, Runtime, menu::MenuId};

use super::MenuItem;

impl<R: Runtime> MenuItem<R> {
  /// Create a new menu item.
  ///
  /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
  ///   for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn new<M, T, A>(
    manager: &M,
    text: T,
    enabled: bool,
    accelerator: Option<A>,
  ) -> crate::Result<Self>
  where
    M: Manager<R>,
    T: AsRef<str>,
    A: AsRef<str>,
  {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.as_ref().to_owned();
    let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());

    let item = handle.run_on_main_thread_blocking(move || {
      let item = muda::MenuItem::new(text, enabled, accelerator);
      MenuItemInner::new(app_handle, item)
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Create a new menu item with the specified id.
  ///
  /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
  ///   for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn with_id<M, I, T, A>(
    manager: &M,
    id: I,
    text: T,
    enabled: bool,
    accelerator: Option<A>,
  ) -> crate::Result<Self>
  where
    M: Manager<R>,
    I: Into<MenuId>,
    T: AsRef<str>,
    A: AsRef<str>,
  {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let id = id.into();
    let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());
    let text = text.as_ref().to_owned();

    let item = handle.run_on_main_thread_blocking(move || {
      let item = muda::MenuItem::with_id(id.clone(), text, enabled, accelerator);
      MenuItemInner::new(app_handle, item)
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Get the text for this menu item.
  pub fn text(&self) -> crate::Result<String> {
    self.with_inner_blocking(|i| i.text())
  }

  /// Set the text for this menu item. `text` could optionally contain
  /// an `&` before a character to assign this character as the mnemonic
  /// for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn set_text<S: AsRef<str>>(&self, text: S) -> crate::Result<()> {
    let text = text.as_ref().to_string();
    self.with_inner_blocking(|i| i.set_text(text))
  }

  /// Get whether this menu item is enabled or not.
  pub fn is_enabled(&self) -> crate::Result<bool> {
    self.with_inner_blocking(|i| i.is_enabled())
  }

  /// Enable or disable this menu item.
  pub fn set_enabled(&self, enabled: bool) -> crate::Result<()> {
    self.with_inner_blocking(move |i| i.set_enabled(enabled))
  }

  /// Set this menu item accelerator.
  pub fn set_accelerator<S: AsRef<str>>(&self, accelerator: Option<S>) -> crate::Result<()> {
    let accel = accelerator.and_then(|s| s.as_ref().parse().ok());
    self
      .with_inner_blocking(move |i| i.set_accelerator(accel))?
      .map_err(Into::into)
  }
}
