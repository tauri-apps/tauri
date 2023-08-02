// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{menu::*, AppHandle, Icon, Manager, Runtime};

/// A builder type for [`Menu`]
///
/// # Example
///
/// ```no_run
/// # use tauri::menu::{*, builders::*};
/// tauri::Builder::default()
///   .setup(move |app| {
///     let handle = app.handle();
///     # let icon1 = tauri::Icon::Rgba {
///     #   rgba: Vec::new(),
///     #   width: 0,
///     #   height: 0,
///     # };
///     # let icon2 = icon1.clone();
///     let menu = Menu::new(&handle);
///     let submenu = SubmenuBuilder::new(&handle, "File")
///       .item(&MenuItem::new(&handle, "MenuItem 1", true, None))?
///       .items(&[
///         &CheckMenuItem::new(&handle, "CheckMenuItem 1", true, true, None),
///         &IconMenuItem::new(&handle, "IconMenuItem 1", true, Some(icon1), None),
///       ])?
///       .separator()?
///       .cut()?
///       .copy()?
///       .paste()?
///       .separator()?
///       .text("MenuItem 2")?
///       .check("CheckMenuItem 2")?
///       .icon("IconMenuItem 2", icon2)?
///       .build();
///     menu.append(&submenu);
///     app.set_menu(menu);
///     Ok(())
///   });
/// ```
pub struct SubmenuBuilder<R: Runtime> {
  submenu: Submenu<R>,
  app_handle: AppHandle<R>,
}

impl<R: Runtime> SubmenuBuilder<R> {
  /// Create a new menu builder.
  pub fn new<M: Manager<R>, S: AsRef<str>>(manager: &M, text: S) -> Self {
    Self {
      submenu: Submenu::new(manager, text, true),
      app_handle: manager.app_handle(),
    }
  }

  /// Set the enabled state for submenu.
  pub fn enabled(self, enabled: bool) -> crate::Result<Self> {
    self.submenu.set_enabled(enabled)?;
    Ok(self)
  }

  /// Add this item to the submenu.
  pub fn item(self, item: &dyn IsMenuItem<R>) -> crate::Result<Self> {
    self.submenu.append(item)?;
    Ok(self)
  }

  /// Add these items to the submenu.
  pub fn items(self, items: &[&dyn IsMenuItem<R>]) -> crate::Result<Self> {
    self.submenu.append_items(items)?;
    Ok(self)
  }

  /// Add a [MenuItem] to the submenu.
  pub fn text<S: AsRef<str>>(self, text: S) -> crate::Result<Self> {
    self
      .submenu
      .append(&MenuItem::new(&self.app_handle, text, true, None))?;
    Ok(self)
  }

  /// Add a [CheckMenuItem] to the submenu.
  pub fn check<S: AsRef<str>>(self, text: S) -> crate::Result<Self> {
    self.submenu.append(&CheckMenuItem::new(
      &self.app_handle,
      text,
      true,
      true,
      None,
    ))?;
    Ok(self)
  }

  /// Add an [IconMenuItem] to the submenu.
  pub fn icon<S: AsRef<str>>(self, text: S, icon: Icon) -> crate::Result<Self> {
    self.submenu.append(&IconMenuItem::new(
      &self.app_handle,
      text,
      true,
      Some(icon),
      None,
    ))?;
    Ok(self)
  }

  /// Add an [IconMenuItem] with a native icon to the submenu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux**: Unsupported.
  pub fn native_icon<S: AsRef<str>>(self, text: S, icon: NativeIcon) -> crate::Result<Self> {
    self.submenu.append(&IconMenuItem::with_native_icon(
      &self.app_handle,
      text,
      true,
      Some(icon),
      None,
    ))?;
    Ok(self)
  }

  /// Add Separator menu item to the submenu.
  pub fn separator(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::separator(&self.app_handle))?;
    Ok(self)
  }

  /// Add Copy menu item to the submenu.
  pub fn copy(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::copy(&self.app_handle, None))?;
    Ok(self)
  }

  /// Add Cut menu item to the submenu.
  pub fn cut(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::cut(&self.app_handle, None))?;
    Ok(self)
  }

  /// Add Paste menu item to the submenu.
  pub fn paste(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::paste(&self.app_handle, None))?;
    Ok(self)
  }

  /// Add SelectAll menu item to the submenu.
  pub fn select_all(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::select_all(&self.app_handle, None))?;
    Ok(self)
  }

  /// Add Undo menu item to the submenu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn undo(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::undo(&self.app_handle, None))?;
    Ok(self)
  }
  /// Add Redo menu item to the submenu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn redo(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::redo(&self.app_handle, None))?;
    Ok(self)
  }

  /// Add Minimize window menu item to the submenu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn minimize(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::minimize(&self.app_handle, None))?;
    Ok(self)
  }

  /// Add Maximize window menu item to the submenu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn maximize(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::maximize(&self.app_handle, None))?;
    Ok(self)
  }

  /// Add Fullscreen menu item to the submenu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn fullscreen(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::fullscreen(&self.app_handle, None))?;
    Ok(self)
  }

  /// Add Hide window menu item to the submenu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn hide(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::hide(&self.app_handle, None))?;
    Ok(self)
  }

  /// Add Hide other windows menu item to the submenu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn hide_others(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::hide_others(&self.app_handle, None))?;
    Ok(self)
  }

  /// Add Show all app windows menu item to the submenu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn show_all(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::show_all(&self.app_handle, None))?;
    Ok(self)
  }

  /// Add Close window menu item to the submenu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn close_window(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::close_window(&self.app_handle, None))?;
    Ok(self)
  }

  /// Add Quit app menu item to the submenu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn quit(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::quit(&self.app_handle, None))?;
    Ok(self)
  }

  /// Add About app menu item to the submenu.
  pub fn about(self, metadata: Option<AboutMetadata>) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::about(&self.app_handle, None, metadata))?;
    Ok(self)
  }

  /// Add Services menu item to the submenu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn services(self) -> crate::Result<Self> {
    self
      .submenu
      .append(&PredefinedMenuItem::services(&self.app_handle, None))?;
    Ok(self)
  }

  /// Builds this menu
  pub fn build(self) -> Submenu<R> {
    self.submenu
  }
}
