// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  sync::{Arc, Mutex, TryLockError, Weak},
};

type Handler<E> = Box<dyn Fn(&E) + Send>;

/// Event listeners scoped to a single window or webview, keyed by its label.
///
/// Backs [`crate::Window::on_window_event`] and [`crate::Webview::on_webview_event`].
/// The event loop feeds it from the runtime's [`tauri_runtime::RunEvent`] stream, so
/// listeners run on the main thread, in registration order.
///
/// Each window or webview instance gets its own [`Listeners`] when it is created, and
/// registers through a [`Weak`] reference to it rather than by label. Once the instance is
/// gone its listeners are dropped from here, so a handle that outlived it can no longer
/// register anything - in particular, nothing that would fire for a later window or
/// webview reusing the label.
pub(crate) struct ScopedEventListeners<E> {
  listeners: Mutex<HashMap<String, Arc<Listeners<E>>>>,
}

impl<E> Default for ScopedEventListeners<E> {
  fn default() -> Self {
    Self {
      listeners: Default::default(),
    }
  }
}

impl<E> ScopedEventListeners<E> {
  /// Creates the listener list of a new window or webview with the given label.
  ///
  /// A previous instance's list is removed once it is closed, so one still registered here
  /// means two instances were attached under the same label - e.g. built concurrently, both
  /// passing the label check before either was inserted. The newer one wins, as in the
  /// window and webview stores, but the older one silently losing its listeners is surfaced.
  pub(crate) fn register(&self, label: &str) -> Weak<Listeners<E>> {
    let listeners = Arc::new(Listeners::default());
    let weak = Arc::downgrade(&listeners);
    let replaced = self
      .listeners
      .lock()
      .expect("poisoned scoped listeners")
      .insert(label.to_string(), listeners);
    if replaced.is_some() {
      log::warn!(
        "replacing the event listeners of `{label}`, which is still open: \
         a window or webview with this label was likely created twice concurrently, \
         and the older one will no longer receive its events"
      );
    }
    weak
  }

  /// Runs the listeners registered for the given label.
  pub(crate) fn dispatch(&self, label: &str, event: &E) {
    // the map lock is released before any handler runs: a handler is free to `register`
    // or `remove`, for this label or any other, and holding it here would deadlock them
    let listeners = {
      let listeners = self.listeners.lock().expect("poisoned scoped listeners");
      listeners.get(label).cloned()
    };

    if let Some(listeners) = listeners {
      listeners.dispatch(event);
    }
  }

  /// Drops the listeners registered for the given label, returning them.
  ///
  /// Listeners currently being dispatched are unaffected - they are dropped once that
  /// dispatch returns, rather than being resurrected by it.
  pub(crate) fn remove(&self, label: &str) -> Option<Arc<Listeners<E>>> {
    let mut listeners = self.listeners.lock().expect("poisoned scoped listeners");
    listeners.remove(label)
  }

  /// Like [`Self::remove`], but only if the listeners registered for the given label
  /// still are the given instance's, rather than those of a later one reusing the label.
  pub(crate) fn remove_instance(
    &self,
    label: &str,
    instance: &Weak<Listeners<E>>,
  ) -> Option<Arc<Listeners<E>>> {
    let mut listeners = self.listeners.lock().expect("poisoned scoped listeners");
    if listeners
      .get(label)
      .is_some_and(|listeners| std::ptr::eq(Arc::as_ptr(listeners), instance.as_ptr()))
    {
      listeners.remove(label)
    } else {
      None
    }
  }
}

/// The listeners of a single window or webview instance.
///
/// Handlers are user code, so dispatch must not hold a lock on anything they might
/// reach for. A registration made while these listeners are dispatching - from one of
/// them, or from another thread - parks in `pending` instead of waiting on (or, from the
/// dispatching thread, deadlocking on) `handlers`, and joins the list once dispatch finishes.
pub(crate) struct Listeners<E> {
  handlers: Mutex<Vec<Handler<E>>>,
  pending: Mutex<Vec<Handler<E>>>,
}

impl<E> Default for Listeners<E> {
  fn default() -> Self {
    Self {
      handlers: Default::default(),
      pending: Default::default(),
    }
  }
}

impl<E> Listeners<E> {
  /// Registers an event listener.
  pub(crate) fn add(&self, handler: Handler<E>) {
    // `pending` is held while checking `handlers`, so `dispatch` can't collect the parked
    // handlers in between the failed check and the push, leaving this one behind
    let mut pending = self.pending.lock().expect("poisoned pending listeners");
    match self.handlers.try_lock() {
      Ok(mut handlers) => handlers.push(handler),
      Err(TryLockError::WouldBlock) => pending.push(handler),
      Err(TryLockError::Poisoned(_)) => panic!("poisoned scoped event listeners"),
    }
  }

