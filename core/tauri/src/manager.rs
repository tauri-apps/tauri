// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  app::{AppHandle, GlobalWindowEvent, GlobalWindowEventListener},
  event::{Event, EventHandler, Listeners},
  hooks::{InvokeHandler, OnPageLoad, PageLoadPayload},
  plugin::PluginStore,
  runtime::{
    http::{
      HttpRange, MimeType, Request as HttpRequest, Response as HttpResponse,
      ResponseBuilder as HttpResponseBuilder,
    },
    webview::{FileDropEvent, FileDropHandler, InvokePayload, WebviewRpcHandler, WindowBuilder},
    window::{dpi::PhysicalSize, DetachedWindow, PendingWindow, WindowEvent},
    Icon, Runtime,
  },
  utils::{
    assets::Assets,
    config::{AppUrl, Config, WindowUrl},
    PackageInfo,
  },
  Context, Invoke, StateManager, Window,
};

#[cfg(target_os = "windows")]
use crate::api::path::{resolve_path, BaseDirectory};

use crate::app::{GlobalMenuEventListener, WindowMenuEvent};

use crate::{runtime::menu::Menu, MenuEvent};

use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  fmt,
  fs::create_dir_all,
  io::SeekFrom,
  sync::{Arc, Mutex, MutexGuard},
};
use tauri_macros::default_runtime;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use url::Url;

const WINDOW_RESIZED_EVENT: &str = "tauri://resize";
const WINDOW_MOVED_EVENT: &str = "tauri://move";
const WINDOW_CLOSE_REQUESTED_EVENT: &str = "tauri://close-requested";
const WINDOW_DESTROYED_EVENT: &str = "tauri://destroyed";
const WINDOW_FOCUS_EVENT: &str = "tauri://focus";
const WINDOW_BLUR_EVENT: &str = "tauri://blur";
const WINDOW_SCALE_FACTOR_CHANGED_EVENT: &str = "tauri://scale-change";
const MENU_EVENT: &str = "tauri://menu";

#[default_runtime(crate::Wry, wry)]
pub struct InnerWindowManager<R: Runtime> {
  windows: Mutex<HashMap<String, Window<R>>>,
  pub(crate) plugins: Mutex<PluginStore<R>>,
  listeners: Listeners,
  pub(crate) state: Arc<StateManager>,

  /// The JS message handler.
  invoke_handler: Box<InvokeHandler<R>>,

  /// The page load hook, invoked when the webview performs a navigation.
  on_page_load: Box<OnPageLoad<R>>,

  config: Arc<Config>,
  assets: Arc<dyn Assets>,
  default_window_icon: Option<Vec<u8>>,

  package_info: PackageInfo,
  /// The webview protocols protocols available to all windows.
  uri_scheme_protocols: HashMap<String, Arc<CustomProtocol<R>>>,
  /// The menu set to all windows.
  menu: Option<Menu>,
  /// Menu event listeners to all windows.
  menu_event_listeners: Arc<Vec<GlobalMenuEventListener<R>>>,
  /// Window event listeners to all windows.
  window_event_listeners: Arc<Vec<GlobalWindowEventListener<R>>>,
}

impl<R: Runtime> fmt::Debug for InnerWindowManager<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("InnerWindowManager")
      .field("plugins", &self.plugins)
      .field("state", &self.state)
      .field("config", &self.config)
      .field("default_window_icon", &self.default_window_icon)
      .field("package_info", &self.package_info)
      .field("menu", &self.menu)
      .finish()
  }
}

/// A resolved asset.
pub struct Asset {
  /// The asset bytes.
  pub bytes: Vec<u8>,
  /// The asset's mime type.
  pub mime_type: String,
}

/// Uses a custom URI scheme handler to resolve file requests
pub struct CustomProtocol<R: Runtime> {
  /// Handler for protocol
  #[allow(clippy::type_complexity)]
  pub protocol: Box<
    dyn Fn(&AppHandle<R>, &HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>>
      + Send
      + Sync,
  >,
}

#[default_runtime(crate::Wry, wry)]
#[derive(Debug)]
pub struct WindowManager<R: Runtime> {
  pub inner: Arc<InnerWindowManager<R>>,
  invoke_keys: Arc<Mutex<Vec<u32>>>,
}

impl<R: Runtime> Clone for WindowManager<R> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      invoke_keys: self.invoke_keys.clone(),
    }
  }
}

