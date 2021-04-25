// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  api::{
    assets::Assets,
    config::{Config, WindowUrl},
    path::{resolve_path, BaseDirectory},
    PackageInfo,
  },
  event::{Event, EventHandler, Listeners},
  hooks::{InvokeHandler, InvokeMessage, InvokePayload, OnPageLoad, PageLoadPayload},
  plugin::PluginStore,
  runtime::{
    tag::{tags_to_javascript_array, Tag, ToJavascript},
    webview::{Attributes, CustomProtocol, FileDropEvent, FileDropHandler, WebviewRpcHandler},
    window::{DetachedWindow, PendingWindow},
    Dispatch, Icon, Runtime,
  },
  sealed::ParamsBase,
  Context, Params, Window,
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::marker::PhantomData;
use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  convert::TryInto,
  fs::create_dir_all,
  sync::{Arc, Mutex, MutexGuard},
};
use uuid::Uuid;

pub struct InnerWindowManager<P: Params> {
  windows: Mutex<HashMap<P::Label, Window<P>>>,
  plugins: Mutex<PluginStore<P>>,
  listeners: Listeners<P::Event, P::Label>,

  /// The JS message handler.
  invoke_handler: Box<InvokeHandler<P>>,

  /// The page load hook, invoked when the webview performs a navigation.
  on_page_load: Box<OnPageLoad<P>>,

  config: Config,
  assets: Arc<P::Assets>,
  default_window_icon: Option<Vec<u8>>,

  /// A list of salts that are valid for the current application.
  salts: Mutex<HashSet<Uuid>>,
  package_info: PackageInfo,
  /// The webview protocols protocols available to all windows.
  webview_protocols: HashMap<String, std::sync::Arc<CustomProtocol>>,
}

/// A [Zero Sized Type] marker representing a full [`Params`].
///
/// [Zero Sized Type]: https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts
pub struct Args<E: Tag, L: Tag, A: Assets, R: Runtime> {
  _event: PhantomData<fn() -> E>,
  _label: PhantomData<fn() -> L>,
  _assets: PhantomData<fn() -> A>,
  _runtime: PhantomData<fn() -> R>,
}

impl<E: Tag, L: Tag, A: Assets, R: Runtime> Default for Args<E, L, A, R> {
  fn default() -> Self {
    Self {
      _event: PhantomData,
      _label: PhantomData,
      _assets: PhantomData,
      _runtime: PhantomData,
    }
  }
}

impl<E: Tag, L: Tag, A: Assets, R: Runtime> ParamsBase for Args<E, L, A, R> {}
impl<E: Tag, L: Tag, A: Assets, R: Runtime> Params for Args<E, L, A, R> {
  type Event = E;
  type Label = L;
  type Assets = A;
  type Runtime = R;
}

pub struct WindowManager<P: Params> {
  pub inner: Arc<InnerWindowManager<P>>,
  _marker: Args<P::Event, P::Label, P::Assets, P::Runtime>,
}

impl<P: Params> Clone for WindowManager<P> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      _marker: Args::default(),
    }
  }
}

impl<P: Params> WindowManager<P> {
  pub(crate) fn with_handlers(
    context: Context<P::Assets>,
    plugins: PluginStore<P>,
    invoke_handler: Box<InvokeHandler<P>>,
    on_page_load: Box<OnPageLoad<P>>,
    webview_protocols: HashMap<String, std::sync::Arc<CustomProtocol>>,
  ) -> Self {
    Self {
      inner: Arc::new(InnerWindowManager {
        windows: Mutex::default(),
        plugins: Mutex::new(plugins),
        listeners: Listeners::default(),
        invoke_handler,
        on_page_load,
        config: context.config,
        assets: Arc::new(context.assets),
        default_window_icon: context.default_window_icon,
        salts: Mutex::default(),
        package_info: context.package_info,
        webview_protocols,
      }),
      _marker: Args::default(),
    }
  }

