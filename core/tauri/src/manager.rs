// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  fmt,
  fs::create_dir_all,
  sync::{Arc, Mutex, MutexGuard},
};

use serde::Serialize;
use serde_json::Value as JsonValue;
use serialize_to_javascript::{default_template, DefaultTemplate, Template};
use url::Url;

use tauri_macros::default_runtime;
use tauri_utils::debug_eprintln;
#[cfg(feature = "isolation")]
use tauri_utils::pattern::isolation::RawIsolationPayload;
use tauri_utils::{
  assets::{AssetKey, CspHash},
  config::{Csp, CspDirectiveSources},
  html::{SCRIPT_NONCE_TOKEN, STYLE_NONCE_TOKEN},
};

use crate::hooks::IpcJavascript;
#[cfg(feature = "isolation")]
use crate::hooks::IsolationJavascript;
use crate::pattern::PatternJavascript;
use crate::{
  app::{AppHandle, GlobalWindowEvent, GlobalWindowEventListener},
  event::{assert_event_name_is_valid, Event, EventHandler, Listeners},
  hooks::{InvokeHandler, InvokePayload, InvokeResponder, OnPageLoad, PageLoadPayload},
  plugin::PluginStore,
  runtime::{
    http::{
      MimeType, Request as HttpRequest, Response as HttpResponse,
      ResponseBuilder as HttpResponseBuilder,
    },
    webview::{WebviewIpcHandler, WindowBuilder},
    window::{dpi::PhysicalSize, DetachedWindow, FileDropEvent, PendingWindow},
  },
  utils::{
    assets::Assets,
    config::{AppUrl, Config, WindowUrl},
    PackageInfo,
  },
  Context, EventLoopMessage, Icon, Invoke, Manager, Pattern, Runtime, Scopes, StateManager, Window,
  WindowEvent,
};
use crate::{
  app::{GlobalMenuEventListener, WindowMenuEvent},
  window::WindowEmitArgs,
};

#[cfg(any(target_os = "linux", target_os = "windows"))]
use crate::api::path::{resolve_path, BaseDirectory};

use crate::{runtime::menu::Menu, MenuEvent};

const WINDOW_RESIZED_EVENT: &str = "tauri://resize";
const WINDOW_MOVED_EVENT: &str = "tauri://move";
const WINDOW_CLOSE_REQUESTED_EVENT: &str = "tauri://close-requested";
const WINDOW_DESTROYED_EVENT: &str = "tauri://destroyed";
const WINDOW_FOCUS_EVENT: &str = "tauri://focus";
const WINDOW_BLUR_EVENT: &str = "tauri://blur";
const WINDOW_SCALE_FACTOR_CHANGED_EVENT: &str = "tauri://scale-change";
const WINDOW_THEME_CHANGED: &str = "tauri://theme-changed";
const WINDOW_FILE_DROP_EVENT: &str = "tauri://file-drop";
const WINDOW_FILE_DROP_HOVER_EVENT: &str = "tauri://file-drop-hover";
const WINDOW_FILE_DROP_CANCELLED_EVENT: &str = "tauri://file-drop-cancelled";
const MENU_EVENT: &str = "tauri://menu";

pub(crate) const STRINGIFY_IPC_MESSAGE_FN: &str =
  include_str!("../scripts/stringify-ipc-message-fn.js");

#[derive(Default)]
/// Spaced and quoted Content-Security-Policy hash values.
struct CspHashStrings {
  script: Vec<String>,
  style: Vec<String>,
}

/// Sets the CSP value to the asset HTML if needed (on Linux).
/// Returns the CSP string for access on the response header (on Windows and macOS).
fn set_csp<R: Runtime>(
  asset: &mut String,
  assets: Arc<dyn Assets>,
  asset_path: &AssetKey,
  manager: &WindowManager<R>,
  csp: Csp,
) -> String {
  let mut csp = csp.into();
  let hash_strings =
    assets
      .csp_hashes(asset_path)
      .fold(CspHashStrings::default(), |mut acc, hash| {
        match hash {
          CspHash::Script(hash) => {
            acc.script.push(hash.into());
          }
          CspHash::Style(hash) => {
            acc.style.push(hash.into());
          }
          _csp_hash => {
            debug_eprintln!("Unknown CspHash variant encountered: {:?}", _csp_hash);
          }
        }

        acc
      });

  let dangerous_disable_asset_csp_modification = &manager
    .config()
    .tauri
    .security
    .dangerous_disable_asset_csp_modification;
  if dangerous_disable_asset_csp_modification.can_modify("script-src") {
    replace_csp_nonce(
      asset,
      SCRIPT_NONCE_TOKEN,
      &mut csp,
      "script-src",
      hash_strings.script,
    );
  }

  if dangerous_disable_asset_csp_modification.can_modify("style-src") {
    replace_csp_nonce(
      asset,
      STYLE_NONCE_TOKEN,
      &mut csp,
      "style-src",
      hash_strings.style,
    );
  }

  #[cfg(feature = "isolation")]
  if let Pattern::Isolation { schema, .. } = &manager.inner.pattern {
    let default_src = csp
      .entry("default-src".into())
      .or_insert_with(Default::default);
    default_src.push(crate::pattern::format_real_schema(
      schema,
      manager.config().tauri.security.dangerous_use_http_scheme,
    ));
  }

  Csp::DirectiveMap(csp).to_string()
}

