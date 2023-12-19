// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::AboutMetadata;
use crate::{menu::MenuId, resources::Resource, run_main_thread, AppHandle, Manager, Runtime};

/// A predefined (native) menu item which has a predfined behavior by the OS or by this crate.
pub struct PredefinedMenuItem<R: Runtime> {
  pub(crate) id: MenuId,
  pub(crate) inner: muda::PredefinedMenuItem,
  pub(crate) app_handle: AppHandle<R>,
}

impl<R: Runtime> Clone for PredefinedMenuItem<R> {
  fn clone(&self) -> Self {
    Self {
      id: self.id.clone(),
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
  fn inner_muda(&self) -> &dyn muda::IsMenuItem {
    &self.inner
  }
}

impl<R: Runtime> super::IsMenuItem<R> for PredefinedMenuItem<R> {
  fn kind(&self) -> super::MenuItemKind<R> {
    super::MenuItemKind::Predefined(self.clone())
  }

  fn id(&self) -> &MenuId {
    self.id()
  }
}

impl<R: Runtime> PredefinedMenuItem<R> {
  /// Separator menu item
  pub fn separator<M: Manager<R>>(manager: &M) -> Self {
    let inner = muda::PredefinedMenuItem::separator();
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Copy menu item
  pub fn copy<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::copy(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Cut menu item
  pub fn cut<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::cut(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Paste menu item
  pub fn paste<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::paste(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// SelectAll menu item
  pub fn select_all<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::select_all(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Undo menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn undo<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::undo(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }
  /// Redo menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn redo<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::redo(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Minimize window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn minimize<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::minimize(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Maximize window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn maximize<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::maximize(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Fullscreen menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn fullscreen<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::fullscreen(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Hide window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn hide<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::hide(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Hide other windows menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn hide_others<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::hide_others(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Show all app windows menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn show_all<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::show_all(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Close window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn close_window<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::close_window(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Quit app menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn quit<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::quit(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// About app menu item
  pub fn about<M: Manager<R>>(
    manager: &M,
    text: Option<&str>,
    metadata: Option<AboutMetadata>,
  ) -> Self {
    let inner = muda::PredefinedMenuItem::about(text, metadata.map(Into::into));
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Services menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn services<M: Manager<R>>(manager: &M, text: Option<&str>) -> Self {
    let inner = muda::PredefinedMenuItem::services(text);
    Self {
      id: inner.id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    }
  }

  /// Returns a unique identifier associated with this menu item.
  pub fn id(&self) -> &MenuId {
    &self.id
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

  /// The application handle associated with this type.
  pub fn app_handle(&self) -> &AppHandle<R> {
    &self.app_handle
  }
}

impl<R: Runtime> Resource for PredefinedMenuItem<R> {}
