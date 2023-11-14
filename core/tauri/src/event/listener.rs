// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{Runtime, Webview};

use super::{EmitArgs, Event, EventId};

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
enum Pending<R: Runtime> {
  Unlisten(EventId),
  Listen(EventId, String, Handler<R>),
  Emit(EmitArgs),
}

/// Stored in [`Listeners`] to be called upon when the event that stored it is triggered.
struct Handler<R: Runtime> {
  webview: Option<Webview<R>>,
  callback: Box<dyn Fn(Event) + Send>,
}

/// Holds event handlers and pending event handlers, along with the salts associating them.
struct InnerListeners<R: Runtime> {
  handlers: Mutex<HashMap<String, HashMap<EventId, Handler<R>>>>,
  pending: Mutex<Vec<Pending<R>>>,
  function_name: &'static str,
  listeners_object_name: &'static str,
  next_event_id: Arc<AtomicU32>,
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
        function_name: "__internal_unstable_listeners_function_id__",
        listeners_object_name: "__internal_unstable_listeners_object_id__",
        next_event_id: Default::default(),
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
        Pending::Listen(id, event, handler) => self.listen_with_id(id, event, handler),
        Pending::Emit(args) => {
          self.emit(&args)?;
        }
      }
    }

    Ok(())
  }

  fn listen_with_id(&self, id: EventId, event: String, handler: Handler<R>) {
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
    webview: Option<Webview<R>>,
    handler: F,
  ) -> EventId {
    let id = self.next_event_id();
    let handler = Handler {
      webview,
      callback: Box::new(handler),
    };

    self.listen_with_id(id, event, handler);

    id
  }

  /// Listen to an event and immediately unlisten.
  pub(crate) fn once<F: FnOnce(Event) + Send + 'static>(
    &self,
    event: String,
    webview: Option<Webview<R>>,
    handler: F,
  ) {
    let self_ = self.clone();
    let handler = Cell::new(Some(handler));

    self.listen(event, webview, move |event| {
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

  /// Emits the given event with its payload based on a filter.
  pub(crate) fn emit_filter<F>(&self, emit_args: &EmitArgs, filter: Option<F>) -> crate::Result<()>
  where
    F: Fn(&Webview<R>) -> bool,
  {
    let mut maybe_pending = false;
    match self.inner.handlers.try_lock() {
      Err(_) => self.insert_pending(Pending::Emit(emit_args.clone())),
      Ok(lock) => {
        if let Some(handlers) = lock.get(&emit_args.event_name) {
          let handlers = if let Some(filter) = filter {
            handlers
              .iter()
              .filter(|h| {
                h.1
                  .webview
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
            for (&id, handler) in handlers {
              maybe_pending = true;
              (handler.callback)(self::Event {
                id,
                data: emit_args.payload.clone(),
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
  pub(crate) fn emit(&self, emit_args: &EmitArgs) -> crate::Result<()> {
    self.emit_filter(emit_args, None::<&dyn Fn(&Webview<R>) -> bool>)
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
      listeners.emit(&EmitArgs { event_name: key.clone(), event: serde_json::to_string(&key).unwrap(), source_window_label: "null".into(), payload: serde_json::to_string(&d).unwrap() })?;

      // lock the mutex
      let l = listeners.inner.handlers.lock().unwrap();

      // assert that the key is contained in the listeners map
      assert!(l.contains_key(&key));
    }
  }
}
