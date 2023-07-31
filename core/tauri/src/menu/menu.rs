// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{IsMenuItem, MenuItemKind};
use crate::{menu::run_main_thread, runtime::menu as muda, AppHandle, Runtime};
use muda::ContextMenu;

/// A type that is either a menu bar on the window
/// on Windows and Linux or as a global menu in the menubar on macOS.
pub struct Menu<R: Runtime> {
  pub(crate) inner: muda::Menu,
  pub(crate) app_handle: AppHandle<R>,
}

/// # Safety
///
/// We make sure it always runs on the main thread.
unsafe impl<R: Runtime> Sync for Menu<R> {}
unsafe impl<R: Runtime> Send for Menu<R> {}

impl<R: Runtime> Clone for Menu<R> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      app_handle: self.app_handle.clone(),
    }
  }
}

unsafe impl<R: Runtime> super::ContextMenu for Menu<R> {}
unsafe impl<R: Runtime> super::sealed::ContextMenuBase for Menu<R> {
  fn inner(&self) -> &dyn muda::ContextMenu {
    &self.inner
  }

  #[cfg(windows)]
  fn show_context_menu_for_hwnd(
    &self,
    hwnd: isize,
    position: Option<crate::Position>,
  ) -> crate::Result<()> {
    run_main_thread!(self, |self_: Self| self_
      .inner()
      .show_context_menu_for_hwnd(hwnd, position.map(Into::into)))
  }

  #[cfg(linux)]
  fn show_context_menu_for_gtk_window(
    &self,
    w: &gtk::ApplicationWindow,
    position: Option<Position>,
  ) -> crate::Result<()> {
    run_main_thread!(self, |self_: Self| self_
      .inner()
      .show_context_menu_for_gtk_window(w, position.map(Into::into)))
  }

  #[cfg(target_os = "macos")]
  fn show_context_menu_for_nsview(
    &self,
    view: cocoa::base::id,
    position: Option<Position>,
  ) -> crate::Result<()> {
    run_main_thread!(self, |self_: Self| self_
      .inner()
      .show_context_menu_for_ns_view(view, position.map(Into::into)))
  }
}

impl<R: Runtime> Menu<R> {
  /// Creates a new menu.
  pub fn new(app_handle: &AppHandle<R>) -> Self {
    Self {
      inner: muda::Menu::new(),
      app_handle: app_handle.clone(),
    }
  }

  /// Creates a new menu with given `items`. It calls [`Menu::new`] and [`Menu::append_items`] internally.
  pub fn with_items(
    app_handle: &AppHandle<R>,
    items: &[&dyn IsMenuItem<R>],
  ) -> crate::Result<Self> {
    let menu = Self::new(app_handle);
    menu.append_items(items)?;
    Ok(menu)
  }

  pub(crate) fn inner(&self) -> &muda::Menu {
    &self.inner
  }

  /// Returns a unique identifier associated with this menu.
  pub fn id(&self) -> crate::Result<u32> {
    run_main_thread!(self, |self_: Self| self_.inner.id())
  }

  /// Add a menu item to the end of this menu.
  ///
  /// ## Platform-spcific:
  ///
  /// - **macOS:** Only [`Submenu`] can be added to the menu.
  ///
  /// [`Submenu`]: super::Submenu
  pub fn append(&self, item: &dyn IsMenuItem<R>) -> crate::Result<()> {
    let kind = item.kind();
    run_main_thread!(self, |self_: Self| self_.inner.append(kind.inner().inner()))?
      .map_err(Into::into)
  }

  /// Add menu items to the end of this menu. It calls [`Menu::append`] in a loop internally.
  ///
  /// ## Platform-spcific:
  ///
  /// - **macOS:** Only [`Submenu`] can be added to the menu
  ///
  /// [`Submenu`]: super::Submenu
  pub fn append_items(&self, items: &[&dyn IsMenuItem<R>]) -> crate::Result<()> {
    for item in items {
      self.append(*item)?
    }

    Ok(())
  }

  /// Add a menu item to the beginning of this menu.
  ///
  /// ## Platform-spcific:
  ///
  /// - **macOS:** Only [`Submenu`] can be added to the menu
  ///
  /// [`Submenu`]: super::Submenu
  pub fn prepend(&self, item: &dyn IsMenuItem<R>) -> crate::Result<()> {
    let kind = item.kind();
    run_main_thread!(self, |self_: Self| self_
      .inner
      .prepend(kind.inner().inner()))?
    .map_err(Into::into)
  }

  /// Add menu items to the beginning of this menu. It calls [`Menu::insert_items`] with position of `0` internally.
  ///
  /// ## Platform-spcific:
  ///
  /// - **macOS:** Only [`Submenu`] can be added to the menu
  ///
  /// [`Submenu`]: super::Submenu
  pub fn prepend_items(&self, items: &[&dyn IsMenuItem<R>]) -> crate::Result<()> {
    self.insert_items(items, 0)
  }

  /// Insert a menu item at the specified `postion` in the menu.
  ///
  /// ## Platform-spcific:
  ///
  /// - **macOS:** Only [`Submenu`] can be added to the menu
  ///
  /// [`Submenu`]: super::Submenu
  pub fn insert(&self, item: &dyn IsMenuItem<R>, position: usize) -> crate::Result<()> {
    let kind = item.kind();
    run_main_thread!(self, |self_: Self| self_
      .inner
      .insert(kind.inner().inner(), position))?
    .map_err(Into::into)
  }

  /// Insert menu items at the specified `postion` in the menu.
  ///
  /// ## Platform-spcific:
  ///
  /// - **macOS:** Only [`Submenu`] can be added to the menu
  ///
  /// [`Submenu`]: super::Submenu
  pub fn insert_items(&self, items: &[&dyn IsMenuItem<R>], position: usize) -> crate::Result<()> {
    for (i, item) in items.iter().enumerate() {
      self.insert(*item, position + i)?
    }

    Ok(())
  }

  /// Remove a menu item from this menu.
  pub fn remove(&self, item: &dyn IsMenuItem<R>) -> crate::Result<()> {
    let kind = item.kind();
    run_main_thread!(self, |self_: Self| self_.inner.remove(kind.inner().inner()))?
      .map_err(Into::into)
  }

  /// Returns a list of menu items that has been added to this menu.
  pub fn items(&self) -> crate::Result<Vec<MenuItemKind<R>>> {
    let handle = self.app_handle.clone();
    run_main_thread!(self, |self_: Self| self_
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
      .collect::<Vec<_>>())
  }
}
