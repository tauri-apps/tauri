// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{image::Image, menu::*, Manager, Runtime};

/// A builder type for [`Menu`]
///
/// # Example
///
/// ```no_run
/// use tauri::menu::*;
/// tauri::Builder::default()
///   .setup(move |app| {
///     let handle = app.handle();
///     # let icon1 = tauri::image::Image::new(&[], 0, 0);
///     let menu = MenuBuilder::new(handle)
///       .item(&MenuItem::new(handle, "MenuItem 1", true, None::<&str>)?)
///       .items(&[
///         &CheckMenuItem::new(handle, "CheckMenuItem 1", true, true, None::<&str>)?,
///         &IconMenuItem::new(handle, "IconMenuItem 1", true, Some(icon1), None::<&str>)?,
///       ])
///       .separator()
///       .cut()
///       .copy()
///       .paste()
///       .separator()
///       .text("item2", "MenuItem 2")
///       .check("checkitem2", "CheckMenuItem 2")
///       .icon("iconitem2", "IconMenuItem 2", app.default_window_icon().cloned().unwrap())
///       .build()?;
///     app.set_menu(menu);
///     Ok(())
///   });
/// ```
pub struct MenuBuilder<'m, R: Runtime, M: Manager<R>> {
  id: Option<MenuId>,
  manager: &'m M,
  items: Vec<crate::Result<MenuItemKind<R>>>,
}

impl<'m, R: Runtime, M: Manager<R>> MenuBuilder<'m, R, M> {
  /// Create a new menu builder.
  pub fn new(manager: &'m M) -> Self {
    Self {
      id: None,
      items: Vec::new(),
      manager,
    }
  }

  /// Create a new menu builder with the specified id.
  pub fn with_id<I: Into<MenuId>>(manager: &'m M, id: I) -> Self {
    Self {
      id: Some(id.into()),
      items: Vec::new(),
      manager,
    }
  }

  /// Set the id for this menu.
  pub fn id<I: Into<MenuId>>(mut self, id: I) -> Self {
    self.id.replace(id.into());
    self
  }

  /// Add this item to the menu.
  pub fn item(mut self, item: &dyn IsMenuItem<R>) -> Self {
    self.items.push(Ok(item.kind()));
    self
  }

  /// Add these items to the menu.
  pub fn items(mut self, items: &[&dyn IsMenuItem<R>]) -> Self {
    for item in items {
      self = self.item(*item);
    }
    self
  }

  /// Add a [MenuItem] to the menu.
  pub fn text<I: Into<MenuId>, S: AsRef<str>>(mut self, id: I, text: S) -> Self {
    self
      .items
      .push(MenuItem::with_id(self.manager, id, text, true, None::<&str>).map(|i| i.kind()));
    self
  }

  /// Add a [CheckMenuItem] to the menu.
  pub fn check<I: Into<MenuId>, S: AsRef<str>>(mut self, id: I, text: S) -> Self {
    self.items.push(
      CheckMenuItem::with_id(self.manager, id, text, true, true, None::<&str>).map(|i| i.kind()),
    );
    self
  }

  /// Add an [IconMenuItem] to the menu.
  pub fn icon<I: Into<MenuId>, S: AsRef<str>>(mut self, id: I, text: S, icon: Image<'_>) -> Self {
    self.items.push(
      IconMenuItem::with_id(self.manager, id, text, true, Some(icon), None::<&str>)
        .map(|i| i.kind()),
    );
    self
  }

  /// Add an [IconMenuItem] with a native icon to the menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux**: Unsupported.
  pub fn native_icon<I: Into<MenuId>, S: AsRef<str>>(
    mut self,
    id: I,
    text: S,
    icon: NativeIcon,
  ) -> Self {
    self.items.push(
      IconMenuItem::with_id_and_native_icon(self.manager, id, text, true, Some(icon), None::<&str>)
        .map(|i| i.kind()),
    );
    self
  }

  /// Add Separator menu item to the menu.
  pub fn separator(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::separator(self.manager).map(|i| i.kind()));
    self
  }

  /// Add Copy menu item to the menu.
  pub fn copy(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::copy(self.manager, None).map(|i| i.kind()));
    self
  }

  /// Add Cut menu item to the menu.
  pub fn cut(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::cut(self.manager, None).map(|i| i.kind()));
    self
  }

  /// Add Paste menu item to the menu.
  pub fn paste(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::paste(self.manager, None).map(|i| i.kind()));
    self
  }

  /// Add SelectAll menu item to the menu.
  pub fn select_all(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::select_all(self.manager, None).map(|i| i.kind()));
    self
  }

  /// Add Undo menu item to the menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn undo(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::undo(self.manager, None).map(|i| i.kind()));
    self
  }
  /// Add Redo menu item to the menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn redo(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::redo(self.manager, None).map(|i| i.kind()));
    self
  }

  /// Add Minimize window menu item to the menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn minimize(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::minimize(self.manager, None).map(|i| i.kind()));
    self
  }

  /// Add Maximize window menu item to the menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn maximize(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::maximize(self.manager, None).map(|i| i.kind()));
    self
  }

  /// Add Fullscreen menu item to the menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn fullscreen(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::fullscreen(self.manager, None).map(|i| i.kind()));
    self
  }

  /// Add Hide window menu item to the menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn hide(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::hide(self.manager, None).map(|i| i.kind()));
    self
  }

  /// Add Hide other windows menu item to the menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn hide_others(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::hide_others(self.manager, None).map(|i| i.kind()));
    self
  }

  /// Add Show all app windows menu item to the menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn show_all(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::show_all(self.manager, None).map(|i| i.kind()));
    self
  }

  /// Add Close window menu item to the menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn close_window(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::close_window(self.manager, None).map(|i| i.kind()));
    self
  }

  /// Add Quit app menu item to the menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn quit(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::quit(self.manager, None).map(|i| i.kind()));
    self
  }

  /// Add About app menu item to the menu.
  pub fn about(mut self, metadata: Option<AboutMetadata<'_>>) -> Self {
    self
      .items
      .push(PredefinedMenuItem::about(self.manager, None, metadata).map(|i| i.kind()));
    self
  }

  /// Add Services menu item to the menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn services(mut self) -> Self {
    self
      .items
      .push(PredefinedMenuItem::services(self.manager, None).map(|i| i.kind()));
    self
  }

  /// Builds this menu
  pub fn build(self) -> crate::Result<Menu<R>> {
    let menu = if let Some(id) = self.id {
      Menu::with_id(self.manager, id)?
    } else {
      Menu::new(self.manager)?
    };

    for item in self.items {
      let item = item?;
      menu.append(&item)?;
    }

    Ok(menu)
  }
}
