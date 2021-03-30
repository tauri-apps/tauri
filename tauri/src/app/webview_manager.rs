use crate::{
  api::{assets::Assets, config::Config},
  app::webview::AttributesPrivate,
  event::{emit_function_name, EventPayload, HandlerId, Listeners},
  plugin::PluginStore,
  runtime::Dispatch,
  Attributes, CustomProtocol, FileDropEvent, Icon, InvokeHandler, InvokeMessage, InvokePayload,
  PageLoadHook, PageLoadPayload, PendingWindow, RpcRequest, WindowUrl,
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{
  borrow::Cow,
  collections::HashSet,
  convert::TryInto,
  fmt,
  hash::{Hash, Hasher},
  ops::Deref,
  str::FromStr,
  sync::{Arc, Mutex},
};

#[allow(missing_docs)]
pub trait Tag: Hash + Eq + FromStr + fmt::Display + Clone + Send + Sync + 'static {}
impl<T> Tag for T where T: Hash + Eq + FromStr + fmt::Display + Clone + Send + Sync + 'static {}

#[allow(missing_docs)]
pub struct InnerWindowManager<E: Tag, L: Tag, D: Dispatch> {
  windows: Arc<Mutex<HashSet<Window<E, L, D>>>>,
  plugins: PluginStore<E, L, D>,
  listeners: Listeners<E, L>,

  /// The JS message handler.
  invoke_handler: Arc<Mutex<Option<Box<InvokeHandler<E, L, D>>>>>,

  ///// The setup hook, invoked when the webviews have been created.
  //setup: Option<Box<SetupHook>>,
  /// The page load hook, invoked when the webview performs a navigation.
  on_page_load: Arc<Mutex<Option<Box<PageLoadHook<E, L, D>>>>>,

  config: Arc<Config>,
}

impl<E: Tag, L: Tag, D: Dispatch> Clone for InnerWindowManager<E, L, D> {
  fn clone(&self) -> Self {
    Self {
      windows: self.windows.clone(),
      plugins: self.plugins.clone(),
      listeners: self.listeners.clone(),
      invoke_handler: self.invoke_handler.clone(),
      on_page_load: self.on_page_load.clone(),
      config: self.config.clone(),
    }
  }
}

impl<E: Tag, L: Tag, D: Dispatch> InnerWindowManager<E, L, D> {
  pub(crate) fn new(
    config: Arc<Config>,
    invoke: Option<Box<InvokeHandler<E, L, D>>>,
    page_load: Option<Box<PageLoadHook<E, L, D>>>,
  ) -> Self {
    Self {
      windows: Arc::new(Mutex::new(HashSet::new())),
      plugins: PluginStore::default(),
      listeners: Listeners::new(),
      config,
      invoke_handler: Arc::new(Mutex::new(invoke)),
      on_page_load: Arc::new(Mutex::new(page_load)),
    }
  }

  pub(crate) fn initialize_plugins(&self) -> crate::Result<()> {
    self.plugins.initialize(&self.config.plugins)
  }

  /// Runs the [invoke handler](AppBuilder::invoke_handler) if defined.
  pub fn run_invoke_handler(&self, message: InvokeMessage<E, L, D>) {
    if let Some(hook) = &*self.invoke_handler.lock().expect("poisoned invoke_handler") {
      hook(message)
    }
  }

  /// Runs the on page load hook if defined.
  fn run_on_page_load(&self, window: Window<E, L, D>, payload: PageLoadPayload) {
    if let Some(hook) = &*self.on_page_load.lock().expect("poisoned on_page_load") {
      hook(window, payload)
    }
  }

  pub(crate) fn attach_window(&self, dispatch: D, label: L) -> Window<E, L, D> {
    let window = Window::new(self.clone(), dispatch, label);

    // drop asap
    {
      self
        .windows
        .lock()
        .expect("poisoned window manager")
        .insert(window.clone());
    }

    self.plugins.created(window.clone());
    window
  }

  pub(crate) fn prepare_window<A: Assets + 'static>(
    &self,
    mut pending: PendingWindow<L, D>,
    dwi: Option<Vec<u8>>,
    assets: Arc<A>,
    pending_labels: &[String],
  ) -> crate::Result<PendingWindow<L, D>> {
    let (is_local, url) = match &pending.url {
      WindowUrl::App => (true, self.get_url(assets.deref())),
      WindowUrl::Custom(url) => (&url[0..8] == "tauri://", url.clone()),
    };

    let (builder, rpc_handler, custom_protocol) = if is_local {
      let plugin_init = self.plugins.initialization_script();
      let is_init_global = self.config.build.with_global_tauri;
      let mut attributes = pending
        .attributes.clone()
        .url(url)
        .initialization_script(&initialization_script(&plugin_init, is_init_global))
        .initialization_script(&format!(
          r#"
              window.__TAURI__.__windows = {window_labels_array}.map(function (label) {{ return {{ label: label }} }});
              window.__TAURI__.__currentWindow = {{ label: "{current_window_label}" }}
            "#,
          window_labels_array =
          serde_json::to_string(pending_labels)?,
          current_window_label = pending.label.clone(),
        ));

      if !attributes.has_icon() {
        if let Some(default_window_icon) = dwi {
          let icon = Icon::Raw(default_window_icon);
          let icon = icon.try_into().expect("infallible icon convert failed");
          attributes = attributes.icon(icon);
        }
      }

      let manager = self.clone();
      let rpc_handler: Box<dyn Fn(D, L, RpcRequest) + Send> =
        Box::new(move |dispatcher, label, request: RpcRequest| {
          let window = Window::new(manager.clone(), dispatcher, label);
          let command = request.command.clone();
          let arg = request
            .params
            .unwrap()
            .as_array_mut()
            .unwrap()
            .first_mut()
            .unwrap_or(&mut JsonValue::Null)
            .take();
          match serde_json::from_value::<InvokePayload>(arg) {
            Ok(message) => {
              let _ = window.on_message(command, message);
            }
            Err(e) => {
              let error: crate::Error = e.into();
              let _ = window.eval(&format!(
                r#"console.error({})"#,
                JsonValue::String(error.to_string())
              ));
            }
          };
        });

      let assets = assets.clone();
      let bundle_identifier = self.config.tauri.bundle.identifier.clone();
      let custom_protocol = CustomProtocol {
        name: "tauri".into(),
        handler: Box::new(move |path| {
          let mut path = path
            .to_string()
            .replace(&format!("tauri://{}", bundle_identifier), "");
          if path.ends_with('/') {
            path.pop();
          }
          let path = if path.is_empty() {
            // if the url is `tauri://${appId}`, we should load `index.html`
            "index.html".to_string()
          } else {
            // skip leading `/`
            path.chars().skip(1).collect::<String>()
          };

          let asset_response = assets
            .get(&path)
            .ok_or(crate::Error::AssetNotFound(path))
            .map(Cow::into_owned);
          match asset_response {
            Ok(asset) => Ok(asset),
            Err(e) => {
              #[cfg(debug_assertions)]
              eprintln!("{:?}", e); // TODO log::error!
              Err(e)
            }
          }
        }),
      };
      (attributes, Some(rpc_handler), Some(custom_protocol))
    } else {
      (pending.attributes.clone().url(url), None, None)
    };

    // TODO: one of the signatures needs to change to allow sending events from this closure,
    // or the file_drop handler must be able to be set after getting the window dispatch proxy
    let manager = self.clone();
    let file_drop_handler: Box<dyn Fn(FileDropEvent, D, L) -> bool + Send> =
      Box::new(move |event, d, l| {
        let manager = manager.clone();
        crate::async_runtime::block_on(async move {
          let window = Window::new(manager.clone(), d.clone(), l);
          let _ = match event {
            FileDropEvent::Hovered(paths) => {
              let hover: E = "tauri://file-drop"
                .parse()
                .unwrap_or_else(|_| panic!("todo: invalid event str"));
              window.emit(hover, Some(paths))
            }
            FileDropEvent::Dropped(paths) => {
              let drop: E = "tauri://file-drop-hover"
                .parse()
                .unwrap_or_else(|_| panic!("todo: invalid event str"));
              window.emit(drop, Some(paths))
            }
            FileDropEvent::Cancelled => {
              let cancel: E = "tauri://file-drop-cancelled"
                .parse()
                .unwrap_or_else(|_| panic!("todo: invalid event str"));
              window.emit(cancel, Some(()))
            }
          };
        });
        true
      });

    pending.set_attributes(builder);
    pending.set_rpc_handler(rpc_handler);
    pending.set_custom_protocol(custom_protocol);
    pending.set_file_drop(file_drop_handler);

    Ok(pending)
  }

  // setup content for dev-server
  #[cfg(dev)]
  fn get_url(&self, assets: &impl Assets) -> String {
    if self.config.build.dev_path.starts_with("http") {
      self.config.build.dev_path.clone()
    } else {
      let path = "index.html";
      format!(
        "data:text/html;base64,{}",
        base64::encode(
          assets
            .get(&path)
            .ok_or_else(|| crate::Error::AssetNotFound(path.to_string()))
            .map(std::borrow::Cow::into_owned)
            .expect("Unable to find `index.html` under your devPath folder")
        )
      )
    }
  }

  #[cfg(custom_protocol)]
  fn get_url(&self, _: &impl Assets) -> String {
    format!("tauri://{}", self.config.tauri.bundle.identifier)
  }
}

