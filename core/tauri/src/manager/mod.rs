// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  borrow::Cow,
  collections::HashMap,
  fmt,
  sync::{Arc, Mutex, MutexGuard},
};

use serde::Serialize;
use url::Url;

use tauri_macros::default_runtime;
use tauri_utils::debug_eprintln;
use tauri_utils::{
  assets::{AssetKey, CspHash},
  config::{Csp, CspDirectiveSources},
  html::{SCRIPT_NONCE_TOKEN, STYLE_NONCE_TOKEN},
};

use crate::{
  app::{AppHandle, GlobalWindowEventListener, OnPageLoad},
  command::RuntimeAuthority,
  event::{assert_event_name_is_valid, Event, EventId, EventTarget, Listeners},
  ipc::{Invoke, InvokeHandler, InvokeResponder},
  plugin::PluginStore,
  utils::{
    assets::Assets,
    config::{AppUrl, Config, WebviewUrl},
    PackageInfo,
  },
  Context, Pattern, Runtime, StateManager, Window,
};
use crate::{event::EmitArgs, resources::ResourceTable, Webview};

#[cfg(desktop)]
mod menu;
#[cfg(all(desktop, feature = "tray-icon"))]
mod tray;
pub mod webview;
pub mod window;

#[derive(Default)]
/// Spaced and quoted Content-Security-Policy hash values.
struct CspHashStrings {
  script: Vec<String>,
  style: Vec<String>,
}

/// Sets the CSP value to the asset HTML if needed (on Linux).
/// Returns the CSP string for access on the response header (on Windows and macOS).
#[allow(clippy::borrowed_box)]
fn set_csp<R: Runtime>(
  asset: &mut String,
  assets: &Box<dyn Assets>,
  asset_path: &AssetKey,
  manager: &AppManager<R>,
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
  if let Pattern::Isolation { schema, .. } = &*manager.pattern {
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
    #[cfg(target_pointer_width = "64")]
    let mut raw = [0u8; 8];
    #[cfg(target_pointer_width = "32")]
    let mut raw = [0u8; 4];
    #[cfg(target_pointer_width = "16")]
    let mut raw = [0u8; 2];
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

/// A resolved asset.
pub struct Asset {
  /// The asset bytes.
  pub bytes: Vec<u8>,
  /// The asset's mime type.
  pub mime_type: String,
  /// The `Content-Security-Policy` header value.
  pub csp_header: Option<String>,
}

#[default_runtime(crate::Wry, wry)]
pub struct AppManager<R: Runtime> {
  pub runtime_authority: RuntimeAuthority,
  pub window: window::WindowManager<R>,
  pub webview: webview::WebviewManager<R>,
  #[cfg(all(desktop, feature = "tray-icon"))]
  pub tray: tray::TrayManager<R>,
  #[cfg(desktop)]
  pub menu: menu::MenuManager<R>,

  pub(crate) plugins: Mutex<PluginStore<R>>,
  pub listeners: Listeners,
  pub state: Arc<StateManager>,
  pub config: Config,
  pub assets: Box<dyn Assets>,

  pub app_icon: Option<Vec<u8>>,

  pub package_info: PackageInfo,

  /// Application pattern.
  pub pattern: Arc<Pattern>,

  /// Application Resources Table
  pub(crate) resources_table: Arc<Mutex<ResourceTable>>,
}

impl<R: Runtime> fmt::Debug for AppManager<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut d = f.debug_struct("AppManager");

    d.field("window", &self.window)
      .field("plugins", &self.plugins)
      .field("state", &self.state)
      .field("config", &self.config)
      .field("app_icon", &self.app_icon)
      .field("package_info", &self.package_info)
      .field("pattern", &self.pattern);

    #[cfg(all(desktop, feature = "tray-icon"))]
    {
      d.field("tray", &self.tray);
    }

    d.finish()
  }
}

