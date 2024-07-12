// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{Runtime, Webview};

use super::{EmitArgs, Event, EventId, EventTarget};

use std::{
  boxed::Box,
  cell::Cell,
  collections::{HashMap, HashSet},
  sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
  },
};

/// What to do with the pending handler when resolving it?
enum Pending {
  Unlisten(EventId),
  Listen {
    id: EventId,
    event: String,
    handler: Handler,
  },
  Emit(EmitArgs),
}

/// Stored in [`Listeners`] to be called upon, when the event that stored it, is triggered.
struct Handler {
  target: EventTarget,
  callback: Box<dyn Fn(Event) + Send>,
}

impl Handler {
  fn new<F: Fn(Event) + Send + 'static>(target: EventTarget, callback: F) -> Self {
    Self {
      target,
      callback: Box::new(callback),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct JsHandler {
  target: EventTarget,
  id: EventId,
}

impl JsHandler {
  fn new(target: EventTarget, id: EventId) -> Self {
    Self { target, id }
  }
}

type WebviewLabel = String;
type EventName = String;

/// Holds event handlers and pending event handlers, along with the salts associating them.
struct InnerListeners {
  pending: Mutex<Vec<Pending>>,
  handlers: Mutex<HashMap<EventName, HashMap<EventId, Handler>>>,
  js_event_listeners: Mutex<HashMap<WebviewLabel, HashMap<EventName, HashSet<JsHandler>>>>,
  function_name: &'static str,
  listeners_object_name: &'static str,
  next_event_id: Arc<AtomicU32>,
}

/// A self-contained event manager.
#[derive(Clone)]
pub struct Listeners {
  inner: Arc<InnerListeners>,
}

impl Default for Listeners {
  fn default() -> Self {
    Self {
      inner: Arc::new(InnerListeners {
        pending: Mutex::default(),
        handlers: Mutex::default(),
        js_event_listeners: Mutex::default(),
        function_name: "__internal_unstable_listeners_function_id__",
        listeners_object_name: "__internal_unstable_listeners_object_id__",
        next_event_id: Default::default(),
      }),
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
        Pending::Listen { id, event, handler } => self.listen_with_id(id, event, handler),
        Pending::Emit(args) => {
          self.emit(args)?;
        }
      }
    }

    Ok(())
  }

  fn listen_with_id(&self, id: EventId, event: String, handler: Handler) {
    match self.inner.handlers.try_lock() {
      Err(_) => self.insert_pending(Pending::Listen { id, event, handler }),
      Ok(mut lock) => {
        lock.entry(event).or_default().insert(id, handler);
      }
    }
  }

  /// Adds an event listener.
  pub(crate) fn listen<F: Fn(Event) + Send + 'static>(
    &self,
    event: String,
    target: EventTarget,
    handler: F,
  ) -> EventId {
    let id = self.next_event_id();
    let handler = Handler::new(target, handler);
    self.listen_with_id(id, event, handler);
    id
  }

