// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Menu types and utility functions

// TODO(muda-migration): figure out js events

pub mod builders;
mod menu;
mod normal;
pub use menu::Menu;
pub use normal::MenuItem;

pub use crate::runtime::menu::{AboutMetadata, MenuEvent};
use crate::Runtime;

use crate::runtime::menu as muda;

macro_rules! getter {
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

pub(crate) use getter;

/// An enumeration of all menu item kinds that could be added to
/// a [`Menu`] or [`Submenu`]
pub enum MenuItemKind<R: Runtime> {
  /// Normal menu item
  MenuItem(MenuItem<R>),
}

impl<R: Runtime> MenuItemKind<R> {
  /// Returns a unique identifier associated with this menu item.
  pub fn id(&self) -> crate::Result<u32> {
    match self {
      MenuItemKind::MenuItem(i) => i.id(),
    }
  }

  pub(crate) fn inner(&self) -> &dyn IsMenuItem<R> {
    match self {
      MenuItemKind::MenuItem(i) => i,
    }
  }

  /// Casts this item to a [`MenuItem`], and returns `None` if it wasn't.
  pub fn as_menuitem(&self) -> Option<&MenuItem<R>> {
    match self {
      MenuItemKind::MenuItem(i) => Some(i),
    }
  }

  /// Casts this item to a [`MenuItem`], and panics if it wasn't.
  pub fn as_menuitem_unchecked(&self) -> &MenuItem<R> {
    match self {
      MenuItemKind::MenuItem(i) => i,
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
