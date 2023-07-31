// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Menu types and utility functions

// TODO(muda-migration): add default menu
// TODO(muda-migration): figure out js events

pub mod builders;
mod check;
mod icon;
mod menu;
mod normal;
mod predefined;
mod submenu;
pub use check::CheckMenuItem;
pub use icon::IconMenuItem;
pub use menu::Menu;
pub use normal::MenuItem;
pub use predefined::PredefinedMenuItem;
pub use submenu::Submenu;

pub use crate::runtime::menu::{icon::NativeIcon, AboutMetadata, MenuEvent};
use crate::Runtime;

use crate::runtime::menu as muda;

macro_rules! run_main_thread {
  ($self:ident, $ex:expr) => {{
    use std::sync::mpsc::channel;

    let (tx, rx) = channel();
    let self_ = $self.clone();
    let task = move || {
      let _ = tx.send($ex(self_));
    };
    $self.app_handle.run_on_main_thread(Box::new(task))?;
    rx.recv().map_err(|_| crate::Error::FailedToReceiveMessage)
  }};
}

pub(crate) use run_main_thread;

/// An enumeration of all menu item kinds that could be added to
/// a [`Menu`] or [`Submenu`]
pub enum MenuItemKind<R: Runtime> {
  /// Normal menu item
  MenuItem(MenuItem<R>),
  /// Submenu menu item
  Submenu(Submenu<R>),
  /// Predefined menu item
  Predefined(PredefinedMenuItem<R>),
  /// Check menu item
  Check(CheckMenuItem<R>),
  /// Icon menu item
  Icon(IconMenuItem<R>),
}

impl<R: Runtime> MenuItemKind<R> {
  /// Returns a unique identifier associated with this menu item.
  pub fn id(&self) -> crate::Result<u32> {
    match self {
      MenuItemKind::MenuItem(i) => i.id(),
      MenuItemKind::Submenu(i) => i.id(),
      MenuItemKind::Predefined(i) => i.id(),
      MenuItemKind::Check(i) => i.id(),
      MenuItemKind::Icon(i) => i.id(),
    }
  }

  pub(crate) fn inner(&self) -> &dyn IsMenuItem<R> {
    match self {
      MenuItemKind::MenuItem(i) => i,
      MenuItemKind::Submenu(i) => i,
      MenuItemKind::Predefined(i) => i,
      MenuItemKind::Check(i) => i,
      MenuItemKind::Icon(i) => i,
    }
  }

  /// Casts this item to a [`MenuItem`], and returns `None` if it wasn't.
  pub fn as_menuitem(&self) -> Option<&MenuItem<R>> {
    match self {
      MenuItemKind::MenuItem(i) => Some(i),
      _ => None,
    }
  }

  /// Casts this item to a [`MenuItem`], and panics if it wasn't.
  pub fn as_menuitem_unchecked(&self) -> &MenuItem<R> {
    match self {
      MenuItemKind::MenuItem(i) => i,
      _ => panic!("Not a MenuItem"),
    }
  }

  /// Casts this item to a [`Submenu`], and returns `None` if it wasn't.
  pub fn as_submenu(&self) -> Option<&Submenu<R>> {
    match self {
      MenuItemKind::Submenu(i) => Some(i),
      _ => None,
    }
  }

  /// Casts this item to a [`Submenu`], and panics if it wasn't.
  pub fn as_submenu_unchecked(&self) -> &Submenu<R> {
    match self {
      MenuItemKind::Submenu(i) => i,
      _ => panic!("Not a Submenu"),
    }
  }

  /// Casts this item to a [`PredefinedMenuItem`], and returns `None` if it wasn't.
  pub fn as_predefined_menuitem(&self) -> Option<&PredefinedMenuItem<R>> {
    match self {
      MenuItemKind::Predefined(i) => Some(i),
      _ => None,
    }
  }

  /// Casts this item to a [`PredefinedMenuItem`], and panics if it wasn't.
  pub fn as_predefined_menuitem_unchecked(&self) -> &PredefinedMenuItem<R> {
    match self {
      MenuItemKind::Predefined(i) => i,
      _ => panic!("Not a PredefinedMenuItem"),
    }
  }

  /// Casts this item to a [`CheckMenuItem`], and returns `None` if it wasn't.
  pub fn as_check_menuitem(&self) -> Option<&CheckMenuItem<R>> {
    match self {
      MenuItemKind::Check(i) => Some(i),
      _ => None,
    }
  }

  /// Casts this item to a [`CheckMenuItem`], and panics if it wasn't.
  pub fn as_check_menuitem_unchecked(&self) -> &CheckMenuItem<R> {
    match self {
      MenuItemKind::Check(i) => i,
      _ => panic!("Not a CheckMenuItem"),
    }
  }

  /// Casts this item to a [`IconMenuItem`], and returns `None` if it wasn't.
  pub fn as_icon_menuitem(&self) -> Option<&IconMenuItem<R>> {
    match self {
      MenuItemKind::Icon(i) => Some(i),
      _ => None,
    }
  }

  /// Casts this item to a [`IconMenuItem`], and panics if it wasn't.
  pub fn as_icon_menuitem_unchecked(&self) -> &IconMenuItem<R> {
    match self {
      MenuItemKind::Icon(i) => i,
      _ => panic!("Not an IconMenuItem"),
    }
  }
}

/// A trait that defines a generic item in a menu, which may be one of [`MenuItemKind`]
///
/// # Safety
///
/// This trait is ONLY meant to be implemented internally by the crate.
pub unsafe trait IsMenuItem<R: Runtime>: sealed::IsMenuItemBase {
  /// Returns the kind of this menu item.
  fn kind(&self) -> MenuItemKind<R>;

  /// Returns a unique identifier associated with this menu.
  fn id(&self) -> crate::Result<u32> {
    self.kind().id()
  }
}

/// A helper trait with methods to help creating a context menu.
///
/// # Safety
///
/// This trait is ONLY meant to be implemented internally by the crate.
pub unsafe trait ContextMenu: sealed::ContextMenuBase + Sync {}

pub(crate) mod sealed {
  use crate::Position;

  pub unsafe trait IsMenuItemBase {
    fn inner(&self) -> &dyn super::muda::IsMenuItem;
  }

  pub unsafe trait ContextMenuBase {
    fn inner(&self) -> &dyn super::muda::ContextMenu;

    #[cfg(windows)]
    fn show_context_menu_for_hwnd(
      &self,
      hwnd: isize,
      position: Option<Position>,
    ) -> crate::Result<()>;

    #[cfg(linux)]
    fn show_context_menu_for_gtk_window(
      &self,
      w: &gtk::ApplicationWindow,
      position: Option<Position>,
    ) -> crate::Result<()>;

    #[cfg(target_os = "macos")]
    fn show_context_menu_for_nsview(
      &self,
      view: cocoa::base::id,
      position: Option<Position>,
    ) -> crate::Result<()>;
  }
}
