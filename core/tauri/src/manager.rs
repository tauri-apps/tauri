// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  app::{AppHandle, GlobalWindowEvent, GlobalWindowEventListener},
  event::{is_event_name_valid, Event, EventHandler, Listeners},
  hooks::{InvokeHandler, InvokePayload, InvokeResponder, OnPageLoad, PageLoadPayload},
  plugin::PluginStore,
  runtime::{
    http::{
      MimeType, Request as HttpRequest, Response as HttpResponse,
      ResponseBuilder as HttpResponseBuilder,
    },
    webview::{FileDropEvent, FileDropHandler, WebviewIpcHandler, WindowBuilder},
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

#[cfg(any(target_os = "linux", target_os = "windows"))]
use crate::api::path::{resolve_path, BaseDirectory};

use crate::app::{GlobalMenuEventListener, WindowMenuEvent};

use crate::{runtime::menu::Menu, MenuEvent};

use regex::{Captures, Regex};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  fmt,
  fs::create_dir_all,
  sync::{Arc, Mutex, MutexGuard},
};
use tauri_macros::default_runtime;
use tauri_utils::{
  assets::{AssetKey, CspHash},
  html::{
    inject_csp, parse as parse_html, CSP_TOKEN, INVOKE_KEY_TOKEN, SCRIPT_NONCE_TOKEN,
    STYLE_NONCE_TOKEN,
  },
};
use url::Url;

const WINDOW_RESIZED_EVENT: &str = "tauri://resize";
const WINDOW_MOVED_EVENT: &str = "tauri://move";
const WINDOW_CLOSE_REQUESTED_EVENT: &str = "tauri://close-requested";
const WINDOW_DESTROYED_EVENT: &str = "tauri://destroyed";
const WINDOW_FOCUS_EVENT: &str = "tauri://focus";
const WINDOW_BLUR_EVENT: &str = "tauri://blur";
const WINDOW_SCALE_FACTOR_CHANGED_EVENT: &str = "tauri://scale-change";
const MENU_EVENT: &str = "tauri://menu";

#[derive(Default)]
/// Spaced and quoted Content-Security-Policy hash values.
struct CspHashStrings {
  script: String,
  style: String,
}