#[cfg(target_os = "linux")]
fn set_html_csp(html: &str, csp: &str) -> String {
  html.replacen(tauri_utils::html::CSP_TOKEN, csp, 1)
}

// inspired by https://github.com/rust-lang/rust/blob/1be5c8f90912c446ecbdc405cbc4a89f9acd20fd/library/alloc/src/str.rs#L260-L297
fn replace_with_callback<F: FnMut() -> String>(
  original: &str,
  pattern: &str,
  mut replacement: F,
) -> String {
  let mut result = String::new();
  let mut last_end = 0;
  for (start, part) in original.match_indices(pattern) {
    result.push_str(unsafe { original.get_unchecked(last_end..start) });
    result.push_str(&replacement());
    last_end = start + part.len();
  }
  result.push_str(unsafe { original.get_unchecked(last_end..original.len()) });
  result
}

fn replace_csp_nonce(
  asset: &mut String,
  token: &str,
  csp: &mut HashMap<String, CspDirectiveSources>,
  directive: &str,
  hashes: Vec<String>,
) {
  let mut nonces = Vec::new();
  *asset = replace_with_callback(asset, token, || {
    let nonce = rand::random::<usize>();
    nonces.push(nonce);
    nonce.to_string()
  });

  if !(nonces.is_empty() && hashes.is_empty()) {
    let nonce_sources = nonces
      .into_iter()
      .map(|n| format!("'nonce-{n}'"))
      .collect::<Vec<String>>();
    let sources = csp.entry(directive.into()).or_default();
    let self_source = "'self'".to_string();
    if !sources.contains(&self_source) {
      sources.push(self_source);
    }
    sources.extend(nonce_sources);
    sources.extend(hashes);
  }
}

#[default_runtime(crate::Wry, wry)]
pub struct InnerWindowManager<R: Runtime> {
  windows: Mutex<HashMap<String, Window<R>>>,
  #[cfg(all(desktop, feature = "system-tray"))]
  pub(crate) trays: Mutex<HashMap<String, crate::SystemTrayHandle<R>>>,
  pub(crate) plugins: Mutex<PluginStore<R>>,
  listeners: Listeners,
  pub(crate) state: Arc<StateManager>,

  /// The JS message handler.
  invoke_handler: Box<InvokeHandler<R>>,

  /// The page load hook, invoked when the webview performs a navigation.
  on_page_load: Box<OnPageLoad<R>>,

  config: Arc<Config>,
  #[cfg(dev)]
  config_parent: Option<std::path::PathBuf>,
  assets: Arc<dyn Assets>,
  pub(crate) default_window_icon: Option<Icon>,
  pub(crate) app_icon: Option<Vec<u8>>,
  pub(crate) tray_icon: Option<Icon>,

  package_info: PackageInfo,
  /// The webview protocols available to all windows.
  uri_scheme_protocols: Mutex<HashMap<String, Arc<CustomProtocol<R>>>>,
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
  /// Application pattern.
  pub(crate) pattern: Pattern,
  /// A runtime generated key to ensure an IPC call comes from an initialized frame.
  invoke_key: String,
}

impl<R: Runtime> fmt::Debug for InnerWindowManager<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("InnerWindowManager")
      .field("plugins", &self.plugins)
      .field("state", &self.state)
      .field("config", &self.config)
      .field("default_window_icon", &self.default_window_icon)
      .field("app_icon", &self.app_icon)
      .field("tray_icon", &self.tray_icon)
      .field("package_info", &self.package_info)
      .field("menu", &self.menu)
      .field("pattern", &self.pattern)
      .field("invoke_key", &self.invoke_key)
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
}

impl<R: Runtime> Clone for WindowManager<R> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

