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
    // the map lock is released before any handler runs: a handler is free to `add` or
    // `remove`, for this label or any other, and holding it here would deadlock them
    let handlers = {
      let listeners = self.listeners.lock().expect("poisoned scoped listeners");
      listeners.get(label).cloned()
    };

    if let Some(handlers) = handlers {
      let handlers = handlers.lock().expect("poisoned scoped event listeners");
      for handler in handlers.iter() {
        handler(event);
      }
    }

    // Add any pending listeners that were parked while dispatching.
    let pending = {
      let mut pending = self.pending.lock().expect("poisoned pending listeners");
      std::mem::take(&mut *pending)
    };
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

#[cfg(test)]
mod tests {
  use super::ScopedEventListeners;
  use std::sync::{Arc, Mutex};

  #[test]
  fn dispatches_in_registration_order_to_the_matching_label_only() {
    let listeners = ScopedEventListeners::<u32>::default();
    let seen = Arc::new(Mutex::new(Vec::new()));

    for tag in ["a1", "a2"] {
      let seen = seen.clone();
      listeners.add("a", Box::new(move |e| seen.lock().unwrap().push((tag, *e))));
    }
    let seen_b = seen.clone();
    listeners.add(
      "b",
      Box::new(move |e| seen_b.lock().unwrap().push(("b1", *e))),
    );

    listeners.dispatch("a", &7);

    assert_eq!(*seen.lock().unwrap(), vec![("a1", 7), ("a2", 7)]);
  }

  /// Creating a window from another window's event handler does exactly this.
  #[test]
  fn registering_for_another_label_while_dispatching() {
    let listeners = Arc::new(ScopedEventListeners::<u32>::default());
    let seen = Arc::new(Mutex::new(Vec::new()));

    let listeners_ = listeners.clone();
    let seen_ = seen.clone();
    listeners.add(
      "a",
      Box::new(move |_| {
        let seen = seen_.clone();
        listeners_.add("b", Box::new(move |e| seen.lock().unwrap().push(*e)));
      }),
    );

    listeners.dispatch("a", &1);
    listeners.dispatch("b", &2);

    assert_eq!(*seen.lock().unwrap(), vec![2]);
  }

  #[test]
  fn registering_for_the_dispatching_label_parks_until_dispatch_is_done() {
    let listeners = Arc::new(ScopedEventListeners::<u32>::default());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let registered = Arc::new(Mutex::new(false));

    let listeners_ = listeners.clone();
    let seen_ = seen.clone();
    listeners.add(
      "a",
      Box::new(move |_| {
        if std::mem::replace(&mut *registered.lock().unwrap(), true) {
          return;
        }
        let seen = seen_.clone();
        listeners_.add("a", Box::new(move |e| seen.lock().unwrap().push(*e)));
      }),
    );

    listeners.dispatch("a", &1);
    // the handler registered mid-dispatch must not run for the event that spawned it
    assert!(seen.lock().unwrap().is_empty());

    listeners.dispatch("a", &2);
    assert_eq!(*seen.lock().unwrap(), vec![2]);
  }

  #[test]
  fn remove_during_dispatch_is_not_undone_by_it() {
    let listeners = Arc::new(ScopedEventListeners::<u32>::default());
    let count = Arc::new(Mutex::new(0));

    let listeners_ = listeners.clone();
    let count_ = count.clone();
    listeners.add(
      "a",
      Box::new(move |_| {
        *count_.lock().unwrap() += 1;
        listeners_.remove("a");
      }),
    );

    listeners.dispatch("a", &1);
    listeners.dispatch("a", &2);
    assert_eq!(*count.lock().unwrap(), 1);
  }
}
