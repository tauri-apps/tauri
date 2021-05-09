// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::hash_map::DefaultHasher,
  fmt::Debug,
  hash::{Hash, Hasher},
};

/// A type that can be derived into a menu id.
pub trait MenuId: Hash + Eq + Debug + Clone + Send + Sync + 'static {}

impl<T> MenuId for T where T: Hash + Eq + Debug + Clone + Send + Sync + 'static {}

/// A window menu.
#[derive(Debug, Clone)]
pub struct Menu<I: MenuId> {
  pub title: String,
  pub items: Vec<MenuItem<I>>,
}

impl<I: MenuId> Menu<I> {
  /// Creates a new window menu with the given title and items.
  pub fn new<T: Into<String>>(title: T, items: Vec<MenuItem<I>>) -> Self {
    Self {
      title: title.into(),
      items,
    }
  }
}

/// A custom menu item.
#[derive(Debug, Clone)]
pub struct CustomMenuItem<I: MenuId> {
  pub id: I,
  pub name: String,
}

impl<I: MenuId> CustomMenuItem<I> {
  /// Create new custom menu item.
  pub fn new<T: Into<String>>(id: I, title: T) -> Self {
    let title = title.into();
    Self { id, name: title }
  }

  #[doc(hidden)]
  pub fn id_value(&self) -> u32 {
    let mut s = DefaultHasher::new();
    self.id.hash(&mut s);
    s.finish() as u32
  }
}

/// System tray menu item.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SystemTrayMenuItem<I: MenuId> {
  /// A custom menu item.
  Custom(CustomMenuItem<I>),
  /// A separator.
  Separator,
}

/// A menu item, bound to a pre-defined action or `Custom` emit an event. Note that status bar only
/// supports `Custom` menu item variants. And on the menu bar, some platforms might not support some
/// of the variants. Unsupported variant will be no-op on such platform.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MenuItem<I: MenuId> {
  /// A custom menu item..
  Custom(CustomMenuItem<I>),

  /// Shows a standard "About" item
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Android / iOS:** Unsupported
  ///
  About(String),

  /// A standard "hide the app" menu item.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Android / iOS:** Unsupported
  ///
  Hide,

  /// A standard "Services" menu item.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Linux / Android / iOS:** Unsupported
  ///
  Services,

  /// A "hide all other windows" menu item.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Linux / Android / iOS:** Unsupported
  ///
  HideOthers,

  /// A menu item to show all the windows for this app.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Linux / Android / iOS:** Unsupported
  ///
  ShowAll,

  /// Close the current window.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Android / iOS:** Unsupported
  ///
  CloseWindow,

  /// A "quit this app" menu icon.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Android / iOS:** Unsupported
  ///
  Quit,

  /// A menu item for enabling copying (often text) from responders.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Android / iOS:** Unsupported
  ///
  Copy,

  /// A menu item for enabling cutting (often text) from responders.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Android / iOS:** Unsupported
  ///
  Cut,

  /// An "undo" menu item; particularly useful for supporting the cut/copy/paste/undo lifecycle
  /// of events.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Linux / Android / iOS:** Unsupported
  ///
  Undo,

  /// An "redo" menu item; particularly useful for supporting the cut/copy/paste/undo lifecycle
  /// of events.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Linux / Android / iOS:** Unsupported
  ///
  Redo,

  /// A menu item for selecting all (often text) from responders.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Android / iOS:** Unsupported
  ///
  SelectAll,

  /// A menu item for pasting (often text) into responders.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Android / iOS:** Unsupported
  ///
  Paste,

  /// A standard "enter full screen" item.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Linux / Android / iOS:** Unsupported
  ///
  EnterFullScreen,

  /// An item for minimizing the window with the standard system controls.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Android / iOS:** Unsupported
  ///
  Minimize,

  /// An item for instructing the app to zoom
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Linux / Android / iOS:** Unsupported
  ///
  Zoom,

  /// Represents a Separator
  ///
  /// ## Platform-specific
  ///
  /// - **Windows / Android / iOS:** Unsupported
  ///
  Separator,
}
