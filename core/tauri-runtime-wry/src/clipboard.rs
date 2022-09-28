// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Clipboard implementation.

use crate::{getter, Context, Message};

use std::sync::{
  mpsc::{channel, Sender},
  Arc, Mutex,
};

use tauri_runtime::{ClipboardManager, Result, UserEvent};
pub use wry::application::clipboard::Clipboard;

#[derive(Debug, Clone)]
pub enum ClipboardMessage {
  WriteText(String, Sender<()>),
  ReadText(Sender<Option<String>>),
}

#[derive(Debug, Clone)]
pub struct ClipboardManagerWrapper<T: UserEvent> {
  pub context: Context<T>,
}

// SAFETY: this is safe since the `Context` usage is guarded on `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for ClipboardManagerWrapper<T> {}

impl<T: UserEvent> ClipboardManager for ClipboardManagerWrapper<T> {
  fn read_text(&self) -> Result<Option<String>> {
    let (tx, rx) = channel();
    getter!(self, rx, Message::Clipboard(ClipboardMessage::ReadText(tx)))
  }

  fn write_text<V: Into<String>>(&mut self, text: V) -> Result<()> {
    let (tx, rx) = channel();
    getter!(
      self,
      rx,
      Message::Clipboard(ClipboardMessage::WriteText(text.into(), tx))
    )?;
    Ok(())
  }
}

pub fn handle_clipboard_message(
  message: ClipboardMessage,
  clipboard_manager: &Arc<Mutex<Clipboard>>,
) {
  match message {
    ClipboardMessage::WriteText(text, tx) => {
      clipboard_manager.lock().unwrap().write_text(text);
      tx.send(()).unwrap();
    }
    ClipboardMessage::ReadText(tx) => tx
      .send(clipboard_manager.lock().unwrap().read_text())
      .unwrap(),
  }
}
