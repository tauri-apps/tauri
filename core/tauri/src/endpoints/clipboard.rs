// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeContext;
#[cfg(any(clipboard_write_text, clipboard_read_text))]
use crate::runtime::ClipboardManager;
use crate::Runtime;
use serde::Deserialize;
use tauri_macros::{module_command_handler, CommandModule};

/// The API descriptor.
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", content = "data", rename_all = "camelCase")]
pub enum Cmd {
  /// Write a text string to the clipboard.
  WriteText(String),
  /// Read clipboard content as text.
  ReadText,
}

impl Cmd {
  #[module_command_handler(clipboard_write_text, "clipboard > writeText")]
  fn write_text<R: Runtime>(context: InvokeContext<R>, text: String) -> crate::Result<()> {
    Ok(
      context
        .window
        .app_handle
        .clipboard_manager()
        .write_text(text)?,
    )
  }

  #[module_command_handler(clipboard_read_text, "clipboard > readText")]
  fn read_text<R: Runtime>(context: InvokeContext<R>) -> crate::Result<Option<String>> {
    Ok(context.window.app_handle.clipboard_manager().read_text()?)
  }
}