fn replace_csp_nonce(
  asset: &mut String,
  token: &str,
  csp: &mut String,
  csp_attr: &str,
  hashes: String,
) {
  let regex = Regex::new(token).unwrap();
  let mut nonces = Vec::new();
  *asset = regex
    .replace_all(asset, |_: &Captures<'_>| {
      let nonce = rand::random::<usize>();
      nonces.push(nonce);
      nonce.to_string()
    })
    .to_string();

  if !(nonces.is_empty() && hashes.is_empty()) {
    let attr = format!(
      "{} 'self'{}{}",
      csp_attr,
      if nonces.is_empty() {
        "".into()
      } else {
        format!(
          " {}",
          nonces
            .into_iter()
            .map(|n| format!("'nonce-{}'", n))
            .collect::<Vec<String>>()
            .join(" ")
        )
      },
      hashes
    );
    if csp.contains(csp_attr) {
      *csp = csp.replace(csp_attr, &attr);
    } else {
      csp.push_str("; ");
      csp.push_str(&attr);
    }
  }
}

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
  /// Responder for invoke calls.
  invoke_responder: Arc<InvokeResponder<R>>,
  /// The script that initializes the invoke system.
  invoke_initialization_script: String,
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
  /// The `Content-Security-Policy` header value.
  pub csp_header: Option<String>,
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
    (invoke_responder, invoke_initialization_script): (Arc<InvokeResponder<R>>, String),
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
        invoke_responder,
        invoke_initialization_script,
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

  /// The invoke responder.
  pub(crate) fn invoke_responder(&self) -> Arc<InvokeResponder<R>> {
    self.inner.invoke_responder.clone()
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

  fn csp(&self) -> Option<String> {
    if cfg!(feature = "custom-protocol") {
      self.inner.config.tauri.security.csp.clone()
    } else {
      self
        .inner
        .config
        .tauri
        .security
        .dev_csp
        .clone()
        .or_else(|| self.inner.config.tauri.security.csp.clone())
    }
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
    webview_attributes =
      webview_attributes.initialization_script(&self.inner.invoke_initialization_script);
    if is_init_global {
      webview_attributes = webview_attributes.initialization_script(&format!(
        "(function () {{
        const __TAURI_INVOKE_KEY__ = {key};
        {bundle_script}
        }})()",
        key = self.generate_invoke_key(),
        bundle_script = include_str!("../scripts/bundle.js"),
      ));
    }
    webview_attributes = webview_attributes
      .initialization_script(&format!(
        r#"
          if (!window.__TAURI__) {{
            window.__TAURI__ = {{}}
          }}
          window.__TAURI__.__windows = {window_labels_array}.map(function (label) {{ return {{ label: label }} }});
          window.__TAURI__.__currentWindow = {{ label: {current_window_label} }}
        "#,
        window_labels_array = serde_json::to_string(pending_labels)?,
        current_window_label = serde_json::to_string(&label)?,
      ))
      .initialization_script(&self.initialization_script(&plugin_init));

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

    #[cfg(protocol_asset)]
    if !registered_scheme_protocols.contains(&"asset".into()) {
      use tokio::io::{AsyncReadExt, AsyncSeekExt};
      use url::Position;
      let asset_scope = self.state().get::<crate::Scopes>().asset_protocol.clone();
      let window_url = Url::parse(&pending.url).unwrap();
      let window_origin =
        if cfg!(windows) && window_url.scheme() != "http" && window_url.scheme() != "https" {
          format!("https://{}.localhost", window_url.scheme())
        } else {
          format!(
            "{}://{}{}",
            window_url.scheme(),
            window_url.host().unwrap(),
            if let Some(port) = window_url.port() {
              format!(":{}", port)
            } else {
              "".into()
            }
          )
        };
      pending.register_uri_scheme_protocol("asset", move |request| {
        let parsed_path = Url::parse(request.uri())?;
        let filtered_path = &parsed_path[..Position::AfterPath];
        #[cfg(target_os = "windows")]
        let path = filtered_path.replace("asset://localhost/", "");
        #[cfg(not(target_os = "windows"))]
        let path = filtered_path.replace("asset://", "");
        let path = percent_encoding::percent_decode(path.as_bytes())
          .decode_utf8_lossy()
          .to_string();

        if !asset_scope.is_allowed(&path) {
          return HttpResponseBuilder::new()
            .status(403)
            .body(Vec::with_capacity(0));
        }

        let path_for_data = path.clone();

        let mut response =
          HttpResponseBuilder::new().header("Access-Control-Allow-Origin", &window_origin);

        // handle 206 (partial range) http request
        if let Some(range) = request.headers().get("range").cloned() {
          let mut status_code = 200;
          let path_for_data = path_for_data.clone();
          let (headers, status_code, data) = crate::async_runtime::safe_block_on(async move {
            let mut headers = HashMap::new();
            let mut buf = Vec::new();
            let mut file = tokio::fs::File::open(path_for_data.clone()).await.unwrap();
            // Get the file size
            let file_size = file.metadata().await.unwrap().len();
            // parse the range
            let range =
              crate::runtime::http::HttpRange::parse(range.to_str().unwrap(), file_size).unwrap();

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
                real_length = std::cmp::min(file_size - range.start, 1024 * 400);
              }

              // last byte we are reading, the length of the range include the last byte
              // who should be skipped on the header
              let last_byte = range.start + real_length - 1;
              // partial content
              status_code = 206;

              headers.insert("Connection", "Keep-Alive".into());
              headers.insert("Accept-Ranges", "bytes".into());
              headers.insert("Content-Length", real_length.to_string());
              headers.insert(
                "Content-Range",
                format!("bytes {}-{}/{}", range.start, last_byte, file_size),
              );

              file
                .seek(std::io::SeekFrom::Start(range.start))
                .await
                .unwrap();
              file.take(real_length).read_to_end(&mut buf).await.unwrap();
            }

            (headers, status_code, buf)
          });

          for (k, v) in headers {
            response = response.header(k, v);
          }

          if !data.is_empty() {
            let mime_type = MimeType::parse(&data, &path);
            return response.mimetype(&mime_type).status(status_code).body(data);
          }
        }

        if let Ok(data) =
          crate::async_runtime::safe_block_on(async move { tokio::fs::read(path_for_data).await })
        {
          let mime_type = MimeType::parse(&data, &path);
          response.mimetype(&mime_type).body(data)
        } else {
          response.status(404).body(Vec::new())
        }
      });
    }

    Ok(pending)
  }

  fn prepare_ipc_handler(&self, app_handle: AppHandle<R>) -> WebviewIpcHandler<R> {
    let manager = self.clone();
    Box::new(move |window, request| {
      let window = Window::new(manager.clone(), window, app_handle.clone());

      match serde_json::from_str::<InvokePayload>(&request) {
        Ok(message) => {
          let _ = window.on_message(message);
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

    let mut asset_path = AssetKey::from(path.as_str());

    let asset_response = assets
      .get(&path.as_str().into())
      .or_else(|| {
        let fallback = format!("{}/index.html", path.as_str()).into();
        let asset = assets.get(&fallback);
        asset_path = fallback;
        asset
      })
      .or_else(|| {
        #[cfg(debug_assertions)]
        eprintln!("Asset `{}` not found; fallback to index.html", path); // TODO log::error!
        let fallback = AssetKey::from("index.html");
        let asset = assets.get(&fallback);
        asset_path = fallback;
        asset
      })
      .ok_or_else(|| crate::Error::AssetNotFound(path.clone()))
      .map(Cow::into_owned);

    let mut csp_header = None;

    match asset_response {
      Ok(asset) => {
        let final_data = if is_javascript || is_html {
          let mut asset = String::from_utf8_lossy(&asset).into_owned();
          asset = asset.replacen(INVOKE_KEY_TOKEN, &self.generate_invoke_key().to_string(), 1);

          if is_html {
            if let Some(mut csp) = self.csp() {
              let hash_strings = self.inner.assets.csp_hashes(&asset_path).fold(
                CspHashStrings::default(),
                |mut acc, hash| {
                  match hash {
                    CspHash::Script(hash) => {
                      acc.script.push(' ');
                      acc.script.push_str(hash);
                    }
                    csp_hash => {
                      #[cfg(debug_assertions)]
                      eprintln!("Unknown CspHash variant encountered: {:?}", csp_hash)
                    }
                  }

                  acc
                },
              );

              replace_csp_nonce(
                &mut asset,
                SCRIPT_NONCE_TOKEN,
                &mut csp,
                "script-src",
                hash_strings.script,
              );
              replace_csp_nonce(
                &mut asset,
                STYLE_NONCE_TOKEN,
                &mut csp,
                "style-src",
                hash_strings.style,
              );

              asset = asset.replace(CSP_TOKEN, &csp);
              csp_header.replace(csp);
            }
          }

          asset.as_bytes().to_vec()
        } else {
          asset
        };
        let mime_type = MimeType::parse(&final_data, &path);
        Ok(Asset {
          bytes: final_data.to_vec(),
          mime_type,
          csp_header,
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
        // ignore query string and fragment
        .next()
        .unwrap()
        .to_string()
        .replace("tauri://localhost", "");
      let asset = manager.get_asset(path)?;
      let mut response = HttpResponseBuilder::new().mimetype(&asset.mime_type);
      if let Some(csp) = asset.csp_header {
        response = response.header("Content-Security-Policy", csp);
      }
      response.body(asset.bytes)
    })
  }

  fn prepare_file_drop(&self, app_handle: AppHandle<R>) -> FileDropHandler<R> {
    let manager = self.clone();
    Box::new(move |event, window| {
      let window = Window::new(manager.clone(), window, app_handle.clone());
      let _ = match event {
        FileDropEvent::Hovered(paths) => window.emit_and_trigger("tauri://file-drop-hover", paths),
        FileDropEvent::Dropped(paths) => window.emit_and_trigger("tauri://file-drop", paths),
        FileDropEvent::Cancelled => window.emit_and_trigger("tauri://file-drop-cancelled", ()),
        _ => unimplemented!(),
      };
      true
    })
  }

  fn initialization_script(&self, plugin_initialization_script: &str) -> String {
    let key = self.generate_invoke_key();
    format!(
      r#"
      {core_script}
      {event_initialization_script}
      if (document.readyState === 'complete') {{
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
      (std::sync::Arc::new(|_, _, _, _| ()), "".into()),
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
    let (is_local, mut url) = match &pending.webview_attributes.url {
      WindowUrl::App(path) => {
        let url = self.get_url();
        (
          true,
          // ignore "index.html" just to simplify the url
          if path.to_str() != Some("index.html") {
            url
              .join(&*path.to_string_lossy())
              .map_err(crate::Error::InvalidUrl)
              // this will never fail
              .unwrap()
          } else {
            url.into_owned()
          },
        )
      }
      WindowUrl::External(url) => (url.scheme() == "tauri", url.clone()),
      _ => unimplemented!(),
    };

    if let Some(csp) = self.csp() {
      if url.scheme() == "data" {
        if let Ok(data_url) = data_url::DataUrl::process(url.as_str()) {
          let (body, _) = data_url.decode_to_vec().unwrap();
          let html = String::from_utf8_lossy(&body).into_owned();
          // naive way to check if it's an html
          if html.contains('<') && html.contains('>') {
            let mut document = parse_html(html);
            inject_csp(&mut document, &csp);
            url.set_path(&format!("text/html,{}", document.to_string()));
          }
        }
      }
    }

    if is_local {
      let label = pending.label.clone();
      pending = self.prepare_pending_window(pending, &label, pending_labels, app_handle.clone())?;
      pending.ipc_handler = Some(self.prepare_ipc_handler(app_handle.clone()));
    }

    if pending.webview_attributes.file_drop_handler_enabled {
      pending.file_drop_handler = Some(self.prepare_file_drop(app_handle));
    }

    pending.url = url.to_string();

    // in `Windows`, we need to force a data_directory
    // but we do respect user-specification
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    if pending.webview_attributes.data_directory.is_none() {
      let local_app_data = resolve_path(
        &self.inner.config,
        &self.inner.package_info,
        self.inner.state.get::<crate::Env>().inner(),
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
    assert!(is_event_name_valid(event));
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
    assert!(is_event_name_valid(event));
    self.inner.listeners.trigger(event, window, data)
  }

  pub fn listen<F: Fn(Event) + Send + 'static>(
    &self,
    event: String,
    window: Option<String>,
    handler: F,
  ) -> EventHandler {
    assert!(is_event_name_valid(&event));
    self.inner.listeners.listen(event, window, handler)
  }

  pub fn once<F: Fn(Event) + Send + 'static>(
    &self,
    event: String,
    window: Option<String>,
    handler: F,
  ) -> EventHandler {
    assert!(is_event_name_valid(&event));
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
    WindowEvent::Resized(size) => window.emit_and_trigger(WINDOW_RESIZED_EVENT, size)?,
    WindowEvent::Moved(position) => window.emit_and_trigger(WINDOW_MOVED_EVENT, position)?,
    WindowEvent::CloseRequested {
      label: _,
      signal_tx,
    } => {
      if window.has_js_listener(WINDOW_CLOSE_REQUESTED_EVENT) {
        signal_tx.send(true).unwrap();
      }
      window.emit_and_trigger(WINDOW_CLOSE_REQUESTED_EVENT, ())?;
    }
    WindowEvent::Destroyed => {
      window.emit_and_trigger(WINDOW_DESTROYED_EVENT, ())?;
      let label = window.label();
      for window in manager.inner.windows.lock().unwrap().values() {
        window.eval(&format!(
          r#"window.__TAURI__.__windows = window.__TAURI__.__windows.filter(w => w.label !== "{}");"#,
          label
        ))?;
      }
    }
    WindowEvent::Focused(focused) => window.emit_and_trigger(
      if *focused {
        WINDOW_FOCUS_EVENT
      } else {
        WINDOW_BLUR_EVENT
      },
      (),
    )?,
    WindowEvent::ScaleFactorChanged {
      scale_factor,
      new_inner_size,
      ..
    } => window.emit_and_trigger(
      WINDOW_SCALE_FACTOR_CHANGED_EVENT,
      ScaleFactorChanged {
        scale_factor: *scale_factor,
        size: *new_inner_size,
      },
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
  window.emit_and_trigger(MENU_EVENT, event.menu_item_id.clone())
}
