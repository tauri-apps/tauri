use crate::{
  api::{assets::Assets, config::Config},
  app::{
    sealed::{ManagedExt, ManagerExt},
    webview::AttributesPrivate,
  },
  event::{EventPayload, HandlerId, Listeners},
  plugin::PluginStore,
  runtime::Dispatch,
  Attributes, Context, CustomProtocol, FileDropEvent, FileDropHandler, Icon, InvokeHandler,
  InvokeMessage, InvokePayload, Managed, Manager, OnPageLoad, PageLoadPayload, PendingWindow,
  Runtime, RuntimeOrDispatch, WebviewRpcHandler, WindowUrl,
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{
  borrow::Cow,
  collections::HashSet,
  convert::TryInto,
  fmt,
  hash::{Hash, Hasher},
  str::FromStr,
  sync::{Arc, Mutex},
};

#[allow(missing_docs)]
pub trait Tag:
  Hash + Eq + FromStr + fmt::Display + fmt::Debug + Clone + Send + Sync + 'static
{
}
impl<T> Tag for T where
  T: Hash + Eq + FromStr + fmt::Display + fmt::Debug + Clone + Send + Sync + 'static
{
}

pub struct InnerWindowManager<M: Manager> {
  windows: Mutex<HashSet<Window<M>>>,
  plugins: Mutex<PluginStore<M>>,
  listeners: Listeners<M::Event, M::Label>,

  /// The JS message handler.
  invoke_handler: Box<InvokeHandler<M>>,

  /// The page load hook, invoked when the webview performs a navigation.
  on_page_load: Box<OnPageLoad<M>>,

  config: Config,
  assets: Arc<M::Assets>,
  default_window_icon: Option<Vec<u8>>,
}

pub struct WindowManager<E, L, A, R>
where
  E: Tag,
  L: Tag,
  A: Assets + 'static,
  R: Runtime,
{
  pub(crate) inner: Arc<InnerWindowManager<Self>>,
}

impl<E, L, A, R> Clone for WindowManager<E, L, A, R>
where
  E: Tag,
  L: Tag,
  A: Assets + 'static,
  R: Runtime,
{
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<E, L, A, R> WindowManager<E, L, A, R>
where
  E: Tag,
  L: Tag,
  A: Assets,
  R: Runtime,
{
  pub(crate) fn with_handlers(
    context: Context<A>,
    invoke_handler: Box<InvokeHandler<Self>>,
    on_page_load: Box<OnPageLoad<Self>>,
  ) -> Self {
    Self {
      inner: Arc::new(InnerWindowManager {
        windows: Mutex::default(),
        plugins: Mutex::default(),
        listeners: Listeners::default(),
        invoke_handler,
        on_page_load,
        config: context.config,
        assets: Arc::new(context.assets),
        default_window_icon: context.default_window_icon,
      }),
    }
  }

  // setup content for dev-server
  #[cfg(dev)]
  fn get_url(&self) -> String {
    if self.inner.config.build.dev_path.starts_with("http") {
      self.inner.config.build.dev_path.clone()
    } else {
      let path = "index.html";
      format!(
        "data:text/html;base64,{}",
        base64::encode(
          self
            .inner
            .assets
            .get(&path)
            .ok_or_else(|| crate::Error::AssetNotFound(path.to_string()))
            .map(Cow::into_owned)
            .expect("Unable to find `index.html` under your devPath folder")
        )
      )
    }
  }

  #[cfg(custom_protocol)]
  fn get_url(&self) -> String {
    format!("tauri://{}", self.inner.config.tauri.bundle.identifier)
  }

  fn prepare_attributes(
    &self,
    attrs: <R::Dispatcher as Dispatch>::Attributes,
    url: String,
    label: L,
    pending_labels: &HashSet<L>,
  ) -> crate::Result<<R::Dispatcher as Dispatch>::Attributes> {
    let is_init_global = self.inner.config.build.with_global_tauri;
    let plugin_init = self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .initialization_script();

    let mut attributes = attrs
      .url(url)
      .initialization_script(&initialization_script(&plugin_init, is_init_global))
      .initialization_script(&format!(
        r#"
              window.__TAURI__.__windows = {window_labels_array}.map(function (label) {{ return {{ label: label }} }});
              window.__TAURI__.__currentWindow = {{ label: {current_window_label} }}
            "#,
        window_labels_array = tags_to_js_string_array(pending_labels)?,
        current_window_label = tag_to_js_string(&label)?,
      ));

    if !attributes.has_icon() {
      if let Some(default_window_icon) = &self.inner.default_window_icon {
        let icon = Icon::Raw(default_window_icon.clone());
        let icon = icon.try_into().expect("infallible icon convert failed");
        attributes = attributes.icon(icon);
      }
    }

    Ok(attributes)
  }

  fn prepare_rpc_handler(&self) -> WebviewRpcHandler<Self> {
    let manager = self.clone();
    Box::new(move |window, request| {
      let window = manager.attach_window(window);
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
      }
    })
  }

  fn prepare_custom_protocol(&self) -> CustomProtocol {
    let assets = self.inner.assets.clone();
    let bundle_identifier = self.inner.config.tauri.bundle.identifier.clone();
    CustomProtocol {
      name: "tauri".into(),
      handler: Box::new(move |path| {
        let mut path = path
          .split('?')
          // ignore query string
          .next()
          .unwrap()
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
    }
  }

  fn prepare_file_drop(&self) -> FileDropHandler<Self> {
    let manager = self.clone();
    Box::new(move |event, window| {
      let manager = manager.clone();
      crate::async_runtime::block_on(async move {
        let window = manager.attach_window(window);
        let _ = match event {
          FileDropEvent::Hovered(paths) => {
            // todo: how should we handle this?
            let hover: E = "tauri://file-drop"
              .parse()
              .unwrap_or_else(|_| panic!("todo: invalid event str"));

            window.emit(&hover, Some(paths))
          }
          FileDropEvent::Dropped(paths) => {
            // todo: how should we handle this?
            let drop: E = "tauri://file-drop-hover"
              .parse()
              .unwrap_or_else(|_| panic!("todo: invalid event str"));

            window.emit(&drop, Some(paths))
          }
          FileDropEvent::Cancelled => {
            // todo: how should we handle this?
            let cancel: E = "tauri://file-drop-cancelled"
              .parse()
              .unwrap_or_else(|_| panic!("todo: invalid event str"));

            window.emit(&cancel, Some(()))
          }
        };
      });
      true
    })
  }
}

#[cfg(test)]
mod test {
  use super::WindowManager;
  use crate::{flavors::Wry, generate_context};

  #[test]
  fn check_get_url() {
    let context = generate_context!("test/fixture/src-tauri/tauri.conf.json", crate::Context);
    let manager: WindowManager<String, String, _, Wry> =
      WindowManager::with_handlers(context, Box::new(|_| ()), Box::new(|_, _| ()));

    #[cfg(custom_protocol)]
    assert_eq!(manager.get_url(), "tauri://studio.tauri.example");

    #[cfg(dev)]
    {
      use crate::app::sealed::ManagerExt;
      assert_eq!(manager.get_url(), manager.config().build.dev_path);
    }
  }
}

pub(crate) fn tag_to_js_string(tag: &impl Tag) -> crate::Result<String> {
  Ok(serde_json::to_string(&tag.to_string())?)
}

pub(crate) fn tags_to_js_string_array(tags: &HashSet<impl Tag>) -> crate::Result<String> {
  let tags = tags
    .iter()
    .map(ToString::to_string)
    .collect::<Vec<String>>();

  Ok(serde_json::to_string(&tags)?)
}

impl<E, L, A, R> ManagerExt<Self> for WindowManager<E, L, A, R>
where
  E: Tag,
  L: Tag,
  A: Assets + 'static,
  R: Runtime,
{
  fn run_invoke_handler(&self, message: InvokeMessage<Self>) {
    (self.inner.invoke_handler)(message);
  }

  fn run_on_page_load(&self, window: Window<Self>, payload: PageLoadPayload) {
    (self.inner.on_page_load)(window.clone(), payload.clone());
    self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .on_page_load(window, payload);
  }

  fn extend_api(&self, command: String, message: InvokeMessage<Self>) {
    self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .extend_api(command, message);
  }

  fn initialize_plugins(&self) -> crate::Result<()> {
    self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .initialize(&self.inner.config.plugins)
  }

  fn prepare_window(
    &self,
    mut pending: PendingWindow<Self>,
    pending_labels: &HashSet<L>,
  ) -> crate::Result<PendingWindow<Self>> {
    let (is_local, url) = match &pending.url {
      WindowUrl::App => (true, self.get_url()),
      WindowUrl::Custom(url) => (&url[0..8] == "tauri://", url.clone()),
    };

    let attributes = pending.attributes.clone();
    if is_local {
      let label = pending.label.clone();
      pending.set_attributes(self.prepare_attributes(attributes, url, label, pending_labels)?);
      pending.set_rpc_handler(self.prepare_rpc_handler());
      pending.set_custom_protocol(self.prepare_custom_protocol());
    } else {
      pending.set_attributes(attributes.url(url));
    }

    pending.set_file_drop(self.prepare_file_drop());

    Ok(pending)
  }

  fn attach_window(&self, window: DetachedWindow<Self>) -> Window<Self> {
    let window = Window {
      window,
      manager: self.clone(),
    };

    // insert the window into our manager
    {
      self
        .inner
        .windows
        .lock()
        .expect("poisoned window manager")
        .insert(window.clone());
    }

    // let plugins know that a new window has been added to the manager
    {
      self
        .inner
        .plugins
        .lock()
        .expect("poisoned plugin store")
        .created(window.clone());
    }

    window
  }

  fn emit_filter<S: Serialize + Clone, F: Fn(&Window<Self>) -> bool>(
    &self,
    event: E,
    payload: Option<S>,
    filter: F,
  ) -> crate::Result<()> {
    self
      .inner
      .windows
      .lock()
      .expect("poisoned window manager")
      .iter()
      .filter(|&w| filter(w))
      .try_for_each(|window| window.emit(&event, payload.clone()))
  }

  fn labels(&self) -> HashSet<L> {
    self
      .inner
      .windows
      .lock()
      .expect("poisoned window manager")
      .iter()
      .map(|w| w.window.label.clone())
      .collect()
  }

  fn config(&self) -> &Config {
    &self.inner.config
  }

  fn unlisten(&self, handler_id: HandlerId) {
    self.inner.listeners.unlisten(handler_id)
  }

  fn trigger(&self, event: E, window: Option<L>, data: Option<String>) {
    self.inner.listeners.trigger(event, window, data)
  }

  fn listen<F: Fn(EventPayload) + Send + 'static>(
    &self,
    event: E,
    window: Option<L>,
    handler: F,
  ) -> HandlerId {
    self.inner.listeners.listen(event, window, handler)
  }

  fn once<F: Fn(EventPayload) + Send + 'static>(&self, event: E, window: Option<L>, handler: F) {
    self.inner.listeners.once(event, window, handler)
  }
}

impl<E, L, A, R> Manager for WindowManager<E, L, A, R>
where
  E: Tag,
  L: Tag,
  A: Assets,
  R: Runtime,
{
  type Event = E;
  type Label = L;
  type Assets = A;
  type Runtime = R;
}

#[allow(missing_docs)]
pub struct DetachedWindow<M: Manager> {
  pub label: M::Label,
  pub dispatcher: <M::Runtime as Runtime>::Dispatcher,
}

impl<M: Manager> Clone for DetachedWindow<M> {
  fn clone(&self) -> Self {
    Self {
      label: self.label.clone(),
      dispatcher: self.dispatcher.clone(),
    }
  }
}

impl<M: Manager> Eq for DetachedWindow<M> {}
impl<M: Manager> PartialEq for DetachedWindow<M> {
  fn eq(&self, other: &Self) -> bool {
    self.label.eq(&other.label)
  }
}

impl<M: Manager> Hash for DetachedWindow<M> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.label.hash(state)
  }
}

