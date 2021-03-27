use crate::api::config::Config;
use crate::event::EventScope;
use crate::plugin::PluginStore;
use crate::{
  event::{emit_function_name, EventPayload, HandlerId, Listeners},
  runtime::{Dispatch, Runtime},
  Icon, InvokeHandler, InvokeMessage, InvokePayload, PageLoadHook, PageLoadPayload, PendingWindow,
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{
  collections::HashSet,
  convert::TryInto,
  fmt::Display,
  hash::{Hash, Hasher},
  str::FromStr,
  sync::{Arc, Mutex},
};

pub trait Label: Hash + Eq + Display + FromStr + Serialize + Clone + Send + Sync + 'static {}
impl<T> Label for T where
  T: Hash + Eq + Display + FromStr + Serialize + Clone + Send + Sync + 'static
{
}

#[derive(Clone)]
pub struct WindowManager<E: Label, L: Label, R: Runtime> {
  windows: Arc<Mutex<HashSet<Window<E, L, R>>>>,
  plugins: PluginStore<E, L, R>,
  listeners: Listeners<E, L>,

  /// The JS message handler.
  invoke_handler: Arc<Mutex<Option<Box<InvokeHandler<E, L, R>>>>>,

  ///// The setup hook, invoked when the webviews have been created.
  //setup: Option<Box<SetupHook>>,
  /// The page load hook, invoked when the webview performs a navigation.
  on_page_load: Arc<Mutex<Option<Box<PageLoadHook<E, L, R>>>>>,

  config: Arc<Config>,
}

impl<E: Label, L: Label, R: Runtime> WindowManager<E, L, R> {
  pub fn new(
    config: Arc<Config>,
    invoke: Option<Box<InvokeHandler<E, L, R>>>,
    page_load: Option<Box<PageLoadHook<E, L, R>>>,
  ) -> Self {
    Self {
      windows: Arc::new(Mutex::new(HashSet::new())),
      plugins: PluginStore::new(),
      listeners: Listeners::new(),
      config,
      invoke_handler: Arc::new(Mutex::new(invoke)),
      on_page_load: Arc::new(Mutex::new(page_load)),
      // todo: init these
      //setup: None,
    }
  }

  /// Runs the [invoke handler](AppBuilder::invoke_handler) if defined.
  pub fn run_invoke_handler(&self, message: InvokeMessage<E, L, R>) {
    if let Some(hook) = &*self.invoke_handler.lock().expect("poisoned invoke_handler") {
      hook(message)
    }
  }

  /// Runs the on page load hook if defined.
  fn run_on_page_load(&self, window: Window<E, L, R>, payload: PageLoadPayload) {
    if let Some(hook) = &*self.on_page_load.lock().expect("poisoned on_page_load") {
      hook(window, payload)
    }
  }
}

/// A single webview window.
pub struct Window<E: Label, L: Label, R: Runtime> {
  label: L,
  dispatcher: R::Dispatcher,
  manager: Arc<WindowManager<E, L, R>>,
}

impl<E: Label, L: Label, R: Runtime> Clone for Window<E, L, R> {
  fn clone(&self) -> Self {
    Self {
      label: self.label.clone(),
      dispatcher: self.dispatcher.clone(),
      manager: self.manager.clone(),
    }
  }
}

impl<E: Label, L: Label, R: Runtime> Hash for Window<E, L, R> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.label.hash(state)
  }
}

impl<E: Label, L: Label, R: Runtime> Eq for Window<E, L, R> {}
impl<E: Label, L: Label, R: Runtime> PartialEq for Window<E, L, R> {
  fn eq(&self, other: &Self) -> bool {
    self.label.eq(&other.label)
  }
}

impl<E: Label, L: Label, R: Runtime> Window<E, L, R> {
  pub(crate) fn new(
    manager: Arc<WindowManager<E, L, R>>,
    dispatcher: R::Dispatcher,
    label: L,
  ) -> Self {
    Self {
      manager,
      label,
      dispatcher,
    }
  }

  pub fn dispatcher(&self) -> R::Dispatcher {
    self.dispatcher.clone()
  }

  /// The label of the window tied to this dispatcher.
  pub fn label(&self) -> &L {
    &self.label
  }

  /// Listen to a webview event.
  pub fn listen<F>(&self, scope: EventScope, event: E, handler: F) -> HandlerId
  where
    F: Fn(EventPayload) + Send + 'static,
  {
    let l = &self.manager.listeners;
    match scope {
      EventScope::Global => l.listen(event, handler),
      EventScope::Window => l.listen_window(self.label.clone(), event, handler),
    }
  }

  /// Listen to a webview event and unlisten after the first event.
  pub fn once<F>(&self, scope: EventScope, event: E, handler: F)
  where
    F: Fn(EventPayload) + Send + 'static,
  {
    let l = &self.manager.listeners;
    match scope {
      EventScope::Global => l.once(event, handler),
      EventScope::Window => l.once_window(self.label.clone(), event, handler),
    }
  }

  /// Unregister the event listener with the given id.
  pub fn unlisten(&self, scope: EventScope, handler_id: HandlerId) {
    let l = &self.manager.listeners;
    match scope {
      EventScope::Global => l.unlisten(handler_id),
      EventScope::Window => l.unlisten_window(&self.label, handler_id),
    }
  }

  /// Emits an event to the webview.
  pub fn emit<S: Serialize>(&self, event: E, payload: Option<S>) -> crate::Result<()> {
    let js_payload = match payload {
      Some(payload_value) => serde_json::to_value(payload_value)?,
      None => JsonValue::Null,
    };

    self.eval(&format!(
      "window['{}']({{event: '{}', payload: {}}}, '{}')",
      emit_function_name(),
      event.to_string(),
      js_payload,
      crate::salt::generate()
    ))?;

    Ok(())
  }

  /// Emits an event from the webview.
  pub(crate) fn trigger(&self, scope: EventScope, event: E, data: Option<String>) {
    let l = &self.manager.listeners;
    match scope {
      EventScope::Global => l.trigger(event, data),
      EventScope::Window => l.trigger_window(&self.label, event, data),
    }
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
    self.dispatcher.set_icon(icon.try_into()?)
  }

  pub async fn create_window(&self, pending: PendingWindow<L, R>) -> crate::Result<Self> {
    let mut dispatcher = self.dispatcher.clone();
    let manager = self.manager.clone();
    let label = pending.label.clone();
    let dispatcher = dispatcher.create_window(pending)?;
    let window = Window::new(manager.clone(), dispatcher, label);

    // drop lock asap
    {
      manager
        .windows
        .lock()
        .expect("poisoned window manager windows")
        .insert(window.clone());
    }

    manager.plugins.created(window.clone());
    Ok(window)
  }

  pub(crate) fn on_message(self, command: String, payload: InvokePayload) -> crate::Result<()> {
    let manager = self.manager.clone();
    if &command == "__initialized" {
      let payload: PageLoadPayload = serde_json::from_value(payload.inner)?;
      manager.run_on_page_load(self.clone(), payload.clone());
      manager.plugins.on_page_load(self, payload);
    } else {
      let message = InvokeMessage::new(self, command.to_string(), payload);
      if let Some(module) = &message.payload.tauri_module {
        let module = module.to_string();
        crate::endpoints::handle(module, message, &manager.config);
      } else if command.starts_with("plugin:") {
        manager.plugins.extend_api(command, message);
      } else {
        manager.run_invoke_handler(message);
      }
    }
    Ok(())
  }
}
