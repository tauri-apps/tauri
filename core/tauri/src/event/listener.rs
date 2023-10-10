// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{Runtime, Window};

use super::{Event, EventHandler};

use serde::Serialize;
use std::{
  boxed::Box,
  cell::Cell,
  collections::HashMap,
  sync::{Arc, Mutex},
};
use uuid::Uuid;

/// What to do with the pending handler when resolving it?
enum Pending<R: Runtime> {
  Unlisten(EventHandler),
  Listen(EventHandler, String, Handler<R>),
  Emit(String, Option<String>),
}

/// Stored in [`Listeners`] to be called upon when the event that stored it is triggered.
struct Handler<R: Runtime> {
  window: Option<Window<R>>,
  callback: Box<dyn Fn(Event) + Send>,
}

/// Holds event handlers and pending event handlers, along with the salts associating them.
struct InnerListeners<R: Runtime> {
  handlers: Mutex<HashMap<String, HashMap<EventHandler, Handler<R>>>>,
  pending: Mutex<Vec<Pending<R>>>,
  function_name: Uuid,
  listeners_object_name: Uuid,
}

/// A self-contained event manager.
pub struct Listeners<R: Runtime> {
  inner: Arc<InnerListeners<R>>,
}

impl<R: Runtime> Default for Listeners<R> {
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

impl<R: Runtime> Clone for Listeners<R> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<R: Runtime> Listeners<R> {
  /// Randomly generated function name to represent the JavaScript event function.
  pub(crate) fn function_name(&self) -> String {
    self.inner.function_name.to_string()
  }

  /// Randomly generated listener object name to represent the JavaScript event listener object.
  pub(crate) fn listeners_object_name(&self) -> String {
    self.inner.listeners_object_name.to_string()
  }

  /// Insert a pending event action to the queue.
  fn insert_pending(&self, action: Pending<R>) {
    self
      .inner
      .pending
      .lock()
      .expect("poisoned pending event queue")
      .push(action)
  }

  /// Finish all pending event actions.
  fn flush_pending(&self) -> crate::Result<()> {
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
        Pending::Emit(ref event, payload) => {
          self.emit(event, payload)?;
        }
      }
    }

    Ok(())
  }

  fn listen_(&self, id: EventHandler, event: String, handler: Handler<R>) {
    match self.inner.handlers.try_lock() {
      Err(_) => self.insert_pending(Pending::Listen(id, event, handler)),
      Ok(mut lock) => {
        lock.entry(event).or_default().insert(id, handler);
      }
    }
  }

  /// Adds an event listener.
  pub(crate) fn listen<F: Fn(Event) + Send + 'static>(
    &self,
    event: String,
    window: Option<Window<R>>,
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

  /// Listen to an event and immediately unlisten.
  pub(crate) fn once<F: FnOnce(Event) + Send + 'static>(
    &self,
    event: String,
    window: Option<Window<R>>,
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

  /// Emits the given event with its payload based on a filter.
  pub(crate) fn emit_filter<S, F>(
    &self,
    event: &str,
    payload: Option<S>,
    filter: Option<F>,
  ) -> crate::Result<()>
  where
    S: Serialize + Clone,
    F: Fn(&Window<R>) -> bool,
  {
    let mut maybe_pending = false;
    match self.inner.handlers.try_lock() {
      Err(_) => self.insert_pending(Pending::Emit(
        event.to_owned(),
        payload.map(|p| serde_json::to_string(&p)).transpose()?,
      )),
      Ok(lock) => {
        if let Some(handlers) = lock.get(event) {
          let handlers = if let Some(filter) = filter {
            handlers
              .iter()
              .filter(|h| {
                h.1
                  .window
                  .as_ref()
                  .map(|w| {
                    // clippy sees this as redundant closure but
                    // fixing it will result in a compiler error
                    #[allow(clippy::redundant_closure)]
                    filter(w)
                  })
                  .unwrap_or(false)
              })
              .collect::<Vec<_>>()
          } else {
            handlers.iter().collect::<Vec<_>>()
          };

          if !handlers.is_empty() {
            let data = payload.map(|p| serde_json::to_string(&p)).transpose()?;

            for (&id, handler) in handlers {
              maybe_pending = true;
              (handler.callback)(self::Event {
                id,
                data: data.clone(),
              })
            }
          }
        }
      }
    }

    if maybe_pending {
      self.flush_pending()?;
    }

    Ok(())
  }

  /// Emits the given event with its payload.
  pub(crate) fn emit<S>(&self, event: &str, payload: Option<S>) -> crate::Result<()>
  where
    S: Serialize + Clone,
  {
    self.emit_filter(event, payload, None::<&dyn Fn(&Window<R>) -> bool>)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::MockRuntime;
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
      let listeners: Listeners<MockRuntime> = Default::default();
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
       let listeners: Listeners<MockRuntime> = Default::default();
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
      let listeners: Listeners<MockRuntime> = Default::default();
      // call listen with key and the event_fn dummy func
      listeners.listen(key.clone(), None, event_fn);
      // call on event with key and d.
      listeners.emit(&key,  Some(d))?;

      // lock the mutex
      let l = listeners.inner.handlers.lock().unwrap();

      // assert that the key is contained in the listeners map
      assert!(l.contains_key(&key));
    }
  }
}
