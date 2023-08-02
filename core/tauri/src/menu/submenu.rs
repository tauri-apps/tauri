// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{IsMenuItem, MenuItemKind};
use crate::{run_main_thread, runtime::menu as muda, AppHandle, Manager, Position, Runtime};
use muda::ContextMenu;

/// A type that is a submenu inside a [`Menu`] or [`Submenu`]
///
/// [`Menu`]: super::Menu
/// [`Submenu`]: super::Submenu
pub struct Submenu<R: Runtime> {
  pub(crate) id: u32,
  pub(crate) inner: muda::Submenu,
  pub(crate) app_handle: AppHandle<R>,
}

/// # Safety
///
/// We make sure it always runs on the main thread.
unsafe impl<R: Runtime> Sync for Submenu<R> {}
unsafe impl<R: Runtime> Send for Submenu<R> {}

impl<R: Runtime> Clone for Submenu<R> {
  fn clone(&self) -> Self {
    Self {
      id: self.id,
      inner: self.inner.clone(),
      app_handle: self.app_handle.clone(),
    }
  }
}

impl<R: Runtime> super::sealed::IsMenuItemBase for Submenu<R> {
  fn inner(&self) -> &dyn muda::IsMenuItem {
    &self.inner
  }
}

impl<R: Runtime> super::IsMenuItem<R> for Submenu<R> {
  fn kind(&self) -> super::MenuItemKind<R> {
    super::MenuItemKind::Submenu(self.clone())
  }

  fn id(&self) -> u32 {
    self.id
  }
}

impl<R: Runtime> super::ContextMenu for Submenu<R> {
  fn popup<T: Runtime, P: Into<Position>>(
    &self,
    window: crate::Window<T>,
    position: Option<P>,
  ) -> crate::Result<()> {
    let position = position.map(Into::into).map(Into::into);
    run_main_thread!(self, move |self_: Self| {
      #[cfg(target_os = "macos")]
      if let Ok(view) = window.ns_view() {
        self_
          .inner()
          .show_context_menu_for_nsview(view as _, position);
      }

      #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
      ))]
      if let Ok(w) = window.gtk_window() {
        self_.inner().show_context_menu_for_gtk_window(&w, position);
      }

      #[cfg(windows)]
      if let Ok(hwnd) = window.hwnd() {
        self_.inner().show_context_menu_for_hwnd(hwnd.0, position)
      }
    })
  }
}
impl<R: Runtime> super::sealed::ContextMenuBase for Submenu<R> {
  fn inner(&self) -> &dyn muda::ContextMenu {
    &self.inner
  }

  fn inner_owned(&self) -> Box<dyn muda::ContextMenu> {
    Box::new(self.clone().inner)
  }
}

impl<R: Runtime> Submenu<R> {
  /// Creates a new submenu.
  pub fn new<M: Manager<R>, S: AsRef<str>>(manager: &M, text: S, enabled: bool) -> Self {
    let submenu = muda::Submenu::new(text, enabled);
    Self {
      id: submenu.id(),
      inner: submenu,
      app_handle: manager.app_handle(),
    }
  }

  /// Creates a new menu with given `items`. It calls [`Submenu::new`] and [`Submenu::append_items`] internally.
  pub fn with_items<M: Manager<R>, S: AsRef<str>>(
    manager: &M,
    text: S,
    enabled: bool,
    items: &[&dyn IsMenuItem<R>],
  ) -> crate::Result<Self> {
    let menu = Self::new(manager, text, enabled);
    menu.append_items(items)?;
    Ok(menu)
  }

  pub(crate) fn inner(&self) -> &muda::Submenu {
    &self.inner
  }

  /// The application handle associated with this type.
  pub fn app_handle(&self) -> AppHandle<R> {
    self.app_handle.clone()
  }

  /// Returns a unique identifier associated with this submenu.
  pub fn id(&self) -> u32 {
    self.id
  }

  /// Add a menu item to the end of this submenu.
  pub fn append(&self, item: &dyn IsMenuItem<R>) -> crate::Result<()> {
    let kind = item.kind();
    run_main_thread!(self, |self_: Self| self_.inner.append(kind.inner().inner()))?
      .map_err(Into::into)
  }

