use proton_ui::Handle;
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct EventHandler {
  on_event: Box<dyn FnOnce(String)>,
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

pub fn listen<F: FnOnce(String) + 'static>(id: &'static str, handler: F) {
  LISTENERS.with(|listeners| {
    let mut l = listeners.lock().unwrap();
    l.insert(
      id.to_string(),
      EventHandler {
        on_event: Box::new(handler),
      },
    );
  });
}

pub fn emit<T: 'static>(webview_handle: Handle<T>, event: &'static str, mut payload: String) {
  let salt = crate::salt::generate();
  if payload == "" {
    payload = "void 0".to_string();
  }

  webview_handle
    .dispatch(move |_webview| {
      _webview.eval(&format!(
        "window['{}']({{type: '{}', payload: {}}}, '{}')",
        emit_function_name(),
        event,
        payload,
        salt
      ))
    })
    .unwrap();
}

pub fn on_event(event: String, data: String) {
  LISTENERS.with(|l| {
    let mut listeners = l.lock().unwrap();

    let key = event.clone();

    if listeners.contains_key(&event) {
      let handler = listeners.remove(&event).unwrap();
      (handler.on_event)(data);
    }

    listeners.remove(&key);
  });
}
