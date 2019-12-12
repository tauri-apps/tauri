use std::boxed::Box;
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

use futures_executor::ThreadPool;
use once_cell::sync::OnceCell;
use web_view::Handle;

struct EventHandler(Box<dyn FnMut(String) -> dyn Future<Output=()>>);

static LISTENERS: Arc<Mutex<HashMap<String, EventHandler>>> = Default::default();
static THREAD_POOL: OnceCell<ThreadPool> = OnceCell::new();
static EMIT_FUNCTION_NAME: String = uuid::Uuid::new_v4().to_string();
static EVENT_LISTENERS_OBJECT_NAME: String = uuid::Uuid::new_v4().to_string();
static EVENT_QUEUE_OBJECT_NAME: String = uuid::Uuid::new_v4().to_string();


pub fn emit_function_name() -> String {
  EMIT_FUNCTION_NAME.to_string()
}

pub fn event_listeners_object_name() -> String {
  EVENT_LISTENERS_OBJECT_NAME.to_string()
}

pub fn event_queue_object_name() -> String {
  EVENT_QUEUE_OBJECT_NAME.to_string()
}

pub fn listen<F: FnMut(String) -> Fut + 'static, Fut: Future<Output=()>>(id: &'static str, handler: F) {
    let mut l = LISTENERS.lock().unwrap();
    l.insert(
      id.to_string(),
      EventHandler(Box::new(handler))
    );
}

pub fn emit<T: 'static>(webview_handle: &Handle<T>, event: &'static str, mut payload: String) {
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
  let mut l = LISTENERS.lock().unwrap();

  let key = event.clone();

  if l.contains_key(&key) {
    let handler = l.get_mut(&key).unwrap();
    let future = Box::new((handler.0)(data));
    if let Some(pool) = THREAD_POOL.get() {
      (*pool).spawn_ok(future);
    }
  }
}

pub fn start_threadpool<S: Into<String>>(num_threads: usize, prefix: S) -> Result<(), std::io::Error> {
  if let None = THREAD_POOL.get() {
    let pool = ThreadPool::builder()
        .pool_size(num_threads)
        .name_prefix(prefix)
        .create()?;

    THREAD_POOL.set(pool);
  }

  Ok(())
}
