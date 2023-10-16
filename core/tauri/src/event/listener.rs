// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{Event, EventId};

use std::{
  boxed::Box,
  cell::Cell,
  collections::HashMap,
  sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
  },
};

/// What to do with the pending handler when resolving it?
enum Pending {
  Unlisten(EventId),
  Listen(EventId, String, Handler),
  Trigger(String, Option<String>, Option<String>),
}

/// Stored in [`Listeners`] to be called upon when the event that stored it is triggered.
struct Handler {
  window: Option<String>,
  callback: Box<dyn Fn(Event) + Send>,
}

/// Holds event handlers and pending event handlers, along with the salts associating them.
struct InnerListeners {
  handlers: Mutex<HashMap<String, HashMap<EventId, Handler>>>,
  pending: Mutex<Vec<Pending>>,
  function_name: &'static str,
  listeners_object_name: &'static str,
  next_event_id: Arc<AtomicU32>,
}

/// A self-contained event manager.
pub struct Listeners {
  inner: Arc<InnerListeners>,
}

impl Default for Listeners {
  fn default() -> Self {
    Self {
      inner: Arc::new(InnerListeners {
        handlers: Mutex::default(),
        pending: Mutex::default(),
        function_name: "_listeners_function_id_",
        listeners_object_name: "_listeners_object_id_",
        next_event_id: Arc::new(AtomicU32::new(2)),
      }),
    }
  }
}

impl Clone for Listeners {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl Listeners {
  pub(crate) fn next_event_id(&self) -> EventId {
    self.inner.next_event_id.fetch_add(1, Ordering::Relaxed)
  }

  /// Function name to represent the JavaScript event function.
  pub(crate) fn function_name(&self) -> &str {
    self.inner.function_name
  }

  /// Listener object name to represent the JavaScript event listener object.
  pub(crate) fn listeners_object_name(&self) -> &str {
    self.inner.listeners_object_name
  }

  /// Insert a pending event action to the queue.
  fn insert_pending(&self, action: Pending) {
    self
      .inner
      .pending
      .lock()
      .expect("poisoned pending event queue")
      .push(action)
  }

  /// Finish all pending event actions.
  fn flush_pending(&self) {
    let pending = {
      let mut lock = self
        .inner
        .pending
        .lock()
        .expect("poisoned pending event queue");
      std::mem::take(&mut *lock)
    };

    for action in pending {
      match action {
        Pending::Unlisten(id) => self.unlisten(id),
        Pending::Listen(id, event, handler) => self.listen_(id, event, handler),
        Pending::Trigger(ref event, window, payload) => self.trigger(event, window, payload),
      }
    }
  }

  fn listen_(&self, id: EventId, event: String, handler: Handler) {
    match self.inner.handlers.try_lock() {
      Err(_) => self.insert_pending(Pending::Listen(id, event, handler)),
      Ok(mut lock) => {
        lock.entry(event).or_default().insert(id, handler);
      }
    }
  }

  /// Adds an event listener for JS events.
  pub(crate) fn listen<F: Fn(Event) + Send + 'static>(
    &self,
    event: String,
    window: Option<String>,
    handler: F,
  ) -> EventId {
    let id = self.next_event_id();
    let handler = Handler {
      window,
      callback: Box::new(handler),
    };

    self.listen_(id, event, handler);

    id
  }

  /// Listen to a JS event and immediately unlisten.
  pub(crate) fn once<F: FnOnce(Event) + Send + 'static>(
    &self,
    event: String,
    window: Option<String>,
    handler: F,
  ) {
    let self_ = self.clone();
    let handler = Cell::new(Some(handler));

    self.listen(event, window, move |event| {
      self_.unlisten(event.id);
      let handler = handler
        .take()
        .expect("attempted to call handler more than once");
      handler(event)
    });
  }

  /// Removes an event listener.
  pub(crate) fn unlisten(&self, id: EventId) {
    match self.inner.handlers.try_lock() {
      Err(_) => self.insert_pending(Pending::Unlisten(id)),
      Ok(mut lock) => lock.values_mut().for_each(|handler| {
        handler.remove(&id);
      }),
    }
  }

  /// Triggers the given global event with its payload.
  pub(crate) fn trigger(&self, event: &str, window: Option<String>, payload: Option<String>) {
    let mut maybe_pending = false;
    match self.inner.handlers.try_lock() {
      Err(_) => self.insert_pending(Pending::Trigger(event.to_owned(), window, payload)),
      Ok(lock) => {
        if let Some(handlers) = lock.get(event) {
          for (&id, handler) in handlers {
            if handler.window.is_none() || window == handler.window {
              maybe_pending = true;
              (handler.callback)(self::Event {
                id,
                data: payload.clone(),
              })
            }
          }
        }
      }
    }

    if maybe_pending {
      self.flush_pending();
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use proptest::prelude::*;

  // dummy event handler function
  fn event_fn(s: Event) {
    println!("{s:?}");
  }

  proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    // check to see if listen() is properly passing keys into the LISTENERS map
    #[test]
    fn listeners_check_key(e in "[a-z]+") {
      let listeners: Listeners = Default::default();
      // clone e as the key
      let key = e.clone();
      // pass e and an dummy func into listen
      listeners.listen(e, None, event_fn);

      // lock mutex
      let l = listeners.inner.handlers.lock().unwrap();

      // check if the generated key is in the map
      assert!(l.contains_key(&key));
    }

    // check to see if listen inputs a handler function properly into the LISTENERS map.
    #[test]
    fn listeners_check_fn(e in "[a-z]+") {
       let listeners: Listeners = Default::default();
       // clone e as the key
       let key = e.clone();
       // pass e and an dummy func into listen
       listeners.listen(e, None, event_fn);

       // lock mutex
       let mut l = listeners.inner.handlers.lock().unwrap();

       // check if l contains key
       if l.contains_key(&key) {
        // grab key if it exists
        let handler = l.get_mut(&key);
        // check to see if we get back a handler or not
        match handler {
          // pass on Some(handler)
          Some(_) => {},
          // Fail on None
          None => panic!("handler is None")
        }
      }
    }

    // check to see if on_event properly grabs the stored function from listen.
    #[test]
    fn check_on_event(key in "[a-z]+", d in "[a-z]+") {
      let listeners: Listeners = Default::default();
      // call listen with e and the event_fn dummy func
      listeners.listen(key.clone(), None, event_fn);
      // call on event with e and d.
      listeners.trigger(&key, None, Some(d));

      // lock the mutex
      let l = listeners.inner.handlers.lock().unwrap();

      // assert that the key is contained in the listeners map
      assert!(l.contains_key(&key));
    }
  }
}
