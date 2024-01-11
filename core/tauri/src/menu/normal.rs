// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::MudaMenuItem;
use crate::{menu::MenuId, resources::Resource, AppHandle, Manager, Runtime};
use crate::{run_item_main_thread, run_main_thread};

/// A menu item inside a [`Menu`] or [`Submenu`] and contains only text.
///
/// [`Menu`]: super::Menu
/// [`Submenu`]: super::Submenu
pub struct MenuItem<R: Runtime> {
  pub(crate) id: MenuId,
  pub(crate) inner: MudaMenuItem,
  pub(crate) app_handle: AppHandle<R>,
}

impl<R: Runtime> Drop for MenuItem<R> {
  fn drop(&mut self) {
    let item = self.inner.take();
    let _ = run_item_main_thread!(self, |_: Self| { drop(item) });
  }
}

/// # Safety
///
/// We make sure it always runs on the main thread.
unsafe impl<R: Runtime> Sync for MenuItem<R> {}
unsafe impl<R: Runtime> Send for MenuItem<R> {}

impl<R: Runtime> Clone for MenuItem<R> {
  fn clone(&self) -> Self {
    Self {
      id: self.id.clone(),
      inner: self.inner.clone(),
      app_handle: self.app_handle.clone(),
    }
  }
}

impl<R: Runtime> super::sealed::IsMenuItemBase for MenuItem<R> {
  fn inner_muda(&self) -> &dyn muda::IsMenuItem {
    self.inner.as_ref()
  }
}

impl<R: Runtime> super::IsMenuItem<R> for MenuItem<R> {
  fn kind(&self) -> super::MenuItemKind<R> {
    super::MenuItemKind::MenuItem(self.clone())
  }

  fn id(&self) -> &MenuId {
    &self.id
  }
}

impl<R: Runtime> MenuItem<R> {
  /// Create a new menu item.
  ///
  /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
  /// for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
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
    let app_handle = manager.app_handle().clone();

    let text = text.as_ref().to_owned();
    let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());
    let item = run_main_thread!(
      app_handle,
      MudaMenuItem::new(muda::MenuItem::new(text, enabled, accelerator))
    )?;

    Ok(Self {
      id: item.as_ref().id().clone(),
      inner: item,
      app_handle,
    })
  }

  /// Create a new menu item with the specified id.
  ///
  /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
  /// for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
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
    let app_handle = manager.app_handle().clone();

    let id = id.into();
    let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());
    let text = text.as_ref().to_owned();
    let item = run_main_thread!(
      app_handle,
      MudaMenuItem::new(muda::MenuItem::with_id(id, text, enabled, accelerator))
    )?;

    Ok(Self {
      id: item.as_ref().id().clone(),
      inner: item,
      app_handle: manager.app_handle().clone(),
    })
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
    run_item_main_thread!(self, |self_: Self| self_.inner.as_ref().text())
  }

  /// Set the text for this menu item. `text` could optionally contain
  /// an `&` before a character to assign this character as the mnemonic
  /// for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn set_text<S: AsRef<str>>(&self, text: S) -> crate::Result<()> {
    let text = text.as_ref().to_string();
    run_item_main_thread!(self, |self_: Self| self_.inner.as_ref().set_text(text))
  }

  /// Get whether this menu item is enabled or not.
  pub fn is_enabled(&self) -> crate::Result<bool> {
    run_item_main_thread!(self, |self_: Self| self_.inner.as_ref().is_enabled())
  }

  /// Enable or disable this menu item.
  pub fn set_enabled(&self, enabled: bool) -> crate::Result<()> {
    run_item_main_thread!(self, |self_: Self| self_
      .inner
      .as_ref()
      .set_enabled(enabled))
  }

  /// Set this menu item accelerator.
  pub fn set_accelerator<S: AsRef<str>>(&self, accelerator: Option<S>) -> crate::Result<()> {
    let accel = accelerator.and_then(|s| s.as_ref().parse().ok());
    run_item_main_thread!(self, |self_: Self| self_
      .inner
      .as_ref()
      .set_accelerator(accel))?
    .map_err(Into::into)
  }
}

impl<R: Runtime> Resource for MenuItem<R> {}