impl<R: Runtime> AppManager<R> {
  #[allow(clippy::too_many_arguments, clippy::type_complexity)]
  pub(crate) fn with_handlers(
    #[allow(unused_mut)] mut context: Context<impl Assets>,
    plugins: PluginStore<R>,
    invoke_handler: Box<InvokeHandler<R>>,
    on_page_load: Option<Arc<OnPageLoad<R>>>,
    uri_scheme_protocols: HashMap<String, Arc<webview::UriSchemeProtocol<R>>>,
    state: StateManager,
    window_event_listeners: Vec<GlobalWindowEventListener<R>>,
    #[cfg(desktop)] window_menu_event_listeners: HashMap<
      String,
      crate::app::GlobalMenuEventListener<Window<R>>,
    >,
    (invoke_responder, invoke_initialization_script): (Option<Arc<InvokeResponder<R>>>, String),
  ) -> Self {
    // generate a random isolation key at runtime
    #[cfg(feature = "isolation")]
    if let Pattern::Isolation { ref mut key, .. } = &mut context.pattern {
      *key = uuid::Uuid::new_v4().to_string();
    }

    Self {
      runtime_authority: RuntimeAuthority::new(context.resolved_acl),
      window: window::WindowManager {
        windows: Mutex::default(),
        default_icon: context.default_window_icon,
        event_listeners: Arc::new(window_event_listeners),
      },
      webview: webview::WebviewManager {
        webviews: Mutex::default(),
        invoke_handler,
        on_page_load,
        uri_scheme_protocols: Mutex::new(uri_scheme_protocols),
        invoke_responder,
        invoke_initialization_script,
      },
      #[cfg(all(desktop, feature = "tray-icon"))]
      tray: tray::TrayManager {
        icon: context.tray_icon,
        icons: Default::default(),
        global_event_listeners: Default::default(),
        event_listeners: Default::default(),
      },
      #[cfg(desktop)]
      menu: menu::MenuManager {
        menus: Default::default(),
        menu: Default::default(),
        global_event_listeners: Default::default(),
        event_listeners: Mutex::new(window_menu_event_listeners),
      },
      plugins: Mutex::new(plugins),
      listeners: Listeners::default(),
      state: Arc::new(state),
      config: context.config,
      assets: context.assets,
      app_icon: context.app_icon,
      package_info: context.package_info,
      pattern: Arc::new(context.pattern),
      resources_table: Arc::default(),
    }
  }

  /// State managed by the application.
  pub(crate) fn state(&self) -> Arc<StateManager> {
    self.state.clone()
  }

  /// Get the base path to serve data from.
  ///
  /// * In dev mode, this will be based on the `devPath` configuration value.
  /// * Otherwise, this will be based on the `distDir` configuration value.
  #[cfg(not(dev))]
  fn base_path(&self) -> &AppUrl {
    &self.config.build.dist_dir
  }

  #[cfg(dev)]
  fn base_path(&self) -> &AppUrl {
    &self.config.build.dev_path
  }