  /// Get a locked handle to the windows.
  pub(crate) fn windows_lock(&self) -> MutexGuard<'_, HashMap<P::Label, Window<P>>> {
    self.inner.windows.lock().expect("poisoned window manager")
  }

  // setup content for dev-server
  #[cfg(dev)]
  fn get_url(&self) -> String {
    if self.inner.config.build.dev_path.starts_with("http") {
      self.inner.config.build.dev_path.clone()
    } else {
      "tauri://localhost".into()
    }
  }

  #[cfg(custom_protocol)]
  fn get_url(&self) -> String {
    "tauri://localhost".into()
  }

  fn prepare_attributes(
    &self,
    attrs: <<P::Runtime as Runtime>::Dispatcher as Dispatch>::Attributes,
    url: String,
    label: P::Label,
    pending_labels: &[P::Label],
  ) -> crate::Result<<<P::Runtime as Runtime>::Dispatcher as Dispatch>::Attributes> {
    let is_init_global = self.inner.config.build.with_global_tauri;
    let plugin_init = self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .initialization_script();

    let mut attributes = attrs
      .url(url)
      .initialization_script(&self.initialization_script(&plugin_init, is_init_global))
      .initialization_script(&format!(
        r#"
              window.__TAURI__.__windows = {window_labels_array}.map(function (label) {{ return {{ label: label }} }});
              window.__TAURI__.__currentWindow = {{ label: {current_window_label} }}
            "#,
        window_labels_array = tags_to_javascript_array(pending_labels)?,
        current_window_label = label.to_javascript()?,
      ));

    if !attributes.has_icon() {
      if let Some(default_window_icon) = &self.inner.default_window_icon {
        let icon = Icon::Raw(default_window_icon.clone());
        let icon = icon.try_into().expect("infallible icon convert failed");
        attributes = attributes.icon(icon);
      }
    }

    for (name, protocol) in &self.inner.webview_protocols {
      if !attributes.has_webview_protocol(name) {
        let protocol = protocol.clone();
        attributes =
          attributes.register_webview_protocol(name.clone(), move |p| (protocol.handler)(p));
      }
    }

    if !attributes.has_webview_protocol("tauri") {
      attributes =
        attributes.register_webview_protocol("tauri", self.prepare_webview_protocol().handler);
    }

    let local_app_data = resolve_path(
      &self.inner.config.tauri.bundle.identifier,
      Some(BaseDirectory::LocalData),
    );
    if let Ok(user_data_dir) = local_app_data {
      // Make sure the directory exist without panic
      if create_dir_all(&user_data_dir).is_ok() {
        attributes = attributes.user_data_path(Some(user_data_dir));
      }
    }

    Ok(attributes)
  }

  fn prepare_rpc_handler(&self) -> WebviewRpcHandler<P> {
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

  fn prepare_webview_protocol(&self) -> CustomProtocol {
    let assets = self.inner.assets.clone();
    CustomProtocol {
      handler: Box::new(move |path| {
        let mut path = path
          .split('?')
          // ignore query string
          .next()
          .unwrap()
          .to_string()
          .replace("tauri://localhost", "");
        if path.ends_with('/') {
          path.pop();
        }
        let path = if path.is_empty() {
          // if the url is `tauri://localhost`, we should load `index.html`
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

  fn prepare_file_drop(&self) -> FileDropHandler<P> {
    let manager = self.clone();
    Box::new(move |event, window| {
      let manager = manager.clone();
      crate::async_runtime::block_on(async move {
        let window = manager.attach_window(window);
        let _ = match event {
          FileDropEvent::Hovered(paths) => {
            window.emit_internal("tauri://file-drop".to_string(), Some(paths))
          }
          FileDropEvent::Dropped(paths) => {
            window.emit_internal("tauri://file-drop-hover".to_string(), Some(paths))
          }
          FileDropEvent::Cancelled => {
            window.emit_internal("tauri://file-drop-cancelled".to_string(), Some(()))
          }
        };
      });
      true
    })
  }

  fn initialization_script(
    &self,
    plugin_initialization_script: &str,
    with_global_tauri: bool,
  ) -> String {
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
      event_initialization_script = self.event_initialization_script(),
      plugin_initialization_script = plugin_initialization_script
    )
  }

  fn event_initialization_script(&self) -> String {
    return format!(
      "
      window['{queue}'] = [];
      window['{function}'] = function (eventData, salt, ignoreQueue) {{
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
      function = self.inner.listeners.function_name(),
      queue = self.inner.listeners.queue_object_name(),
      listeners = self.inner.listeners.listeners_object_name()
    );
  }
}

#[cfg(test)]
mod test {
  use super::{Args, WindowManager};
  use crate::{generate_context, plugin::PluginStore, runtime::flavors::wry::Wry};

  #[test]
  fn check_get_url() {
    let context = generate_context!("test/fixture/src-tauri/tauri.conf.json", crate);
    let manager: WindowManager<Args<String, String, _, Wry>> = WindowManager::with_handlers(
      context,
      PluginStore::default(),
      Box::new(|_| ()),
      Box::new(|_, _| ()),
      Default::default(),
    );

    #[cfg(custom_protocol)]
    assert_eq!(manager.get_url(), "tauri://localhost");

    #[cfg(dev)]
    assert_eq!(manager.get_url(), manager.config().build.dev_path);
  }
}

impl<P: Params> WindowManager<P> {
  pub fn run_invoke_handler(&self, message: InvokeMessage<P>) {
    (self.inner.invoke_handler)(message);
  }
  pub fn run_on_page_load(&self, window: Window<P>, payload: PageLoadPayload) {
    (self.inner.on_page_load)(window.clone(), payload.clone());
    self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .on_page_load(window, payload);
  }
  pub fn extend_api(&self, message: InvokeMessage<P>) {
    self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .extend_api(message);
  }
  pub fn initialize_plugins(&self) -> crate::Result<()> {
    self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .initialize(&self.inner.config.plugins)
  }

  pub fn prepare_window(
    &self,
    mut pending: PendingWindow<P>,
    pending_labels: &[P::Label],
  ) -> crate::Result<PendingWindow<P>> {
    let (is_local, url) = match &pending.url {
      WindowUrl::App(path) => {
        let url = self.get_url();
        (
          true,
          // ignore "index.html" just to simplify the url
          if path.to_str() != Some("index.html") {
            format!("{}/{}", url, path.to_string_lossy())
          } else {
            url
          },
        )
      }
      WindowUrl::External(url) => (url.as_str().starts_with("tauri://"), url.to_string()),
    };

    let attributes = pending.attributes.clone();
    if is_local {
      let label = pending.label.clone();
      pending.attributes = self.prepare_attributes(attributes, url, label, pending_labels)?;
      pending.rpc_handler = Some(self.prepare_rpc_handler());
    } else {
      pending.attributes = attributes.url(url);
    }

    pending.file_drop_handler = Some(self.prepare_file_drop());

    Ok(pending)
  }
  pub fn attach_window(&self, window: DetachedWindow<P>) -> Window<P> {
    let window = Window::new(self.clone(), window);

    // insert the window into our manager
    {
      self
        .windows_lock()
        .insert(window.label().clone(), window.clone());
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
  pub fn emit_filter_internal<S: Serialize + Clone, F: Fn(&Window<P>) -> bool>(
    &self,
    event: String,
    payload: Option<S>,
    filter: F,
  ) -> crate::Result<()> {
    self
      .windows_lock()
      .values()
      .filter(|&w| filter(w))
      .try_for_each(|window| window.emit_internal(event.clone(), payload.clone()))
  }
  pub fn emit_filter<S: Serialize + Clone, F: Fn(&Window<P>) -> bool>(
    &self,
    event: P::Event,
    payload: Option<S>,
    filter: F,
  ) -> crate::Result<()> {
    self
      .windows_lock()
      .values()
      .filter(|&w| filter(w))
      .try_for_each(|window| window.emit(&event, payload.clone()))
  }
  pub fn labels(&self) -> HashSet<P::Label> {
    self.windows_lock().keys().cloned().collect()
  }
  pub fn config(&self) -> &Config {
    &self.inner.config
  }
  pub fn package_info(&self) -> &PackageInfo {
    &self.inner.package_info
  }
  pub fn unlisten(&self, handler_id: EventHandler) {
    self.inner.listeners.unlisten(handler_id)
  }
  pub fn trigger(&self, event: P::Event, window: Option<P::Label>, data: Option<String>) {
    self.inner.listeners.trigger(event, window, data)
  }
  pub fn listen<F: Fn(Event) + Send + 'static>(
    &self,
    event: P::Event,
    window: Option<P::Label>,
    handler: F,
  ) -> EventHandler {
    self.inner.listeners.listen(event, window, handler)
  }
  pub fn once<F: Fn(Event) + Send + 'static>(
    &self,
    event: P::Event,
    window: Option<P::Label>,
    handler: F,
  ) -> EventHandler {
    self.inner.listeners.once(event, window, handler)
  }
  pub fn event_listeners_object_name(&self) -> String {
    self.inner.listeners.listeners_object_name()
  }
  pub fn event_queue_object_name(&self) -> String {
    self.inner.listeners.queue_object_name()
  }
  pub fn event_emit_function_name(&self) -> String {
    self.inner.listeners.function_name()
  }
  pub fn generate_salt(&self) -> Uuid {
    let salt = Uuid::new_v4();
    self
      .inner
      .salts
      .lock()
      .expect("poisoned salt mutex")
      .insert(salt);
    salt
  }
  pub fn verify_salt(&self, salt: String) -> bool {
    // flat out ignore any invalid uuids
    let uuid: Uuid = match salt.parse() {
      Ok(uuid) => uuid,
      Err(_) => return false,
    };

    // HashSet::remove lets us know if the entry was found
    self
      .inner
      .salts
      .lock()
      .expect("poisoned salt mutex")
      .remove(&uuid)
  }
  pub fn get_window(&self, label: &P::Label) -> Option<Window<P>> {
    self.windows_lock().get(label).cloned()
  }
  pub fn windows(&self) -> HashMap<P::Label, Window<P>> {
    self.windows_lock().clone()
  }
}
