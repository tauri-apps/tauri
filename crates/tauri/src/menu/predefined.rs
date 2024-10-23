// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use super::run_item_main_thread;
use super::{AboutMetadata, PredefinedMenuItem};
use crate::menu::PredefinedMenuItemInner;
use crate::run_main_thread;
use crate::{menu::MenuId, AppHandle, Manager, Runtime};

impl<R: Runtime> PredefinedMenuItem<R> {
  /// Separator menu item
  pub fn separator<M: Manager<R>>(manager: &M) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::separator();
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Copy menu item
  pub fn copy<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::copy(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Cut menu item
  pub fn cut<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::cut(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Paste menu item
  pub fn paste<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::paste(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// SelectAll menu item
  pub fn select_all<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::select_all(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Undo menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn undo<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::undo(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }
  /// Redo menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn redo<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::redo(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Minimize window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn minimize<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::minimize(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Maximize window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn maximize<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::maximize(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Fullscreen menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn fullscreen<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::fullscreen(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Hide window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn hide<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::hide(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Hide other windows menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn hide_others<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::hide_others(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Show all app windows menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn show_all<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::show_all(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Close window menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn close_window<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::close_window(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Quit app menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn quit<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::quit(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// About app menu item
  pub fn about<M: Manager<R>>(
    manager: &M,
    text: Option<&str>,
    metadata: Option<AboutMetadata<'_>>,
  ) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let metadata = match metadata {
      Some(m) => Some(m.try_into()?),
      None => None,
    };

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::about(text.as_deref(), metadata);
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Services menu item
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux:** Unsupported.
  pub fn services<M: Manager<R>>(manager: &M, text: Option<&str>) -> crate::Result<Self> {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.map(|t| t.to_owned());

    let item = run_main_thread!(handle, || {
      let item = muda::PredefinedMenuItem::services(text.as_deref());
      PredefinedMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Returns a unique identifier associated with this menu item.
  pub fn id(&self) -> &MenuId {
    &self.0.id
  }

  /// Get the text for this menu item.
  pub fn text(&self) -> crate::Result<String> {
    run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().text())
  }

  /// Set the text for this menu item. `text` could optionally contain
  /// an `&` before a character to assign this character as the mnemonic
  /// for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn set_text<S: AsRef<str>>(&self, text: S) -> crate::Result<()> {
    let text = text.as_ref().to_string();
    run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().set_text(text))
  }

  /// The application handle associated with this type.
  pub fn app_handle(&self) -> &AppHandle<R> {
    &self.0.app_handle
  }
}
