// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use super::MenuId;

/// A window menu.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Menu<I: MenuId> {
  pub items: Vec<MenuEntry<I>>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Submenu<I: MenuId> {
  pub title: String,
  pub enabled: bool,
  pub inner: Menu<I>,
}

impl<I: MenuId> Submenu<I> {
  /// Creates a new submenu with the given title and menu items.
  pub fn new<S: Into<String>>(title: S, menu: Menu<I>) -> Self {
    Self {
      title: title.into(),
      enabled: true,
      inner: menu,
    }
  }
}

impl<I: MenuId> Default for Menu<I> {
  fn default() -> Self {
    Self { items: Vec::new() }
  }
}

impl<I: MenuId> Menu<I> {
  /// Creates a new window menu.
  pub fn new() -> Self {
    Default::default()
  }

  /// Adds the custom menu item to the menu.
  pub fn add_item(mut self, item: CustomMenuItem<I>) -> Self {
    self.items.push(MenuEntry::CustomItem(item));
    self
  }

  /// Adds a native item to the menu.
  pub fn add_native_item(mut self, item: MenuItem) -> Self {
    self.items.push(MenuEntry::NativeItem(item));
    self
  }

  /// Adds an entry with submenu.
  pub fn add_submenu(mut self, submenu: Submenu<I>) -> Self {
    self.items.push(MenuEntry::Submenu(submenu));
    self
  }
}

/// A custom menu item.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CustomMenuItem<I: MenuId> {
  pub id: I,
  pub title: String,
  pub keyboard_accelerator: Option<String>,
  pub enabled: bool,
  pub selected: bool,
}

impl<I: MenuId> CustomMenuItem<I> {
  /// Create new custom menu item.
  pub fn new<T: Into<String>>(id: I, title: T) -> Self {
    Self {
      id,
      title: title.into(),
      keyboard_accelerator: None,
      enabled: true,
      selected: false,
    }
  }

  #[doc(hidden)]
  pub fn id_value(&self) -> u32 {
    let mut s = DefaultHasher::new();
    self.id.hash(&mut s);
    s.finish() as u32
  }
}

/// A system tray menu.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SystemTrayMenu<I: MenuId> {
  pub items: Vec<SystemTrayMenuEntry<I>>,
}

impl<I: MenuId> Default for SystemTrayMenu<I> {
  fn default() -> Self {
    Self { items: Vec::new() }
  }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SystemTraySubmenu<I: MenuId> {
  pub title: String,
  pub enabled: bool,
  pub inner: SystemTrayMenu<I>,
}

impl<I: MenuId> SystemTraySubmenu<I> {
  /// Creates a new submenu with the given title and menu items.
  pub fn new<S: Into<String>>(title: S, menu: SystemTrayMenu<I>) -> Self {
    Self {
      title: title.into(),
      enabled: true,
      inner: menu,
    }
  }
}

impl<I: MenuId> SystemTrayMenu<I> {
  /// Creates a new system tray menu.
  pub fn new() -> Self {
    Default::default()
  }

  /// Adds the custom menu item to the system tray menu.
  pub fn add_item(mut self, item: CustomMenuItem<I>) -> Self {
    self.items.push(SystemTrayMenuEntry::CustomItem(item));
    self
  }

  /// Adds a native item to the system tray menu.
  pub fn add_native_item(mut self, item: SystemTrayMenuItem) -> Self {
    self.items.push(SystemTrayMenuEntry::NativeItem(item));
    self
  }

  /// Adds an entry with submenu.
  pub fn add_submenu(mut self, submenu: SystemTraySubmenu<I>) -> Self {
    self.items.push(SystemTrayMenuEntry::Submenu(submenu));
    self
  }
}

/// An entry on the system tray menu.
#[derive(Debug, Clone)]
pub enum SystemTrayMenuEntry<I: MenuId> {
  /// A custom item.
  CustomItem(CustomMenuItem<I>),
  /// A native item.
  NativeItem(SystemTrayMenuItem),
  /// An entry with submenu.
  Submenu(SystemTraySubmenu<I>),
}

/// System tray menu item.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SystemTrayMenuItem {
  /// A separator.
  Separator,
}

/// An entry on the system tray menu.
#[derive(Debug, Clone)]
pub enum MenuEntry<I: MenuId> {
  /// A custom item.
  CustomItem(CustomMenuItem<I>),
  /// A native item.
  NativeItem(MenuItem),
  /// An entry with submenu.
  Submenu(Submenu<I>),
}

/// A menu item, bound to a pre-defined action or `Custom` emit an event. Note that status bar only
/// supports `Custom` menu item variants. And on the menu bar, some platforms might not support some
/// of the variants. Unsupported variant will be no-op on such platform.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MenuItem {
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
