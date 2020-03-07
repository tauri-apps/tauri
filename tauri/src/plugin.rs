use std::sync::{Arc, Mutex};
use web_view::WebView;

pub trait Plugin {
  #[allow(unused_variables)]
  fn created(&self, webview: &mut WebView<'_, ()>) {}

  #[allow(unused_variables)]
  fn ready(&self, webview: &mut WebView<'_, ()>) {}

  #[allow(unused_variables)]
  fn extend_api(&self, webview: &mut WebView<'_, ()>, payload: &str) {}
}

thread_local!(static PLUGINS: Arc<Mutex<Vec<Box<dyn Plugin>>>> = Default::default());

pub fn register(ext: impl Plugin + 'static) {
  PLUGINS.with(|plugins| {
    let mut exts = plugins.lock().unwrap();
    exts.push(Box::new(ext));
  });
}

fn run_plugin<T: FnMut(&Box<dyn Plugin>)>(mut callback: T) {
  PLUGINS.with(|plugins| {
    let exts = plugins.lock().unwrap();
    for ext in exts.iter() {
      callback(ext);
    }
  });
}

pub(crate) fn created(webview: &mut WebView<'_, ()>) {
  run_plugin(|ext| {
    ext.created(webview);
  });
}

pub(crate) fn ready(webview: &mut WebView<'_, ()>) {
  run_plugin(|ext| {
    ext.ready(webview);
  });
}

pub(crate) fn extend_api(webview: &mut WebView<'_, ()>, arg: &str) {
   run_plugin(|ext| {
    ext.extend_api(webview, arg);
  });
}
