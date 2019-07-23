use std::collections::HashMap;
use std::boxed::Box;
use std::sync::Mutex;

struct EventHandler {
  on_event: Box<Fn(String)>
}

thread_local!(static LISTENERS: Mutex<HashMap<String, EventHandler>> = Mutex::new(HashMap::new()));

pub fn prompt<F: Fn(String) + 'static>(id: String, handler: F) {
  LISTENERS.with(|listeners| {
    listeners.lock().unwrap().insert(id, EventHandler {
      on_event: Box::new(handler)
    });
  });
}

pub fn answer(id: String, data: String) {
  LISTENERS.with(|listeners| {
    let mut l = listeners.lock().unwrap();
    match l.get(&id) {
      Some(handler) => {
        (handler.on_event)(data);
        l.remove(&id);
      }
      None => {}
    }
  });
}