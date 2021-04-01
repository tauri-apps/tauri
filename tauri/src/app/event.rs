use crate::{Manager, Tag};
use once_cell::sync::Lazy;
use std::{
  boxed::Box,
  collections::HashMap,
  fmt,
  hash::Hash,
  sync::{Arc, Mutex},
};
use uuid::Uuid;

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

struct Handler<L: Tag> {
  window: Option<L>,
  callback: Box<dyn Fn(EventPayload) + Send>,
}

type Handlers<E, L> = HashMap<E, HashMap<HandlerId, Handler<L>>>;

#[derive(Clone)]
pub struct Listeners<M: Manager> {
  inner: Arc<Mutex<Handlers<M::Event, M::Label>>>,
}

impl<M: Manager> Default for Listeners<M> {
  fn default() -> Self {
    Self {
      inner: Arc::new(Mutex::default()),
    }
  }
}

impl<M: Manager> Listeners<M> {
  /// Adds an event listener for JS events.
  pub fn listen<F: Fn(EventPayload) + Send + 'static>(
    &self,
    event: M::Event,
    window: Option<M::Label>,
    handler: F,
  ) -> HandlerId {
    let id = HandlerId::default();
    let handler = Handler {
      window,
      callback: Box::new(handler),
    };
    self
      .inner
      .lock()
      .expect("poisoned event mutex")
      .entry(event)
      .or_default()
      .insert(id, handler);
    id
  }

  /// Listen to a JS event and immediately unlisten.
  pub fn once<F: Fn(EventPayload) + Send + 'static>(
    &self,
    event: M::Event,
    window: Option<M::Label>,
    handler: F,
  ) {
    let self_ = self.clone();
    self.listen(event, window, move |e| {
      self_.unlisten(e.id);
      handler(e);
    });
  }

  /// Removes an event listener.
  pub fn unlisten(&self, handler_id: HandlerId) {
    self
      .inner
      .lock()
      .expect("poisoned event mutex")
      .values_mut()
      .for_each(|handler| {
        handler.remove(&handler_id);
      })
  }

  /// Triggers the given global event with its payload.
  pub(crate) fn trigger(&self, event: M::Event, window: Option<M::Label>, data: Option<String>) {
    if let Some(handlers) = self.inner.lock().expect("poisoned event mutex").get(&event) {
      for (&id, handler) in handlers {
        if window.is_none() || window == handler.window {
          let data = data.clone();
          let payload = EventPayload { id, data };
          (handler.callback)(payload)
        }
      }
    }
  }
}

static EMIT_FUNCTION_NAME: Lazy<Uuid> = Lazy::new(Uuid::new_v4);
static EVENT_LISTENERS_OBJECT_NAME: Lazy<Uuid> = Lazy::new(Uuid::new_v4);
static EVENT_QUEUE_OBJECT_NAME: Lazy<Uuid> = Lazy::new(Uuid::new_v4);

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
