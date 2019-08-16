use std::sync::{Arc, Mutex};
use tauri_ui::WebView;

pub trait Extension {
  #[allow(unused_variables)]
  fn created(&self, webview: &mut WebView<'_, ()>) {}

  #[allow(unused_variables)]
  fn ready(&self, webview: &mut WebView<'_, ()>) {}

  #[allow(unused_variables)]
  fn extend_api(&self, webview: &mut WebView<'_, ()>, payload: &str) {}
}

thread_local!(static EXTENSIONS: Arc<Mutex<Vec<Box<dyn Extension>>>> = Arc::new(Mutex::new(Vec::new())));

pub fn register(ext: impl Extension + 'static) {
  EXTENSIONS.with(|extensions| {
    let mut exts = extensions.lock().unwrap();
    exts.push(Box::new(ext));
  });
}

fn run_ext<T: FnMut(&Box<dyn Extension>)>(mut callback: T) {
  EXTENSIONS.with(|extensions| {
    let exts = extensions.lock().unwrap();
    for ext in exts.iter() {
      callback(ext);
    }
  });
}

pub(crate) fn created(webview: &mut WebView<'_, ()>) {
  run_ext(|ext| {
    ext.created(webview);
  });
}

pub(crate) fn ready(webview: &mut WebView<'_, ()>) {
  run_ext(|ext| {
    ext.ready(webview);
  });
}

pub(crate) fn extend_api(webview: &mut WebView<'_, ()>, arg: &str) {
   run_ext(|ext| {
    ext.extend_api(webview, arg);
  });
}