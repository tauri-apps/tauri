// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Menu types and utility functions

// TODO(muda-migration): look for a way to initalize menu for a window without routing through tauri-runtime-wry
// TODO(muda-migration): figure out js events

pub mod builders;
mod check;
mod icon;
#[allow(clippy::module_inception)]
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
  pub fn id(&self) -> u32 {
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
pub trait IsMenuItem<R: Runtime>: sealed::IsMenuItemBase {
  /// Returns the kind of this menu item.
  fn kind(&self) -> MenuItemKind<R>;

  /// Returns a unique identifier associated with this menu.
  fn id(&self) -> u32 {
    self.kind().id()
  }
}

/// A helper trait with methods to help creating a context menu.
///
/// # Safety
///
/// This trait is ONLY meant to be implemented internally by the crate.
pub trait ContextMenu: sealed::ContextMenuBase + Send + Sync {
  /// Popup this menu as a context menu on the specified window.
  fn popup<R: crate::Runtime, P: Into<crate::Position>>(
    &self,
    window: crate::Window<R>,
    position: Option<P>,
  ) -> crate::Result<()>;
}

pub(crate) mod sealed {

  pub trait IsMenuItemBase {
    fn inner(&self) -> &dyn super::muda::IsMenuItem;
  }

  pub trait ContextMenuBase {
    fn inner(&self) -> &dyn super::muda::ContextMenu;
    fn inner_owned(&self) -> Box<dyn super::muda::ContextMenu>;
  }
}
