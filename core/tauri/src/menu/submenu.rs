// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{IsMenuItem, MenuItemKind};
use crate::{run_main_thread, runtime::menu as muda, AppHandle, Position, Runtime};
use muda::ContextMenu;

/// A type that is a submenu inside a [`Menu`] or [`Submenu`]
///
/// [`Menu`]: super::Menu
/// [`Submenu`]: super::Submenu
pub struct Submenu<R: Runtime> {
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

  fn id(&self) -> crate::Result<u32> {
    self.id()
  }
}

impl<R: Runtime> super::ContextMenu for Submenu<R> {}
impl<R: Runtime> super::sealed::ContextMenuBase for Submenu<R> {
  fn inner(&self) -> &dyn muda::ContextMenu {
    &self.inner
  }

  fn inner_owned(&self) -> Box<dyn muda::ContextMenu> {
    Box::new(self.clone().inner)
  }

  #[cfg(windows)]
  fn show_context_menu_for_hwnd(
    &self,
    hwnd: isize,
    position: Option<Position>,
  ) -> crate::Result<()> {
    run_main_thread!(self, |self_: Self| {
      self_
        .inner()
        .show_context_menu_for_hwnd(hwnd, position.map(Into::into))
    })
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn show_context_menu_for_gtk_window(
    &self,
    w: &gtk::ApplicationWindow,
    position: Option<Position>,
  ) -> crate::Result<()> {
    run_main_thread!(self, |self_: Self| {
      self_
        .inner()
        .show_context_menu_for_gtk_window(w, position.map(Into::into))
    })
  }

  #[cfg(target_os = "macos")]
  fn show_context_menu_for_nsview<T: Runtime>(
    &self,
    window: crate::Window<T>,
    position: Option<Position>,
  ) -> crate::Result<()> {
    run_main_thread!(self, move |self_: Self| {
      if let Ok(view) = window.ns_view() {
        self_
          .inner()
          .show_context_menu_for_nsview(view as _, position.map(Into::into))
      }
    })
  }
}

impl<R: Runtime> Submenu<R> {
  /// Creates a new submenu.
  pub fn new<S: AsRef<str>>(app_handle: &AppHandle<R>, text: S, enabled: bool) -> Self {
    Self {
      inner: muda::Submenu::new(text, enabled),
      app_handle: app_handle.clone(),
    }
  }

  /// Creates a new menu with given `items`. It calls [`Submenu::new`] and [`Submenu::append_items`] internally.
  pub fn with_items<S: AsRef<str>>(
    app_handle: &AppHandle<R>,
    text: S,
    enabled: bool,
    items: &[&dyn IsMenuItem<R>],
  ) -> crate::Result<Self> {
    let menu = Self::new(app_handle, text, enabled);
    menu.append_items(items)?;
    Ok(menu)
  }

  pub(crate) fn inner(&self) -> &muda::Submenu {
    &self.inner
  }

  /// Returns a unique identifier associated with this submenu.
  pub fn id(&self) -> crate::Result<u32> {
    run_main_thread!(self, |self_: Self| self_.inner.id())
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
            inner: i,
            app_handle: handle.clone(),
          }),
          muda::MenuItemKind::Submenu(i) => super::MenuItemKind::Submenu(super::Submenu {
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
            inner: i,
            app_handle: handle.clone(),
          }),
          muda::MenuItemKind::Icon(i) => super::MenuItemKind::Icon(super::IconMenuItem {
            inner: i,
            app_handle: handle.clone(),
          }),
        })
        .collect::<Vec<_>>()
    })
  }
}