impl<R: Runtime> WindowManager<R> {
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn with_handlers(
    #[allow(unused_mut)] mut context: Context<impl Assets>,
    plugins: PluginStore<R>,
    invoke_handler: Box<InvokeHandler<R>>,
    on_page_load: Box<OnPageLoad<R>>,
    uri_scheme_protocols: HashMap<String, Arc<CustomProtocol<R>>>,
    state: StateManager,
    window_event_listeners: Vec<GlobalWindowEventListener<R>>,
    (menu, menu_event_listeners): (Option<Menu>, Vec<GlobalMenuEventListener<R>>),
    (invoke_responder, invoke_initialization_script): (Arc<InvokeResponder<R>>, String),
    invoke_key: String,
  ) -> Self {
    // generate a random isolation key at runtime
    #[cfg(feature = "isolation")]
    if let Pattern::Isolation { ref mut key, .. } = &mut context.pattern {
      *key = uuid::Uuid::new_v4().to_string();
    }

    Self {
      inner: Arc::new(InnerWindowManager {
        windows: Mutex::default(),
        #[cfg(all(desktop, feature = "system-tray"))]
        trays: Default::default(),
        plugins: Mutex::new(plugins),
        listeners: Listeners::default(),
        state: Arc::new(state),
        invoke_handler,
        on_page_load,
        config: Arc::new(context.config),
        #[cfg(dev)]
        config_parent: context.config_parent,
        assets: context.assets,
        default_window_icon: context.default_window_icon,
        app_icon: context.app_icon,
        tray_icon: context.system_tray_icon,
        package_info: context.package_info,
        pattern: context.pattern,
        uri_scheme_protocols: Mutex::new(uri_scheme_protocols),
        menu,
        menu_event_listeners: Arc::new(menu_event_listeners),
        window_event_listeners: Arc::new(window_event_listeners),
        invoke_responder,
        invoke_initialization_script,
        invoke_key,
      }),
    }
  }

  pub(crate) fn pattern(&self) -> &Pattern {
    &self.inner.pattern
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

  pub(crate) fn register_uri_scheme_protocol<N: Into<String>>(
    &self,
    uri_scheme: N,
    protocol: Arc<CustomProtocol<R>>,
  ) {
    let uri_scheme = uri_scheme.into();
    self
      .inner
      .uri_scheme_protocols
      .lock()
      .unwrap()
      .insert(uri_scheme, protocol);
  }

  /// Get the base path to serve data from.
  ///
  /// * In dev mode, this will be based on the `devPath` configuration value.
  /// * Otherwise, this will be based on the `distDir` configuration value.
  #[cfg(not(dev))]
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
  pub(crate) fn get_url(&self) -> Cow<'_, Url> {
    match self.base_path() {
      AppUrl::Url(WindowUrl::External(url)) => Cow::Borrowed(url),
      #[cfg(windows)]
      _ => {
        let scheme = if self.inner.config.tauri.security.dangerous_use_http_scheme {
          "http"
        } else {
          "https"
        };
        Cow::Owned(Url::parse(&format!("{scheme}://tauri.localhost")).unwrap())
      }
      #[cfg(not(windows))]
      _ => Cow::Owned(Url::parse("tauri://localhost").unwrap()),
    }
  }