#[allow(missing_docs)]
pub struct Window<M: Manager> {
  window: DetachedWindow<M>,
  manager: M,
}

impl<M: Manager> Clone for Window<M> {
  fn clone(&self) -> Self {
    Self {
      window: self.window.clone(),
      manager: self.manager.clone(),
    }
  }
}

impl<M: Manager> Hash for Window<M> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.window.label.hash(state)
  }
}

impl<M: Manager> Eq for Window<M> {}
impl<M: Manager> PartialEq for Window<M> {
  fn eq(&self, other: &Self) -> bool {
    self.window.label.eq(&other.window.label)
  }
}

impl<M: Manager> Managed<M> for Window<M> {}
impl<M: Manager> ManagedExt<M> for Window<M> {
  fn manager(&self) -> &M {
    &self.manager
  }

  fn runtime(&mut self) -> RuntimeOrDispatch<'_, M> {
    RuntimeOrDispatch::Dispatch(self.dispatcher())
  }
}

impl<M: Manager> Window<M> {
  /// The current window's dispatcher.
  pub(crate) fn dispatcher(&self) -> <M::Runtime as Runtime>::Dispatcher {
    self.window.dispatcher.clone()
  }

  /// How to handle this window receiving an [`InvokeMessage`].
  pub(crate) fn on_message(self, command: String, payload: InvokePayload) -> crate::Result<()> {
    let manager = self.manager.clone();
    if &command == "__initialized" {
      let payload: PageLoadPayload = serde_json::from_value(payload.inner)?;
      manager.run_on_page_load(self, payload);
    } else {
      let message = InvokeMessage::new(self, command.to_string(), payload);
      if let Some(module) = &message.payload.tauri_module {
        let module = module.to_string();
        crate::endpoints::handle(module, message, manager.config());
      } else if command.starts_with("plugin:") {
        manager.extend_api(command, message);
      } else {
        manager.run_invoke_handler(message);
      }
    }

    Ok(())
  }

