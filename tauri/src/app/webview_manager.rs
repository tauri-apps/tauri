use std::collections::HashMap;

use super::{ApplicationDispatcherExt, Event, Icon, Message};

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
    event: impl AsRef<str>,
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

  /// Updates the window resizable flag.
  pub fn set_resizable(&self, resizable: bool) {
    self.0.send_message(Message::SetResizable(resizable))
  }

  /// Updates the window title.
  pub fn set_title(&self, title: &str) {
    self.0.send_message(Message::SetTitle(title.to_string()))
  }

  /// Maximizes the window.
  pub fn maximize(&self) {
    self.0.send_message(Message::Maximize)
  }

  /// Unmaximizes the window.
  pub fn unmaximize(&self) {
    self.0.send_message(Message::Unmaximize)
  }

  /// Minimizes the window.
  pub fn minimize(&self) {
    self.0.send_message(Message::Minimize)
  }

  /// Unminimizes the window.
  pub fn unminimize(&self) {
    self.0.send_message(Message::Unminimize)
  }

  /// Sets the window visibility to true.
  pub fn show(&self) {
    self.0.send_message(Message::Show)
  }

  /// Sets the window visibility to false.
  pub fn hide(&self) {
    self.0.send_message(Message::Hide)
  }

  /// Sets the window transparent flag.
  pub fn set_transparent(&self, transparent: bool) {
    self.0.send_message(Message::SetTransparent(transparent))
  }

  /// Whether the window should have borders and bars.
  pub fn set_decorations(&self, decorations: bool) {
    self.0.send_message(Message::SetDecorations(decorations))
  }

  /// Whether the window should always be on top of other windows.
  pub fn set_always_on_top(&self, always_on_top: bool) {
    self.0.send_message(Message::SetAlwaysOnTop(always_on_top))
  }

  /// Sets the window width.
  pub fn set_width(&self, width: impl Into<f64>) {
    self.0.send_message(Message::SetWidth(width.into()))
  }

  /// Sets the window height.
  pub fn set_height(&self, height: impl Into<f64>) {
    self.0.send_message(Message::SetHeight(height.into()))
  }

  /// Resizes the window.
  pub fn resize(&self, width: impl Into<f64>, height: impl Into<f64>) {
    self.0.send_message(Message::Resize {
      width: width.into(),
      height: height.into(),
    })
  }

  /// Sets the window min size.
  pub fn set_min_size(&self, min_width: impl Into<f64>, min_height: impl Into<f64>) {
    self.0.send_message(Message::SetMinSize {
      min_width: min_width.into(),
      min_height: min_height.into(),
    })
  }

  /// Sets the window max size.
  pub fn set_max_size(&self, max_width: impl Into<f64>, max_height: impl Into<f64>) {
    self.0.send_message(Message::SetMaxSize {
      max_width: max_width.into(),
      max_height: max_height.into(),
    })
  }

  /// Sets the window x position.
  pub fn set_x(&self, x: impl Into<f64>) {
    self.0.send_message(Message::SetX(x.into()))
  }

  /// Sets the window y position.
  pub fn set_y(&self, y: impl Into<f64>) {
    self.0.send_message(Message::SetY(y.into()))
  }

  /// Sets the window position.
  pub fn set_position(&self, x: impl Into<f64>, y: impl Into<f64>) {
    self.0.send_message(Message::SetPosition {
      x: x.into(),
      y: y.into(),
    })
  }

  /// Sets the window fullscreen state.
  pub fn set_fullscreen(&self, fullscreen: bool) {
    self.0.send_message(Message::SetFullscreen(fullscreen))
  }

  /// Sets the window icon.
  pub fn set_icon(&self, icon: Icon) {
    self.0.send_message(Message::SetIcon(icon))
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

  /// Returns the label of the window associated with the current context.
  pub fn current_window_label(&self) -> &str {
    &self.current_webview_window_label
  }

  /// Gets the webview associated with the current context.
  pub fn current_webview(&self) -> crate::Result<&WebviewDispatcher<A>> {
    self.get_webview(&self.current_webview_window_label)
  }

  /// Gets the webview associated with the given window label.
  pub fn get_webview(&self, window_label: &str) -> crate::Result<&WebviewDispatcher<A>> {
    self
      .dispatchers
      .get(window_label)
      .ok_or(crate::Error::WebviewNotFound)
  }
}