  fn csp(&self) -> Option<Csp> {
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

  fn prepare_pending_window(
    &self,
    mut pending: PendingWindow<EventLoopMessage, R>,
    label: &str,
    window_labels: &[String],
    app_handle: AppHandle<R>,
  ) -> crate::Result<PendingWindow<EventLoopMessage, R>> {
    let is_init_global = self.inner.config.build.with_global_tauri;
    let plugin_init_scripts = self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .initialization_script();

    let pattern_init = PatternJavascript {
      pattern: self.pattern().into(),
    }
    .render_default(&Default::default())?;

    let mut webview_attributes = pending.webview_attributes;

    let ipc_init = IpcJavascript {
      isolation_origin: &match self.pattern() {
        #[cfg(feature = "isolation")]
        Pattern::Isolation { schema, .. } => {
          crate::pattern::format_real_schema(schema, pending.http_scheme)
        }
        _ => "".to_string(),
      },
      invoke_key: self.invoke_key(),
      os_name: std::env::consts::OS,
    }
    .render_default(&Default::default())?;

    let mut window_labels = window_labels.to_vec();
    let l = label.to_string();
    if !window_labels.contains(&l) {
      window_labels.push(l);
    }
    webview_attributes = webview_attributes
      .initialization_script(&self.inner.invoke_initialization_script)
      .initialization_script(&format!(
        r#"
          Object.defineProperty(window, '__TAURI_METADATA__', {{
            value: {{
              __windows: {window_labels_array}.map(function (label) {{ return {{ label: label }} }}),
              __currentWindow: {{ label: {current_window_label} }}
            }}
          }})
        "#,
        window_labels_array = serde_json::to_string(&window_labels)?,
        current_window_label = serde_json::to_string(&label)?,
      ))
      .initialization_script(&self.initialization_script(&ipc_init.into_string(),&pattern_init.into_string(),is_init_global)?);

    for plugin_init_script in plugin_init_scripts {
      webview_attributes = webview_attributes.initialization_script(&plugin_init_script);
    }

    #[cfg(feature = "isolation")]
    if let Pattern::Isolation { schema, .. } = self.pattern() {
      webview_attributes = webview_attributes.initialization_script(
        &IsolationJavascript {
          isolation_src: &crate::pattern::format_real_schema(schema, pending.http_scheme),
          style: tauri_utils::pattern::isolation::IFRAME_STYLE,
        }
        .render_default(&Default::default())?
        .into_string(),
      );
    }

    pending.webview_attributes = webview_attributes;

    let mut registered_scheme_protocols = Vec::new();

    for (uri_scheme, protocol) in &*self.inner.uri_scheme_protocols.lock().unwrap() {
      registered_scheme_protocols.push(uri_scheme.clone());
      let protocol = protocol.clone();
      let app_handle = Mutex::new(app_handle.clone());
      pending.register_uri_scheme_protocol(uri_scheme.clone(), move |p| {
        (protocol.protocol)(&app_handle.lock().unwrap(), p)
      });
    }

    let window_url = Url::parse(&pending.url).unwrap();
    let window_origin = if window_url.scheme() == "data" {
      "null".into()
    } else if cfg!(windows) && window_url.scheme() != "http" && window_url.scheme() != "https" {
      let scheme = if pending.http_scheme { "http" } else { "https" };
      format!("{scheme}://{}.localhost", window_url.scheme())
    } else {
      format!(
        "{}://{}{}",
        window_url.scheme(),
        window_url.host().unwrap(),
        window_url
          .port()
          .map(|p| format!(":{p}"))
          .unwrap_or_default()
      )
    };

    if !registered_scheme_protocols.contains(&"tauri".into()) {
      let web_resource_request_handler = pending.web_resource_request_handler.take();
      pending.register_uri_scheme_protocol(
        "tauri",
        self.prepare_uri_scheme_protocol(&window_origin, web_resource_request_handler),
      );
      registered_scheme_protocols.push("tauri".into());
    }

    #[cfg(protocol_asset)]
    if !registered_scheme_protocols.contains(&"asset".into()) {
      let asset_scope = self.state().get::<crate::Scopes>().asset_protocol.clone();
      pending.register_uri_scheme_protocol("asset", move |request| {
        crate::asset_protocol::asset_protocol_handler(
          request,
          asset_scope.clone(),
          window_origin.clone(),
        )
      });
    }

    #[cfg(feature = "isolation")]
    if let Pattern::Isolation {
      assets,
      schema,
      key: _,
      crypto_keys,
    } = &self.inner.pattern
    {
      let assets = assets.clone();
      let schema_ = schema.clone();
      let url_base = format!("{schema_}://localhost");
      let aes_gcm_key = *crypto_keys.aes_gcm().raw();

      pending.register_uri_scheme_protocol(schema, move |request| {
        match request_to_path(request, &url_base).as_str() {
          "index.html" => match assets.get(&"index.html".into()) {
            Some(asset) => {
              let asset = String::from_utf8_lossy(asset.as_ref());
              let template = tauri_utils::pattern::isolation::IsolationJavascriptRuntime {
                runtime_aes_gcm_key: &aes_gcm_key,
                stringify_ipc_message_fn: STRINGIFY_IPC_MESSAGE_FN,
              };
              match template.render(asset.as_ref(), &Default::default()) {
                Ok(asset) => HttpResponseBuilder::new()
                  .mimetype("text/html")
                  .body(asset.into_string().as_bytes().to_vec()),
                Err(_) => HttpResponseBuilder::new()
                  .status(500)
                  .mimetype("text/plain")
                  .body(Vec::new()),
              }
            }

            None => HttpResponseBuilder::new()
              .status(404)
              .mimetype("text/plain")
              .body(Vec::new()),
          },
          _ => HttpResponseBuilder::new()
            .status(404)
            .mimetype("text/plain")
            .body(Vec::new()),
        }
      });
    }

    Ok(pending)
  }

  fn prepare_ipc_handler(
    &self,
    app_handle: AppHandle<R>,
  ) -> WebviewIpcHandler<EventLoopMessage, R> {
    let manager = self.clone();
    Box::new(move |window, #[allow(unused_mut)] mut request| {
      #[cfg(feature = "tracing")]
      let _span =
        tracing::trace_span!("ipc::request", kind = "post-message", request = request).entered();

      let window = Window::new(manager.clone(), window, app_handle.clone());

      #[cfg(feature = "isolation")]
      if let Pattern::Isolation { crypto_keys, .. } = manager.pattern() {
        match RawIsolationPayload::try_from(request.as_str())
          .and_then(|raw| crypto_keys.decrypt(raw))
        {
          Ok(json) => request = json,
          Err(e) => {
            let error: crate::Error = e.into();
            let _ = window.eval(&format!(
              r#"console.error({})"#,
              JsonValue::String(error.to_string())
            ));
            return;
          }
        }
      }

      match serde_json::from_str::<InvokePayload>(&request) {
        Ok(message) => {
          #[cfg(feature = "tracing")]
          let _span = tracing::trace_span!("ipc::request::handle", cmd = message.cmd).entered();

          let _ = window.on_message(message);
        }
        Err(e) => {
          #[cfg(feature = "tracing")]
          tracing::trace!("ipc::request::error {}", e);
          let error: crate::Error = e.into();
          let _ = window.eval(&format!(
            r#"console.error({})"#,
            JsonValue::String(error.to_string())
          ));
        }
      }
    })
  }