fn initialization_script(plugin_initialization_script: &str, with_global_tauri: bool) -> String {
  format!(
    r#"
      {bundle_script}
      {core_script}
      {event_initialization_script}
      if (window.rpc) {{
        window.__TAURI__.invoke("__initialized", {{ url: window.location.href }})
      }} else {{
        window.addEventListener('DOMContentLoaded', function () {{
          window.__TAURI__.invoke("__initialized", {{ url: window.location.href }})
        }})
      }}
      {plugin_initialization_script}
    "#,
    core_script = include_str!("../../scripts/core.js"),
    bundle_script = if with_global_tauri {
      include_str!("../../scripts/bundle.js")
    } else {
      ""
    },
    event_initialization_script = event_initialization_script(),
    plugin_initialization_script = plugin_initialization_script
  )
}

fn event_initialization_script() -> String {
  return format!(
    "
      window['{queue}'] = [];
      window['{fn}'] = function (eventData, salt, ignoreQueue) {{
      const listeners = (window['{listeners}'] && window['{listeners}'][eventData.event]) || []
      if (!ignoreQueue && listeners.length === 0) {{
        window['{queue}'].push({{
          eventData: eventData,
          salt: salt
        }})
      }}

      if (listeners.length > 0) {{
        window.__TAURI__.invoke('tauri', {{
          __tauriModule: 'Internal',
          message: {{
            cmd: 'validateSalt',
            salt: salt
          }}
        }}).then(function (flag) {{
          if (flag) {{
            for (let i = listeners.length - 1; i >= 0; i--) {{
              const listener = listeners[i]
              eventData.id = listener.id
              listener.handler(eventData)
            }}
          }}
        }})
      }}
    }}
    ",
    fn = crate::event::emit_function_name(),
    queue = crate::event::event_queue_object_name(),
    listeners = crate::event::event_listeners_object_name()
  );
}

