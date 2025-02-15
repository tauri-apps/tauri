// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use super::run_item_main_thread;
use super::Submenu;
use super::{sealed::ContextMenuBase, IsMenuItem, MenuItemKind};
use crate::menu::SubmenuInner;
use crate::run_main_thread;
use crate::{AppHandle, Manager, Position, Runtime, Window};
use muda::{ContextMenu, MenuId};

impl<R: Runtime> super::ContextMenu for Submenu<R> {
  #[cfg(target_os = "windows")]
  fn hpopupmenu(&self) -> crate::Result<isize> {
    run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().hpopupmenu())
  }

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

impl<R: Runtime> ContextMenuBase for Submenu<R> {
  fn popup_inner<T: Runtime, P: Into<crate::Position>>(
    &self,
    window: crate::Window<T>,
    position: Option<P>,
  ) -> crate::Result<()> {
    let position = position.map(Into::into);
    run_item_main_thread!(self, move |self_: Self| {
      #[cfg(target_os = "macos")]
      if let Ok(view) = window.ns_view() {
        unsafe {
          self_
            .inner()
            .show_context_menu_for_nsview(view as _, position);
        }
      }

      #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
      ))]
      if let Ok(w) = window.gtk_window() {
        self_
          .inner()
          .show_context_menu_for_gtk_window(w.as_ref(), position);
      }

      #[cfg(windows)]
      if let Ok(hwnd) = window.hwnd() {
        unsafe {
          self_
            .inner()
            .show_context_menu_for_hwnd(hwnd.0 as _, position)
        }
      }
    })
  }

  fn inner_context(&self) -> &dyn muda::ContextMenu {
    (*self.0).as_ref()
  }

  fn inner_context_owned(&self) -> Box<dyn muda::ContextMenu> {
    Box::new((*self.0).as_ref().clone())
  }
}

impl<R: Runtime> Submenu<R> {
  /// Creates a new submenu.
  pub fn new<M: Manager<R>, S: AsRef<str>>(
    manager: &M,
    text: S,
    enabled: bool,
  ) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.as_ref().to_owned();

    let submenu = run_main_thread!(handle, || {
      let submenu = muda::Submenu::new(text, enabled);
      SubmenuInner {
        id: submenu.id().clone(),
        inner: Some(submenu),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(submenu)))
  }

  /// Creates a new submenu with the specified id.
  pub fn with_id<M: Manager<R>, I: Into<MenuId>, S: AsRef<str>>(
    manager: &M,
    id: I,
    text: S,
    enabled: bool,
  ) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let id = id.into();
    let text = text.as_ref().to_owned();

    let submenu = run_main_thread!(handle, || {
      let submenu = muda::Submenu::with_id(id.clone(), text, enabled);
      SubmenuInner {
        id,
        inner: Some(submenu),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(submenu)))
  }

  /// Creates a new menu with given `items`. It calls [`Submenu::new`] and [`Submenu::append_items`] internally.
  pub fn with_items<M: Manager<R>, S: AsRef<str>>(
    manager: &M,
    text: S,
    enabled: bool,
    items: &[&dyn IsMenuItem<R>],
  ) -> crate::Result<Self> {
    let menu = Self::new(manager, text, enabled)?;
    menu.append_items(items)?;
    Ok(menu)
  }

  /// Creates a new menu with the specified id and given `items`.
  /// It calls [`Submenu::new`] and [`Submenu::append_items`] internally.
  pub fn with_id_and_items<M: Manager<R>, I: Into<MenuId>, S: AsRef<str>>(
    manager: &M,
    id: I,
    text: S,
    enabled: bool,
    items: &[&dyn IsMenuItem<R>],
  ) -> crate::Result<Self> {
    let menu = Self::with_id(manager, id, text, enabled)?;
    menu.append_items(items)?;
    Ok(menu)
  }

  pub(crate) fn inner(&self) -> &muda::Submenu {
    (*self.0).as_ref()
  }

  /// The application handle associated with this type.
  pub fn app_handle(&self) -> &AppHandle<R> {
    &self.0.app_handle
  }

  /// Returns a unique identifier associated with this submenu.
  pub fn id(&self) -> &MenuId {
    &self.0.id
  }