  /// Get the base URL to use for webview requests.
  ///
  /// In dev mode, this will be based on the `devPath` configuration value.
  pub(crate) fn get_url(&self) -> Cow<'_, Url> {
    match self.base_path() {
      AppUrl::Url(WebviewUrl::External(url)) => Cow::Borrowed(url),
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
      self.config.tauri.security.csp.clone()
    } else {
      self
        .config
        .tauri
        .security
        .dev_csp
        .clone()
        .or_else(|| self.config.tauri.security.csp.clone())
    }
  }

  pub fn get_asset(&self, mut path: String) -> Result<Asset, Box<dyn std::error::Error>> {
    let assets = &self.assets;
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
            csp_header.replace(set_csp(&mut asset, &self.assets, &asset_path, self, csp));
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

  pub(crate) fn listeners(&self) -> &Listeners {
    &self.listeners
  }

  pub fn run_invoke_handler(&self, invoke: Invoke<R>) -> bool {
    (self.webview.invoke_handler)(invoke)
  }

  pub fn extend_api(&self, plugin: &str, invoke: Invoke<R>) -> bool {
    self
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .extend_api(plugin, invoke)
  }

  pub fn initialize_plugins(&self, app: &AppHandle<R>) -> crate::Result<()> {
    self
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .initialize_all(app, &self.config.plugins)
  }

  pub fn config(&self) -> &Config {
    &self.config
  }

  pub fn package_info(&self) -> &PackageInfo {
    &self.package_info
  }

  pub fn listen<F: Fn(Event) + Send + 'static>(
    &self,
    event: String,
    target: EventTarget,
    handler: F,
  ) -> EventId {
    assert_event_name_is_valid(&event);
    self.listeners().listen(event, target, handler)
  }

  pub fn unlisten(&self, id: EventId) {
    self.listeners().unlisten(id)
  }

  pub fn once<F: FnOnce(Event) + Send + 'static>(
    &self,
    event: String,
    target: EventTarget,
    handler: F,
  ) {
    assert_event_name_is_valid(&event);
    self.listeners().once(event, target, handler)
  }

  pub fn emit_filter<S, F>(&self, event: &str, payload: S, filter: F) -> crate::Result<()>
  where
    S: Serialize + Clone,
    F: Fn(&EventTarget) -> bool,
  {
    assert_event_name_is_valid(event);

    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("emit::run").entered();
    let emit_args = EmitArgs::new(event, payload)?;

    let listeners = self.listeners();

    listeners.try_for_each_js(
      event,
      self.webview.webviews_lock().values(),
      |webview, target| {
        if filter(target) {
          webview.emit_js(&emit_args, target)
        } else {
          Ok(())
        }
      },
    )?;

    listeners.emit_filter(emit_args, Some(filter))?;

    Ok(())
  }

  pub fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> crate::Result<()> {
    assert_event_name_is_valid(event);

    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("emit::run").entered();
    let emit_args = EmitArgs::new(event, payload)?;

    let listeners = self.listeners();

    listeners.try_for_each_js(
      event,
      self.webview.webviews_lock().values(),
      |webview, target| webview.emit_js(&emit_args, target),
    )?;

    listeners.emit(emit_args)?;

    Ok(())
  }

  pub fn get_window(&self, label: &str) -> Option<Window<R>> {
    self.window.windows_lock().get(label).cloned()
  }

  pub fn get_focused_window(&self) -> Option<Window<R>> {
    self
      .window
      .windows_lock()
      .iter()
      .find(|w| w.1.is_focused().unwrap_or(false))
      .map(|w| w.1.clone())
  }

  pub(crate) fn on_window_close(&self, label: &str) {
    if let Some(window) = self.window.windows_lock().remove(label) {
      for webview in window.webviews() {
        self.webview.webviews_lock().remove(webview.label());
      }
    }
  }

  pub fn windows(&self) -> HashMap<String, Window<R>> {
    self.window.windows_lock().clone()
  }

  pub fn get_webview(&self, label: &str) -> Option<Webview<R>> {
    self.webview.webviews_lock().get(label).cloned()
  }

  pub fn webviews(&self) -> HashMap<String, Webview<R>> {
    self.webview.webviews_lock().clone()
  }

  /// Resources table managed by the application.
  pub(crate) fn resources_table(&self) -> MutexGuard<'_, ResourceTable> {
    self
      .resources_table
      .lock()
      .expect("poisoned window manager")
  }
}