  /// The label of this window.
  pub fn label(&self) -> &M::Label {
    &self.window.label
  }

  /// Emits an event to the current window.
  pub fn emit<S: Serialize>(&self, event: &M::Event, payload: Option<S>) -> crate::Result<()> {
    let js_payload = match payload {
      Some(payload_value) => serde_json::to_value(payload_value)?,
      None => JsonValue::Null,
    };

    self.eval(&format!(
      "window['{}']({{event: {}, payload: {}}}, '{}')",
      crate::event::emit_function_name(),
      tag_to_js_string(event)?,
      js_payload,
      crate::salt::generate()
    ))?;

    Ok(())
  }

  /// Emits an event on all windows except this one.
  pub fn emit_others<S: Serialize + Clone>(
    &self,
    event: M::Event,
    payload: Option<S>,
  ) -> crate::Result<()> {
    self.manager.emit_filter(event, payload, |w| w != self)
  }

  /// Listen to an event on this window.
  pub fn listen<F>(&self, event: M::Event, handler: F) -> HandlerId
  where
    F: Fn(EventPayload) + Send + 'static,
  {
    let label = self.window.label.clone();
    self.manager.listen(event, Some(label), handler)
  }

  /// Listen to a an event on this window a single time.
  pub fn once<F>(&self, event: M::Event, handler: F)
  where
    F: Fn(EventPayload) + Send + 'static,
  {
    let label = self.window.label.clone();
    self.manager.once(event, Some(label), handler)
  }

