// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::AboutMetadata;
use crate::{run_main_thread, runtime::menu as muda, AppHandle, Runtime};

/// A predefined (native) menu item which has a predfined behavior by the OS or by this crate.
pub struct PredefinedMenuItem<R: Runtime> {
  pub(crate) inner: muda::PredefinedMenuItem,
  pub(crate) app_handle: AppHandle<R>,
}

impl<R: Runtime> Clone for PredefinedMenuItem<R> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      app_handle: self.app_handle.clone(),
    }
  }
}

/// # Safety
///
/// We make sure it always runs on the main thread.
unsafe impl<R: Runtime> Sync for PredefinedMenuItem<R> {}
unsafe impl<R: Runtime> Send for PredefinedMenuItem<R> {}

impl<R: Runtime> super::sealed::IsMenuItemBase for PredefinedMenuItem<R> {
  fn inner(&self) -> &dyn muda::IsMenuItem {
    &self.inner
  }
}

impl<R: Runtime> super::IsMenuItem<R> for PredefinedMenuItem<R> {
  fn kind(&self) -> super::MenuItemKind<R> {
    super::MenuItemKind::Predefined(self.clone())
  }

  fn id(&self) -> crate::Result<u32> {
    self.id()
  }
}

impl<R: Runtime> PredefinedMenuItem<R> {
  /// Separator menu item
  pub fn separator(app_handle: &AppHandle<R>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::separator(),
      app_handle: app_handle.clone(),
    }
  }

  /// Copy menu item
  pub fn copy(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::copy(text),
      app_handle: app_handle.clone(),
    }
  }

  /// Cut menu item
  pub fn cut(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::cut(text),
      app_handle: app_handle.clone(),
    }
  }

  /// Paste menu item
  pub fn paste(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::paste(text),
      app_handle: app_handle.clone(),
    }
  }

  /// SelectAll menu item
  pub fn select_all(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::select_all(text),
      app_handle: app_handle.clone(),
    }
  }

  /// Undo menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn undo(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::undo(text),
      app_handle: app_handle.clone(),
    }
  }
  /// Redo menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn redo(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::redo(text),
      app_handle: app_handle.clone(),
    }
  }

  /// Minimize window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn minimize(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::minimize(text),
      app_handle: app_handle.clone(),
    }
  }

  /// Maximize window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn maximize(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::maximize(text),
      app_handle: app_handle.clone(),
    }
  }

  /// Fullscreen menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn fullscreen(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::fullscreen(text),
      app_handle: app_handle.clone(),
    }
  }

  /// Hide window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn hide(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::hide(text),
      app_handle: app_handle.clone(),
    }
  }

  /// Hide other windows menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn hide_others(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::hide_others(text),
      app_handle: app_handle.clone(),
    }
  }

  /// Show all app windows menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn show_all(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::show_all(text),
      app_handle: app_handle.clone(),
    }
  }

  /// Close window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn close_window(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::show_all(text),
      app_handle: app_handle.clone(),
    }
  }

  /// Quit app menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn quit(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::quit(text),
      app_handle: app_handle.clone(),
    }
  }

  /// About app menu item
  pub fn about(
    app_handle: &AppHandle<R>,
    text: Option<&str>,
    metadata: Option<AboutMetadata>,
  ) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::about(text, metadata),
      app_handle: app_handle.clone(),
    }
  }

  /// Services menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn services(app_handle: &AppHandle<R>, text: Option<&str>) -> Self {
    Self {
      inner: muda::PredefinedMenuItem::services(text),
      app_handle: app_handle.clone(),
    }
  }

  /// Returns a unique identifier associated with this menu item.
  pub fn id(&self) -> crate::Result<u32> {
    Ok(0)
  }

  /// Get the text for this menu item.
  pub fn text(&self) -> crate::Result<String> {
    run_main_thread!(self, |self_: Self| self_.inner.text())
  }

  /// Set the text for this menu item. `text` could optionally contain
  /// an `&` before a character to assign this character as the mnemonic
  /// for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn set_text<S: AsRef<str>>(&self, text: S) -> crate::Result<()> {
    let text = text.as_ref().to_string();
    run_main_thread!(self, |self_: Self| self_.inner.set_text(text))
  }
}
