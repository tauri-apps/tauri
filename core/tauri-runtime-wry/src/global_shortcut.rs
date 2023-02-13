// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Global shortcut implementation.

use std::{
  collections::HashMap,
  fmt,
  sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
  },
};

use crate::{getter, Context, GlobalHotKeyManagerWrapper, Message};

#[cfg(desktop)]
pub use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager};
use tauri_runtime::{Error, GlobalShortcutManager, Result, UserEvent};

pub type GlobalShortcutListeners = Arc<Mutex<HashMap<u32, Box<dyn Fn() + Send>>>>;

#[derive(Debug, Clone)]
pub enum GlobalShortcutMessage {
  Register(HotKey, Sender<Result<GlobalShortcutWrapper>>),
  Unregister(GlobalShortcutWrapper, Sender<Result<()>>),
  UnregisterAll(Vec<GlobalShortcutWrapper>, Sender<Result<()>>),
}

#[derive(Debug, Clone, Copy)]
pub struct GlobalShortcutWrapper(HotKey);

// SAFETY: usage outside of main thread is guarded, we use the event loop on such cases.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for GlobalShortcutWrapper {}

/// Wrapper around [`WryShortcutManager`].
#[derive(Clone)]
pub struct GlobalShortcutManagerHandle<T: UserEvent> {
  pub context: Context<T>,
  pub shortcuts: Arc<Mutex<HashMap<String, (u32, GlobalShortcutWrapper)>>>,
  pub listeners: GlobalShortcutListeners,
}

// SAFETY: this is safe since the `Context` usage is guarded on `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for GlobalShortcutManagerHandle<T> {}

impl<T: UserEvent> fmt::Debug for GlobalShortcutManagerHandle<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("GlobalShortcutManagerHandle")
      .field("context", &self.context)
      .field("shortcuts", &self.shortcuts)
      .finish()
  }
}

impl<T: UserEvent> GlobalShortcutManager for GlobalShortcutManagerHandle<T> {
  fn is_registered(&self, shorcut: &str) -> Result<bool> {
    Ok(self.shortcuts.lock().unwrap().contains_key(shorcut))
  }
  fn register<F: Fn() + Send + 'static>(&mut self, shorcut: &str, handler: F) -> Result<()> {
    let hotkey: HotKey = shorcut.parse().expect("invalid shorcut");
    let id = hotkey.id();
    let (tx, rx) = channel();
    let shortcut = getter!(
      self,
      rx,
      Message::GlobalShortcut(GlobalShortcutMessage::Register(hotkey, tx))
    )??;

    self.listeners.lock().unwrap().insert(id, Box::new(handler));
    self
      .shortcuts
      .lock()
      .unwrap()
      .insert(shorcut.into(), (id, shortcut));

    Ok(())
  }

  fn unregister_all(&mut self) -> Result<()> {
    let (tx, rx) = channel();
    getter!(
      self,
      rx,
      Message::GlobalShortcut(GlobalShortcutMessage::UnregisterAll(
        self
          .shortcuts
          .lock()
          .unwrap()
          .values()
          .into_iter()
          .map(|h| h.1)
          .collect(),
        tx
      ))
    )??;
    self.listeners.lock().unwrap().clear();
    self.shortcuts.lock().unwrap().clear();
    Ok(())
  }

  fn unregister(&mut self, hotkey: &str) -> Result<()> {
    if let Some((hotkey_id, shortcut)) = self.shortcuts.lock().unwrap().remove(hotkey) {
      let (tx, rx) = channel();
      getter!(
        self,
        rx,
        Message::GlobalShortcut(GlobalShortcutMessage::Unregister(shortcut, tx))
      )??;
      self.listeners.lock().unwrap().remove(&hotkey_id);
    }
    Ok(())
  }
}

pub fn handle_global_shortcut_message(
  message: GlobalShortcutMessage,
  global_shortcut_manager: &Arc<Mutex<GlobalHotKeyManagerWrapper>>,
) {
  match message {
    GlobalShortcutMessage::Register(shortcut, tx) => tx
      .send(
        global_shortcut_manager
          .lock()
          .unwrap()
          .0
          .register(shortcut)
          .map(|_| GlobalShortcutWrapper(shortcut))
          .map_err(|e| Error::GlobalShortcut(Box::new(e))),
      )
      .unwrap(),
    GlobalShortcutMessage::Unregister(shortcut, tx) => tx
      .send(
        global_shortcut_manager
          .lock()
          .unwrap()
          .0
          .unregister(shortcut.0)
          .map_err(|e| Error::GlobalShortcut(Box::new(e))),
      )
      .unwrap(),
    GlobalShortcutMessage::UnregisterAll(shortcuts, tx) => tx
      .send(
        global_shortcut_manager
          .lock()
          .unwrap()
          .0
          .unregister_all(shortcuts.iter().map(|s| s.0).collect::<Vec<_>>().as_slice())
          .map_err(|e| Error::GlobalShortcut(Box::new(e))),
      )
      .unwrap(),
  }
}