  /// Triggers an event on this window.
  pub(crate) fn trigger(&self, event: M::Event, data: Option<String>) {
    let label = self.window.label.clone();
    self.manager.trigger(event, Some(label), data)
  }

  /// Evaluates JavaScript on this window.
  pub fn eval(&self, js: &str) -> crate::Result<()> {
    self.window.dispatcher.eval_script(js)
  }

  /// Determines if this window should be resizable.
  pub fn set_resizable(&self, resizable: bool) -> crate::Result<()> {
    self.window.dispatcher.set_resizable(resizable)
  }

  /// Set this window's title.
  pub fn set_title(&self, title: &str) -> crate::Result<()> {
    self.window.dispatcher.set_title(title.to_string())
  }

  /// Maximizes this window.
  pub fn maximize(&self) -> crate::Result<()> {
    self.window.dispatcher.maximize()
  }

  /// Un-maximizes this window.
  pub fn unmaximize(&self) -> crate::Result<()> {
    self.window.dispatcher.unmaximize()
  }

  /// Minimizes this window.
  pub fn minimize(&self) -> crate::Result<()> {
    self.window.dispatcher.minimize()
  }

  /// Un-minimizes this window.
  pub fn unminimize(&self) -> crate::Result<()> {
    self.window.dispatcher.unminimize()
  }