  pub fn asset_iter(&self) -> Box<dyn Iterator<Item = (&&str, &&[u8])> + '_> {
    self.inner.assets.iter()
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

    let mut asset_path = AssetKey::from(path.as_str());

    let asset_response = assets
      .get(&path.as_str().into())
      .or_else(|| {
        eprintln!("Asset `{path}` not found; fallback to {path}.html");
        let fallback = format!("{}.html", path.as_str()).into();
        let asset = assets.get(&fallback);
        asset_path = fallback;
        asset
      })
      .or_else(|| {
        debug_eprintln!(
          "Asset `{}` not found; fallback to {}/index.html",
          path,
          path
        );
        let fallback = format!("{}/index.html", path.as_str()).into();
        let asset = assets.get(&fallback);
        asset_path = fallback;
        asset
      })
      .or_else(|| {
        debug_eprintln!("Asset `{}` not found; fallback to index.html", path);
        let fallback = AssetKey::from("index.html");
        let asset = assets.get(&fallback);
        asset_path = fallback;
        asset
      })
      .ok_or_else(|| crate::Error::AssetNotFound(path.clone()))
      .map(Cow::into_owned);

    let mut csp_header = None;
    let is_html = asset_path.as_ref().ends_with(".html");

    match asset_response {
      Ok(asset) => {
        let final_data = if is_html {
          let mut asset = String::from_utf8_lossy(&asset).into_owned();
          if let Some(csp) = self.csp() {
            csp_header.replace(set_csp(
              &mut asset,
              self.inner.assets.clone(),
              &asset_path,
              self,
              csp,
            ));
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
        debug_eprintln!("{:?}", e); // TODO log::error!
        Err(Box::new(e))
      }
    }
  }

  #[allow(clippy::type_complexity)]
  fn prepare_uri_scheme_protocol(
    &self,
    window_origin: &str,
    web_resource_request_handler: Option<
      Box<dyn Fn(&HttpRequest, &mut HttpResponse) + Send + Sync>,
    >,
  ) -> Box<dyn Fn(&HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> + Send + Sync>
  {
    let manager = self.clone();
    let window_origin = window_origin.to_string();
    Box::new(move |request| {
      let path = request
        .uri()
        .split(&['?', '#'][..])
        // ignore query string and fragment
        .next()
        .unwrap()
        .strip_prefix("tauri://localhost")
        .map(|p| p.to_string())
        // the `strip_prefix` only returns None when a request is made to `https://tauri.$P` on Windows
        // where `$P` is not `localhost/*`
        .unwrap_or_else(|| "".to_string());
      let asset = manager.get_asset(path)?;
      let mut builder = HttpResponseBuilder::new()
        .header("Access-Control-Allow-Origin", &window_origin)
        .mimetype(&asset.mime_type);
      if let Some(csp) = &asset.csp_header {
        builder = builder.header("Content-Security-Policy", csp);
      }
      let mut response = builder.body(asset.bytes)?;
      if let Some(handler) = &web_resource_request_handler {
        handler(request, &mut response);

        // if it's an HTML file, we need to set the CSP meta tag on Linux
        #[cfg(target_os = "linux")]
        if let Some(response_csp) = response.headers().get("Content-Security-Policy") {
          let response_csp = String::from_utf8_lossy(response_csp.as_bytes());
          let body = set_html_csp(&String::from_utf8_lossy(response.body()), &response_csp);
          *response.body_mut() = body.as_bytes().to_vec();
        }
      } else {
        #[cfg(target_os = "linux")]
        {
          if let Some(csp) = &asset.csp_header {
            let body = set_html_csp(&String::from_utf8_lossy(response.body()), csp);
            *response.body_mut() = body.as_bytes().to_vec();
          }
        }
      }
      Ok(response)
    })
  }

  fn initialization_script(
    &self,
    ipc_script: &str,
    pattern_script: &str,
    with_global_tauri: bool,
  ) -> crate::Result<String> {
    #[derive(Template)]
    #[default_template("../scripts/init.js")]
    struct InitJavascript<'a> {
      #[raw]
      pattern_script: &'a str,
      #[raw]
      ipc_script: &'a str,
      #[raw]
      bundle_script: &'a str,
      // A function to immediately listen to an event.
      #[raw]
      listen_function: &'a str,
      #[raw]
      core_script: &'a str,
      #[raw]
      event_initialization_script: &'a str,
      #[raw]
      freeze_prototype: &'a str,
      #[raw]
      hotkeys: &'a str,
    }

    #[derive(Template)]
    #[default_template("../scripts/core.js")]
    struct CoreJavascript<'a> {
      os_name: &'a str,
      protocol_scheme: &'a str,
    }

    #[derive(Template)]
    #[default_template("../scripts/toggle-devtools.js")]
    struct ToggleDevtoolsScript<'a> {
      os_name: &'a str,
    }

    let bundle_script = if with_global_tauri {
      include_str!("../scripts/bundle.global.js")
    } else {
      ""
    };