/// A single webview window that is not attached to a window manager.
pub struct Window<E: Tag, L: Tag, D: Dispatch> {
  label: L,
  dispatcher: D,
  manager: InnerWindowManager<E, L, D>,
}

impl<E: Tag, L: Tag, D: Dispatch> Clone for Window<E, L, D> {
  fn clone(&self) -> Self {
    Self {
      label: self.label.clone(),
      dispatcher: self.dispatcher.clone(),
      manager: self.manager.clone(),
    }
  }
}

impl<E: Tag, L: Tag, D: Dispatch> Hash for Window<E, L, D> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.label.hash(state)
  }
}

impl<E: Tag, L: Tag, D: Dispatch> Eq for Window<E, L, D> {}
impl<E: Tag, L: Tag, D: Dispatch> PartialEq for Window<E, L, D> {
  fn eq(&self, other: &Self) -> bool {
    self.label.eq(&other.label)
  }
}

impl<E: Tag, L: Tag, D: Dispatch> Window<E, L, D> {
  pub(crate) fn new(manager: InnerWindowManager<E, L, D>, dispatcher: D, label: L) -> Self {
    Self {
      manager,
      label,
      dispatcher,
    }
  }

  pub(crate) fn dispatcher(&self) -> D {
    self.dispatcher.clone()
  }

  /// The label of the window tied to this dispatcher.
  pub fn label(&self) -> &L {
    &self.label
  }

  /// Listen to a webview event.
  pub fn listen<F>(&self, event: E, handler: F) -> HandlerId
  where
    F: Fn(EventPayload) + Send + 'static,
  {
    self.manager.listeners.listen(event, None, handler)
  }

  /// Listen to a webview event and unlisten after the first event.
  pub fn once<F>(&self, event: E, handler: F)
  where
    F: Fn(EventPayload) + Send + 'static,
  {
    self.manager.listeners.once(event, None, handler)
  }

  /// Unregister the event listener with the given id.
  pub fn unlisten(&self, handler_id: HandlerId) {
    self.manager.listeners.unlisten(handler_id)
  }

  /// Emits an event to the webview.
  pub(crate) fn emit<S: Serialize>(&self, event: E, payload: Option<S>) -> crate::Result<()> {
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
  pub(crate) fn trigger(&self, event: E, data: Option<String>) {
    self.manager.listeners.trigger(event, None, data)
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

  pub(crate) fn emit_all<S: Serialize + Clone>(
    &self,
    event: E,
    payload: Option<S>,
  ) -> crate::Result<()> {
    self
      .manager
      .windows
      .lock()
      .expect("poisoned manager window mutex")
      .iter()
      .try_for_each(|window| window.emit(event.clone(), payload.clone()))
  }

  /// emits an event on all windows except the current one
  pub(crate) fn emit_others<S>(&self, event: E, payload: Option<S>) -> crate::Result<()>
  where
    S: Serialize + Clone,
  {
    self
      .manager
      .windows
      .lock()
      .expect("poisoned manager window mutex")
      .iter()
      .try_for_each(|window| {
        if window != self {
          window.emit(event.clone(), payload.clone())
        } else {
          Ok(())
        }
      })
  }

  #[allow(missing_docs)]
  pub async fn create_window(self, pending: PendingWindow<L, D>) -> crate::Result<Self> {
    let mut dispatcher = self.dispatcher.clone();
    let manager = self.manager.clone();
    let label = pending.label.clone();
    let dispatcher = dispatcher.create_window(pending)?;
    let window = manager.attach_window(dispatcher, label);

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