impl<R: Runtime> WindowManager<R> {
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn with_handlers(
    context: Context<impl Assets>,
    plugins: PluginStore<R>,
    invoke_handler: Box<InvokeHandler<R>>,
    on_page_load: Box<OnPageLoad<R>>,
    uri_scheme_protocols: HashMap<String, Arc<CustomProtocol<R>>>,
    state: StateManager,
    window_event_listeners: Vec<GlobalWindowEventListener<R>>,
    (menu, menu_event_listeners): (Option<Menu>, Vec<GlobalMenuEventListener<R>>),
  ) -> Self {
    Self {
      inner: Arc::new(InnerWindowManager {
        windows: Mutex::default(),
        plugins: Mutex::new(plugins),
        listeners: Listeners::default(),
        state: Arc::new(state),
        invoke_handler,
        on_page_load,
        config: Arc::new(context.config),
        assets: context.assets,
        default_window_icon: context.default_window_icon,
        package_info: context.package_info,
        uri_scheme_protocols,
        menu,
        menu_event_listeners: Arc::new(menu_event_listeners),
        window_event_listeners: Arc::new(window_event_listeners),
      }),
      invoke_keys: Default::default(),
    }
  }

  /// Get a locked handle to the windows.
  pub(crate) fn windows_lock(&self) -> MutexGuard<'_, HashMap<String, Window<R>>> {
    self.inner.windows.lock().expect("poisoned window manager")
  }

  /// State managed by the application.
  pub(crate) fn state(&self) -> Arc<StateManager> {
    self.inner.state.clone()
  }

  /// Get the base path to serve data from.
  ///
  /// * In dev mode, this will be based on the `devPath` configuration value.
  /// * Otherwise, this will be based on the `distDir` configuration value.
  #[cfg(custom_protocol)]
  fn base_path(&self) -> &AppUrl {
    &self.inner.config.build.dist_dir
  }

  #[cfg(dev)]
  fn base_path(&self) -> &AppUrl {
    &self.inner.config.build.dev_path
  }

  /// Get the base URL to use for webview requests.
  ///
  /// In dev mode, this will be based on the `devPath` configuration value.
  fn get_url(&self) -> Cow<'_, Url> {
    match self.base_path() {
      AppUrl::Url(WindowUrl::External(url)) => Cow::Borrowed(url),
      _ => Cow::Owned(Url::parse("tauri://localhost").unwrap()),
    }
  }

  fn generate_invoke_key(&self) -> u32 {
    let key = rand::random();
    self.invoke_keys.lock().unwrap().push(key);
    key
  }

  /// Checks whether the invoke key is valid or not.
  ///
  /// An invoke key is valid if it was generated by this manager instance.
  pub(crate) fn verify_invoke_key(&self, key: u32) -> bool {
    self.invoke_keys.lock().unwrap().contains(&key)
  }

  fn prepare_pending_window(
    &self,
    mut pending: PendingWindow<R>,
    label: &str,
    pending_labels: &[String],
    app_handle: AppHandle<R>,
  ) -> crate::Result<PendingWindow<R>> {
    let is_init_global = self.inner.config.build.with_global_tauri;
    let plugin_init = self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .initialization_script();

    let mut webview_attributes = pending.webview_attributes;
    webview_attributes = webview_attributes
      .initialization_script(&self.initialization_script(&plugin_init, is_init_global))
      .initialization_script(&format!(
        r#"
          window.__TAURI__.__windows = {window_labels_array}.map(function (label) {{ return {{ label: label }} }});
          window.__TAURI__.__currentWindow = {{ label: {current_window_label} }}
        "#,
        window_labels_array = serde_json::to_string(pending_labels)?,
        current_window_label = serde_json::to_string(&label)?,
      ));

    #[cfg(dev)]
    {
      webview_attributes = webview_attributes.initialization_script(&format!(
        "window.__TAURI_INVOKE_KEY__ = {}",
        self.generate_invoke_key()
      ));
    }

    pending.webview_attributes = webview_attributes;

    if !pending.window_builder.has_icon() {
      if let Some(default_window_icon) = &self.inner.default_window_icon {
        let icon = Icon::Raw(default_window_icon.clone());
        pending.window_builder = pending.window_builder.icon(icon)?;
      }
    }

    if pending.window_builder.get_menu().is_none() {
      if let Some(menu) = &self.inner.menu {
        pending = pending.set_menu(menu.clone());
      }
    }

    let mut registered_scheme_protocols = Vec::new();

    for (uri_scheme, protocol) in &self.inner.uri_scheme_protocols {
      registered_scheme_protocols.push(uri_scheme.clone());
      let protocol = protocol.clone();
      let app_handle = Mutex::new(app_handle.clone());
      pending.register_uri_scheme_protocol(uri_scheme.clone(), move |p| {
        (protocol.protocol)(&app_handle.lock().unwrap(), p)
      });
    }

    if !registered_scheme_protocols.contains(&"tauri".into()) {
      pending.register_uri_scheme_protocol("tauri", self.prepare_uri_scheme_protocol());
      registered_scheme_protocols.push("tauri".into());
    }
    if !registered_scheme_protocols.contains(&"asset".into()) {
      pending.register_uri_scheme_protocol("asset", move |request| {
        #[cfg(target_os = "windows")]
        let path = request.uri().replace("asset://localhost/", "");
        #[cfg(not(target_os = "windows"))]
        let path = request.uri().replace("asset://", "");
        let path = percent_encoding::percent_decode(path.as_bytes())
          .decode_utf8_lossy()
          .to_string();
        let path_for_data = path.clone();

        // handle 206 (partial range) http request
        if let Some(range) = request.headers().get("range") {
          let mut status_code = 200;
          let path_for_data = path_for_data.clone();
          let mut response = HttpResponseBuilder::new();
          let (response, status_code, data) = crate::async_runtime::block_on(async move {
            let mut buf = Vec::new();
            let mut file = tokio::fs::File::open(path_for_data.clone()).await.unwrap();
            // Get the file size
            let file_size = file.metadata().await.unwrap().len();
            // parse the range
            let range = HttpRange::parse(range.to_str().unwrap(), file_size).unwrap();

            // FIXME: Support multiple ranges
            // let support only 1 range for now
            let first_range = range.first();
            if let Some(range) = first_range {
              let mut real_length = range.length;
              // prevent max_length;
              // specially on webview2
              if range.length > file_size / 3 {
                // max size sent (400ko / request)
                // as it's local file system we can afford to read more often
                real_length = 1024 * 400;
              }

              // last byte we are reading, the length of the range include the last byte
              // who should be skipped on the header
              let last_byte = range.start + real_length - 1;
              // partial content
              status_code = 206;

              response = response
                .header("Connection", "Keep-Alive")
                .header("Accept-Ranges", "bytes")
                .header("Content-Length", real_length)
                .header(
                  "Content-Range",
                  format!("bytes {}-{}/{}", range.start, last_byte, file_size),
                );

              file.seek(SeekFrom::Start(range.start)).await.unwrap();
              file.take(real_length).read_to_end(&mut buf).await.unwrap();
            }

            (response, status_code, buf)
          });

          if !data.is_empty() {
            let mime_type = MimeType::parse(&data, &path);
            return response.mimetype(&mime_type).status(status_code).body(data);
          }
        }

        let data =
          crate::async_runtime::block_on(async move { tokio::fs::read(path_for_data).await })?;
        let mime_type = MimeType::parse(&data, &path);
        HttpResponseBuilder::new().mimetype(&mime_type).body(data)
      });
    }

    Ok(pending)
  }

  fn prepare_rpc_handler(&self, app_handle: AppHandle<R>) -> WebviewRpcHandler<R> {
    let manager = self.clone();
    Box::new(move |window, request| {
      let window = Window::new(manager.clone(), window, app_handle.clone());
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

  pub fn get_asset(&self, mut path: String) -> Result<Asset, Box<dyn std::error::Error>> {
    let assets = &self.inner.assets;
    if path.ends_with('/') {
      path.pop();
    }
    path = percent_encoding::percent_decode(path.as_bytes())
      .decode_utf8_lossy()
      .to_string();
    let path = if path.is_empty() {
      // if the url is `tauri://localhost`, we should load `index.html`
      "index.html".to_string()
    } else {
      // skip leading `/`
      path.chars().skip(1).collect::<String>()
    };
    let is_javascript = path.ends_with(".js") || path.ends_with(".cjs") || path.ends_with(".mjs");
    let is_html = path.ends_with(".html");

    let asset_response = assets
      .get(&path.as_str().into())
      .or_else(|| assets.get(&format!("{}/index.html", path.as_str()).into()))
      .or_else(|| {
        #[cfg(debug_assertions)]
        eprintln!("Asset `{}` not found; fallback to index.html", path); // TODO log::error!
        assets.get(&"index.html".into())
      })
      .ok_or_else(|| crate::Error::AssetNotFound(path.clone()))
      .map(Cow::into_owned);

    match asset_response {
      Ok(asset) => {
        let final_data = match is_javascript || is_html {
          true => String::from_utf8_lossy(&asset)
            .into_owned()
            .replacen(
              "__TAURI__INVOKE_KEY_TOKEN__",
              &self.generate_invoke_key().to_string(),
              1,
            )
            .as_bytes()
            .to_vec(),
          false => asset,
        };
        let mime_type = MimeType::parse(&final_data, &path);
        Ok(Asset {
          bytes: final_data,
          mime_type,
        })
      }
      Err(e) => {
        #[cfg(debug_assertions)]
        eprintln!("{:?}", e); // TODO log::error!
        Err(Box::new(e))
      }
    }
  }

  #[allow(clippy::type_complexity)]
  fn prepare_uri_scheme_protocol(
    &self,
  ) -> Box<dyn Fn(&HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> + Send + Sync>
  {
    let manager = self.clone();
    Box::new(move |request| {
      let path = request
        .uri()
        .split(&['?', '#'][..])
        // ignore query string
        .next()
        .unwrap()
        .to_string()
        .replace("tauri://localhost", "");
      let asset = manager.get_asset(path)?;
      HttpResponseBuilder::new()
        .mimetype(&asset.mime_type)
        .body(asset.bytes)
    })
  }

  fn prepare_file_drop(&self, app_handle: AppHandle<R>) -> FileDropHandler<R> {
    let manager = self.clone();
    Box::new(move |event, window| {
      let manager = manager.clone();
      let app_handle = app_handle.clone();
      crate::async_runtime::block_on(async move {
        let window = Window::new(manager.clone(), window, app_handle);
        let _ = match event {
          FileDropEvent::Hovered(paths) => window.emit("tauri://file-drop-hover", Some(paths)),
          FileDropEvent::Dropped(paths) => window.emit("tauri://file-drop", Some(paths)),
          FileDropEvent::Cancelled => window.emit("tauri://file-drop-cancelled", Some(())),
          _ => unimplemented!(),
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
    let key = self.generate_invoke_key();
    format!(
      r#"
      (function () {{
        const __TAURI_INVOKE_KEY__ = {key};
        {bundle_script}
      }})()
      {core_script}
      {event_initialization_script}
      if (window.rpc) {{
        window.__TAURI_INVOKE__("__initialized", {{ url: window.location.href }}, {key})
      }} else {{
        window.addEventListener('DOMContentLoaded', function () {{
          window.__TAURI_INVOKE__("__initialized", {{ url: window.location.href }}, {key})
        }})
      }}
      {plugin_initialization_script}
    "#,
      key = key,
      core_script = include_str!("../scripts/core.js").replace("_KEY_VALUE_", &key.to_string()),
      bundle_script = if with_global_tauri {
        include_str!("../scripts/bundle.js")
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
      window['{function}'] = function (eventData) {{
      const listeners = (window['{listeners}'] && window['{listeners}'][eventData.event]) || []

      for (let i = listeners.length - 1; i >= 0; i--) {{
        const listener = listeners[i]
        eventData.id = listener.id
        listener.handler(eventData)
      }}
    }}
    ",
      function = self.inner.listeners.function_name(),
      listeners = self.inner.listeners.listeners_object_name()
    );
  }
}

#[cfg(test)]
mod test {
  use super::WindowManager;
  use crate::{generate_context, plugin::PluginStore, StateManager, Wry};

  #[test]
  fn check_get_url() {
    let context = generate_context!("test/fixture/src-tauri/tauri.conf.json", crate);
    let manager: WindowManager<Wry> = WindowManager::with_handlers(
      context,
      PluginStore::default(),
      Box::new(|_| ()),
      Box::new(|_, _| ()),
      Default::default(),
      StateManager::new(),
      Default::default(),
      Default::default(),
    );

    #[cfg(custom_protocol)]
    assert_eq!(manager.get_url().to_string(), "tauri://localhost");

    #[cfg(dev)]
    assert_eq!(manager.get_url().to_string(), "http://localhost:4000/");
  }
}

impl<R: Runtime> WindowManager<R> {
  pub fn run_invoke_handler(&self, invoke: Invoke<R>) {
    (self.inner.invoke_handler)(invoke);
  }

  pub fn run_on_page_load(&self, window: Window<R>, payload: PageLoadPayload) {
    (self.inner.on_page_load)(window.clone(), payload.clone());
    self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .on_page_load(window, payload);
  }

  pub fn extend_api(&self, invoke: Invoke<R>) {
    self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .extend_api(invoke);
  }

  pub fn initialize_plugins(&self, app: &AppHandle<R>) -> crate::Result<()> {
    self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .initialize(app, &self.inner.config.plugins)
  }

  pub fn prepare_window(
    &self,
    app_handle: AppHandle<R>,
    mut pending: PendingWindow<R>,
    pending_labels: &[String],
  ) -> crate::Result<PendingWindow<R>> {
    if self.windows_lock().contains_key(&pending.label) {
      return Err(crate::Error::WindowLabelAlreadyExists(pending.label));
    }
    let (is_local, url) = match &pending.webview_attributes.url {
      WindowUrl::App(path) => {
        let url = self.get_url();
        (
          true,
          // ignore "index.html" just to simplify the url
          if path.to_str() != Some("index.html") {
            url
              .join(&*path.to_string_lossy())
              .map_err(crate::Error::InvalidUrl)?
              .to_string()
          } else {
            url.to_string()
          },
        )
      }
      WindowUrl::External(url) => (url.scheme() == "tauri", url.to_string()),
      _ => unimplemented!(),
    };

    if is_local {
      let label = pending.label.clone();
      pending = self.prepare_pending_window(pending, &label, pending_labels, app_handle.clone())?;
      pending.rpc_handler = Some(self.prepare_rpc_handler(app_handle.clone()));
    }

    if pending.webview_attributes.file_drop_handler_enabled {
      pending.file_drop_handler = Some(self.prepare_file_drop(app_handle));
    }

    pending.url = url;

    // in `Windows`, we need to force a data_directory
    // but we do respect user-specification
    #[cfg(target_os = "windows")]
    if pending.webview_attributes.data_directory.is_none() {
      let local_app_data = resolve_path(
        &self.inner.config,
        &self.inner.package_info,
        &self.inner.config.tauri.bundle.identifier,
        Some(BaseDirectory::LocalData),
      );
      if let Ok(user_data_dir) = local_app_data {
        pending.webview_attributes.data_directory = Some(user_data_dir);
      }
    }

    // make sure the directory is created and available to prevent a panic
    if let Some(user_data_dir) = &pending.webview_attributes.data_directory {
      if !user_data_dir.exists() {
        create_dir_all(user_data_dir)?;
      }
    }

    Ok(pending)
  }

  pub fn attach_window(&self, app_handle: AppHandle<R>, window: DetachedWindow<R>) -> Window<R> {
    let window = Window::new(self.clone(), window, app_handle);

    let window_ = window.clone();
    let window_event_listeners = self.inner.window_event_listeners.clone();
    let manager = self.clone();
    window.on_window_event(move |event| {
      let _ = on_window_event(&window_, &manager, event);
      for handler in window_event_listeners.iter() {
        handler(GlobalWindowEvent {
          window: window_.clone(),
          event: event.clone(),
        });
      }
    });
    {
      let window_ = window.clone();
      let menu_event_listeners = self.inner.menu_event_listeners.clone();
      window.on_menu_event(move |event| {
        let _ = on_menu_event(&window_, &event);
        for handler in menu_event_listeners.iter() {
          handler(WindowMenuEvent {
            window: window_.clone(),
            menu_item_id: event.menu_item_id.clone(),
          });
        }
      });
    }

    // insert the window into our manager
    {
      self
        .windows_lock()
        .insert(window.label().to_string(), window.clone());
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

  pub(crate) fn on_window_close(&self, label: &str) {
    self.windows_lock().remove(label);
  }

  pub fn emit_filter<S, F>(&self, event: &str, payload: S, filter: F) -> crate::Result<()>
  where
    S: Serialize + Clone,
    F: Fn(&Window<R>) -> bool,
  {
    self
      .windows_lock()
      .values()
      .filter(|&w| filter(w))
      .try_for_each(|window| window.emit(event, payload.clone()))
  }

  pub fn labels(&self) -> HashSet<String> {
    self.windows_lock().keys().cloned().collect()
  }

  pub fn config(&self) -> Arc<Config> {
    self.inner.config.clone()
  }

  pub fn package_info(&self) -> &PackageInfo {
    &self.inner.package_info
  }

  pub fn unlisten(&self, handler_id: EventHandler) {
    self.inner.listeners.unlisten(handler_id)
  }

  pub fn trigger(&self, event: &str, window: Option<String>, data: Option<String>) {
    self.inner.listeners.trigger(event, window, data)
  }

  pub fn listen<F: Fn(Event) + Send + 'static>(
    &self,
    event: String,
    window: Option<String>,
    handler: F,
  ) -> EventHandler {
    self.inner.listeners.listen(event, window, handler)
  }
  pub fn once<F: Fn(Event) + Send + 'static>(
    &self,
    event: String,
    window: Option<String>,
    handler: F,
  ) -> EventHandler {
    self.inner.listeners.once(event, window, handler)
  }

  pub fn event_listeners_object_name(&self) -> String {
    self.inner.listeners.listeners_object_name()
  }

  pub fn event_emit_function_name(&self) -> String {
    self.inner.listeners.function_name()
  }

  pub fn get_window(&self, label: &str) -> Option<Window<R>> {
    self.windows_lock().get(label).cloned()
  }

  pub fn windows(&self) -> HashMap<String, Window<R>> {
    self.windows_lock().clone()
  }
}

fn on_window_event<R: Runtime>(
  window: &Window<R>,
  manager: &WindowManager<R>,
  event: &WindowEvent,
) -> crate::Result<()> {
  match event {
    WindowEvent::Resized(size) => window.emit(WINDOW_RESIZED_EVENT, Some(size))?,
    WindowEvent::Moved(position) => window.emit(WINDOW_MOVED_EVENT, Some(position))?,
    WindowEvent::CloseRequested => {
      window.emit(WINDOW_CLOSE_REQUESTED_EVENT, Some(()))?;
    }
    WindowEvent::Destroyed => {
      window.emit(WINDOW_DESTROYED_EVENT, Some(()))?;
      let label = window.label();
      for window in manager.inner.windows.lock().unwrap().values() {
        window.eval(&format!(
          r#"window.__TAURI__.__windows = window.__TAURI__.__windows.filter(w => w.label !== "{}");"#,
          label
        ))?;
      }
    }
    WindowEvent::Focused(focused) => window.emit(
      if *focused {
        WINDOW_FOCUS_EVENT
      } else {
        WINDOW_BLUR_EVENT
      },
      Some(()),
    )?,
    WindowEvent::ScaleFactorChanged {
      scale_factor,
      new_inner_size,
      ..
    } => window.emit(
      WINDOW_SCALE_FACTOR_CHANGED_EVENT,
      Some(ScaleFactorChanged {
        scale_factor: *scale_factor,
        size: *new_inner_size,
      }),
    )?,
    _ => unimplemented!(),
  }
  Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScaleFactorChanged {
  scale_factor: f64,
  size: PhysicalSize<u32>,
}

fn on_menu_event<R: Runtime>(window: &Window<R>, event: &MenuEvent) -> crate::Result<()> {
  window.emit(MENU_EVENT, Some(event.menu_item_id.clone()))
}
