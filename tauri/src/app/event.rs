use crate::app::webview_manager::Label;
use once_cell::sync::Lazy;
use std::{
  boxed::Box,
  collections::HashMap,
  fmt,
  hash::{Hash, Hasher},
  sync::{Arc, Mutex},
};
use uuid::Uuid;

pub enum EventScope {
  Global,
  Window,
}

/// A randomly generated id that represents an event handler.
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HandlerId(Uuid);

impl Default for HandlerId {
  fn default() -> Self {
    Self(Uuid::new_v4())
  }
}

impl fmt::Display for HandlerId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

type Handler = Box<dyn Fn(EventPayload) + Send + 'static>;

#[derive(Clone)]
pub struct Listeners<E: Label, L: Label> {
  global: Arc<Mutex<HashMap<E, HashMap<HandlerId, Handler>>>>,
  window: Arc<Mutex<HashMap<L, HashMap<E, HashMap<HandlerId, Handler>>>>>,
}

impl<E: Label, L: Label> Listeners<E, L> {
  /// Create an empty set of listeners
  pub fn new() -> Self {
    Self {
      global: Arc::new(Mutex::new(HashMap::new())),
      window: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  /// Adds a global event listener for JS events.
  pub fn listen<F: Fn(EventPayload) + Send + 'static>(&self, event: E, handler: F) -> HandlerId {
    let id = HandlerId::default();
    self
      .global
      .lock()
      .expect(&format!("poisoned event mutex"))
      .entry(event)
      .or_default()
      .insert(id, Box::new(handler));
    id
  }

  /// Adds a window event listener for JS events.
  pub fn listen_window<F: Fn(EventPayload) + Send + 'static>(
    &self,
    window: L,
    event: E,
    handler: F,
  ) -> HandlerId {
    let id = HandlerId::default();
    self
      .window
      .lock()
      .expect(&format!("poisoned event mutex"))
      .entry(window)
      .or_default()
      .entry(event)
      .or_default()
      .insert(id, Box::new(handler));
    id
  }

  /// Listen to a global JS event and immediately unlisten.
  pub fn once<F: Fn(EventPayload) + Send + 'static>(&self, event: E, handler: F) {
    let self_ = self.clone();
    self.listen(event, move |e| {
      self_.unlisten(e.id);
      handler(e);
    });
  }

  /// Listen to an JS event on a window and immediately unlisten.
  pub fn once_window<F: Fn(EventPayload) + Send + 'static>(&self, window: L, event: E, handler: F) {
    let self_ = self.clone();
    self.listen_window(window, event, move |e| {
      self_.unlisten(e.id);
      handler(e);
    });
  }

  /// Removes a global event listener.
  pub fn unlisten(&self, handler_id: HandlerId) {
    self
      .global
      .lock()
      .expect("poisoned event mutex")
      .values_mut()
      .for_each(|handler| {
        handler.remove(&handler_id);
      })
  }

  /// Removes a window event listener.
  pub fn unlisten_window(&self, window: &L, handler_id: HandlerId) {
    if let Some(handlers) = self
      .window
      .lock()
      .expect("poisoned event mutex")
      .get_mut(window)
    {
      for h in handlers.values_mut() {
        h.remove(&handler_id);
      }
    }
  }

  /// Triggers the given global event with its payload.
  pub(crate) fn trigger(&self, event: E, data: Option<String>) {
    if let Some(handlers) = self
      .global
      .lock()
      .expect("poisoned event mutex")
      .get(&event)
    {
      for (&id, handler) in handlers {
        let data = data.clone();
        let payload = EventPayload { id, data };
        handler(payload)
      }
    }
  }

  /// Triggers the given global event with its payload.
  pub(crate) fn trigger_window(&self, window: &L, event: E, data: Option<String>) {
    if let Some(handlers) = self
      .window
      .lock()
      .expect("poisoned event mutex")
      .get(window)
      .and_then(|window| window.get(&event))
    {
      for (&id, handler) in handlers {
        let data = data.clone();
        let payload = EventPayload { id, data };
        handler(payload)
      }
    }
  }
}

static EMIT_FUNCTION_NAME: Lazy<Uuid> = Lazy::new(|| Uuid::new_v4());
static EVENT_LISTENERS_OBJECT_NAME: Lazy<Uuid> = Lazy::new(|| Uuid::new_v4());
static EVENT_QUEUE_OBJECT_NAME: Lazy<Uuid> = Lazy::new(|| Uuid::new_v4());

/// the emit JS function name
pub fn emit_function_name() -> String {
  EMIT_FUNCTION_NAME.to_string()
}

/// the event listeners JS object name
pub fn event_listeners_object_name() -> String {
  EVENT_LISTENERS_OBJECT_NAME.to_string()
}

/// the event queue JS object name
pub fn event_queue_object_name() -> String {
  EVENT_QUEUE_OBJECT_NAME.to_string()
}

#[derive(Debug, Clone)]
pub struct EventPayload {
  id: HandlerId,
  data: Option<String>,
}

impl EventPayload {
  /// The event identifier.
  pub fn id(&self) -> HandlerId {
    self.id
  }

  /// The event payload.
  pub fn payload(&self) -> Option<&String> {
    self.data.as_ref()
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use proptest::prelude::*;

  // dummy event handler function
  fn event_fn(s: EventPayload) {
    println!("{:?}", s);
  }

  proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    // check to see if listen() is properly passing keys into the LISTENERS map
    #[test]
    fn listeners_check_key(e in "[a-z]+") {
      let listeners: Listeners<String> = Default::default();
      // clone e as the key
      let key = e.clone();
      // pass e and an dummy func into listen
      listeners.listen(e, None, event_fn);

      // lock mutex
      let l = listeners.window.lock().unwrap();

      // check if the generated key is in the map
      assert_eq!(l.contains_key(&key), true);
    }

    // check to see if listen inputs a handler function properly into the LISTENERS map.
    #[test]
    fn listeners_check_fn(e in "[a-z]+") {
       let listeners: Listeners<String> = Default::default();
       // clone e as the key
       let key = e.clone();
       // pass e and an dummy func into listen
       listenerslisten(e, None, event_fn);

       // lock mutex
       let mut l = listeners.window.lock().unwrap();

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
      let listeners: Listeners<String> = Default::default();
      // clone e as the key
      let key = e.clone();
      // call listen with e and the event_fn dummy func
      listeners.listen(e.clone(), None, event_fn);
      // call on event with e and d.
      listeners.on_event(e, None, Some(d));

      // lock the mutex
      let l = listeners.window.lock().unwrap();

      // assert that the key is contained in the listeners map
      assert!(l.contains_key(&key));
    }
  }
}
