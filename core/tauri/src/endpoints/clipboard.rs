// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;
#[cfg(any(clipboard_write_text, clipboard_read_text))]
use crate::runtime::ClipboardManager;
use crate::{runtime::Runtime, window::Window};
use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", content = "data", rename_all = "camelCase")]
pub enum Cmd {
  /// Write a text string to the clipboard.
  WriteText(String),
  /// Read clipboard content as text.
  ReadText,
}

impl Cmd {
  #[allow(unused_variables)]
  pub fn run<R: Runtime>(self, window: Window<R>) -> crate::Result<InvokeResponse> {
    match self {
      #[cfg(clipboard_write_text)]
      Self::WriteText(text) => Ok(
        window
          .app_handle
          .clipboard_manager()
          .write_text(text)?
          .into(),
      ),
      #[cfg(not(clipboard_write_text))]
      Self::WriteText(_) => Err(crate::Error::ApiNotAllowlisted(
        "clipboard > readText".to_string(),
      )),

      #[cfg(clipboard_read_text)]
      Self::ReadText => Ok(window.app_handle.clipboard_manager().read_text()?.into()),
      #[cfg(not(clipboard_read_text))]
      Self::ReadText => Err(crate::Error::ApiNotAllowlisted(
        "clipboard > writeText".to_string(),
      )),
    }
  }
}
