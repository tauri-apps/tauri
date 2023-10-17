// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  borrow::Cow,
  collections::{HashMap, HashSet},
  fmt,
  fs::create_dir_all,
  path::PathBuf,
  sync::{Arc, Mutex, MutexGuard},
};

#[cfg(desktop)]
use crate::menu::{Menu, MenuId};
#[cfg(all(desktop, feature = "tray-icon"))]
use crate::tray::{TrayIcon, TrayIconId};
use serde::Serialize;
use serialize_to_javascript::{default_template, DefaultTemplate, Template};
use url::Url;

use tauri_macros::default_runtime;
use tauri_utils::debug_eprintln;
use tauri_utils::{
  assets::{AssetKey, CspHash},
  config::{Csp, CspDirectiveSources},
  html::{SCRIPT_NONCE_TOKEN, STYLE_NONCE_TOKEN},
};

use crate::{
  app::{
    AppHandle, GlobalWindowEvent, GlobalWindowEventListener, OnPageLoad, PageLoadPayload,
    UriSchemeResponder,
  },
  event::{assert_event_name_is_valid, Event, EventId, Listeners},
  ipc::{Invoke, InvokeHandler, InvokeResponder},
  pattern::PatternJavascript,
  plugin::PluginStore,
  runtime::{
    webview::WindowBuilder,
    window::{
      dpi::{PhysicalPosition, PhysicalSize},
      DetachedWindow, FileDropEvent, PendingWindow,
    },
  },
  utils::{
    assets::Assets,
    config::{AppUrl, Config, WindowUrl},
    PackageInfo,
  },
  Context, EventLoopMessage, Icon, Manager, Pattern, Runtime, Scopes, StateManager, Window,
  WindowEvent,
};

#[cfg(desktop)]
use crate::app::GlobalMenuEventListener;
#[cfg(all(desktop, feature = "tray-icon"))]
use crate::app::GlobalTrayIconEventListener;

#[cfg(any(target_os = "linux", target_os = "windows"))]
use crate::path::BaseDirectory;

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

pub(crate) const PROCESS_IPC_MESSAGE_FN: &str =
  include_str!("../scripts/process-ipc-message-fn.js");

// we need to proxy the dev server on mobile because we can't use `localhost`, so we use the local IP address
// and we do not get a secure context without the custom protocol that proxies to the dev server
// additionally, we need the custom protocol to inject the initialization scripts on Android
// must also keep in sync with the `let mut response` assignment in prepare_uri_scheme_protocol
pub(crate) const PROXY_DEV_SERVER: bool = cfg!(all(dev, mobile));

#[cfg(feature = "isolation")]
#[derive(Template)]
#[default_template("../scripts/isolation.js")]
pub(crate) struct IsolationJavascript<'a> {
  pub(crate) isolation_src: &'a str,
  pub(crate) style: &'a str,
}