  /// Show this window.
  pub fn show(&self) -> crate::Result<()> {
    self.window.dispatcher.show()
  }

  /// Hide this window.
  pub fn hide(&self) -> crate::Result<()> {
    self.window.dispatcher.hide()
  }

  /// Closes this window.
  pub fn close(&self) -> crate::Result<()> {
    self.window.dispatcher.close()
  }

  /// Determines if this window should be [decorated].
  ///
  /// [decorated]: https://en.wikipedia.org/wiki/Window_(computing)#Window_decoration
  pub fn set_decorations(&self, decorations: bool) -> crate::Result<()> {
    self.window.dispatcher.set_decorations(decorations)
  }

  /// Determines if this window should always be on top of other windows.
  pub fn set_always_on_top(&self, always_on_top: bool) -> crate::Result<()> {
    self.window.dispatcher.set_always_on_top(always_on_top)
  }

  /// Sets this window's width.
  pub fn set_width(&self, width: impl Into<f64>) -> crate::Result<()> {
    self.window.dispatcher.set_width(width.into())
  }

  /// Sets this window's height.
  pub fn set_height(&self, height: impl Into<f64>) -> crate::Result<()> {
    self.window.dispatcher.set_height(height.into())
  }

  /// Resizes this window.
  pub fn resize(&self, width: impl Into<f64>, height: impl Into<f64>) -> crate::Result<()> {
    self.window.dispatcher.resize(width.into(), height.into())
  }

  /// Sets this window's minimum size.
  pub fn set_min_size(
    &self,
    min_width: impl Into<f64>,
    min_height: impl Into<f64>,
  ) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_min_size(min_width.into(), min_height.into())
  }

  /// Sets this window's maximum size.
  pub fn set_max_size(
    &self,
    max_width: impl Into<f64>,
    max_height: impl Into<f64>,
  ) -> crate::Result<()> {
    self
      .window
      .dispatcher
      .set_max_size(max_width.into(), max_height.into())
  }

  /// Sets this window's x position.
  pub fn set_x(&self, x: impl Into<f64>) -> crate::Result<()> {
    self.window.dispatcher.set_x(x.into())
  }

  /// Sets this window's y position.
  pub fn set_y(&self, y: impl Into<f64>) -> crate::Result<()> {
    self.window.dispatcher.set_y(y.into())
  }

  /// Sets this window's position.
  pub fn set_position(&self, x: impl Into<f64>, y: impl Into<f64>) -> crate::Result<()> {
    self.window.dispatcher.set_position(x.into(), y.into())
  }

  /// Determines if this window should be fullscreen.
  pub fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()> {
    self.window.dispatcher.set_fullscreen(fullscreen)
  }

  /// Sets this window' icon.
  pub fn set_icon(&self, icon: Icon) -> crate::Result<()> {
    self.window.dispatcher.set_icon(icon.try_into()?)
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
