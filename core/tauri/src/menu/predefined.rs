// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::AboutMetadata;
use super::{run_item_main_thread, run_main_thread, MudaPredefinedMenuItem};
use crate::{menu::MenuId, resources::Resource, AppHandle, Manager, Runtime};

/// A predefined (native) menu item which has a predfined behavior by the OS or by this crate.
pub struct PredefinedMenuItem<R: Runtime> {
  pub(crate) id: MenuId,
  pub(crate) inner: MudaPredefinedMenuItem,
  pub(crate) app_handle: AppHandle<R>,
}

impl<R: Runtime> Drop for PredefinedMenuItem<R> {
  fn drop(&mut self) {
    let item = self.inner.take();
    let _ = run_item_main_thread!(self, |_: Self| { drop(item) });
  }
}

/// # Safety
///
/// We make sure it always runs on the main thread.
unsafe impl<R: Runtime> Sync for PredefinedMenuItem<R> {}
unsafe impl<R: Runtime> Send for PredefinedMenuItem<R> {}

impl<R: Runtime> Clone for PredefinedMenuItem<R> {
  fn clone(&self) -> Self {
    Self {
      id: self.id.clone(),
      inner: self.inner.clone(),
      app_handle: self.app_handle.clone(),
    }
  }
}

impl<R: Runtime> super::sealed::IsMenuItemBase for PredefinedMenuItem<R> {
  fn inner_muda(&self) -> &dyn muda::IsMenuItem {
    self.inner.as_ref()
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
  pub fn separator<M: Manager<R>>(manager: &M) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::separator())
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle: manager.app_handle().clone(),
    })
  }

  /// Copy menu item
  pub fn copy<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::copy(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// Cut menu item
  pub fn cut<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::cut(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// Paste menu item
  pub fn paste<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::paste(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// SelectAll menu item
  pub fn select_all<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::select_all(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// Undo menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn undo<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::undo(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }
  /// Redo menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn redo<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::redo(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// Minimize window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn minimize<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::minimize(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// Maximize window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn maximize<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::maximize(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// Fullscreen menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn fullscreen<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::fullscreen(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// Hide window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn hide<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::hide(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// Hide other windows menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn hide_others<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::hide_others(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// Show all app windows menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn show_all<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::show_all(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// Close window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn close_window<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::close_window(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// Quit app menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn quit<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::quit(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// About app menu item
  pub fn about<M: Manager<R>>(
    manager: &M,
    text: Option<&str>,
    metadata: Option<AboutMetadata>,
  ) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::about(
        text.as_deref(),
        metadata.map(Into::into)
      ))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// Services menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn services<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let app_handle = manager.app_handle().clone();

    let text = text.map(|t| t.to_owned());
    let inner = run_main_thread!(
      app_handle,
      MudaPredefinedMenuItem::new(muda::PredefinedMenuItem::services(text.as_deref()))
    )?;

    Ok(Self {
      id: inner.as_ref().id().clone(),
      inner,
      app_handle,
    })
  }

  /// Returns a unique identifier associated with this menu item.
  pub fn id(&self) -> &MenuId {
    &self.id
  }

  /// Get the text for this menu item.
  pub fn text(&self) -> crate::Result<String> {
    run_item_main_thread!(self, |self_: Self| self_.inner.as_ref().text())
  }

  /// Set the text for this menu item. `text` could optionally contain
  /// an `&` before a character to assign this character as the mnemonic
  /// for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn set_text<S: AsRef<str>>(&self, text: S) -> crate::Result<()> {
    let text = text.as_ref().to_string();
    run_item_main_thread!(self, |self_: Self| self_.inner.as_ref().set_text(text))
  }

  /// The application handle associated with this type.
  pub fn app_handle(&self) -> &AppHandle<R> {
    &self.app_handle
  }
}

impl<R: Runtime> Resource for PredefinedMenuItem<R> {}
