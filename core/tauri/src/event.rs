// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  boxed::Box,
  cell::Cell,
  collections::HashMap,
  fmt,
  hash::Hash,
  sync::{Arc, Mutex},
};
use uuid::Uuid;

/// Checks if an event name is valid.
pub fn is_event_name_valid(event: &str) -> bool {
  event
    .chars()
    .all(|c| c.is_alphanumeric() || c == '-' || c == '/' || c == ':' || c == '_')
}

pub fn assert_event_name_is_valid(event: &str) {
  assert!(
    is_event_name_valid(event),
    "Event name must include only alphanumeric characters, `-`, `/`, `:` and `_`."
  );
}

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
enum Pending {
  Unlisten(EventHandler),
  Listen(EventHandler, String, Handler),
  Trigger(String, Option<String>, Option<String>),
}

/// Stored in [`Listeners`] to be called upon when the event that stored it is triggered.
struct Handler {
  window: Option<String>,
  callback: Box<dyn Fn(Event) + Send>,
}

/// Holds event handlers and pending event handlers, along with the salts associating them.
struct InnerListeners {
  handlers: Mutex<HashMap<String, HashMap<EventHandler, Handler>>>,
  pending: Mutex<Vec<Pending>>,
  function_name: Uuid,
  listeners_object_name: Uuid,
}

/// A self-contained event manager.
pub(crate) struct Listeners {
  inner: Arc<InnerListeners>,
}

impl Default for Listeners {
  fn default() -> Self {
    Self {
      inner: Arc::new(InnerListeners {
        handlers: Mutex::default(),
        pending: Mutex::default(),
        function_name: Uuid::new_v4(),
        listeners_object_name: Uuid::new_v4(),
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
  /// Randomly generated function name to represent the JavaScript event function.
  pub(crate) fn function_name(&self) -> String {
    self.inner.function_name.to_string()
  }

  /// Randomly generated listener object name to represent the JavaScript event listener object.
  pub(crate) fn listeners_object_name(&self) -> String {
    self.inner.listeners_object_name.to_string()
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

  fn listen_(&self, id: EventHandler, event: String, handler: Handler) {
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
  pub(crate) fn once<F: FnOnce(Event) + Send + 'static>(
    &self,
    event: String,
    window: Option<String>,
    handler: F,
  ) -> EventHandler {
    let self_ = self.clone();
    let handler = Cell::new(Some(handler));

    self.listen(event, window, move |event| {
      self_.unlisten(event.id);
      let handler = handler
        .take()
        .expect("attempted to call handler more than once");
      handler(event)
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

pub fn unlisten_js(listeners_object_name: String, event_name: String, event_id: u32) -> String {
  format!(
    "
      (function () {{
        const listeners = (window['{listeners_object_name}'] || {{}})['{event_name}']
        if (listeners) {{
          const index = window['{listeners_object_name}']['{event_name}'].findIndex(e => e.id === {event_id})
          if (index > -1) {{
            window['{listeners_object_name}']['{event_name}'].splice(index, 1)
          }}
        }}
      }})()
    ",
  )
}

pub fn listen_js(
  listeners_object_name: String,
  event: String,
  event_id: u32,
  window_label: Option<String>,
  handler: String,
) -> String {
  format!(
    "
    (function () {{
      if (window['{listeners}'] === void 0) {{
        Object.defineProperty(window, '{listeners}', {{ value: Object.create(null) }});
      }}
      if (window['{listeners}'][{event}] === void 0) {{
        Object.defineProperty(window['{listeners}'], {event}, {{ value: [] }});
      }}
      const eventListeners = window['{listeners}'][{event}]
      const listener = {{
        id: {event_id},
        windowLabel: {window_label},
        handler: {handler}
      }};
      eventListeners.push(listener);
    }})()
  ",
    listeners = listeners_object_name,
    window_label = if let Some(l) = window_label {
      crate::runtime::window::assert_label_is_valid(&l);
      format!("'{l}'")
    } else {
      "null".to_owned()
    },
  )
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
