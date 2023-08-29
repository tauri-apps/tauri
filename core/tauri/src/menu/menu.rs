// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::sealed::ContextMenuBase;
use super::{AboutMetadata, IsMenuItem, MenuItemKind, PredefinedMenuItem, Submenu};
use crate::resources::Resource;
use crate::Window;
use crate::{run_main_thread, AppHandle, Manager, Position, Runtime};
use muda::ContextMenu;
use muda::MenuId;

/// Expected submenu id of the Window menu for macOS.
pub const WINDOW_SUBMENU_ID: &str = "__tauri_window_menu__";
/// Expected submenu id of the Help menu for macOS.
pub const HELP_SUBMENU_ID: &str = "__tauri_help_menu__";

/// A type that is either a menu bar on the window
/// on Windows and Linux or as a global menu in the menubar on macOS.
pub struct Menu<R: Runtime> {
  pub(crate) id: MenuId,
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
      id: self.id.clone(),
      inner: self.inner.clone(),
      app_handle: self.app_handle.clone(),
    }
  }
}

impl<R: Runtime> super::ContextMenu for Menu<R> {
  fn popup<T: Runtime>(&self, window: Window<T>) -> crate::Result<()> {
    self.popup_inner(window, None::<Position>)
  }

  fn popup_at<T: Runtime, P: Into<Position>>(
    &self,
    window: Window<T>,
    position: P,
  ) -> crate::Result<()> {
    self.popup_inner(window, Some(position))
  }
}

impl<R: Runtime> ContextMenuBase for Menu<R> {
  fn popup_inner<T: Runtime, P: Into<crate::Position>>(
    &self,
    window: crate::Window<T>,
    position: Option<P>,
  ) -> crate::Result<()> {
    let position = position.map(Into::into).map(super::into_position);
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
  fn inner_context(&self) -> &dyn muda::ContextMenu {
    &self.inner
  }

  fn inner_context_owned(&self) -> Box<dyn muda::ContextMenu> {
    Box::new(self.inner.clone())
  }
}

impl<R: Runtime> Menu<R> {
  /// Creates a new menu.
  pub fn new<M: Manager<R>>(manager: &M) -> Self {
    let menu = muda::Menu::new();
    Self {
      id: menu.id().clone(),
      inner: menu,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Creates a new menu with the specified id.
  pub fn with_id<M: Manager<R>, I: Into<MenuId>>(manager: &M, id: I) -> Self {
    let menu = muda::Menu::with_id(id);
    Self {
      id: menu.id().clone(),
      inner: menu,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Creates a new menu with given `items`. It calls [`Menu::new`] and [`Menu::append_items`] internally.
  pub fn with_items<M: Manager<R>>(
    manager: &M,
    items: &[&dyn IsMenuItem<R>],
  ) -> crate::Result<Self> {
    let menu = Self::new(manager);
    menu.append_items(items)?;
    Ok(menu)
  }

  /// Creates a new menu with the specified id and given `items`.
  /// It calls [`Menu::new`] and [`Menu::append_items`] internally.
  pub fn with_id_and_items<M: Manager<R>, I: Into<MenuId>>(
    manager: &M,
    id: I,
    items: &[&dyn IsMenuItem<R>],
  ) -> crate::Result<Self> {
    let menu = Self::with_id(manager, id);
    menu.append_items(items)?;
    Ok(menu)
  }

  /// Creates a menu filled with default menu items and submenus.
  pub fn default(app_handle: &AppHandle<R>) -> crate::Result<Self> {
    let pkg_info = app_handle.package_info();
    let config = app_handle.config();
    let about_metadata = AboutMetadata {
      name: Some(pkg_info.name.clone()),
      version: Some(pkg_info.version.to_string()),
      copyright: config.tauri.bundle.copyright.clone(),
      authors: config.tauri.bundle.publisher.clone().map(|p| vec![p]),
      ..Default::default()
    };

    let window_menu = Submenu::with_id_and_items(
      app_handle,
      WINDOW_SUBMENU_ID,
      "Window",
      true,
      &[
        &PredefinedMenuItem::minimize(app_handle, None),
        &PredefinedMenuItem::maximize(app_handle, None),
        #[cfg(target_os = "macos")]
        &PredefinedMenuItem::separator(app_handle),
        &PredefinedMenuItem::close_window(app_handle, None),
      ],
    )?;

    let help_menu = Submenu::with_id_and_items(
      app_handle,
      HELP_SUBMENU_ID,
      "Help",
      true,
      &[
        #[cfg(not(target_os = "macos"))]
        &PredefinedMenuItem::about(app_handle, None, Some(about_metadata)),
      ],
    )?;

    let menu = Menu::with_items(
      app_handle,
      &[
        #[cfg(target_os = "macos")]
        &Submenu::with_items(
          app_handle,
          pkg_info.name.clone(),
          true,
          &[
            &PredefinedMenuItem::about(app_handle, None, Some(about_metadata)),
            &PredefinedMenuItem::separator(app_handle),
            &PredefinedMenuItem::services(app_handle, None),
            &PredefinedMenuItem::separator(app_handle),
            &PredefinedMenuItem::hide(app_handle, None),
            &PredefinedMenuItem::hide_others(app_handle, None),
            &PredefinedMenuItem::separator(app_handle),
            &PredefinedMenuItem::quit(app_handle, None),
          ],
        )?,
        #[cfg(not(any(
          target_os = "linux",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "netbsd",
          target_os = "openbsd"
        )))]
        &Submenu::with_items(
          app_handle,
          "File",
          true,
          &[
            &PredefinedMenuItem::close_window(app_handle, None),
            #[cfg(not(target_os = "macos"))]
            &PredefinedMenuItem::quit(app_handle, None),
          ],
        )?,
        &Submenu::with_items(
          app_handle,
          "Edit",
          true,
          &[
            &PredefinedMenuItem::undo(app_handle, None),
            &PredefinedMenuItem::redo(app_handle, None),
            &PredefinedMenuItem::separator(app_handle),
            &PredefinedMenuItem::cut(app_handle, None),
            &PredefinedMenuItem::copy(app_handle, None),
            &PredefinedMenuItem::paste(app_handle, None),
            &PredefinedMenuItem::select_all(app_handle, None),
          ],
        )?,
        #[cfg(target_os = "macos")]
        &Submenu::with_items(
          app_handle,
          "View",
          true,
          &[&PredefinedMenuItem::fullscreen(app_handle, None)],
        )?,
        &window_menu,
        &help_menu,
      ],
    )?;

    Ok(menu)
  }

