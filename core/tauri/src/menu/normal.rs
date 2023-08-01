// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{run_main_thread, runtime::menu as muda, AppHandle, Runtime};

/// A menu item inside a [`Menu`] or [`Submenu`] and contains only text.
///
/// [`Menu`]: super::Menu
/// [`Submenu`]: super::Submenu
pub struct MenuItem<R: Runtime> {
  pub(crate) inner: muda::MenuItem,
  pub(crate) app_handle: AppHandle<R>,
}

impl<R: Runtime> Clone for MenuItem<R> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      app_handle: self.app_handle.clone(),
    }
  }
}

/// # Safety
///
/// We make sure it always runs on the main thread.
unsafe impl<R: Runtime> Sync for MenuItem<R> {}
unsafe impl<R: Runtime> Send for MenuItem<R> {}

impl<R: Runtime> super::sealed::IsMenuItemBase for MenuItem<R> {
  fn inner(&self) -> &dyn muda::IsMenuItem {
    &self.inner
  }
}

impl<R: Runtime> super::IsMenuItem<R> for MenuItem<R> {
  fn kind(&self) -> super::MenuItemKind<R> {
    super::MenuItemKind::MenuItem(self.clone())
  }

  fn id(&self) -> crate::Result<u32> {
    self.id()
  }
}

impl<R: Runtime> MenuItem<R> {
  /// Create a new menu item.
  ///
  /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
  /// for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn new<S: AsRef<str>>(
    app_handle: &AppHandle<R>,
    text: S,
    enabled: bool,
    acccelerator: Option<S>,
  ) -> Self {
    Self {
      inner: muda::MenuItem::new(
        text,
        enabled,
        acccelerator.and_then(|s| s.as_ref().parse().ok()),
      ),
      app_handle: app_handle.clone(),
    }
  }

  /// Returns a unique identifier associated with this menu item.
  pub fn id(&self) -> crate::Result<u32> {
    run_main_thread!(self, |self_: Self| self_.inner.id())
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
}
