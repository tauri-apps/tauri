// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Global shortcut implementation.

use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

use tauri_runtime::{Error, Result, UserEvent};
#[cfg(desktop)]
pub use wry::application::{
  accelerator::{Accelerator, AcceleratorId},
  global_shortcut::{GlobalShortcut as WryGlobalShortcut, ShortcutManager as WryShortcutManager},
};

use crate::{getter, Context, Message};

pub type GlobalShortcutListenerStore = Arc<Mutex<HashMap<AcceleratorId, Box<dyn Fn() + Send>>>>;

#[derive(Debug, Clone)]
pub enum GlobalShortcutMessage {
  IsRegistered(Accelerator, Sender<bool>),
  Register(Accelerator, Sender<Result<GlobalShortcut>>),
  Unregister(GlobalShortcut, Sender<Result<()>>),
  UnregisterAll(Sender<Result<()>>),
}

#[derive(Debug, Clone)]
pub struct GlobalShortcut(WryGlobalShortcut);

// SAFETY: usage outside of main thread is guarded, we use the event loop on such cases.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for GlobalShortcut {}

/// Wrapper around [`WryShortcutManager`].
#[derive(Clone)]
pub struct GlobalShortcutManager<T: UserEvent> {
  pub context: Context<T>,
  pub shortcuts_store: Arc<Mutex<HashMap<String, (AcceleratorId, GlobalShortcut)>>>,
  pub listeners_store: GlobalShortcutListenerStore,
}

// SAFETY: this is safe since the `Context` usage is guarded on `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for GlobalShortcutManager<T> {}

impl<T: UserEvent> fmt::Debug for GlobalShortcutManager<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("GlobalShortcutManager")
      .field("context", &self.context)
      .field("shortcut_store", &self.shortcuts_store)
      .finish()
  }
}

impl<T: UserEvent> tauri_runtime::GlobalShortcutManager for GlobalShortcutManager<T> {
  fn is_registered(&self, accelerator: &str) -> Result<bool> {
    let (tx, rx) = channel();
    getter!(
      self,
      rx,
      Message::GlobalShortcut(GlobalShortcutMessage::IsRegistered(
        accelerator.parse().expect("invalid accelerator"),
        tx
      ))
    )
  }

  fn register<F: Fn() + Send + 'static>(&mut self, accelerator: &str, handler: F) -> Result<()> {
    let wry_accelerator: Accelerator = accelerator.parse().expect("invalid accelerator");
    let id = wry_accelerator.clone().id();
    let (tx, rx) = channel();
    let shortcut = getter!(
      self,
      rx,
      Message::GlobalShortcut(GlobalShortcutMessage::Register(wry_accelerator, tx))
    )??;

    self
      .listeners_store
      .lock()
      .unwrap()
      .insert(id, Box::new(handler));
    self
      .shortcuts_store
      .lock()
      .unwrap()
      .insert(accelerator.into(), (id, shortcut));

    Ok(())
  }

  fn unregister_all(&mut self) -> Result<()> {
    let (tx, rx) = channel();
    getter!(
      self,
      rx,
      Message::GlobalShortcut(GlobalShortcutMessage::UnregisterAll(tx))
    )??;
    self.listeners_store.lock().unwrap().clear();
    self.shortcuts_store.lock().unwrap().clear();
    Ok(())
  }

  fn unregister(&mut self, accelerator: &str) -> Result<()> {
    if let Some((accelerator_id, shortcut)) =
      self.shortcuts_store.lock().unwrap().remove(accelerator)
    {
      let (tx, rx) = channel();
      getter!(
        self,
        rx,
        Message::GlobalShortcut(GlobalShortcutMessage::Unregister(shortcut, tx))
      )??;
      self.listeners_store.lock().unwrap().remove(&accelerator_id);
    }
    Ok(())
  }
}

pub fn handle_global_shortcut_message(
  message: GlobalShortcutMessage,
  shortcut_manager: &Arc<Mutex<WryShortcutManager>>,
) {
  match message {
    GlobalShortcutMessage::IsRegistered(accelerator, tx) => tx
      .send(shortcut_manager.lock().unwrap().is_registered(&accelerator))
      .unwrap(),
    GlobalShortcutMessage::Register(accelerator, tx) => tx
      .send(
        shortcut_manager
          .lock()
          .unwrap()
          .register(accelerator)
          .map(GlobalShortcut)
          .map_err(|e| Error::GlobalShortcut(Box::new(e))),
      )
      .unwrap(),
    GlobalShortcutMessage::Unregister(shortcut, tx) => tx
      .send(
        shortcut_manager
          .lock()
          .unwrap()
          .unregister(shortcut.0)
          .map_err(|e| Error::GlobalShortcut(Box::new(e))),
      )
      .unwrap(),
    GlobalShortcutMessage::UnregisterAll(tx) => tx
      .send(
        shortcut_manager
          .lock()
          .unwrap()
          .unregister_all()
          .map_err(|e| Error::GlobalShortcut(Box::new(e))),
      )
      .unwrap(),
  }
}

pub fn handle_global_shortcut_event<T: UserEvent>(
  accelerator_id: AcceleratorId,
  global_shortcut_manager: &GlobalShortcutManager<T>,
) {
  for (id, handler) in &*global_shortcut_manager.listeners_store.lock().unwrap() {
    if accelerator_id == *id {
      handler();
    }
  }
}
