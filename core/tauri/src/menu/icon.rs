// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use super::{IconMenuItem, NativeIcon};
use crate::menu::IconMenuItemInner;
use crate::{menu::MenuId, AppHandle, Icon, Manager, Runtime};
use crate::{run_item_main_thread, run_main_thread};

impl<R: Runtime> IconMenuItem<R> {
  /// Create a new menu item.
  ///
  /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
  /// for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn new<M, T, A>(
    manager: &M,
    text: T,
    enabled: bool,
    icon: Option<Icon>,
    accelerator: Option<A>,
  ) -> crate::Result<Self>
  where
    M: Manager<R>,
    T: AsRef<str>,
    A: AsRef<str>,
  {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.as_ref().to_owned();
    let icon = icon.and_then(|i| i.try_into().ok());
    let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());

    let item = run_main_thread!(handle, || {
      let item = muda::IconMenuItem::new(text, enabled, icon, accelerator);
      IconMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Create a new menu item with the specified id.
  ///
  /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
  /// for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn with_id<M, I, T, A>(
    manager: &M,
    id: I,
    text: T,
    enabled: bool,
    icon: Option<Icon>,
    accelerator: Option<A>,
  ) -> crate::Result<Self>
  where
    M: Manager<R>,
    I: Into<MenuId>,
    T: AsRef<str>,
    A: AsRef<str>,
  {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let id = id.into();
    let text = text.as_ref().to_owned();
    let icon = icon.and_then(|i| i.try_into().ok());
    let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());

    let item = run_main_thread!(handle, || {
      let item = muda::IconMenuItem::with_id(id.clone(), text, enabled, icon, accelerator);
      IconMenuItemInner {
        id,
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Create a new icon menu item but with a native icon.
  ///
  /// See [`IconMenuItem::new`] for more info.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux**: Unsupported.
  pub fn with_native_icon<M, T, A>(
    manager: &M,
    text: T,
    enabled: bool,
    native_icon: Option<NativeIcon>,
    accelerator: Option<A>,
  ) -> crate::Result<Self>
  where
    M: Manager<R>,
    T: AsRef<str>,
    A: AsRef<str>,
  {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.as_ref().to_owned();
    let icon = native_icon.map(Into::into);
    let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());

    let item = run_main_thread!(handle, || {
      let item = muda::IconMenuItem::with_native_icon(text, enabled, icon, accelerator);
      IconMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Create a new icon menu item with the specified id but with a native icon.
  ///
  /// See [`IconMenuItem::new`] for more info.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux**: Unsupported.
  pub fn with_id_and_native_icon<M, I, T, A>(
    manager: &M,
    id: I,
    text: T,
    enabled: bool,
    native_icon: Option<NativeIcon>,
    accelerator: Option<A>,
  ) -> crate::Result<Self>
  where
    M: Manager<R>,
    I: Into<MenuId>,
    T: AsRef<str>,
    A: AsRef<str>,
  {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let id = id.into();
    let text = text.as_ref().to_owned();
    let icon = native_icon.map(Into::into);
    let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());

    let item = run_main_thread!(handle, || {
      let item =
        muda::IconMenuItem::with_id_and_native_icon(id.clone(), text, enabled, icon, accelerator);
      IconMenuItemInner {
        id,
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// The application handle associated with this type.
  pub fn app_handle(&self) -> &AppHandle<R> {
    &self.0.app_handle
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

  /// Get whether this menu item is enabled or not.
  pub fn is_enabled(&self) -> crate::Result<bool> {
    run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().is_enabled())
  }

  /// Enable or disable this menu item.
  pub fn set_enabled(&self, enabled: bool) -> crate::Result<()> {
    run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().set_enabled(enabled))
  }

  /// Set this menu item accelerator.
  pub fn set_accelerator<S: AsRef<str>>(&self, accelerator: Option<S>) -> crate::Result<()> {
    let accel = accelerator.and_then(|s| s.as_ref().parse().ok());
    run_item_main_thread!(self, |self_: Self| (*self_.0)
      .as_ref()
      .set_accelerator(accel))?
    .map_err(Into::into)
  }

  /// Change this menu item icon or remove it.
  pub fn set_icon(&self, icon: Option<Icon>) -> crate::Result<()> {
    run_item_main_thread!(self, |self_: Self| (*self_.0)
      .as_ref()
      .set_icon(icon.and_then(|i| i.try_into().ok())))
  }

  /// Change this menu item icon to a native image or remove it.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows / Linux**: Unsupported.
  pub fn set_native_icon(&self, _icon: Option<NativeIcon>) -> crate::Result<()> {
    #[cfg(target_os = "macos")]
    return run_item_main_thread!(self, |self_: Self| (*self_.0)
      .as_ref()
      .set_native_icon(_icon.map(Into::into)));
    #[allow(unreachable_code)]
    Ok(())
  }
}
