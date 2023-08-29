// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{menu::MenuId, resources::Resource, run_main_thread, AppHandle, Manager, Runtime};

/// A menu item inside a [`Menu`] or [`Submenu`]
/// and usually contains a text and a check mark or a similar toggle
/// that corresponds to a checked and unchecked states.
///
/// [`Menu`]: super::Menu
/// [`Submenu`]: super::Submenu
pub struct CheckMenuItem<R: Runtime> {
  pub(crate) id: MenuId,
  pub(crate) inner: muda::CheckMenuItem,
  pub(crate) app_handle: AppHandle<R>,
}

impl<R: Runtime> Clone for CheckMenuItem<R> {
  fn clone(&self) -> Self {
    Self {
      id: self.id.clone(),
      inner: self.inner.clone(),
      app_handle: self.app_handle.clone(),
    }
  }
}

/// # Safety
///
/// We make sure it always runs on the main thread.
unsafe impl<R: Runtime> Sync for CheckMenuItem<R> {}
unsafe impl<R: Runtime> Send for CheckMenuItem<R> {}

impl<R: Runtime> super::sealed::IsMenuItemBase for CheckMenuItem<R> {
  fn inner_muda(&self) -> &dyn muda::IsMenuItem {
    &self.inner
  }
}

impl<R: Runtime> super::IsMenuItem<R> for CheckMenuItem<R> {
  fn kind(&self) -> super::MenuItemKind<R> {
    super::MenuItemKind::Check(self.clone())
  }

  fn id(&self) -> &MenuId {
    &self.id
  }
}

impl<R: Runtime> CheckMenuItem<R> {
  /// Create a new menu item.
  ///
  /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
  /// for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn new<M: Manager<R>, S: AsRef<str>>(
    manager: &M,
    text: S,
    enabled: bool,
    checked: bool,
    acccelerator: Option<S>,
  ) -> Self {
    let item = muda::CheckMenuItem::new(
      text,
      enabled,
      checked,
      acccelerator.and_then(|s| s.as_ref().parse().ok()),
    );
    Self {
      id: item.id().clone(),
      inner: item,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Create a new menu item with the specified id.
  ///
  /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
  /// for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn with_id<M: Manager<R>, I: Into<MenuId>, S: AsRef<str>>(
    manager: &M,
    id: I,
    text: S,
    enabled: bool,
    checked: bool,
    acccelerator: Option<S>,
  ) -> Self {
    let item = muda::CheckMenuItem::with_id(
      id,
      text,
      enabled,
      checked,
      acccelerator.and_then(|s| s.as_ref().parse().ok()),
    );
    Self {
      id: item.id().clone(),
      inner: item,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// The application handle associated with this type.
  pub fn app_handle(&self) -> &AppHandle<R> {
    &self.app_handle
  }

  /// Returns a unique identifier associated with this menu item.
  pub fn id(&self) -> &MenuId {
    &self.id
  }

  /// Get the text for this menu item.
  pub fn text(&self) -> crate::Result<String> {
    run_main_thread!(self, |self_: Self| self_.inner.text())
  }

  /// Set the text for this menu item. `text` could optionally contain
  /// an `&` before a character to assign this character as the mnemonic
  /// for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn set_text<S: AsRef<str>>(&self, text: S) -> crate::Result<()> {
    let text = text.as_ref().to_string();
    run_main_thread!(self, |self_: Self| self_.inner.set_text(text))
  }

  /// Get whether this menu item is enabled or not.
  pub fn is_enabled(&self) -> crate::Result<bool> {
    run_main_thread!(self, |self_: Self| self_.inner.is_enabled())
  }

  /// Enable or disable this menu item.
  pub fn set_enabled(&self, enabled: bool) -> crate::Result<()> {
    run_main_thread!(self, |self_: Self| self_.inner.set_enabled(enabled))
  }

  /// Set this menu item accelerator.
  pub fn set_accelerator<S: AsRef<str>>(&self, acccelerator: Option<S>) -> crate::Result<()> {
    let accel = acccelerator.and_then(|s| s.as_ref().parse().ok());
    run_main_thread!(self, |self_: Self| self_.inner.set_accelerator(accel))?.map_err(Into::into)
  }

  /// Get whether this check menu item is checked or not.
  pub fn is_checked(&self) -> crate::Result<bool> {
    run_main_thread!(self, |self_: Self| self_.inner.is_checked())
  }

  /// Check or Uncheck this check menu item.
  pub fn set_checked(&self, checked: bool) -> crate::Result<()> {
    run_main_thread!(self, |self_: Self| self_.inner.set_checked(checked))
  }
}

impl<R: Runtime> Resource for CheckMenuItem<R> {}
