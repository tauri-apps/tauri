use proton_ui::WebView;
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::Mutex;

struct EventHandler {
  on_event: Box<Fn(String)>,
}

thread_local!(static LISTENERS: Mutex<HashMap<String, EventHandler>> = Mutex::new(HashMap::new()));
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

pub fn prompt<T: 'static, F: Fn(String) + 'static>(
  webview: &mut WebView<'_, T>,
  id: &'static str,
  mut payload: String,
  handler: F,
) {
  LISTENERS.with(|listeners| {
    listeners.lock().unwrap().insert(
      id.to_string(),
      EventHandler {
        on_event: Box::new(handler),
      },
    );

    let salt = crate::salt::generate();
    if payload == "" {
      payload = "void 0".to_string();
    }

    webview
      .handle()
      .dispatch(move |_webview| {
        _webview.eval(&format!(
          "window['{}']({{type: '{}', payload: {}}}, '{}')",
          prompt_function_name(), id, payload, salt
        ))
      })
      .unwrap();
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
