// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(desktop)]

//! Menu types and utility functions

// TODO(muda-migration): figure out js events

mod builders;
mod check;
mod icon;
#[allow(clippy::module_inception)]
mod menu;
mod normal;
mod predefined;
mod submenu;
pub use builders::*;
pub use check::CheckMenuItem;
pub use icon::IconMenuItem;
pub use menu::{Menu, HELP_SUBMENU_ID, WINDOW_SUBMENU_ID};
pub use normal::MenuItem;
pub use predefined::PredefinedMenuItem;
pub use submenu::Submenu;

use crate::Runtime;
pub use muda::{AboutMetadata, MenuEvent, MenuId, NativeIcon};

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
  pub fn id(&self) -> &MenuId {
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

impl<R: Runtime> Clone for MenuItemKind<R> {
  fn clone(&self) -> Self {
    match self {
      Self::MenuItem(i) => Self::MenuItem(i.clone()),
      Self::Submenu(i) => Self::Submenu(i.clone()),
      Self::Predefined(i) => Self::Predefined(i.clone()),
      Self::Check(i) => Self::Check(i.clone()),
      Self::Icon(i) => Self::Icon(i.clone()),
    }
  }
}

impl<R: Runtime> sealed::IsMenuItemBase for MenuItemKind<R> {
  fn inner(&self) -> &dyn muda::IsMenuItem {
    self.inner().inner()
  }
}

impl<R: Runtime> IsMenuItem<R> for MenuItemKind<R> {
  fn kind(&self) -> MenuItemKind<R> {
    self.clone()
  }

  fn id(&self) -> &MenuId {
    self.id()
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
  fn id(&self) -> &MenuId;
}

/// A helper trait with methods to help creating a context menu.
///
/// # Safety
///
/// This trait is ONLY meant to be implemented internally by the crate.
pub trait ContextMenu: sealed::ContextMenuBase + Send + Sync {
  /// Popup this menu as a context menu on the specified window at the cursor position.
  fn popup<R: crate::Runtime>(&self, window: crate::Window<R>) -> crate::Result<()>;

  /// Popup this menu as a context menu on the specified window at the specified position.
  ///
  /// The position is relative to the window's top-left corner.
  fn popup_at<R: crate::Runtime, P: Into<crate::Position>>(
    &self,
    window: crate::Window<R>,
    position: P,
  ) -> crate::Result<()>;
}

pub(crate) mod sealed {

  pub trait IsMenuItemBase {
    fn inner(&self) -> &dyn muda::IsMenuItem;
  }

  pub trait ContextMenuBase {
    fn inner(&self) -> &dyn muda::ContextMenu;
    fn inner_owned(&self) -> Box<dyn muda::ContextMenu>;
    fn popup_inner<R: crate::Runtime, P: Into<crate::Position>>(
      &self,
      window: crate::Window<R>,
      position: Option<P>,
    ) -> crate::Result<()>;
  }
}

impl TryFrom<crate::Icon> for muda::Icon {
  type Error = crate::Error;

  fn try_from(value: crate::Icon) -> Result<Self, Self::Error> {
    let value: crate::runtime::Icon = value.try_into()?;
    muda::Icon::from_rgba(value.rgba, value.width, value.height).map_err(Into::into)
  }
}

pub(crate) fn into_logical_position<P: crate::Pixel>(
  p: crate::LogicalPosition<P>,
) -> muda::LogicalPosition<P> {
  muda::LogicalPosition { x: p.x, y: p.y }
}

pub(crate) fn into_physical_position<P: crate::Pixel>(
  p: crate::PhysicalPosition<P>,
) -> muda::PhysicalPosition<P> {
  muda::PhysicalPosition { x: p.x, y: p.y }
}

pub(crate) fn into_position(p: crate::Position) -> muda::Position {
  match p {
    crate::Position::Physical(p) => muda::Position::Physical(into_physical_position(p)),
    crate::Position::Logical(p) => muda::Position::Logical(into_logical_position(p)),
  }
}
