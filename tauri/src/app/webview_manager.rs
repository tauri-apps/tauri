use std::collections::HashMap;

use crate::{
  webview::{Event, Message},
  ApplicationDispatcherExt,
};

use serde::Serialize;

/// The webview dispatcher.
#[derive(Clone)]
pub struct WebviewDispatcher<A: Clone>(A);

impl<A: ApplicationDispatcherExt> WebviewDispatcher<A> {
  pub(crate) fn new(dispatcher: A) -> Self {
    Self(dispatcher)
  }

  pub(crate) fn send_event(&self, event: Event) {
    self.0.send_message(Message::Event(event))
  }

  /// Listen to an event.
  pub fn listen<F: FnMut(Option<String>) + Send + 'static>(
    &self,
    event: impl Into<String>,
    handler: F,
  ) {
    super::event::listen(event, handler)
  }

  /// Emits an event.
  pub fn emit<S: Serialize>(
    &self,
    event: impl AsRef<str>,
    payload: Option<S>,
  ) -> crate::Result<()> {
    super::event::emit(&self, event, payload)
  }

  pub(crate) fn on_event(&self, event: String, data: Option<String>) {
    super::event::on_event(event, data)
  }

  /// Evaluates a JS script.
  pub fn eval(&self, js: &str) {
    self.0.send_message(Message::EvalScript(js.to_string()))
  }

  /// Updates the window title.
  pub fn set_title(&self, title: &str) {
    self
      .0
      .send_message(Message::SetWindowTitle(title.to_string()))
  }
}

/// The webview manager.
#[derive(Clone)]
pub struct WebviewManager<A: Clone> {
  dispatchers: HashMap<String, WebviewDispatcher<A>>,
  current_webview_window_label: String,
}

impl<A: ApplicationDispatcherExt> WebviewManager<A> {
  pub(crate) fn new(dispatchers: HashMap<String, WebviewDispatcher<A>>, label: String) -> Self {
    Self {
      dispatchers,
      current_webview_window_label: label,
    }
  }

  /// Gets the webview associated with the current context.
  pub fn current_webview(&self) -> crate::Result<&WebviewDispatcher<A>> {
    self.get_webview(&self.current_webview_window_label)
  }

  /// Gets the webview associated with the given window label.
  pub fn get_webview(&self, label: &str) -> crate::Result<&WebviewDispatcher<A>> {
    self
      .dispatchers
      .get(label)
      .ok_or(crate::Error::WebviewNotFound)
  }
}