#[cfg(desktop)]
impl<R: Runtime> AppManager<R> {
  pub fn remove_menu_from_stash_by_id(&self, id: Option<&crate::menu::MenuId>) {
    if let Some(id) = id {
      let is_used_by_a_window = self
        .window
        .windows_lock()
        .values()
        .any(|w| w.is_menu_in_use(id));
      if !(self.menu.is_menu_in_use(id) || is_used_by_a_window) {
        self.menu.menus_stash_lock().remove(id);
      }
    }
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

#[cfg(test)]
mod test {
  use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
  };

  use crate::{
    generate_context,
    plugin::PluginStore,
    test::{mock_app, MockRuntime},
    App, Manager, StateManager, WebviewWindow, WebviewWindowBuilder, Wry,
  };

  use super::AppManager;

  const WINDOW_LISTEN_ID: &str = "Window::listen";
  const WINDOW_LISTEN_GLOBAL_ID: &str = "Window::listen_global";
  const APP_LISTEN_GLOBAL_ID: &str = "App::listen_global";
  const TEST_EVENT_NAME: &str = "event";

  #[test]
  fn check_get_url() {
    let context = generate_context!("test/fixture/src-tauri/tauri.conf.json", crate);
    let manager: AppManager<Wry> = AppManager::with_handlers(
      context,
      PluginStore::default(),
      Box::new(|_| false),
      None,
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

  struct EventSetup {
    app: App<MockRuntime>,
    webview: WebviewWindow<MockRuntime>,
    tx: Sender<(&'static str, String)>,
    rx: Receiver<(&'static str, String)>,
  }

  fn setup_events() -> EventSetup {
    let app = mock_app();
    let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
      .build()
      .unwrap();

    let (tx, rx) = channel();

    let tx_ = tx.clone();
    webview.listen(TEST_EVENT_NAME, move |evt| {
      tx_
        .send((
          WINDOW_LISTEN_ID,
          serde_json::from_str::<String>(evt.payload()).unwrap(),
        ))
        .unwrap();
    });

    let tx_ = tx.clone();
    webview.listen_global(TEST_EVENT_NAME, move |evt| {
      tx_
        .send((
          WINDOW_LISTEN_GLOBAL_ID,
          serde_json::from_str::<String>(evt.payload()).unwrap(),
        ))
        .unwrap();
    });

    let tx_ = tx.clone();
    app.listen_global(TEST_EVENT_NAME, move |evt| {
      tx_
        .send((
          APP_LISTEN_GLOBAL_ID,
          serde_json::from_str::<String>(evt.payload()).unwrap(),
        ))
        .unwrap();
    });

    EventSetup {
      app,
      webview,
      tx,
      rx,
    }
  }

  fn assert_events(received: &[&str], expected: &[&str]) {
    for e in expected {
      assert!(received.contains(e), "{e} did not receive global event");
    }
    assert_eq!(
      received.len(),
      expected.len(),
      "received {:?} events but expected {:?}",
      received,
      expected
    );
  }

  #[test]
  fn app_global_events() {
    let EventSetup {
      app,
      webview: _,
      tx: _,
      rx,
    } = setup_events();

    let mut received = Vec::new();
    let payload = "global-payload";
    app.emit(TEST_EVENT_NAME, payload).unwrap();
    while let Ok((source, p)) = rx.recv_timeout(Duration::from_secs(1)) {
      assert_eq!(p, payload);
      received.push(source);
    }
    assert_events(
      &received,
      &[
        WINDOW_LISTEN_ID,
        WINDOW_LISTEN_GLOBAL_ID,
        APP_LISTEN_GLOBAL_ID,
      ],
    );
  }

  #[test]
  fn window_global_events() {
    let EventSetup {
      app: _,
      webview,
      tx: _,
      rx,
    } = setup_events();

    let mut received = Vec::new();
    let payload = "global-payload";
    webview.emit(TEST_EVENT_NAME, payload).unwrap();
    while let Ok((source, p)) = rx.recv_timeout(Duration::from_secs(1)) {
      assert_eq!(p, payload);
      received.push(source);
    }
    assert_events(
      &received,
      &[
        WINDOW_LISTEN_ID,
        WINDOW_LISTEN_GLOBAL_ID,
        APP_LISTEN_GLOBAL_ID,
      ],
    );
  }

  #[test]
  fn window_local_events() {
    let EventSetup {
      app,
      webview,
      tx,
      rx,
    } = setup_events();

    let mut received = Vec::new();
    let payload = "global-payload";
    webview
      .emit_to(webview.label(), TEST_EVENT_NAME, payload)
      .unwrap();
    while let Ok((source, p)) = rx.recv_timeout(Duration::from_secs(1)) {
      assert_eq!(p, payload);
      received.push(source);
    }
    assert_events(&received, &[WINDOW_LISTEN_ID]);

    received.clear();
    let other_webview_listen_id = "OtherWebview::listen";
    let other_webview = WebviewWindowBuilder::new(&app, "other", Default::default())
      .build()
      .unwrap();

    other_webview.listen(TEST_EVENT_NAME, move |evt| {
      tx.send((
        other_webview_listen_id,
        serde_json::from_str::<String>(evt.payload()).unwrap(),
      ))
      .unwrap();
    });
    webview
      .emit_to(other_webview.label(), TEST_EVENT_NAME, payload)
      .unwrap();
    while let Ok((source, p)) = rx.recv_timeout(Duration::from_secs(1)) {
      assert_eq!(p, payload);
      received.push(source);
    }
    assert_events(&received, &[other_webview_listen_id]);
  }
}
