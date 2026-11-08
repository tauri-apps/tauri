// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  sync::{Arc, Mutex, PoisonError, TryLockError, Weak},
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
/// Handlers are user code, so no lock on the list is held while they run: dispatch runs
/// over a snapshot of it. A handler registering another one - for this list or any other -
/// or another thread doing so mid-dispatch therefore never waits on (or, from the
/// dispatching thread, deadlocks on) the running dispatch, and the new handler takes effect
/// from the next event.
///
/// Each handler sits behind its own lock instead, since it may not be [`Sync`]. A handler
/// that makes the platform deliver another event for this window or webview synchronously
/// (e.g. pumping a nested event loop) is not re-entered for it - the other handlers still
/// are - rather than deadlocking on its own lock. Dispatch only happens on the event loop
/// thread, so that lock is never contended otherwise.
///
/// A handler panicking poisons its lock, but leaves the handler itself intact, so the
/// poisoning is ignored rather than turning every later event into a panic.
pub(crate) struct Listeners<E> {
  handlers: Mutex<Vec<Arc<Mutex<Handler<E>>>>>,
}

impl<E> Default for Listeners<E> {
  fn default() -> Self {
    Self {
      handlers: Default::default(),
    }
  }
}

impl<E> Listeners<E> {
  /// Registers an event listener.
  pub(crate) fn add(&self, handler: Handler<E>) {
    self
      .handlers
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .push(Arc::new(Mutex::new(handler)));
  }

  /// Runs the listeners.
  pub(crate) fn dispatch(&self, event: &E) {
    let handlers = self
      .handlers
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .clone();
    for handler in handlers {
      match handler.try_lock() {
        Ok(handler) => handler(event),
        Err(TryLockError::Poisoned(handler)) => handler.into_inner()(event),
        // running further up the stack, for the event that led to this one
        Err(TryLockError::WouldBlock) => {}
      }
    }
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
  fn registering_for_the_dispatching_label_takes_effect_from_the_next_event() {
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
  fn a_reentrant_dispatch_runs_every_handler_but_the_one_it_came_from() {
    let listeners = Arc::new(ScopedEventListeners::<u32>::default());
    let seen = Arc::new(Mutex::new(Vec::new()));

    let a = listeners.register("a").upgrade().unwrap();
    let listeners_ = listeners.clone();
    let seen_ = seen.clone();
    a.add(Box::new(move |e| {
      seen_.lock().unwrap().push(("first", *e));
      // e.g. a window API call the platform answers with another event, synchronously
      if *e == 1 {
        listeners_.dispatch("a", &2);
      }
    }));
    let seen_ = seen.clone();
    a.add(Box::new(move |e| {
      seen_.lock().unwrap().push(("second", *e))
    }));

    listeners.dispatch("a", &1);

    assert_eq!(
      *seen.lock().unwrap(),
      vec![("first", 1), ("second", 2), ("second", 1)]
    );
  }

  #[test]
  fn a_panicking_handler_does_not_break_later_registrations_or_events() {
    let listeners = ScopedEventListeners::<u32>::default();
    let seen = Arc::new(Mutex::new(Vec::new()));

    let a = listeners.register("a").upgrade().unwrap();
    let seen_ = seen.clone();
    a.add(Box::new(move |e| {
      seen_.lock().unwrap().push(*e);
      if *e == 1 {
        panic!("handler panic");
      }
    }));

    // e.g. caught by the platform's event loop callback
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      listeners.dispatch("a", &1);
    }));
    assert!(result.is_err());

    let seen_ = seen.clone();
    a.add(Box::new(move |e| seen_.lock().unwrap().push(*e * 10)));
    listeners.dispatch("a", &2);

    assert_eq!(*seen.lock().unwrap(), vec![1, 2, 20]);
  }

  #[test]
  fn a_removed_instance_cannot_register_anymore() {
    let listeners = ScopedEventListeners::<u32>::default();
    let count = Arc::new(Mutex::new(0));

    let old = listeners.register("a");
    let count_ = count.clone();
    old
      .upgrade()
      .unwrap()
      .add(Box::new(move |_| *count_.lock().unwrap() += 1));
    listeners.remove("a");
    assert!(old.upgrade().is_none());

    // the label is reused by a new instance: the old handle must not resolve to it,
    // and the old instance's listeners must not fire for it
    let new = listeners.register("a");
    assert!(old.upgrade().is_none());
    listeners.dispatch("a", &1);
    assert_eq!(*count.lock().unwrap(), 0);

    // nor can the old instance take the new one's listeners down with it
    assert!(listeners.remove_instance("a", &old).is_none());
    assert!(new.upgrade().is_some());
    assert!(listeners.remove_instance("a", &new).is_some());
    assert!(new.upgrade().is_none());
  }
}
