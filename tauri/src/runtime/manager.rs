use crate::{
  api::{
    assets::Assets,
    config::{Config, WindowUrl},
    PackageInfo,
  },
  event::{Event, EventHandler, Listeners},
  hooks::{InvokeHandler, InvokeMessage, InvokePayload, OnPageLoad, PageLoadPayload},
  plugin::PluginStore,
  runtime::{
    sealed::ParamsPrivate,
    tag::{tags_to_javascript_array, Tag, ToJavascript},
    webview::{
      Attributes, AttributesPrivate, CustomProtocol, FileDropEvent, FileDropHandler,
      WebviewRpcHandler,
    },
    window::{DetachedWindow, PendingWindow, Window},
    Context, Dispatch, Icon, Params, Runtime,
  },
};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{
  borrow::Cow,
  collections::HashSet,
  convert::TryInto,
  sync::{Arc, Mutex},
};

pub struct InnerWindowManager<M: Params> {
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
  package_info: PackageInfo,
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
        package_info: context.package_info,
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
    pending_labels: &[L],
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

    // If we are on windows use App Data Local as webview temp dir
    // to prevent any bundled application to failed.
    // Fix: https://github.com/tauri-apps/tauri/issues/1365
    #[cfg(windows)]
    {
      // Should return a path similar to C:\Users\<User>\AppData\Local\<AppName>
      let local_app_data = tauri_api::path::resolve_path(
        self.context.package_info.name,
        Some(tauri_api::path::BaseDirectory::LocalData),
      );
      // Make sure the directory exist without panic
      if let Ok(user_data_dir) = local_app_data {
        if let Ok(()) = std::fs::create_dir_all(&user_data_dir) {
          attributes.user_data_path(Some(user_data_dir));
        }
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
  use crate::{generate_context, runtime::flavor::wry::Wry};

  use super::WindowManager;

  #[test]
  fn check_get_url() {
    let context = generate_context!("test/fixture/src-tauri/tauri.conf.json", crate::Context);
    let manager: WindowManager<String, String, _, Wry> =
      WindowManager::with_handlers(context, Box::new(|_| ()), Box::new(|_, _| ()));

    #[cfg(custom_protocol)]
    assert_eq!(manager.get_url(), "tauri://studio.tauri.example");

    #[cfg(dev)]
    {
      use crate::runtime::sealed::ParamsPrivate;
      assert_eq!(manager.get_url(), manager.config().build.dev_path);
    }
  }
}

impl<E, L, A, R> ParamsPrivate<Self> for WindowManager<E, L, A, R>
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
    pending_labels: &[L],
  ) -> crate::Result<PendingWindow<Self>> {
    let (is_local, url) = match &pending.url {
      WindowUrl::App => (true, self.get_url()),
      // todo: we should probably warn about how custom urls usually need to be valid urls
      // e.g. cannot be relative without a base
      WindowUrl::Custom(url) => (url.len() > 7 && &url[0..8] == "tauri://", url.clone()),
    };

    let attributes = pending.attributes.clone();
    if is_local {
      let label = pending.label.clone();
      pending.attributes = self.prepare_attributes(attributes, url, label, pending_labels)?;
      pending.rpc_handler = Some(self.prepare_rpc_handler());
      pending.custom_protocol = Some(self.prepare_custom_protocol());
    } else {
      pending.attributes = attributes.url(url);
    }

    pending.file_drop_handler = Some(self.prepare_file_drop());

    Ok(pending)
  }

  fn attach_window(&self, window: DetachedWindow<Self>) -> Window<Self> {
    let window = Window::new(self.clone(), window);

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

  fn emit_filter_internal<S: Serialize + Clone, F: Fn(&Window<Self>) -> bool>(
    &self,
    event: String,
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
      .try_for_each(|window| window.emit_internal(event.clone(), payload.clone()))
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
      .map(|w| w.label().clone())
      .collect()
  }

  fn config(&self) -> &Config {
    &self.inner.config
  }

  fn package_info(&self) -> &PackageInfo {
    &self.inner.package_info
  }

  fn unlisten(&self, handler_id: EventHandler) {
    self.inner.listeners.unlisten(handler_id)
  }

  fn trigger(&self, event: E, window: Option<L>, data: Option<String>) {
    self.inner.listeners.trigger(event, window, data)
  }

  fn listen<F: Fn(Event) + Send + 'static>(
    &self,
    event: E,
    window: Option<L>,
    handler: F,
  ) -> EventHandler {
    self.inner.listeners.listen(event, window, handler)
  }

  fn once<F: Fn(Event) + Send + 'static>(&self, event: E, window: Option<L>, handler: F) {
    self.inner.listeners.once(event, window, handler)
  }

  fn event_listeners_object_name(&self) -> String {
    self.inner.listeners.listeners_object_name()
  }

  fn event_queue_object_name(&self) -> String {
    self.inner.listeners.queue_object_name()
  }

  fn event_emit_function_name(&self) -> String {
    self.inner.listeners.function_name()
  }
}

impl<E, L, A, R> Params for WindowManager<E, L, A, R>
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
