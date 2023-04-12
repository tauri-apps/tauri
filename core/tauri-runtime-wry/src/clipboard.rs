// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Clipboard implementation.

use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

use tauri_runtime::{Result, UserEvent};
use wry::application::clipboard::Clipboard as WryClipboard;

use crate::{getter, Context, Message};

#[derive(Debug, Clone)]
pub enum ClipboardMessage {
  WriteText(String, Sender<()>),
  ReadText(Sender<Option<String>>),
}

#[derive(Debug, Clone)]
pub struct ClipboardManager<T: UserEvent> {
  pub context: Context<T>,
}

// SAFETY: this is safe since the `Context` usage is guarded on `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for ClipboardManager<T> {}

impl<T: UserEvent> tauri_runtime::ClipboardManager for ClipboardManager<T> {
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

pub fn handle_clipboard_message(message: ClipboardMessage, clipboard: &Arc<Mutex<WryClipboard>>) {
  match message {
    ClipboardMessage::WriteText(text, tx) => {
      clipboard.lock().unwrap().write_text(text);
      tx.send(()).unwrap();
    }
    ClipboardMessage::ReadText(tx) => tx.send(clipboard.lock().unwrap().read_text()).unwrap(),
  }
}
