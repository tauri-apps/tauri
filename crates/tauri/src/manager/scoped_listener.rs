// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

type Handler<E> = Box<dyn Fn(&E) + Send>;
type Handlers<E> = Arc<Mutex<Vec<Handler<E>>>>;

/// Event listeners scoped to a single window or webview, keyed by its label.
///
/// Backs [`crate::Window::on_window_event`] and [`crate::Webview::on_webview_event`].
/// The event loop feeds it from the runtime's [`tauri_runtime::RunEvent`] stream, so
/// listeners run on the main thread, in registration order.
///
/// Handlers are user code, so dispatch must not hold a lock on anything they might
/// reach for. Each label owns its own handler list: dispatching for one window leaves every
/// other window free to register, which is what creating a window from another window's
/// event handler does. A registration for the label *currently* dispatching parks in
/// `pending` instead of deadlocking, and joins the list once dispatch finishes.
pub(crate) struct ScopedEventListeners<E> {
  listeners: Mutex<HashMap<String, Handlers<E>>>,
  pending: Mutex<Vec<(String, Handler<E>)>>,
}

impl<E> Default for ScopedEventListeners<E> {
  fn default() -> Self {
    Self {
      listeners: Default::default(),
      pending: Default::default(),
    }
  }
}

impl<E> ScopedEventListeners<E> {
  /// Registers an event listener for the given label.
  pub(crate) fn add(&self, label: &str, handler: Handler<E>) {
    let mut listeners = self.listeners.lock().expect("poisoned scoped listeners");

    let handlers = listeners.entry(label.to_string()).or_default().clone();
    let mut handlers = match handlers.try_lock() {
      Ok(handlers) => handlers,
      // this label is dispatching right now, so we are being called from one of its
      // own listeners - park the handler and let `dispatch` pick it up on the way out
      Err(_) => {
        let mut pending = self.pending.lock().expect("poisoned pending listeners");
        pending.push((label.to_string(), handler));
        return;
      }
    };

    handlers.push(handler);
  }

  /// Runs the listeners registered for the given label.
  pub(crate) fn dispatch(&self, label: &str, event: &E) {
    let listeners = self.listeners.lock().expect("poisoned scoped listeners");

    if let Some(handlers) = listeners.get(label).cloned() {
      let handlers = handlers.lock().expect("poisoned scoped event listeners");
      for handler in handlers.iter() {
        handler(event);
      }
    }

    // Add any pending listeners that were parked while dispatching.
    let mut pending = self.pending.lock().expect("poisoned pending  listeners");
    let pending = std::mem::take(&mut *pending);
    for (label, handler) in pending {
      self.add(&label, handler);
    }
  }

  /// Drops the listeners registered for the given label.
  ///
  /// Listeners currently being dispatched are unaffected - they are dropped once that
  /// dispatch returns, rather than being resurrected by it.
  pub(crate) fn remove(&self, label: &str) {
    let mut listeners = self.listeners.lock().expect("poisoned scoped listeners");
    listeners.remove(label);
  }
}