  /// Runs the listeners.
  pub(crate) fn dispatch(&self, event: &E) {
    let mut handlers = self
      .handlers
      .lock()
      .expect("poisoned scoped event listeners");
    for handler in handlers.iter() {
      handler(event);
    }

    // add any listeners that were parked while dispatching
    let mut pending = self.pending.lock().expect("poisoned pending listeners");
    handlers.append(&mut pending);
    // release `handlers` first: an `add` waiting on `pending` must then find `handlers`
    // free, instead of parking a listener there is no dispatch left to collect
    drop(handlers);
    drop(pending);
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

    let a = listeners.register("a").upgrade().unwrap();
    for tag in ["a1", "a2"] {
      let seen = seen.clone();
      a.add(Box::new(move |e| seen.lock().unwrap().push((tag, *e))));
    }
    let seen_b = seen.clone();
    listeners
      .register("b")
      .upgrade()
      .unwrap()
      .add(Box::new(move |e| seen_b.lock().unwrap().push(("b1", *e))));

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
    listeners
      .register("a")
      .upgrade()
      .unwrap()
      .add(Box::new(move |_| {
        let seen = seen_.clone();
        listeners_
          .register("b")
          .upgrade()
          .unwrap()
          .add(Box::new(move |e| seen.lock().unwrap().push(*e)));
      }));

    listeners.dispatch("a", &1);
    listeners.dispatch("b", &2);

    assert_eq!(*seen.lock().unwrap(), vec![2]);
  }

  #[test]
  fn registering_for_the_dispatching_label_parks_until_dispatch_is_done() {
    let listeners = ScopedEventListeners::<u32>::default();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let registered = Arc::new(Mutex::new(false));

    let a = listeners.register("a");
    let a_ = a.clone();
    let seen_ = seen.clone();
    a.upgrade().unwrap().add(Box::new(move |_| {
      if std::mem::replace(&mut *registered.lock().unwrap(), true) {
        return;
      }
      let seen = seen_.clone();
      a_.upgrade()
        .unwrap()
        .add(Box::new(move |e| seen.lock().unwrap().push(*e)));
    }));

    listeners.dispatch("a", &1);
    // the handler registered mid-dispatch must not run for the event that spawned it
    assert!(seen.lock().unwrap().is_empty());

    listeners.dispatch("a", &2);
    assert_eq!(*seen.lock().unwrap(), vec![2]);
  }

  #[test]
  fn registering_from_another_thread_while_dispatching() {
    let listeners = ScopedEventListeners::<u32>::default();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let registered = Arc::new(Mutex::new(false));

    let a = listeners.register("a");
    let a_ = a.clone();
    let seen_ = seen.clone();
    a.upgrade().unwrap().add(Box::new(move |_| {
      if std::mem::replace(&mut *registered.lock().unwrap(), true) {
        return;
      }
      let a = a_.clone();
      let seen = seen_.clone();
      // must neither block on the running dispatch nor get lost
      std::thread::spawn(move || {
        a.upgrade()
          .unwrap()
          .add(Box::new(move |e| seen.lock().unwrap().push(*e)));
      })
      .join()
      .unwrap();
    }));

    listeners.dispatch("a", &1);
    listeners.dispatch("a", &2);
    assert_eq!(*seen.lock().unwrap(), vec![2]);
  }

  #[test]
  fn remove_during_dispatch_is_not_undone_by_it() {
    let listeners = Arc::new(ScopedEventListeners::<u32>::default());
    let count = Arc::new(Mutex::new(0));

    let listeners_ = listeners.clone();
    let count_ = count.clone();
    listeners
      .register("a")
      .upgrade()
      .unwrap()
      .add(Box::new(move |_| {
        *count_.lock().unwrap() += 1;
        listeners_.remove("a");
      }));

    listeners.dispatch("a", &1);
    listeners.dispatch("a", &2);
    assert_eq!(*count.lock().unwrap(), 1);
  }

  #[test]
  fn a_removed_instance_cannot_register_anymore() {
    let listeners = ScopedEventListeners::<u32>::default();
    let count = Arc::new(Mutex::new(0));

    let old = listeners.register("a");
    listeners.remove("a");
    assert!(old.upgrade().is_none());

    // the label is reused by a new instance, which must not see the old one's listeners
    let _new = listeners.register("a");
    if let Some(old) = old.upgrade() {
      let count = count.clone();
      old.add(Box::new(move |_| *count.lock().unwrap() += 1));
    }

    listeners.dispatch("a", &1);
    assert_eq!(*count.lock().unwrap(), 0);
  }
}
