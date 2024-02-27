// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Clipboard implementation.

use std::{
  fmt,
  sync::{Arc, Mutex},
};

pub use arboard::Clipboard;
use tauri_runtime::{ClipboardManager, Result};

#[derive(Clone)]
pub struct ClipboardManagerWrapper {
  pub clipboard: Arc<Mutex<std::result::Result<Clipboard, arboard::Error>>>,
}

impl fmt::Debug for ClipboardManagerWrapper {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ClipboardManagerWrapper").finish()
  }
}

struct ClipboardError(String);
impl std::error::Error for ClipboardError {}
impl fmt::Display for ClipboardError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "ClipboardError: {}", self.0)
  }
}
impl fmt::Debug for ClipboardError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("ClipboardError").field(&self.0).finish()
  }
}
impl From<ClipboardError> for crate::Error {
  fn from(e: ClipboardError) -> crate::Error {
    crate::Error::Clipboard(Box::new(e))
  }
}

impl ClipboardManager for ClipboardManagerWrapper {
  fn read_text(&self) -> Result<Option<String>> {
    self
      .clipboard
      .lock()
      .unwrap()
      .as_mut()
      .map(|c| c.get_text().map(Some))
      .map_err(|e| ClipboardError(e.to_string()))?
      .map_err(|e| ClipboardError(e.to_string()))
      .map_err(Into::into)
  }

  fn write_text<V: Into<String>>(&mut self, text: V) -> Result<()> {
    let text = text.into();
    self
      .clipboard
      .lock()
      .unwrap()
      .as_mut()
      .map(|c| c.set_text(text))
      .map_err(|e| ClipboardError(e.to_string()))?
      .map_err(|e| ClipboardError(e.to_string()))
      .map_err(Into::into)
  }
}
