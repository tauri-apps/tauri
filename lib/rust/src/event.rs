use proton_ui::WebView;
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::Mutex;

struct EventHandler {
  on_event: Box<Fn(String)>,
}

thread_local!(static LISTENERS: Mutex<HashMap<String, EventHandler>> = Mutex::new(HashMap::new()));

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
          "window.protonPrompt({{type: '{}', payload: {}}}, '{}')",
          id, payload, salt
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