  /// Add a menu item to the end of this submenu.
  pub fn append(&self, item: &dyn IsMenuItem<R>) -> crate::Result<()> {
    let kind = item.kind();
    run_item_main_thread!(self, |self_: Self| {
      (*self_.0).as_ref().append(kind.inner().inner_muda())
    })?
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
    run_item_main_thread!(self, |self_: Self| {
      (*self_.0).as_ref().prepend(kind.inner().inner_muda())
    })?
    .map_err(Into::into)
  }

  /// Add menu items to the beginning of this submenu. It calls [`Submenu::insert_items`] with position of `0` internally.
  pub fn prepend_items(&self, items: &[&dyn IsMenuItem<R>]) -> crate::Result<()> {
    self.insert_items(items, 0)
  }

  /// Insert a menu item at the specified `position` in this submenu.
  pub fn insert(&self, item: &dyn IsMenuItem<R>, position: usize) -> crate::Result<()> {
    let kind = item.kind();
    run_item_main_thread!(self, |self_: Self| {
      (*self_.0)
        .as_ref()
        .insert(kind.inner().inner_muda(), position)
    })?
    .map_err(Into::into)
  }

  /// Insert menu items at the specified `position` in this submenu.
  pub fn insert_items(&self, items: &[&dyn IsMenuItem<R>], position: usize) -> crate::Result<()> {
    for (i, item) in items.iter().enumerate() {
      self.insert(*item, position + i)?
    }

    Ok(())
  }

  /// Remove a menu item from this submenu.
  pub fn remove(&self, item: &dyn IsMenuItem<R>) -> crate::Result<()> {
    let kind = item.kind();
    run_item_main_thread!(self, |self_: Self| {
      (*self_.0).as_ref().remove(kind.inner().inner_muda())
    })?
    .map_err(Into::into)
  }

  /// Remove the menu item at the specified position from this submenu and returns it.
  pub fn remove_at(&self, position: usize) -> crate::Result<Option<MenuItemKind<R>>> {
    run_item_main_thread!(self, |self_: Self| {
      (*self_.0)
        .as_ref()
        .remove_at(position)
        .map(|i| MenuItemKind::from_muda(self_.0.app_handle.clone(), i))
    })
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

  /// Returns a list of menu items that has been added to this submenu.
  pub fn items(&self) -> crate::Result<Vec<MenuItemKind<R>>> {
    run_item_main_thread!(self, |self_: Self| {
      (*self_.0)
        .as_ref()
        .items()
        .into_iter()
        .map(|i| MenuItemKind::from_muda(self_.0.app_handle.clone(), i))
        .collect::<Vec<_>>()
    })
  }

  /// Get the text for this submenu.
  pub fn text(&self) -> crate::Result<String> {
    run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().text())
  }

  /// Set the text for this submenu. `text` could optionally contain
  /// an `&` before a character to assign this character as the mnemonic
  /// for this submenu. To display a `&` without assigning a mnemonic, use `&&`.
  pub fn set_text<S: AsRef<str>>(&self, text: S) -> crate::Result<()> {
    let text = text.as_ref().to_string();
    run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().set_text(text))
  }

  /// Get whether this submenu is enabled or not.
  pub fn is_enabled(&self) -> crate::Result<bool> {
    run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().is_enabled())
  }

  /// Enable or disable this submenu.
  pub fn set_enabled(&self, enabled: bool) -> crate::Result<()> {
    run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().set_enabled(enabled))
  }

  /// Set this submenu as the Window menu for the application on macOS.
  ///
  /// This will cause macOS to automatically add window-switching items and
  /// certain other items to the menu.
  #[cfg(target_os = "macos")]
  pub fn set_as_windows_menu_for_nsapp(&self) -> crate::Result<()> {
    run_item_main_thread!(self, |self_: Self| {
      (*self_.0).as_ref().set_as_windows_menu_for_nsapp()
    })?;
    Ok(())
  }

  /// Set this submenu as the Help menu for the application on macOS.
  ///
  /// This will cause macOS to automatically add a search box to the menu.
  ///
  /// If no menu is set as the Help menu, macOS will automatically use any menu
  /// which has a title matching the localized word "Help".
  #[cfg(target_os = "macos")]
  pub fn set_as_help_menu_for_nsapp(&self) -> crate::Result<()> {
    run_item_main_thread!(self, |self_: Self| {
      (*self_.0).as_ref().set_as_help_menu_for_nsapp()
    })?;
    Ok(())
  }
}
