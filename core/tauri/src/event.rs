// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::runtime::tag::Tag;
use std::{
  boxed::Box,
  collections::HashMap,
  fmt,
  hash::Hash,
  sync::{Arc, Mutex},
};
use uuid::Uuid;

/// Represents an event handler.
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EventHandler(Uuid);

impl fmt::Display for EventHandler {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

/// An event that was triggered.
#[derive(Debug, Clone)]
pub struct Event {
  id: EventHandler,
  data: Option<String>,
}

impl Event {
  /// The [`EventHandler`] that was triggered.
  pub fn id(&self) -> EventHandler {
    self.id
  }

  /// The event payload.
  pub fn payload(&self) -> Option<&str> {
    self.data.as_deref()
  }
}

/// What to do with the pending handler when resolving it?
enum Pending<Event: Tag, Window: Tag> {
  Unlisten(EventHandler),
  Listen(EventHandler, Event, Handler<Window>),
  Trigger(Event, Option<Window>, Option<String>),
}

/// Stored in [`Listeners`] to be called upon when the event that stored it is triggered.
struct Handler<Window: Tag> {
  window: Option<Window>,
  callback: Box<dyn Fn(Event) + Send>,
}

/// A collection of handlers. Multiple handlers can represent the same event.
type Handlers<Event, Window> = HashMap<Event, HashMap<EventHandler, Handler<Window>>>;

/// Holds event handlers and pending event handlers, along with the salts associating them.
struct InnerListeners<Event: Tag, Window: Tag> {
  handlers: Mutex<Handlers<Event, Window>>,
  pending: Mutex<Vec<Pending<Event, Window>>>,
  function_name: Uuid,
  listeners_object_name: Uuid,
  queue_object_name: Uuid,
}

/// A self-contained event manager.
pub(crate) struct Listeners<Event: Tag, Window: Tag> {
  inner: Arc<InnerListeners<Event, Window>>,
}

impl<Event: Tag, Window: Tag> Default for Listeners<Event, Window> {
  fn default() -> Self {
    Self {
      inner: Arc::new(InnerListeners {
        handlers: Mutex::default(),
        pending: Mutex::default(),
        function_name: Uuid::new_v4(),
        listeners_object_name: Uuid::new_v4(),
        queue_object_name: Uuid::new_v4(),
      }),
    }
  }
}

impl<Event: Tag, Window: Tag> Clone for Listeners<Event, Window> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<Event: Tag, Window: Tag> Listeners<Event, Window> {
  /// Randomly generated function name to represent the JavaScript event function.
  pub(crate) fn function_name(&self) -> String {
    self.inner.function_name.to_string()
  }

  /// Randomly generated listener object name to represent the JavaScript event listener object.
  pub(crate) fn listeners_object_name(&self) -> String {
    self.inner.listeners_object_name.to_string()
  }

  /// Randomly generated queue object name to represent the JavaScript event queue object.
  pub(crate) fn queue_object_name(&self) -> String {
    self.inner.queue_object_name.to_string()
  }

  /// Insert a pending event action to the queue.
  fn insert_pending(&self, action: Pending<Event, Window>) {
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
        Pending::Trigger(event, window, payload) => self.trigger(event, window, payload),
      }
    }
  }

  fn listen_(&self, id: EventHandler, event: Event, handler: Handler<Window>) {
    match self.inner.handlers.try_lock() {
      Err(_) => self.insert_pending(Pending::Listen(id, event, handler)),
      Ok(mut lock) => {
        lock.entry(event).or_default().insert(id, handler);
      }
    }
  }

  /// Adds an event listener for JS events.
  pub(crate) fn listen<F: Fn(self::Event) + Send + 'static>(
    &self,
    event: Event,
    window: Option<Window>,
    handler: F,
  ) -> EventHandler {
    let id = EventHandler(Uuid::new_v4());
    let handler = Handler {
      window,
      callback: Box::new(handler),
    };

    self.listen_(id, event, handler);

    id
  }

  /// Listen to a JS event and immediately unlisten.
  pub(crate) fn once<F: Fn(self::Event) + Send + 'static>(
    &self,
    event: Event,
    window: Option<Window>,
    handler: F,
  ) -> EventHandler {
    let self_ = self.clone();
    self.listen(event, window, move |event| {
      self_.unlisten(event.id);
      handler(event);
    })
  }

  /// Removes an event listener.
  pub(crate) fn unlisten(&self, handler_id: EventHandler) {
    match self.inner.handlers.try_lock() {
      Err(_) => self.insert_pending(Pending::Unlisten(handler_id)),
      Ok(mut lock) => lock.values_mut().for_each(|handler| {
        handler.remove(&handler_id);
      }),
    }
  }

  /// Triggers the given global event with its payload.
  pub(crate) fn trigger(&self, event: Event, window: Option<Window>, payload: Option<String>) {
    let mut maybe_pending = false;
    match self.inner.handlers.try_lock() {
      Err(_) => self.insert_pending(Pending::Trigger(event, window, payload)),
      Ok(lock) => {
        if let Some(handlers) = lock.get(&event) {
          for (&id, handler) in handlers {
            if window.is_none() || window == handler.window {
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
    println!("{:?}", s);
  }

  proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    // check to see if listen() is properly passing keys into the LISTENERS map
    #[test]
    fn listeners_check_key(e in "[a-z]+") {
      let listeners: Listeners<String, String> = Default::default();
      // clone e as the key
      let key = e.clone();
      // pass e and an dummy func into listen
      listeners.listen(e, None, event_fn);

      // lock mutex
      let l = listeners.inner.handlers.lock().unwrap();

      // check if the generated key is in the map
      assert_eq!(l.contains_key(&key), true);
    }

    // check to see if listen inputs a handler function properly into the LISTENERS map.
    #[test]
    fn listeners_check_fn(e in "[a-z]+") {
       let listeners: Listeners<String, String> = Default::default();
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
    fn check_on_event(e in "[a-z]+", d in "[a-z]+") {
      let listeners: Listeners<String, String> = Default::default();
      // clone e as the key
      let key = e.clone();
      // call listen with e and the event_fn dummy func
      listeners.listen(e.clone(), None, event_fn);
      // call on event with e and d.
      listeners.trigger(e, None, Some(d));

      // lock the mutex
      let l = listeners.inner.handlers.lock().unwrap();

      // assert that the key is contained in the listeners map
      assert!(l.contains_key(&key));
    }
  }
}