#[derive(Template)]
#[default_template("../scripts/ipc.js")]
pub(crate) struct IpcJavascript<'a> {
  pub(crate) isolation_origin: &'a str,
}

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
    default_src.push(crate::pattern::format_real_schema(schema));
  }

  Csp::DirectiveMap(csp).to_string()
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
    let mut raw = [0u8; 8];
    getrandom::getrandom(&mut raw).expect("failed to get random bytes");
    let nonce = usize::from_ne_bytes(raw);
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
  pub(crate) windows: Mutex<HashMap<String, Window<R>>>,
  pub(crate) plugins: Mutex<PluginStore<R>>,
  listeners: Listeners,
  pub(crate) state: Arc<StateManager>,

  /// The JS message handler.
  invoke_handler: Box<InvokeHandler<R>>,

  /// The page load hook, invoked when the webview performs a navigation.
  on_page_load: Box<OnPageLoad<R>>,

  config: Arc<Config>,
  assets: Arc<dyn Assets>,
  pub(crate) default_window_icon: Option<Icon>,
  pub(crate) app_icon: Option<Vec<u8>>,
  #[cfg(all(desktop, feature = "tray-icon"))]
  pub(crate) tray_icon: Option<Icon>,

  package_info: PackageInfo,
  /// The webview protocols available to all windows.
  uri_scheme_protocols: HashMap<String, Arc<UriSchemeProtocol<R>>>,
  /// A set containing a reference to the active menus, including
  /// the app-wide menu and the window-specific menus
  ///
  /// This should be mainly used to acceess [`Menu::haccel`]
  /// to setup the accelerator handling in the event loop
  #[cfg(desktop)]
  pub menus: Arc<Mutex<HashMap<MenuId, Menu<R>>>>,
  /// The menu set to all windows.
  #[cfg(desktop)]
  pub(crate) menu: Arc<Mutex<Option<Menu<R>>>>,
  /// Menu event listeners to all windows.
  #[cfg(desktop)]
  pub(crate) menu_event_listeners: Arc<Mutex<Vec<GlobalMenuEventListener<AppHandle<R>>>>>,
  /// Menu event listeners to specific windows.
  #[cfg(desktop)]
  pub(crate) window_menu_event_listeners:
    Arc<Mutex<HashMap<String, GlobalMenuEventListener<Window<R>>>>>,
  /// Window event listeners to all windows.
  window_event_listeners: Arc<Vec<GlobalWindowEventListener<R>>>,
  /// Tray icons
  #[cfg(all(desktop, feature = "tray-icon"))]
  pub(crate) tray_icons: Arc<Mutex<Vec<TrayIcon<R>>>>,
  /// Global Tray icon event listeners.
  #[cfg(all(desktop, feature = "tray-icon"))]
  pub(crate) global_tray_event_listeners:
    Arc<Mutex<Vec<GlobalTrayIconEventListener<AppHandle<R>>>>>,
  /// Tray icon event listeners.
  #[cfg(all(desktop, feature = "tray-icon"))]
  pub(crate) tray_event_listeners:
    Arc<Mutex<HashMap<TrayIconId, GlobalTrayIconEventListener<TrayIcon<R>>>>>,
  /// Responder for invoke calls.
  invoke_responder: Option<Arc<InvokeResponder<R>>>,
  /// The script that initializes the invoke system.
  invoke_initialization_script: String,
  /// Application pattern.
  pub(crate) pattern: Pattern,
}