  pub(crate) fn inner(&self) -> &muda::Menu {
    &self.inner
  }

  /// The application handle associated with this type.
  pub fn app_handle(&self) -> &AppHandle<R> {
    &self.app_handle
  }

  /// Returns a unique identifier associated with this menu.
  pub fn id(&self) -> &MenuId {
    &self.id
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
    run_main_thread!(self, |self_: Self| self_
      .inner
      .append(kind.inner().inner_muda()))?
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
      .prepend(kind.inner().inner_muda()))?
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
      .insert(kind.inner().inner_muda(), position))?
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
    run_main_thread!(self, |self_: Self| self_
      .inner
      .remove(kind.inner().inner_muda()))?
    .map_err(Into::into)
  }

  /// Remove the menu item at the specified position from this menu and returns it.
  pub fn remove_at(&self, position: usize) -> crate::Result<Option<MenuItemKind<R>>> {
    run_main_thread!(self, |self_: Self| self_
      .inner
      .remove_at(position)
      .map(|i| MenuItemKind::from_muda(self_.app_handle.clone(), i)))
  }

  /// Retrieves the menu item matching the given identifier.
  pub fn get<'a, I>(&self, id: &'a I) -> Option<MenuItemKind<R>>
  where
    I: ?Sized,
    MenuId: PartialEq<&'a I>,
  {
    self
      .items()
      .unwrap_or_default()
      .into_iter()
      .find(|i| i.id() == &id)
  }

  /// Returns a list of menu items that has been added to this menu.
  pub fn items(&self) -> crate::Result<Vec<MenuItemKind<R>>> {
    run_main_thread!(self, |self_: Self| self_
      .inner
      .items()
      .into_iter()
      .map(|i| MenuItemKind::from_muda(self_.app_handle.clone(), i))
      .collect::<Vec<_>>())
  }

  /// Set this menu as the application menu.
  ///
  /// This is an alias for [`AppHandle::set_menu`].
  pub fn set_as_app_menu(&self) -> crate::Result<Option<Menu<R>>> {
    self.app_handle.set_menu(self.clone())
  }

  /// Set this menu as the window menu.
  ///
  /// This is an alias for [`Window::set_menu`].
  pub fn set_as_window_menu(&self, window: &Window<R>) -> crate::Result<Option<Menu<R>>> {
    window.set_menu(self.clone())
  }
}

impl<R: Runtime> Resource for Menu<R> {}
