use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use super::{
  App, ApplicationDispatcherExt, ApplicationExt, Icon, Webview, WebviewBuilderExt,
  WebviewInitializer,
};
use crate::{api::config::WindowUrl, flavors::Wry};

use serde::Serialize;

/// The webview dispatcher.
#[derive(Clone)]
pub struct WebviewDispatcher<A: Clone> {
  dispatcher: A,
  window_label: String,
}

impl<A: ApplicationDispatcherExt> WebviewDispatcher<A> {
  pub(crate) fn new(dispatcher: A, window_label: String) -> Self {
    Self {
      dispatcher,
      window_label,
    }
  }

  /// The label of the window tied to this dispatcher.
  pub fn window_label(&self) -> &str {
    &self.window_label
  }

  /// Listen to a webview event.
  pub fn listen<F: FnMut(Option<String>) + Send + 'static>(
    &self,
    event: impl AsRef<str>,
    handler: F,
  ) {
    super::event::listen(event, Some(self.window_label.to_string()), handler)
  }

  /// Emits an event to the webview.
  pub fn emit<S: Serialize>(
    &self,
    event: impl AsRef<str>,
    payload: Option<S>,
  ) -> crate::Result<()> {
    super::event::emit(&self, event, payload)
  }

  /// Emits an event from the webview.
  pub(crate) fn on_event(&self, event: String, data: Option<String>) {
    super::event::on_event(event, Some(&self.window_label), data)
  }

  /// Evaluates a JS script.
  pub fn eval(&self, js: &str) -> crate::Result<()> {
    self.dispatcher.eval_script(js)
  }

  /// Updates the window resizable flag.
  pub fn set_resizable(&self, resizable: bool) -> crate::Result<()> {
    self.dispatcher.set_resizable(resizable)
  }

  /// Updates the window title.
  pub fn set_title(&self, title: &str) -> crate::Result<()> {
    self.dispatcher.set_title(title.to_string())
  }

  /// Maximizes the window.
  pub fn maximize(&self) -> crate::Result<()> {
    self.dispatcher.maximize()
  }

  /// Unmaximizes the window.
  pub fn unmaximize(&self) -> crate::Result<()> {
    self.dispatcher.unmaximize()
  }

  /// Minimizes the window.
  pub fn minimize(&self) -> crate::Result<()> {
    self.dispatcher.minimize()
  }

  /// Unminimizes the window.
  pub fn unminimize(&self) -> crate::Result<()> {
    self.dispatcher.unminimize()
  }

  /// Sets the window visibility to true.
  pub fn show(&self) -> crate::Result<()> {
    self.dispatcher.show()
  }

  /// Sets the window visibility to false.
  pub fn hide(&self) -> crate::Result<()> {
    self.dispatcher.hide()
  }

  /// Closes the window.
  pub fn close(&self) -> crate::Result<()> {
    self.dispatcher.close()
  }

  /// Whether the window should have borders and bars.
  pub fn set_decorations(&self, decorations: bool) -> crate::Result<()> {
    self.dispatcher.set_decorations(decorations)
  }

  /// Whether the window should always be on top of other windows.
  pub fn set_always_on_top(&self, always_on_top: bool) -> crate::Result<()> {
    self.dispatcher.set_always_on_top(always_on_top)
  }

  /// Sets the window width.
  pub fn set_width(&self, width: impl Into<f64>) -> crate::Result<()> {
    self.dispatcher.set_width(width.into())
  }

  /// Sets the window height.
  pub fn set_height(&self, height: impl Into<f64>) -> crate::Result<()> {
    self.dispatcher.set_height(height.into())
  }

  /// Resizes the window.
  pub fn resize(&self, width: impl Into<f64>, height: impl Into<f64>) -> crate::Result<()> {
    self.dispatcher.resize(width.into(), height.into())
  }

  /// Sets the window min size.
  pub fn set_min_size(
    &self,
    min_width: impl Into<f64>,
    min_height: impl Into<f64>,
  ) -> crate::Result<()> {
    self
      .dispatcher
      .set_min_size(min_width.into(), min_height.into())
  }

  /// Sets the window max size.
  pub fn set_max_size(
    &self,
    max_width: impl Into<f64>,
    max_height: impl Into<f64>,
  ) -> crate::Result<()> {
    self
      .dispatcher
      .set_max_size(max_width.into(), max_height.into())
  }

  /// Sets the window x position.
  pub fn set_x(&self, x: impl Into<f64>) -> crate::Result<()> {
    self.dispatcher.set_x(x.into())
  }

  /// Sets the window y position.
  pub fn set_y(&self, y: impl Into<f64>) -> crate::Result<()> {
    self.dispatcher.set_y(y.into())
  }

  /// Sets the window position.
  pub fn set_position(&self, x: impl Into<f64>, y: impl Into<f64>) -> crate::Result<()> {
    self.dispatcher.set_position(x.into(), y.into())
  }

  /// Sets the window fullscreen state.
  pub fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()> {
    self.dispatcher.set_fullscreen(fullscreen)
  }

  /// Sets the window icon.
  pub fn set_icon(&self, icon: Icon) -> crate::Result<()> {
    self.dispatcher.set_icon(icon)
  }
}

