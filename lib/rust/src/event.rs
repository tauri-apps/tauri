use proton_ui::{Handle, WebView};
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct EventHandler {
  on_event: Box<dyn FnOnce(String)>,
}

thread_local!(static LISTENERS: Arc<Mutex<HashMap<String, EventHandler>>> = Arc::new(Mutex::new(HashMap::new())));

lazy_static! {
  static ref PROMPT_FUNCTION_NAME: String = uuid::Uuid::new_v4().to_string();
  static ref EVENT_LISTENERS_OBJECT_NAME: String = uuid::Uuid::new_v4().to_string();
}

pub fn prompt_function_name() -> String {
  PROMPT_FUNCTION_NAME.to_string()
}

pub fn event_listeners_object_name() -> String {
  EVENT_LISTENERS_OBJECT_NAME.to_string()
}

pub fn prompt<T: 'static, F: FnOnce(String) + 'static>(
  webview: &mut WebView<'_, T>,
  id: &'static str,
  payload: String,
  handler: F,
) {
  LISTENERS.with(|listeners| {
    let mut l = listeners.lock().unwrap();
    l.insert(
      id.to_string(),
      EventHandler {
        on_event: Box::new(handler),
      },
    );
  });

  trigger(webview.handle(), id, payload);
}

pub fn trigger<T: 'static>(webview_handle: Handle<T>, id: &'static str, mut payload: String) {
  let salt = crate::salt::generate();
  if payload == "" {
    payload = "void 0".to_string();
  }

  webview_handle
    .dispatch(move |_webview| {
      _webview.eval(&format!(
        "window['{}']({{type: '{}', payload: {}}}, '{}')",
        prompt_function_name(),
        id,
        payload,
        salt
      ))
    })
    .unwrap();
}

pub fn answer(id: String, data: String, salt: String) {
  if crate::salt::is_valid(salt) {
    LISTENERS.with(|l| {
      let mut listeners = l.lock().unwrap();

      let key = id.clone();

      if listeners.contains_key(&id) {
        let handler = listeners.remove(&id).unwrap();
        (handler.on_event)(data);
      }

      listeners.remove(&key);
    });
  }
}