  /// Add menu items to the end of this submenu. It calls [`Submenu::append`] in a loop internally.
  pub fn append_items(&self, items: &[&dyn IsMenuItem<R>]) -> crate::Result<()> {
    for item in items {
      self.append(*item)?
    }

    Ok(())
  }

  /// Add a menu item to the beginning of this submenu.
  pub fn prepend(&self, item: &dyn IsMenuItem<R>) -> crate::Result<()> {
    let kind = item.kind();
    run_main_thread!(self, |self_: Self| {
      self_.inner.prepend(kind.inner().inner())
    })?
    .map_err(Into::into)
  }

  /// Add menu items to the beginning of this submenu. It calls [`Submenu::insert_items`] with position of `0` internally.
  pub fn prepend_items(&self, items: &[&dyn IsMenuItem<R>]) -> crate::Result<()> {
    self.insert_items(items, 0)
  }

  /// Insert a menu item at the specified `postion` in this submenu.
  pub fn insert(&self, item: &dyn IsMenuItem<R>, position: usize) -> crate::Result<()> {
    let kind = item.kind();
    run_main_thread!(self, |self_: Self| {
      self_.inner.insert(kind.inner().inner(), position)
    })?
    .map_err(Into::into)
  }

  /// Insert menu items at the specified `postion` in this submenu.
  pub fn insert_items(&self, items: &[&dyn IsMenuItem<R>], position: usize) -> crate::Result<()> {
    for (i, item) in items.iter().enumerate() {
      self.insert(*item, position + i)?
    }

    Ok(())
  }

  /// Remove a menu item from this submenu.
  pub fn remove(&self, item: &dyn IsMenuItem<R>) -> crate::Result<()> {
    let kind = item.kind();
    run_main_thread!(self, |self_: Self| self_.inner.remove(kind.inner().inner()))?
      .map_err(Into::into)
  }

  /// Returns a list of menu items that has been added to this submenu.
  pub fn items(&self) -> crate::Result<Vec<MenuItemKind<R>>> {
    let handle = self.app_handle.clone();
    run_main_thread!(self, |self_: Self| {
      self_
        .inner
        .items()
        .into_iter()
        .map(|i| match i {
          muda::MenuItemKind::MenuItem(i) => super::MenuItemKind::MenuItem(super::MenuItem {
            id: i.id(),
            inner: i,
            app_handle: handle.clone(),
          }),
          muda::MenuItemKind::Submenu(i) => super::MenuItemKind::Submenu(super::Submenu {
            id: i.id(),
            inner: i,
            app_handle: handle.clone(),
          }),
          muda::MenuItemKind::Predefined(i) => {
            super::MenuItemKind::Predefined(super::PredefinedMenuItem {
              inner: i,
              app_handle: handle.clone(),
            })
          }
          muda::MenuItemKind::Check(i) => super::MenuItemKind::Check(super::CheckMenuItem {
            id: i.id(),
            inner: i,
            app_handle: handle.clone(),
          }),
          muda::MenuItemKind::Icon(i) => super::MenuItemKind::Icon(super::IconMenuItem {
            id: i.id(),
            inner: i,
            app_handle: handle.clone(),
          }),
        })
        .collect::<Vec<_>>()
    })
  }

  /// Get the text for this submenu.
  pub fn text(&self) -> crate::Result<String> {
    run_main_thread!(self, |self_: Self| self_.inner.text())
  }

  /// Set the text for this submenu. `text` could optionally contain
  /// an `&` before a character to assign this character as the mnemonic
  /// for this submenu. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn set_text<S: AsRef<str>>(&self, text: S) -> crate::Result<()> {
    let text = text.as_ref().to_string();
    run_main_thread!(self, |self_: Self| self_.inner.set_text(text))
  }

  /// Get whether this submenu is enabled or not.
  pub fn is_enabled(&self) -> crate::Result<bool> {
    run_main_thread!(self, |self_: Self| self_.inner.is_enabled())
  }

  /// Enable or disable this submenu.
  pub fn set_enabled(&self, enabled: bool) -> crate::Result<()> {
    run_main_thread!(self, |self_: Self| self_.inner.set_enabled(enabled))
  }
}
