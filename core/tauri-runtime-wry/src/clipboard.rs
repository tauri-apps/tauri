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
  pub clipboard: Arc<Mutex<Clipboard>>,
}

impl fmt::Debug for ClipboardManagerWrapper {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ClipboardManagerWrapper").finish()
  }
}

impl ClipboardManager for ClipboardManagerWrapper {
  fn read_text(&self) -> Result<Option<String>> {
    Ok(self.clipboard.lock().unwrap().get_text().ok())
  }

  fn write_text<V: Into<String>>(&mut self, text: V) -> Result<()> {
    self
      .clipboard
      .lock()
      .unwrap()
      .set_text(text.into())
      .map_err(|e| crate::Error::Clipboard(Box::new(e)))
  }
}
