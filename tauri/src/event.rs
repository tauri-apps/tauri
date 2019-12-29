use std::boxed::Box;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use web_view::Handle;

struct EventHandler {
  on_event: Box<dyn FnMut(String)>,
}

thread_local!(static LISTENERS: Arc<Mutex<HashMap<String, EventHandler>>> = Arc::new(Mutex::new(HashMap::new())));

lazy_static! {
  static ref EMIT_FUNCTION_NAME: String = uuid::Uuid::new_v4().to_string();
  static ref EVENT_LISTENERS_OBJECT_NAME: String = uuid::Uuid::new_v4().to_string();
  static ref EVENT_QUEUE_OBJECT_NAME: String = uuid::Uuid::new_v4().to_string();
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

pub fn listen<F: FnMut(String) + 'static>(id: String, handler: F) {
  LISTENERS.with(|listeners| {
    let mut l = listeners
      .lock()
      .expect("Failed to lock listeners: listen()");
    l.insert(
      id,
      EventHandler {
        on_event: Box::new(handler),
      },
    );
  });
}

pub fn emit<T: 'static>(webview_handle: &Handle<T>, event: String, mut payload: String) {
  let salt = crate::salt::generate();
  if payload == "" {
    payload = "void 0".to_string();
  }

  webview_handle
    .dispatch(move |_webview| {
      _webview.eval(&format!(
        "window['{}']({{type: '{}', payload: {}}}, '{}')",
        emit_function_name(),
        event.as_str(),
        payload,
        salt
      ))
    })
    .expect("Failed to dispatch JS from emit");
}

pub fn on_event(event: String, data: String) {
  LISTENERS.with(|listeners| {
    let mut l = listeners
      .lock()
      .expect("Failed to lock listeners: on_event()");

    let key = event.clone();

    if l.contains_key(&key) {
      let handler = l.get_mut(&key).expect("Failed to get mutable handler");
      (handler.on_event)(data);
    }
  });
}

#[cfg(test)]
mod test {
  use crate::event::*;
  use proptest::prelude::*;

  fn event_fn(s: String) {
    println!("{}-works", s)
  }

  proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    // check to see if listen() is properly passing keys into the LISTENERS map
    fn listeners_check_key(e in ".+") {
      // clone e as the key
      let key = e.clone();
      // pass e and an empty closure into listen
      listen(e, event_fn);

      // open listeners
      LISTENERS.with(|lis| {
        // lock mutex
        let l = lis.lock().unwrap();

        // check if the generated key is in the map
        assert_eq!(l.contains_key(&key), true);
      });
    }

    // #[test]
    // fn listeners_check_fn(e in "[a-z]+") {
    //    // clone e as the key
    //    let key = e.clone();
    //    // pass e and an empty closure into listen
    //    listen(e, event_fn);

    //    // open listeners
    //    LISTENERS.with(|lis| {
    //      // lock mutex
    //     let mut l = lis.lock().unwrap();

    //     if l.contains_key(&key) {
    //       let handler = l.get_mut(&key).unwrap();

    //     }
    //   });
    // }
  }
}
