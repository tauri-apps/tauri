// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeContext;
use crate::{api::ipc::CallbackFn, Runtime};
use serde::Deserialize;
use tauri_macros::{module_command_handler, CommandModule};

#[cfg(global_shortcut_all)]
use crate::runtime::GlobalShortcutManager;

/// The API descriptor.
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// Register a global shortcut.
  Register {
    shortcut: String,
    handler: CallbackFn,
  },
  /// Register a list of global shortcuts.
  RegisterAll {
    shortcuts: Vec<String>,
    handler: CallbackFn,
  },
  /// Unregister a global shortcut.
  Unregister { shortcut: String },
  /// Unregisters all registered shortcuts.
  UnregisterAll,
  /// Determines whether the given hotkey is registered or not.
  IsRegistered { shortcut: String },
}

impl Cmd {
  #[module_command_handler(global_shortcut_all, "globalShortcut > all")]
  fn register<R: Runtime>(
    context: InvokeContext<R>,
    shortcut: String,
    handler: CallbackFn,
  ) -> crate::Result<()> {
    let mut manager = context.window.app_handle.global_shortcut_manager();
    register_shortcut(context.window, &mut manager, shortcut, handler)?;
    Ok(())
  }

  #[module_command_handler(global_shortcut_all, "globalShortcut > all")]
  fn register_all<R: Runtime>(
    context: InvokeContext<R>,
    shortcuts: Vec<String>,
    handler: CallbackFn,
  ) -> crate::Result<()> {
    let mut manager = context.window.app_handle.global_shortcut_manager();
    for shortcut in shortcuts {
      register_shortcut(context.window.clone(), &mut manager, shortcut, handler)?;
    }
    Ok(())
  }

  #[module_command_handler(global_shortcut_all, "globalShortcut > all")]
  fn unregister<R: Runtime>(context: InvokeContext<R>, shortcut: String) -> crate::Result<()> {
    context
      .window
      .app_handle
      .global_shortcut_manager()
      .unregister(&shortcut)?;
    Ok(())
  }

  #[module_command_handler(global_shortcut_all, "globalShortcut > all")]
  fn unregister_all<R: Runtime>(context: InvokeContext<R>) -> crate::Result<()> {
    context
      .window
      .app_handle
      .global_shortcut_manager()
      .unregister_all()?;
    Ok(())
  }

  #[module_command_handler(global_shortcut_all, "globalShortcut > all")]
  fn is_registered<R: Runtime>(context: InvokeContext<R>, shortcut: String) -> crate::Result<bool> {
    Ok(
      context
        .window
        .app_handle
        .global_shortcut_manager()
        .is_registered(&shortcut)?,
    )
  }
}

#[cfg(global_shortcut_all)]
fn register_shortcut<R: Runtime>(
  window: crate::Window<R>,
  manager: &mut R::GlobalShortcutManager,
  shortcut: String,
  handler: CallbackFn,
) -> crate::Result<()> {
  let accelerator = shortcut.clone();
  manager.register(&shortcut, move || {
    let callback_string = crate::api::ipc::format_callback(handler, &accelerator)
      .expect("unable to serialize shortcut string to json");
    let _ = window.eval(callback_string.as_str());
  })?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::api::ipc::CallbackFn;

  #[tauri_macros::module_command_test(global_shortcut_all, "globalShortcut > all")]
  #[quickcheck_macros::quickcheck]
  fn register(shortcut: String, handler: CallbackFn) {
    let ctx = crate::test::mock_invoke_context();
    super::Cmd::register(ctx.clone(), shortcut.clone(), handler).unwrap();
    assert!(super::Cmd::is_registered(ctx, shortcut).unwrap());
  }

  #[tauri_macros::module_command_test(global_shortcut_all, "globalShortcut > all")]
  #[quickcheck_macros::quickcheck]
  fn register_all(shortcuts: Vec<String>, handler: CallbackFn) {
    let ctx = crate::test::mock_invoke_context();
    super::Cmd::register_all(ctx.clone(), shortcuts.clone(), handler).unwrap();
    for shortcut in shortcuts {
      assert!(super::Cmd::is_registered(ctx.clone(), shortcut).unwrap(),);
    }
  }

  #[tauri_macros::module_command_test(global_shortcut_all, "globalShortcut > all")]
  #[quickcheck_macros::quickcheck]
  fn unregister(shortcut: String) {
    let ctx = crate::test::mock_invoke_context();
    super::Cmd::register(ctx.clone(), shortcut.clone(), CallbackFn(0)).unwrap();
    super::Cmd::unregister(ctx.clone(), shortcut.clone()).unwrap();
    assert!(!super::Cmd::is_registered(ctx, shortcut).unwrap());
  }

  #[tauri_macros::module_command_test(global_shortcut_all, "globalShortcut > all")]
  #[quickcheck_macros::quickcheck]
  fn unregister_all() {
    let shortcuts = vec!["CTRL+X".to_string(), "SUPER+C".to_string(), "D".to_string()];
    let ctx = crate::test::mock_invoke_context();
    super::Cmd::register_all(ctx.clone(), shortcuts.clone(), CallbackFn(0)).unwrap();
    super::Cmd::unregister_all(ctx.clone()).unwrap();
    for shortcut in shortcuts {
      assert!(!super::Cmd::is_registered(ctx.clone(), shortcut).unwrap(),);
    }
  }

  #[tauri_macros::module_command_test(global_shortcut_all, "globalShortcut > all")]
  #[quickcheck_macros::quickcheck]
  fn is_registered(shortcut: String) {
    let ctx = crate::test::mock_invoke_context();
    assert!(!super::Cmd::is_registered(ctx.clone(), shortcut.clone()).unwrap(),);
    super::Cmd::register(ctx.clone(), shortcut.clone(), CallbackFn(0)).unwrap();
    assert!(super::Cmd::is_registered(ctx, shortcut).unwrap());
  }
}