  /// Listen to an event and immediately unlisten.
  pub(crate) fn once<F: FnOnce(Event) + Send + 'static>(
    &self,
    event: String,
    target: EventTarget,
    handler: F,
  ) -> EventId {
    let self_ = self.clone();
    let handler = Cell::new(Some(handler));

    self.listen(event, target, move |event| {
      let id = event.id;
      let handler = handler
        .take()
        .expect("attempted to call handler more than once");
      handler(event);
      self_.unlisten(id);
    })
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
  pub(crate) fn emit_filter<F>(&self, emit_args: EmitArgs, filter: Option<F>) -> crate::Result<()>
  where
    F: Fn(&EventTarget) -> bool,
  {
    let mut maybe_pending = false;

    match self.inner.handlers.try_lock() {
      Err(_) => self.insert_pending(Pending::Emit(emit_args)),
      Ok(lock) => {
        if let Some(handlers) = lock.get(&emit_args.event_name) {
          let handlers = handlers.iter();
          let handlers = handlers.filter(|(_, h)| match_any_or_filter(&h.target, &filter));
          for (&id, Handler { callback, .. }) in handlers {
            maybe_pending = true;
            (callback)(Event::new(id, emit_args.payload.clone()))
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
  pub(crate) fn emit(&self, emit_args: EmitArgs) -> crate::Result<()> {
    self.emit_filter(emit_args, None::<&dyn Fn(&EventTarget) -> bool>)
  }

  pub(crate) fn listen_js(
    &self,
    event: &str,
    source_webview_label: &str,
    target: EventTarget,
    id: EventId,
  ) {
    let mut listeners = self.inner.js_event_listeners.lock().unwrap();
    listeners
      .entry(source_webview_label.to_string())
      .or_default()
      .entry(event.to_string())
      .or_default()
      .insert(JsHandler::new(target, id));
  }

  pub(crate) fn unlisten_js(&self, event: &str, id: EventId) {
    let mut js_listeners = self.inner.js_event_listeners.lock().unwrap();
    let js_listeners = js_listeners.values_mut();
    for js_listeners in js_listeners {
      if let Some(handlers) = js_listeners.get_mut(event) {
        handlers.retain(|h| h.id != id);

        if handlers.is_empty() {
          js_listeners.remove(event);
        }
      }
    }
  }

  pub(crate) fn has_js_listener<F: Fn(&EventTarget) -> bool>(
    &self,
    event: &str,
    filter: F,
  ) -> bool {
    let js_listeners = self.inner.js_event_listeners.lock().unwrap();
    js_listeners.values().any(|events| {
      events
        .get(event)
        .map(|handlers| handlers.iter().any(|handler| filter(&handler.target)))
        .unwrap_or(false)
    })
  }

  pub(crate) fn emit_js_filter<'a, R, I, F>(
    &self,
    mut webviews: I,
    event: &str,
    emit_args: &EmitArgs,
    filter: Option<F>,
  ) -> crate::Result<()>
  where
    R: Runtime,
    I: Iterator<Item = &'a Webview<R>>,
    F: Fn(&EventTarget) -> bool,
  {
    let js_listeners = self.inner.js_event_listeners.lock().unwrap();
    webviews.try_for_each(|webview| {
      if let Some(handlers) = js_listeners.get(webview.label()).and_then(|s| s.get(event)) {
        let ids = handlers
          .iter()
          .filter(|handler| match_any_or_filter(&handler.target, &filter))
          .map(|handler| handler.id)
          .collect::<Vec<_>>();
        webview.emit_js(emit_args, &ids)?;
      }

      Ok(())
    })
  }

  pub(crate) fn emit_js<'a, R, I>(
    &self,
    webviews: I,
    event: &str,
    emit_args: &EmitArgs,
  ) -> crate::Result<()>
  where
    R: Runtime,
    I: Iterator<Item = &'a Webview<R>>,
  {
    self.emit_js_filter(
      webviews,
      event,
      emit_args,
      None::<&dyn Fn(&EventTarget) -> bool>,
    )
  }
}

#[inline(always)]
fn match_any_or_filter<F: Fn(&EventTarget) -> bool>(
  target: &EventTarget,
  filter: &Option<F>,
) -> bool {
  *target == EventTarget::Any || filter.as_ref().map(|f| f(target)).unwrap_or(true)
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::event::EventTarget;
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
      listeners.listen(e, EventTarget::Any, event_fn);

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
       listeners.listen(e, EventTarget::Any, event_fn);

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
      // call listen with key and the event_fn dummy func
      listeners.listen(key.clone(), EventTarget::Any, event_fn);
      // call on event with key and d.
      listeners.emit(EmitArgs {
        event_name: key.clone(),
        event: serde_json::to_string(&key).unwrap(),
        payload: serde_json::to_string(&d).unwrap()
      })?;

      // lock the mutex
      let l = listeners.inner.handlers.lock().unwrap();

      // assert that the key is contained in the listeners map
      assert!(l.contains_key(&key));
    }
  }
}
