// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{menu::getter, runtime::menu as muda, AppHandle, Runtime};

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

unsafe impl<R: Runtime> Sync for MenuItem<R> {}
unsafe impl<R: Runtime> Send for MenuItem<R> {}

unsafe impl<R: Runtime> super::sealed::IsMenuItemBase for MenuItem<R> {
  fn inner(&self) -> &dyn muda::IsMenuItem {
    &self.inner
  }
}

unsafe impl<R: Runtime> super::IsMenuItem<R> for MenuItem<R> {
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
        acccelerator.and_then(|s| {
          let s = s.as_ref();
          s.parse().ok()
        }),
      ),
      app_handle: app_handle.clone(),
    }
  }

  /// Returns a unique identifier associated with this menu item.
  pub fn id(&self) -> crate::Result<u32> {
    getter!(self, |self_: Self| self_.inner.id())
  }

  /// Set the text for this menu item. `text` could optionally contain
  /// an `&` before a character to assign this character as the mnemonic
  /// for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn set_text<S: AsRef<str>>(&self, text: S) -> crate::Result<()> {
    let text = text.as_ref().to_string();
    getter!(self, |self_: Self| self_.inner.set_text(text))
  }
}
