// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;
use crate::{
  runtime::{ClipboardManager, Runtime},
  window::Window,
};
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
  pub fn run<R: Runtime>(self, window: Window<R>) -> crate::Result<InvokeResponse> {
    let mut clipboard = window.app_handle.clipboard_manager();
    match self {
      Self::WriteText(text) => Ok(clipboard.write_text(text)?.into()),
      Self::ReadText => Ok(clipboard.read_text()?.into()),
    }
  }
}