    let freeze_prototype = if self.inner.config.tauri.security.freeze_prototype {
      include_str!("../scripts/freeze_prototype.js")
    } else {
      ""
    };

    #[cfg(any(debug_assertions, feature = "devtools"))]
    let hotkeys = ToggleDevtoolsScript {
      os_name: std::env::consts::OS,
    }
    .render_default(&Default::default())?
    .into_string();
    #[cfg(not(any(debug_assertions, feature = "devtools")))]
    let hotkeys = String::default();

    InitJavascript {
      pattern_script,
      ipc_script,
      bundle_script,
      listen_function: &format!(
        "function listen(eventName, cb) {{ {} }}",
        crate::event::listen_js(
          self.event_listeners_object_name(),
          "eventName".into(),
          0,
          None,
          "window['_' + window.__TAURI__.transformCallback(cb) ]".into()
        )
      ),
      core_script: &CoreJavascript {
        os_name: std::env::consts::OS,
        protocol_scheme: if self.inner.config.tauri.security.dangerous_use_http_scheme {
          "http"
        } else {
          "https"
        },
      }
      .render_default(&Default::default())?
      .into_string(),
      event_initialization_script: &self.event_initialization_script(),
      freeze_prototype,
      hotkeys: &hotkeys,
    }
    .render_default(&Default::default())
    .map(|s| s.into_string())
    .map_err(Into::into)
  }

  fn event_initialization_script(&self) -> String {
    format!(
      "
      Object.defineProperty(window, '{function}', {{
        value: function (eventData) {{
          const listeners = (window['{listeners}'] && window['{listeners}'][eventData.event]) || []

          for (let i = listeners.length - 1; i >= 0; i--) {{
            const listener = listeners[i]
            if (listener.windowLabel === null || eventData.windowLabel === null || listener.windowLabel === eventData.windowLabel) {{
              eventData.id = listener.id
              listener.handler(eventData)
            }}
          }}
        }}
      }});
    ",
      function = self.event_emit_function_name(),
      listeners = self.event_listeners_object_name()
    )
  }

  pub(crate) fn invoke_key(&self) -> &str {
    &self.inner.invoke_key
  }
}

#[cfg(test)]
mod test {
  use crate::{generate_context, plugin::PluginStore, StateManager, Wry};

