// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;
use crate::{Runtime, Window};
use serde::Deserialize;

#[cfg(global_shortcut_all)]
use crate::runtime::GlobalShortcutManager;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// Register a global shortcut.
  Register { shortcut: String, handler: String },
  /// Register a list of global shortcuts.
  RegisterAll {
    shortcuts: Vec<String>,
    handler: String,
  },
  /// Unregister a global shortcut.
  Unregister { shortcut: String },
  /// Unregisters all registered shortcuts.
  UnregisterAll,
  /// Determines whether the given hotkey is registered or not.
  IsRegistered { shortcut: String },
}

#[cfg(global_shortcut_all)]
fn register_shortcut<R: Runtime>(
  window: Window<R>,
  manager: &mut R::GlobalShortcutManager,
  shortcut: String,
  handler: String,
) -> crate::Result<()> {
  let accelerator = shortcut.clone();
  manager.register(&shortcut, move || {
    let callback_string = crate::api::rpc::format_callback(handler.to_string(), &accelerator)
      .expect("unable to serialize shortcut string to json");
    let _ = window.eval(callback_string.as_str());
  })?;
  Ok(())
}

#[cfg(not(global_shortcut_all))]
impl Cmd {
  pub fn run<R: Runtime>(self, _window: Window<R>) -> crate::Result<InvokeResponse> {
    Err(crate::Error::ApiNotAllowlisted(
      "globalShortcut > all".to_string(),
    ))
  }
}

#[cfg(global_shortcut_all)]
impl Cmd {
  pub fn run<R: Runtime>(self, window: Window<R>) -> crate::Result<InvokeResponse> {
    match self {
      Self::Register { shortcut, handler } => {
        let mut manager = window.app_handle.global_shortcut_manager();
        register_shortcut(window, &mut manager, shortcut, handler)?;
        Ok(().into())
      }
      Self::RegisterAll { shortcuts, handler } => {
        let mut manager = window.app_handle.global_shortcut_manager();
        for shortcut in shortcuts {
          register_shortcut(window.clone(), &mut manager, shortcut, handler.clone())?;
        }
        Ok(().into())
      }
      Self::Unregister { shortcut } => {
        window
          .app_handle
          .global_shortcut_manager()
          .unregister(&shortcut)?;
        Ok(().into())
      }
      Self::UnregisterAll => {
        window
          .app_handle
          .global_shortcut_manager()
          .unregister_all()?;
        Ok(().into())
      }
      Self::IsRegistered { shortcut } => Ok(
        window
          .app_handle
          .global_shortcut_manager()
          .is_registered(&shortcut)?
          .into(),
      ),
    }
  }
}
