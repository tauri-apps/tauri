use std::boxed::Box;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use web_view::Handle;
use once_cell::sync::Lazy;

struct EventHandler {
  on_event: Box<dyn FnMut(Option<String>) + Send>,
}

type Listeners = Arc<Mutex<HashMap<String, EventHandler>>>;

lazy_static! {
  static ref EMIT_FUNCTION_NAME: String = uuid::Uuid::new_v4().to_string();
  static ref EVENT_LISTENERS_OBJECT_NAME: String = uuid::Uuid::new_v4().to_string();
  static ref EVENT_QUEUE_OBJECT_NAME: String = uuid::Uuid::new_v4().to_string();
}

fn listeners() -> &'static Listeners {
  static LISTENERS: Lazy<Listeners> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));
  &LISTENERS
}

pub fn emit_function_name() -> String {
  EMIT_FUNCTION_NAME.to_string()
}

pub fn event_listeners_object_name() -> String {
  EVENT_LISTENERS_OBJECT_NAME.to_string()
}

pub fn event_queue_object_name() -> String {
  EVENT_QUEUE_OBJECT_NAME.to_string()
}

pub fn listen<F: FnMut(Option<String>) + Send + 'static>(id: String, handler: F) {
  let mut l = listeners()
    .lock()
    .expect("Failed to lock listeners: listen()");
  l.insert(
    id,
    EventHandler {
      on_event: Box::new(handler),
    },
  );
}

pub fn emit<T: 'static>(webview_handle: &Handle<T>, event: String, payload: Option<String>) {
  let salt = crate::salt::generate();

  let js_payload = if let Some(payload_str) = payload {
    payload_str
  } else {
    "void 0".to_string()
  };

  webview_handle
    .dispatch(move |_webview| {
      _webview.eval(&format!(
        "window['{}']({{type: '{}', payload: {}}}, '{}')",
        emit_function_name(),
        event.as_str(),
        js_payload,
        salt
      ))
    })
    .expect("Failed to dispatch JS from emit");
}

pub fn on_event(event: String, data: Option<String>) {
  let mut l = listeners()
    .lock()
    .expect("Failed to lock listeners: on_event()");

  if l.contains_key(&event) {
    let handler = l.get_mut(&event).expect("Failed to get mutable handler");
    (handler.on_event)(data);
  }
}

#[cfg(test)]
mod test {
  use crate::event::*;
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
      listen(e, event_fn);

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
       listen(e, event_fn);

       // lock mutex
       let mut l = listeners().lock().unwrap();

       // check if l contains key
       if l.contains_key(&key) {
        // grab key if it exists
        let handler = l.get_mut(&key);
        // check to see if we get back a handler or not
        match handler {
          // pass on Some(handler)
          Some(_) => assert!(true),
          // Fail on None
          None => assert!(false)
        }
      }
    }

    #[test]
    // check to see if on_event properly grabs the stored function from listen.
    fn check_on_event(e in "[a-z]+", d in "[a-z]+") {
      // clone e as the key
      let key = e.clone();
      // call listen with e and the event_fn dummy func
      listen(e.clone(), event_fn);
      // call on event with e and d.
      on_event(e, Some(d));

      // lock the mutex
      let l = listeners().lock().unwrap();

      // assert that the key is contained in the listeners map
      assert!(l.contains_key(&key));
    }
  }
}