/// The webview manager.
pub struct WebviewManager<A = Wry>
where
  A: ApplicationExt,
{
  application: Arc<App<A>>,
  dispatchers: Arc<Mutex<HashMap<String, WebviewDispatcher<A::Dispatcher>>>>,
  current_webview_window_label: String,
}

impl<A: ApplicationExt> Clone for WebviewManager<A> {
  fn clone(&self) -> Self {
    Self {
      application: self.application.clone(),
      dispatchers: self.dispatchers.clone(),
      current_webview_window_label: self.current_webview_window_label.to_string(),
    }
  }
}

impl<A: ApplicationExt + 'static> WebviewManager<A> {
  pub(crate) fn new(
    application: Arc<App<A>>,
    dispatchers: Arc<Mutex<HashMap<String, WebviewDispatcher<A::Dispatcher>>>>,
    label: String,
  ) -> Self {
    Self {
      application,
      dispatchers,
      current_webview_window_label: label,
    }
  }

  /// Returns the label of the window associated with the current context.
  pub fn current_window_label(&self) -> &str {
    &self.current_webview_window_label
  }

  /// Gets the webview associated with the current context.
  pub fn current_webview(&self) -> crate::Result<WebviewDispatcher<A::Dispatcher>> {
    self.get_webview(&self.current_webview_window_label)
  }

  /// Gets the webview associated with the given window label.
  pub fn get_webview(&self, window_label: &str) -> crate::Result<WebviewDispatcher<A::Dispatcher>> {
    self
      .dispatchers
      .lock()
      .unwrap()
      .get(window_label)
      .ok_or(crate::Error::WebviewNotFound)
      .map(|d| d.clone())
  }

  /// Creates a new webview.
  pub async fn create_webview<F: FnOnce(A::WebviewBuilder) -> crate::Result<A::WebviewBuilder>>(
    &self,
    label: String,
    url: WindowUrl,
    f: F,
  ) -> crate::Result<WebviewDispatcher<A::Dispatcher>> {
    let builder = f(A::WebviewBuilder::new())?;
    let webview = Webview {
      url,
      label: label.to_string(),
      builder,
    };
    self
      .application
      .window_labels
      .lock()
      .unwrap()
      .push(label.to_string());
    let (webview_builder, rpc_handler, custom_protocol, file_drop_handler) =
      self.application.init_webview(webview)?;

    let window_dispatcher = self.current_webview()?.dispatcher.create_webview(
      webview_builder,
      rpc_handler,
      custom_protocol,
      file_drop_handler,
    )?;
    let webview_manager = Self::new(
      self.application.clone(),
      self.dispatchers.clone(),
      label.to_string(),
    );
    self
      .application
      .on_webview_created(
        label.to_string(),
        window_dispatcher.clone(),
        webview_manager,
      )
      .await;
    Ok(WebviewDispatcher::new(window_dispatcher, label))
  }

  /// Listen to a global event.
  /// An event from any webview will trigger the handler.
  pub fn listen<F: FnMut(Option<String>) + Send + 'static>(
    &self,
    event: impl AsRef<str>,
    handler: F,
  ) {
    super::event::listen(event, None, handler)
  }

  /// Emits an event to all webviews.
  pub fn emit<S: Serialize + Clone>(
    &self,
    event: impl AsRef<str>,
    payload: Option<S>,
  ) -> crate::Result<()> {
    for dispatcher in self.dispatchers.lock().unwrap().values() {
      super::event::emit(&dispatcher, event.as_ref(), payload.clone())?;
    }
    Ok(())
  }

  pub(crate) fn emit_except<S: Serialize + Clone>(
    &self,
    except_label: String,
    event: impl AsRef<str>,
    payload: Option<S>,
  ) -> crate::Result<()> {
    for dispatcher in self.dispatchers.lock().unwrap().values() {
      if dispatcher.window_label != except_label {
        super::event::emit(&dispatcher, event.as_ref(), payload.clone())?;
      }
    }
    Ok(())
  }

  /// Emits a global event from the webview.
  pub(crate) fn on_event(&self, event: String, data: Option<String>) {
    super::event::on_event(event, None, data)
  }
}
