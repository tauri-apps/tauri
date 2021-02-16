use std::{
  boxed::Box,
  collections::HashMap,
  sync::{Arc, Mutex},
};

use crate::ApplicationDispatcherExt;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::Value as JsonValue;

/// An event handler.
struct EventHandler {
  /// A event handler might be global or tied to a window.
  window_label: Option<String>,
  /// The on event callback.
  on_event: Box<dyn FnMut(Option<String>) + Send>,
}

type Listeners = Arc<Mutex<HashMap<String, Vec<EventHandler>>>>;

lazy_static! {
  static ref EMIT_FUNCTION_NAME: String = uuid::Uuid::new_v4().to_string();
  static ref EVENT_LISTENERS_OBJECT_NAME: String = uuid::Uuid::new_v4().to_string();
  static ref EVENT_QUEUE_OBJECT_NAME: String = uuid::Uuid::new_v4().to_string();
}

/// Gets the listeners map.
fn listeners() -> &'static Listeners {
  static LISTENERS: Lazy<Listeners> = Lazy::new(Default::default);
  &LISTENERS
}

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

/// Adds an event listener for JS events.
pub fn listen<F: FnMut(Option<String>) + Send + 'static>(
  id: impl AsRef<str>,
  window_label: Option<String>,
  handler: F,
) {
  let mut l = listeners()
    .lock()
    .expect("Failed to lock listeners: listen()");
  let handler = EventHandler {
    window_label,
    on_event: Box::new(handler),
  };
  if let Some(listeners) = l.get_mut(id.as_ref()) {
    listeners.push(handler);
  } else {
    l.insert(id.as_ref().to_string(), vec![handler]);
  }
}

/// Emits an event to JS.
pub fn emit<D: ApplicationDispatcherExt, S: Serialize>(
  webview_dispatcher: &crate::WebviewDispatcher<D>,
  event: impl AsRef<str>,
  payload: Option<S>,
) -> crate::Result<()> {
  let salt = crate::salt::generate();

  let js_payload = if let Some(payload_value) = payload {
    serde_json::to_value(payload_value)?
  } else {
    JsonValue::Null
  };

  webview_dispatcher.eval(&format!(
    "window['{}']({{type: '{}', payload: {}}}, '{}')",
    emit_function_name(),
    event.as_ref(),
    js_payload,
    salt
  ));

  Ok(())
}

/// Triggers the given event with its payload.
pub fn on_event(event: String, window_label: Option<&str>, data: Option<String>) {
  let mut l = listeners()
    .lock()
    .expect("Failed to lock listeners: on_event()");

  if l.contains_key(&event) {
    let listeners = l.get_mut(&event).expect("Failed to get mutable handler");
    for handler in listeners {
      if let Some(target_window_label) = window_label {
        // if the emitted event targets a specifid window, only triggers the listeners associated to that window
        if handler.window_label.as_deref() == Some(target_window_label) {
          (handler.on_event)(data.clone())
        }
      } else {
        // otherwise triggers all listeners
        (handler.on_event)(data.clone())
      }
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use proptest::prelude::*;

  // dummy event handler function
  fn event_fn(s: Option<String>) {
    println!("{:?}", s);
  }

  proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    // check to see if listen() is properly passing keys into the LISTENERS map
    fn listeners_check_key(e in "[a-z]+") {
      // clone e as the key
      let key = e.clone();
      // pass e and an dummy func into listen
      listen(e, None, event_fn);

      // lock mutex
      let l = listeners().lock().unwrap();

      // check if the generated key is in the map
      assert_eq!(l.contains_key(&key), true);
    }

    #[test]
    // check to see if listen inputs a handler function properly into the LISTENERS map.
    fn listeners_check_fn(e in "[a-z]+") {
       // clone e as the key
       let key = e.clone();
       // pass e and an dummy func into listen
       listen(e, None, event_fn);

       // lock mutex
       let mut l = listeners().lock().unwrap();

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

    #[test]
    // check to see if on_event properly grabs the stored function from listen.
    fn check_on_event(e in "[a-z]+", d in "[a-z]+") {
      // clone e as the key
      let key = e.clone();
      // call listen with e and the event_fn dummy func
      listen(e.clone(), None, event_fn);
      // call on event with e and d.
      on_event(e, None, Some(d));

      // lock the mutex
      let l = listeners().lock().unwrap();

      // assert that the key is contained in the listeners map
      assert!(l.contains_key(&key));
    }
  }
}