impl<R: Runtime> fmt::Debug for InnerWindowManager<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut d = f.debug_struct("InnerWindowManager");

    d.field("plugins", &self.plugins)
      .field("state", &self.state)
      .field("config", &self.config)
      .field("default_window_icon", &self.default_window_icon)
      .field("app_icon", &self.app_icon)
      .field("package_info", &self.package_info)
      .field("pattern", &self.pattern);

    #[cfg(all(desktop, feature = "tray-icon"))]
    d.field("tray_icon", &self.tray_icon);

    d.finish()
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
pub struct UriSchemeProtocol<R: Runtime> {
  /// Handler for protocol
  #[allow(clippy::type_complexity)]
  pub protocol:
    Box<dyn Fn(&AppHandle<R>, http::Request<Vec<u8>>, UriSchemeResponder) + Send + Sync>,
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
  #[allow(clippy::too_many_arguments, clippy::type_complexity)]
  pub(crate) fn with_handlers(
    #[allow(unused_mut)] mut context: Context<impl Assets>,
    plugins: PluginStore<R>,
    invoke_handler: Box<InvokeHandler<R>>,
    on_page_load: Box<OnPageLoad<R>>,
    uri_scheme_protocols: HashMap<String, Arc<UriSchemeProtocol<R>>>,
    state: StateManager,
    window_event_listeners: Vec<GlobalWindowEventListener<R>>,
    #[cfg(desktop)] window_menu_event_listeners: HashMap<
      String,
      GlobalMenuEventListener<Window<R>>,
    >,
    (invoke_responder, invoke_initialization_script): (Option<Arc<InvokeResponder<R>>>, String),
  ) -> Self {
    // generate a random isolation key at runtime
    #[cfg(feature = "isolation")]
    if let Pattern::Isolation { ref mut key, .. } = &mut context.pattern {
      *key = uuid::Uuid::new_v4().to_string();
    }

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
        app_icon: context.app_icon,
        #[cfg(all(desktop, feature = "tray-icon"))]
        tray_icon: context.tray_icon,
        package_info: context.package_info,
        pattern: context.pattern,
        uri_scheme_protocols,
        #[cfg(desktop)]
        menus: Default::default(),
        #[cfg(desktop)]
        menu: Default::default(),
        #[cfg(desktop)]
        menu_event_listeners: Default::default(),
        #[cfg(desktop)]
        window_menu_event_listeners: Arc::new(Mutex::new(window_menu_event_listeners)),
        window_event_listeners: Arc::new(window_event_listeners),
        #[cfg(all(desktop, feature = "tray-icon"))]
        tray_icons: Default::default(),
        #[cfg(all(desktop, feature = "tray-icon"))]
        global_tray_event_listeners: Default::default(),
        #[cfg(all(desktop, feature = "tray-icon"))]
        tray_event_listeners: Default::default(),
        invoke_responder,
        invoke_initialization_script,
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

  #[cfg(desktop)]
  pub(crate) fn prepare_window_menu_creation_handler(
    &self,
    window_menu: Option<&crate::window::WindowMenu<R>>,
  ) -> Option<impl Fn(tauri_runtime::window::RawWindow<'_>)> {
    {
      if let Some(menu) = window_menu {
        self
          .menus_stash_lock()
          .insert(menu.menu.id().clone(), menu.menu.clone());
      }
    }

    #[cfg(target_os = "macos")]
    return None;

    #[cfg_attr(target_os = "macos", allow(unused_variables, unreachable_code))]
    if let Some(menu) = &window_menu {
      let menu = menu.menu.clone();
      Some(move |raw: tauri_runtime::window::RawWindow<'_>| {
        #[cfg(target_os = "windows")]
        let _ = menu.inner().init_for_hwnd(raw.hwnd as _);
        #[cfg(any(
          target_os = "linux",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "netbsd",
          target_os = "openbsd"
        ))]
        let _ = menu
          .inner()
          .init_for_gtk_window(raw.gtk_window, raw.default_vbox);
      })
    } else {
      None
    }
  }

  /// App-wide menu.
  #[cfg(desktop)]
  pub(crate) fn menu_lock(&self) -> MutexGuard<'_, Option<Menu<R>>> {
    self.inner.menu.lock().expect("poisoned window manager")
  }

  /// Menus stash.
  #[cfg(desktop)]
  pub(crate) fn menus_stash_lock(&self) -> MutexGuard<'_, HashMap<MenuId, Menu<R>>> {
    self.inner.menus.lock().expect("poisoned window manager")
  }

  #[cfg(desktop)]
  pub(crate) fn is_menu_in_use<I: PartialEq<MenuId>>(&self, id: &I) -> bool {
    self
      .menu_lock()
      .as_ref()
      .map(|m| id.eq(m.id()))
      .unwrap_or(false)
  }

  /// Menus stash.
  #[cfg(desktop)]
  pub(crate) fn insert_menu_into_stash(&self, menu: &Menu<R>) {
    self
      .menus_stash_lock()
      .insert(menu.id().clone(), menu.clone());
  }

  #[cfg(desktop)]
  pub(crate) fn remove_menu_from_stash_by_id(&self, id: Option<&MenuId>) {
    if let Some(id) = id {
      let is_used_by_a_window = self.windows_lock().values().any(|w| w.is_menu_in_use(id));
      if !(self.is_menu_in_use(id) || is_used_by_a_window) {
        self.menus_stash_lock().remove(id);
      }
    }
  }

  /// The invoke responder.
  pub(crate) fn invoke_responder(&self) -> Option<Arc<InvokeResponder<R>>> {
    self.inner.invoke_responder.clone()
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
      _ => self.protocol_url(),
    }
  }

  pub(crate) fn protocol_url(&self) -> Cow<'_, Url> {
    if cfg!(windows) || cfg!(target_os = "android") {
      Cow::Owned(Url::parse("http://tauri.localhost").unwrap())
    } else {
      Cow::Owned(Url::parse("tauri://localhost").unwrap())
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
    let plugin_init = self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .initialization_script();

    let pattern_init = PatternJavascript {
      pattern: self.pattern().into(),
    }
    .render_default(&Default::default())?;

    let ipc_init = IpcJavascript {
      isolation_origin: &match self.pattern() {
        #[cfg(feature = "isolation")]
        Pattern::Isolation { schema, .. } => crate::pattern::format_real_schema(schema),
        _ => "".to_string(),
      },
    }
    .render_default(&Default::default())?;

    let mut webview_attributes = pending.webview_attributes;

    let mut window_labels = window_labels.to_vec();
    let l = label.to_string();
    if !window_labels.contains(&l) {
      window_labels.push(l);
    }
    webview_attributes = webview_attributes
      .initialization_script(
        r#"
        if (!window.__TAURI_INTERNALS__) {
          Object.defineProperty(window, '__TAURI_INTERNALS__', {
            value: {
              plugins: {}
            }
          })
        }
      "#,
      )
      .initialization_script(&self.inner.invoke_initialization_script)
      .initialization_script(&format!(
        r#"
          Object.defineProperty(window.__TAURI_INTERNALS__, 'metadata', {{
            value: {{
              windows: {window_labels_array}.map(function (label) {{ return {{ label: label }} }}),
              currentWindow: {{ label: {current_window_label} }}
            }}
          }})
        "#,
        window_labels_array = serde_json::to_string(&window_labels)?,
        current_window_label = serde_json::to_string(&label)?,
      ))
      .initialization_script(&self.initialization_script(
        &ipc_init.into_string(),
        &pattern_init.into_string(),
        &plugin_init,
        is_init_global,
      )?);

    #[cfg(feature = "isolation")]
    if let Pattern::Isolation { schema, .. } = self.pattern() {
      webview_attributes = webview_attributes.initialization_script(
        &IsolationJavascript {
          isolation_src: &crate::pattern::format_real_schema(schema),
          style: tauri_utils::pattern::isolation::IFRAME_STYLE,
        }
        .render_default(&Default::default())?
        .into_string(),
      );
    }

    pending.webview_attributes = webview_attributes;

    let mut registered_scheme_protocols = Vec::new();

    for (uri_scheme, protocol) in &self.inner.uri_scheme_protocols {
      registered_scheme_protocols.push(uri_scheme.clone());
      let protocol = protocol.clone();
      let app_handle = Mutex::new(app_handle.clone());
      pending.register_uri_scheme_protocol(uri_scheme.clone(), move |p, responder| {
        (protocol.protocol)(
          &app_handle.lock().unwrap(),
          p,
          UriSchemeResponder(responder),
        )
      });
    }

    let window_url = Url::parse(&pending.url).unwrap();
    let window_origin = if window_url.scheme() == "data" {
      "null".into()
    } else if (cfg!(windows) || cfg!(target_os = "android"))
      && window_url.scheme() != "http"
      && window_url.scheme() != "https"
    {
      format!("http://{}.localhost", window_url.scheme())
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
      let protocol =
        crate::protocol::tauri::get(self, &window_origin, web_resource_request_handler);
      pending.register_uri_scheme_protocol("tauri", move |request, responder| {
        protocol(request, UriSchemeResponder(responder))
      });
      registered_scheme_protocols.push("tauri".into());
    }

    if !registered_scheme_protocols.contains(&"ipc".into()) {
      let protocol = crate::ipc::protocol::get(self.clone(), pending.label.clone());
      pending.register_uri_scheme_protocol("ipc", move |request, responder| {
        protocol(request, UriSchemeResponder(responder))
      });
      registered_scheme_protocols.push("ipc".into());
    }

    #[cfg(feature = "protocol-asset")]
    if !registered_scheme_protocols.contains(&"asset".into()) {
      let asset_scope = self.state().get::<crate::Scopes>().asset_protocol.clone();
      let protocol = crate::protocol::asset::get(asset_scope.clone(), window_origin.clone());
      pending.register_uri_scheme_protocol("asset", move |request, responder| {
        protocol(request, UriSchemeResponder(responder))
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
      let protocol = crate::protocol::isolation::get(assets.clone(), *crypto_keys.aes_gcm().raw());
      pending.register_uri_scheme_protocol(schema, move |request, responder| {
        protocol(request, UriSchemeResponder(responder))
      });
    }

    Ok(pending)
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
        debug_eprintln!("Asset `{path}` not found; fallback to {path}.html");
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
        let mime_type = tauri_utils::mime_type::MimeType::parse(&final_data, &path);
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

  fn initialization_script(
    &self,
    ipc_script: &str,
    pattern_script: &str,
    plugin_initialization_script: &str,
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
      #[raw]
      core_script: &'a str,
      #[raw]
      event_initialization_script: &'a str,
      #[raw]
      plugin_initialization_script: &'a str,
      #[raw]
      freeze_prototype: &'a str,
    }

    #[derive(Template)]
    #[default_template("../scripts/core.js")]
    struct CoreJavascript<'a> {
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

    InitJavascript {
      pattern_script,
      ipc_script,
      bundle_script,
      core_script: &CoreJavascript {
        os_name: std::env::consts::OS,
      }
      .render_default(&Default::default())?
      .into_string(),
      event_initialization_script: &self.event_initialization_script(),
      plugin_initialization_script,
      freeze_prototype,
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
      function = self.listeners().function_name(),
      listeners = self.listeners().listeners_object_name()
    )
  }

  pub(crate) fn listeners(&self) -> &Listeners {
    &self.inner.listeners
  }
}

impl<R: Runtime> WindowManager<R> {
  pub fn run_invoke_handler(&self, invoke: Invoke<R>) -> bool {
    (self.inner.invoke_handler)(invoke)
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

  pub fn extend_api(&self, plugin: &str, invoke: Invoke<R>) -> bool {
    self
      .inner
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .extend_api(plugin, invoke)
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
    mut pending: PendingWindow<EventLoopMessage, R>,
    window_labels: &[String],
  ) -> crate::Result<PendingWindow<EventLoopMessage, R>> {
    if self.windows_lock().contains_key(&pending.label) {
      return Err(crate::Error::WindowLabelAlreadyExists(pending.label));
    }
    #[allow(unused_mut)] // mut url only for the data-url parsing
    let mut url = match &pending.webview_attributes.url {
      WindowUrl::App(path) => {
        let url = if PROXY_DEV_SERVER {
          Cow::Owned(Url::parse("tauri://localhost").unwrap())
        } else {
          self.get_url()
        };
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
      WindowUrl::External(url) => {
        let config_url = self.get_url();
        let is_local = config_url.make_relative(url).is_some();
        let mut url = url.clone();
        if is_local && PROXY_DEV_SERVER {
          url.set_scheme("tauri").unwrap();
          url.set_host(Some("localhost")).unwrap();
        }
        url
      }
      _ => unimplemented!(),
    };

    #[cfg(not(feature = "window-data-url"))]
    if url.scheme() == "data" {
      return Err(crate::Error::InvalidWindowUrl(
        "data URLs are not supported without the `window-data-url` feature.",
      ));
    }

    #[cfg(feature = "window-data-url")]
    if let Some(csp) = self.csp() {
      if url.scheme() == "data" {
        if let Ok(data_url) = data_url::DataUrl::process(url.as_str()) {
          let (body, _) = data_url.decode_to_vec().unwrap();
          let html = String::from_utf8_lossy(&body).into_owned();
          // naive way to check if it's an html
          if html.contains('<') && html.contains('>') {
            let mut document = tauri_utils::html::parse(html);
            tauri_utils::html::inject_csp(&mut document, &csp.to_string());
            url.set_path(&format!("{},{}", mime::TEXT_HTML, document.to_string()));
          }
        }
      }
    }

    pending.url = url.to_string();

    if !pending.window_builder.has_icon() {
      if let Some(default_window_icon) = self.inner.default_window_icon.clone() {
        pending.window_builder = pending
          .window_builder
          .icon(default_window_icon.try_into()?)?;
      }
    }

    #[cfg(target_os = "android")]
    {
      pending = pending.on_webview_created(move |ctx| {
        let plugin_manager = ctx
          .env
          .call_method(
            ctx.activity,
            "getPluginManager",
            "()Lapp/tauri/plugin/PluginManager;",
            &[],
          )?
          .l()?;

        // tell the manager the webview is ready
        ctx.env.call_method(
          plugin_manager,
          "onWebViewCreated",
          "(Landroid/webkit/WebView;)V",
          &[ctx.webview.into()],
        )?;

        Ok(())
      });
    }

    let label = pending.label.clone();
    pending = self.prepare_pending_window(
      pending,
      &label,
      window_labels,
      #[allow(clippy::redundant_clone)]
      app_handle.clone(),
    )?;

    #[cfg(any(target_os = "macos", target_os = "ios", not(ipc_custom_protocol)))]
    {
      pending.ipc_handler = Some(crate::ipc::protocol::message_handler(self.clone()));
    }

    // in `Windows`, we need to force a data_directory
    // but we do respect user-specification
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    if pending.webview_attributes.data_directory.is_none() {
      let local_app_data = app_handle.path().resolve(
        &self.inner.config.tauri.bundle.identifier,
        BaseDirectory::LocalData,
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
    let manager = self.inner.clone();
    let label = pending.label.clone();
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
        if !handler(url) {
          return false;
        }
      }
      let window = manager.windows.lock().unwrap().get(&label).cloned();
      if let Some(w) = window {
        manager
          .plugins
          .lock()
          .expect("poisoned plugin store")
          .on_navigation(&w, url)
      } else {
        true
      }
    }));

    Ok(pending)
  }

  pub(crate) fn attach_window(
    &self,
    app_handle: AppHandle<R>,
    window: DetachedWindow<EventLoopMessage, R>,
    #[cfg(desktop)] menu: Option<crate::window::WindowMenu<R>>,
  ) -> Window<R> {
    let window = Window::new(
      self.clone(),
      window,
      app_handle,
      #[cfg(desktop)]
      menu,
    );

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

    #[cfg(target_os = "ios")]
    {
      window
        .with_webview(|w| {
          unsafe { crate::ios::on_webview_created(w.inner() as _, w.view_controller() as _) };
        })
        .expect("failed to run on_webview_created hook");
    }

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
    assert_event_name_is_valid(event);
    self
      .windows_lock()
      .values()
      .filter(|&w| filter(w))
      .try_for_each(|window| window.emit_internal(event, source_window_label, payload.clone()))
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

  pub fn package_info(&self) -> &PackageInfo {
    &self.inner.package_info
  }

  pub fn trigger(&self, event: &str, window: Option<String>, data: Option<String>) {
    assert_event_name_is_valid(event);
    self.listeners().trigger(event, window, data)
  }

  pub fn listen<F: Fn(Event) + Send + 'static>(
    &self,
    event: String,
    window: Option<String>,
    handler: F,
  ) -> EventId {
    assert_event_name_is_valid(&event);
    self.listeners().listen(event, window, handler)
  }

  pub fn once<F: FnOnce(Event) + Send + 'static>(
    &self,
    event: String,
    window: Option<String>,
    handler: F,
  ) {
    assert_event_name_is_valid(&event);
    self.listeners().once(event, window, handler)
  }

  pub fn unlisten(&self, id: EventId) {
    self.listeners().unlisten(id)
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

#[derive(Serialize, Clone)]
struct FileDropPayload<'a> {
  paths: &'a Vec<PathBuf>,
  position: &'a PhysicalPosition<f64>,
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
          r#"(function () {{ const metadata = window.__TAURI_INTERNALS__.metadata; if (metadata != null) {{ metadata.windows = window.__TAURI_INTERNALS__.metadata.windows.filter(w => w.label !== "{label}"); }} }})()"#,
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
      FileDropEvent::Hovered { paths, position } => {
        let payload = FileDropPayload { paths, position };
        window.emit(WINDOW_FILE_DROP_HOVER_EVENT, payload)?
      }
      FileDropEvent::Dropped { paths, position } => {
        let scopes = window.state::<Scopes>();
        for path in paths {
          if path.is_file() {
            let _ = scopes.allow_file(path);
          } else {
            let _ = scopes.allow_directory(path, false);
          }
        }
        let payload = FileDropPayload { paths, position };
        window.emit(WINDOW_FILE_DROP_EVENT, payload)?
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
      Box::new(|_| false),
      Box::new(|_, _| ()),
      Default::default(),
      StateManager::new(),
      Default::default(),
      Default::default(),
      (None, "".into()),
    );

    #[cfg(custom_protocol)]
    {
      assert_eq!(
        manager.get_url().to_string(),
        if cfg!(windows) || cfg!(target_os = "android") {
          "http://tauri.localhost/"
        } else {
          "tauri://localhost"
        }
      );
    }

    #[cfg(dev)]
    assert_eq!(manager.get_url().to_string(), "http://localhost:4000/");
  }
}