  use super::WindowManager;

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
      crate::generate_invoke_key().unwrap(),
    );

    #[cfg(custom_protocol)]
    {
      assert_eq!(
        manager.get_url().to_string(),
        if cfg!(windows) {
          "https://tauri.localhost/"
        } else {
          "tauri://localhost"
        }
      );
    }

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
      .initialize_all(app, &self.inner.config.plugins)
  }

  pub fn prepare_window(
    &self,
    app_handle: AppHandle<R>,
    mut pending: PendingWindow<EventLoopMessage, R>,
    window_labels: &[String],
  ) -> crate::Result<PendingWindow<EventLoopMessage, R>> {
    if self.windows_lock().contains_key(&pending.label) {
      return Err(crate::Error::WindowLabelAlreadyExists(pending.label));
    }
    #[allow(unused_mut)] // mut url only for the data-url parsing
    let mut url = match &pending.webview_attributes.url {
      WindowUrl::App(path) => {
        let url = self.get_url();
        // ignore "index.html" just to simplify the url
        if path.to_str() != Some("index.html") {
          url
            .join(&path.to_string_lossy())
            .map_err(crate::Error::InvalidUrl)
            // this will never fail
            .unwrap()
        } else {
          url.into_owned()
        }
      }
      WindowUrl::External(url) => url.clone(),
      #[cfg(feature = "window-data-url")]
      WindowUrl::DataUrl(url) => url.clone(),
      _ => unimplemented!(),
    };

    #[cfg(not(feature = "window-data-url"))]
    if url.scheme() == "data" {
      return Err(crate::Error::InvalidWindowUrl(
        "data URLs are not supported without the `window-data-url` feature.",
      ));
    }

    match (
      url.scheme(),
      tauri_utils::html::extract_html_content(url.as_str()),
    ) {
      #[cfg(feature = "window-data-url")]
      ("data", Some(html_string)) => {
        // There is an issue with the external DataUrl where HTML containing special characters
        // are not correctly processed. A workaround is to first percent encode the html string,
        // before it processed by DataUrl.
        let encoded_string = percent_encoding::utf8_percent_encode(html_string, percent_encoding::NON_ALPHANUMERIC).to_string();
        let url = data_url::DataUrl::process(&format!("data:text/html,{}", encoded_string))
          .map_err(|_| crate::Error::InvalidWindowUrl("Failed to process data url"))
          .and_then(|data_url| {
            data_url
              .decode_to_vec()
              .map_err(|_| crate::Error::InvalidWindowUrl("Failed to decode processed data url"))
          })
          .and_then(|(body, _)| {
            let html = String::from_utf8_lossy(&body).into_owned();
            let mut document = tauri_utils::html::parse(html);
            if let Some(csp) = self.csp() {
              tauri_utils::html::inject_csp(&mut document, &csp.to_string());
            }
            // decode back to raw html, as the content should be fully decoded
            // when passing to wry / tauri-runtime-wry, which will be responsible
            // for handling the encoding based on the OS.
            let encoded_html = document.to_string();
            Ok(
              percent_encoding::percent_decode_str(encoded_html.as_str())
                .decode_utf8_lossy()
                .to_string(),
            )
          })
          .unwrap_or(html_string.to_string());
        pending.url = format!("data:text/html,{}", url);
      }
      _ => {
        pending.url = url.to_string();
      }
    };

    if !pending.window_builder.has_icon() {
      if let Some(default_window_icon) = self.inner.default_window_icon.clone() {
        pending.window_builder = pending
          .window_builder
          .icon(default_window_icon.try_into()?)?;
      }
    }

    if pending.window_builder.get_menu().is_none() {
      if let Some(menu) = &self.inner.menu {
        pending = pending.set_menu(menu.clone());
      }
    }

    let label = pending.label.clone();
    pending = self.prepare_pending_window(pending, &label, window_labels, app_handle.clone())?;
    pending.ipc_handler = Some(self.prepare_ipc_handler(app_handle));

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

    #[cfg(feature = "isolation")]
    let pattern = self.pattern().clone();
    let navigation_handler = pending.navigation_handler.take();
    pending.navigation_handler = Some(Box::new(move |url| {
      // always allow navigation events for the isolation iframe and do not emit them for consumers
      #[cfg(feature = "isolation")]
      if let Pattern::Isolation { schema, .. } = &pattern {
        if url.scheme() == schema
          && url.domain() == Some(crate::pattern::ISOLATION_IFRAME_SRC_DOMAIN)
        {
          return true;
        }
      }
      if let Some(handler) = &navigation_handler {
        handler(url)
      } else {
        true
      }
    }));

    Ok(pending)
  }

  pub fn attach_window(
    &self,
    app_handle: AppHandle<R>,
    window: DetachedWindow<EventLoopMessage, R>,
  ) -> Window<R> {
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
    let manager = self.inner.clone();
    let window_ = window.clone();
    // run on main thread so the plugin store doesn't dead lock with the event loop handler in App
    let _ = window.run_on_main_thread(move || {
      manager
        .plugins
        .lock()
        .expect("poisoned plugin store")
        .created(window_);
    });

    window
  }

  pub(crate) fn on_window_close(&self, label: &str) {
    self.windows_lock().remove(label);
  }

  pub fn emit_filter<S, F>(
    &self,
    event: &str,
    source_window_label: Option<&str>,
    payload: S,
    filter: F,
  ) -> crate::Result<()>
  where
    S: Serialize + Clone,
    F: Fn(&Window<R>) -> bool,
  {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("emit::run").entered();
    let emit_args = WindowEmitArgs::from(event, source_window_label, payload)?;
    assert_event_name_is_valid(event);
    self
      .windows()
      .values()
      .filter(|&w| filter(w))
      .try_for_each(|window| window.emit_internal(&emit_args))
  }

  pub fn eval_script_all<S: Into<String>>(&self, script: S) -> crate::Result<()> {
    let script = script.into();
    self
      .windows_lock()
      .values()
      .try_for_each(|window| window.eval(&script))
  }

  pub fn labels(&self) -> HashSet<String> {
    self.windows_lock().keys().cloned().collect()
  }

  pub fn config(&self) -> Arc<Config> {
    self.inner.config.clone()
  }

  #[cfg(dev)]
  pub fn config_parent(&self) -> Option<&std::path::PathBuf> {
    self.inner.config_parent.as_ref()
  }

  pub fn package_info(&self) -> &PackageInfo {
    &self.inner.package_info
  }

  pub fn unlisten(&self, handler_id: EventHandler) {
    self.inner.listeners.unlisten(handler_id)
  }

  pub fn trigger(&self, event: &str, window: Option<String>, data: Option<String>) {
    assert_event_name_is_valid(event);
    self.inner.listeners.trigger(event, window, data)
  }

  pub fn listen<F: Fn(Event) + Send + 'static>(
    &self,
    event: String,
    window: Option<String>,
    handler: F,
  ) -> EventHandler {
    assert_event_name_is_valid(&event);
    self.inner.listeners.listen(event, window, handler)
  }

  pub fn once<F: FnOnce(Event) + Send + 'static>(
    &self,
    event: String,
    window: Option<String>,
    handler: F,
  ) -> EventHandler {
    assert_event_name_is_valid(&event);
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

  pub fn get_focused_window(&self) -> Option<Window<R>> {
    self
      .windows_lock()
      .iter()
      .find(|w| w.1.is_focused().unwrap_or(false))
      .map(|w| w.1.clone())
  }

  pub fn windows(&self) -> HashMap<String, Window<R>> {
    self.windows_lock().clone()
  }
}

/// Tray APIs
#[cfg(all(desktop, feature = "system-tray"))]
impl<R: Runtime> WindowManager<R> {
  pub fn get_tray(&self, id: &str) -> Option<crate::SystemTrayHandle<R>> {
    self.inner.trays.lock().unwrap().get(id).cloned()
  }

  pub fn trays(&self) -> HashMap<String, crate::SystemTrayHandle<R>> {
    self.inner.trays.lock().unwrap().clone()
  }

  pub fn attach_tray(&self, id: String, tray: crate::SystemTrayHandle<R>) {
    self.inner.trays.lock().unwrap().insert(id, tray);
  }

  pub fn get_tray_by_runtime_id(&self, id: u16) -> Option<(String, crate::SystemTrayHandle<R>)> {
    let trays = self.inner.trays.lock().unwrap();
    let iter = trays.iter();
    for (tray_id, tray) in iter {
      if tray.id == id {
        return Some((tray_id.clone(), tray.clone()));
      }
    }
    None
  }
}

fn on_window_event<R: Runtime>(
  window: &Window<R>,
  manager: &WindowManager<R>,
  event: &WindowEvent,
) -> crate::Result<()> {
  match event {
    WindowEvent::Resized(size) => window.emit(WINDOW_RESIZED_EVENT, size)?,
    WindowEvent::Moved(position) => window.emit(WINDOW_MOVED_EVENT, position)?,
    WindowEvent::CloseRequested { api } => {
      if window.has_js_listener(Some(window.label().into()), WINDOW_CLOSE_REQUESTED_EVENT) {
        api.prevent_close();
      }
      window.emit(WINDOW_CLOSE_REQUESTED_EVENT, ())?;
    }
    WindowEvent::Destroyed => {
      window.emit(WINDOW_DESTROYED_EVENT, ())?;
      let label = window.label();
      let windows_map = manager.inner.windows.lock().unwrap();
      let windows = windows_map.values();
      for window in windows {
        window.eval(&format!(
          r#"(function () {{ const metadata = window.__TAURI_METADATA__; if (metadata != null) {{ metadata.__windows = window.__TAURI_METADATA__.__windows.filter(w => w.label !== "{label}"); }} }})()"#
        ))?;
      }
    }
    WindowEvent::Focused(focused) => window.emit(
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
    } => window.emit(
      WINDOW_SCALE_FACTOR_CHANGED_EVENT,
      ScaleFactorChanged {
        scale_factor: *scale_factor,
        size: *new_inner_size,
      },
    )?,
    WindowEvent::FileDrop(event) => match event {
      FileDropEvent::Hovered(paths) => window.emit(WINDOW_FILE_DROP_HOVER_EVENT, paths)?,
      FileDropEvent::Dropped(paths) => {
        let scopes = window.state::<Scopes>();
        for path in paths {
          if path.is_file() {
            let _ = scopes.allow_file(path);
          } else {
            let _ = scopes.allow_directory(path, false);
          }
        }
        window.emit(WINDOW_FILE_DROP_EVENT, paths)?
      }
      FileDropEvent::Cancelled => window.emit(WINDOW_FILE_DROP_CANCELLED_EVENT, ())?,
      _ => unimplemented!(),
    },
    WindowEvent::ThemeChanged(theme) => window.emit(WINDOW_THEME_CHANGED, theme.to_string())?,
  }
  Ok(())
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScaleFactorChanged {
  scale_factor: f64,
  size: PhysicalSize<u32>,
}

fn on_menu_event<R: Runtime>(window: &Window<R>, event: &MenuEvent) -> crate::Result<()> {
  window.emit(MENU_EVENT, event.menu_item_id.clone())
}

#[cfg(feature = "isolation")]
fn request_to_path(request: &tauri_runtime::http::Request, base_url: &str) -> String {
  let mut path = request
    .uri()
    .split(&['?', '#'][..])
    // ignore query string
    .next()
    .unwrap()
    .trim_start_matches(base_url)
    .to_string();

  if path.ends_with('/') {
    path.pop();
  }

  let path = percent_encoding::percent_decode(path.as_bytes())
    .decode_utf8_lossy()
    .to_string();

  if path.is_empty() {
    // if the url has no path, we should load `index.html`
    "index.html".to_string()
  } else {
    // skip leading `/`
    path.chars().skip(1).collect()
  }
}

#[cfg(test)]
mod tests {
  use super::replace_with_callback;

  #[test]
  fn string_replace_with_callback() {
    let mut tauri_index = 0;
    #[allow(clippy::single_element_loop)]
    for (src, pattern, replacement, result) in [(
      "tauri is awesome, tauri is amazing",
      "tauri",
      || {
        tauri_index += 1;
        tauri_index.to_string()
      },
      "1 is awesome, 2 is amazing",
    )] {
      assert_eq!(replace_with_callback(src, pattern, replacement), result);
    }
  }
}
