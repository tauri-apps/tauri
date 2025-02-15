// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
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
use tauri_utils::{
  assets::{AssetKey, CspHash},
  config::{Csp, CspDirectiveSources},
  html::{SCRIPT_NONCE_TOKEN, STYLE_NONCE_TOKEN},
};

use crate::resources::ResourceTable;
use crate::{
  app::{
    AppHandle, ChannelInterceptor, GlobalWebviewEventListener, GlobalWindowEventListener,
    OnPageLoad,
  },
  event::{EmitArgs, Event, EventId, EventTarget, Listeners},
  ipc::{Invoke, InvokeHandler, RuntimeAuthority},
  plugin::PluginStore,
  utils::{config::Config, PackageInfo},
  Assets, Context, EventName, Pattern, Runtime, StateManager, Webview, Window,
};

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
pub(crate) fn set_csp<R: Runtime>(
  asset: &mut String,
  assets: &impl std::borrow::Borrow<dyn Assets<R>>,
  asset_path: &AssetKey,
  manager: &AppManager<R>,
  csp: Csp,
) -> HashMap<String, CspDirectiveSources> {
  let mut csp = csp.into();
  let hash_strings =
    assets
      .borrow()
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
            log::debug!("Unknown CspHash variant encountered: {:?}", _csp_hash);
          }
        }

        acc
      });

  let dangerous_disable_asset_csp_modification = &manager
    .config()
    .app
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

  csp
}

// inspired by <https://github.com/rust-lang/rust/blob/1be5c8f90912c446ecbdc405cbc4a89f9acd20fd/library/alloc/src/str.rs#L260-L297>
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
#[non_exhaustive]
pub struct Asset {
  /// The asset bytes.
  pub bytes: Vec<u8>,
  /// The asset's mime type.
  pub mime_type: String,
  /// The `Content-Security-Policy` header value.
  pub csp_header: Option<String>,
}

impl Asset {
  /// The asset bytes.
  pub fn bytes(&self) -> &[u8] {
    &self.bytes
  }

  /// The asset's mime type.
  pub fn mime_type(&self) -> &str {
    &self.mime_type
  }

  /// The `Content-Security-Policy` header value.
  pub fn csp_header(&self) -> Option<&str> {
    self.csp_header.as_deref()
  }
}

#[default_runtime(crate::Wry, wry)]
pub struct AppManager<R: Runtime> {
  pub runtime_authority: Mutex<RuntimeAuthority>,
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
  #[cfg(dev)]
  pub config_parent: Option<std::path::PathBuf>,
  pub assets: Box<dyn Assets<R>>,

  pub app_icon: Option<Vec<u8>>,

  pub package_info: PackageInfo,

  /// Application pattern.
  pub pattern: Arc<Pattern>,

  /// Global API scripts collected from plugins.
  pub plugin_global_api_scripts: Arc<Option<&'static [&'static str]>>,

  /// Application Resources Table
  pub(crate) resources_table: Arc<Mutex<ResourceTable>>,

  /// Runtime-generated invoke key.
  pub(crate) invoke_key: String,

  pub(crate) channel_interceptor: Option<ChannelInterceptor<R>>,
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

pub(crate) enum EmitPayload<'a, S: Serialize> {
  Serialize(&'a S),
  Str(String),
}

impl<R: Runtime> AppManager<R> {
  #[allow(clippy::too_many_arguments, clippy::type_complexity)]
  pub(crate) fn with_handlers(
    #[allow(unused_mut)] mut context: Context<R>,
    plugins: PluginStore<R>,
    invoke_handler: Box<InvokeHandler<R>>,
    on_page_load: Option<Arc<OnPageLoad<R>>>,
    uri_scheme_protocols: HashMap<String, Arc<webview::UriSchemeProtocol<R>>>,
    state: StateManager,
    #[cfg(desktop)] menu_event_listener: Vec<crate::app::GlobalMenuEventListener<AppHandle<R>>>,
    #[cfg(all(desktop, feature = "tray-icon"))] tray_icon_event_listeners: Vec<
      crate::app::GlobalTrayIconEventListener<AppHandle<R>>,
    >,
    window_event_listeners: Vec<GlobalWindowEventListener<R>>,
    webiew_event_listeners: Vec<GlobalWebviewEventListener<R>>,
    #[cfg(desktop)] window_menu_event_listeners: HashMap<
      String,
      crate::app::GlobalMenuEventListener<Window<R>>,
    >,
    invoke_initialization_script: String,
    channel_interceptor: Option<ChannelInterceptor<R>>,
    invoke_key: String,
  ) -> Self {
    // generate a random isolation key at runtime
    #[cfg(feature = "isolation")]
    if let Pattern::Isolation { ref mut key, .. } = &mut context.pattern {
      *key = uuid::Uuid::new_v4().to_string();
    }

    Self {
      runtime_authority: Mutex::new(context.runtime_authority),
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
        event_listeners: Arc::new(webiew_event_listeners),
        invoke_initialization_script,
        invoke_key: invoke_key.clone(),
      },
      #[cfg(all(desktop, feature = "tray-icon"))]
      tray: tray::TrayManager {
        icon: context.tray_icon,
        icons: Default::default(),
        global_event_listeners: Mutex::new(tray_icon_event_listeners),
        event_listeners: Default::default(),
      },
      #[cfg(desktop)]
      menu: menu::MenuManager {
        menus: Default::default(),
        menu: Default::default(),
        global_event_listeners: Mutex::new(menu_event_listener),
        event_listeners: Mutex::new(window_menu_event_listeners),
      },
      plugins: Mutex::new(plugins),
      listeners: Listeners::default(),
      state: Arc::new(state),
      config: context.config,
      #[cfg(dev)]
      config_parent: context.config_parent,
      assets: context.assets,
      app_icon: context.app_icon,
      package_info: context.package_info,
      pattern: Arc::new(context.pattern),
      plugin_global_api_scripts: Arc::new(context.plugin_global_api_scripts),
      resources_table: Arc::default(),
      invoke_key,
      channel_interceptor,
    }
  }

  /// State managed by the application.
  pub(crate) fn state(&self) -> Arc<StateManager> {
    self.state.clone()
  }

  /// Get the base path to serve data from.
  ///
  /// * In dev mode, this will be based on the `devUrl` configuration value.
  /// * Otherwise, this will be based on the `frontendDist` configuration value.
  #[cfg(not(dev))]
  fn base_path(&self) -> Option<&Url> {
    use crate::utils::config::FrontendDist;
    match self.config.build.frontend_dist.as_ref() {
      Some(FrontendDist::Url(url)) => Some(url),
      _ => None,
    }
  }

  #[cfg(dev)]
  fn base_path(&self) -> Option<&Url> {
    self.config.build.dev_url.as_ref()
  }

  pub(crate) fn protocol_url(&self, https: bool) -> Cow<'_, Url> {
    if cfg!(windows) || cfg!(target_os = "android") {
      let scheme = if https { "https" } else { "http" };
      Cow::Owned(Url::parse(&format!("{scheme}://tauri.localhost")).unwrap())
    } else {
      Cow::Owned(Url::parse("tauri://localhost").unwrap())
    }
  }

  /// Get the base URL to use for webview requests.
  ///
  /// In dev mode, this will be based on the `devUrl` configuration value.
  pub(crate) fn get_url(&self, https: bool) -> Cow<'_, Url> {
    match self.base_path() {
      Some(url) => Cow::Borrowed(url),
      _ => self.protocol_url(https),
    }
  }

  fn csp(&self) -> Option<Csp> {
    if !crate::is_dev() {
      self.config.app.security.csp.clone()
    } else {
      self
        .config
        .app
        .security
        .dev_csp
        .clone()
        .or_else(|| self.config.app.security.csp.clone())
    }
  }

  pub fn get_asset(
    &self,
    mut path: String,
    _use_https_schema: bool,
  ) -> Result<Asset, Box<dyn std::error::Error>> {
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
        log::debug!("Asset `{path}` not found; fallback to {path}.html");
        let fallback = format!("{}.html", path.as_str()).into();
        let asset = assets.get(&fallback);
        asset_path = fallback;
        asset
      })
      .or_else(|| {
        log::debug!(
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
        log::debug!("Asset `{}` not found; fallback to index.html", path);
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
            #[allow(unused_mut)]
            let mut csp_map = set_csp(&mut asset, &self.assets, &asset_path, self, csp);
            #[cfg(feature = "isolation")]
            if let Pattern::Isolation { schema, .. } = &*self.pattern {
              let default_src = csp_map
                .entry("default-src".into())
                .or_insert_with(Default::default);
              default_src.push(crate::pattern::format_real_schema(
                schema,
                _use_https_schema,
              ));
            }

            csp_header.replace(Csp::DirectiveMap(csp_map).to_string());
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
        log::error!("{:?}", e);
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

  #[cfg(dev)]
  pub fn config_parent(&self) -> Option<&std::path::PathBuf> {
    self.config_parent.as_ref()
  }

  pub fn package_info(&self) -> &PackageInfo {
    &self.package_info
  }

  /// # Panics
  /// Will panic if `event` contains characters other than alphanumeric, `-`, `/`, `:` and `_`
  pub(crate) fn listen<F: Fn(Event) + Send + 'static>(
    &self,
    event: EventName,
    target: EventTarget,
    handler: F,
  ) -> EventId {
    self.listeners().listen(event, target, handler)
  }

  /// # Panics
  /// Will panic if `event` contains characters other than alphanumeric, `-`, `/`, `:` and `_`
  pub(crate) fn once<F: FnOnce(Event) + Send + 'static>(
    &self,
    event: EventName,
    target: EventTarget,
    handler: F,
  ) -> EventId {
    self.listeners().once(event, target, handler)
  }

  pub fn unlisten(&self, id: EventId) {
    self.listeners().unlisten(id)
  }

  #[cfg_attr(
    feature = "tracing",
    tracing::instrument("app::emit", skip(self, payload))
  )]
  pub(crate) fn emit<S: Serialize>(
    &self,
    event: EventName<&str>,
    payload: EmitPayload<'_, S>,
  ) -> crate::Result<()> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("emit::run").entered();
    let emit_args = match payload {
      EmitPayload::Serialize(payload) => EmitArgs::new(event, payload)?,
      EmitPayload::Str(payload) => EmitArgs::new_str(event, payload)?,
    };

    let listeners = self.listeners();
    let webviews = self
      .webview
      .webviews_lock()
      .values()
      .cloned()
      .collect::<Vec<_>>();

    listeners.emit_js(webviews.iter(), &emit_args)?;
    listeners.emit(emit_args)?;

    Ok(())
  }

  #[cfg_attr(
    feature = "tracing",
    tracing::instrument("app::emit::filter", skip(self, payload, filter))
  )]
  pub(crate) fn emit_filter<S, F>(
    &self,
    event: EventName<&str>,
    payload: EmitPayload<'_, S>,
    filter: F,
  ) -> crate::Result<()>
  where
    S: Serialize,
    F: Fn(&EventTarget) -> bool,
  {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("emit::run").entered();
    let emit_args = match payload {
      EmitPayload::Serialize(payload) => EmitArgs::new(event, payload)?,
      EmitPayload::Str(payload) => EmitArgs::new_str(event, payload)?,
    };

    let listeners = self.listeners();

    listeners.emit_js_filter(
      self.webview.webviews_lock().values(),
      &emit_args,
      Some(&filter),
    )?;

    listeners.emit_filter(emit_args, Some(filter))?;

    Ok(())
  }

  #[cfg_attr(
    feature = "tracing",
    tracing::instrument("app::emit::to", skip(self, target, payload), fields(target))
  )]
  pub(crate) fn emit_to<I, S>(
    &self,
    target: I,
    event: EventName<&str>,
    payload: EmitPayload<'_, S>,
  ) -> crate::Result<()>
  where
    I: Into<EventTarget>,
    S: Serialize,
  {
    let target = target.into();
    #[cfg(feature = "tracing")]
    tracing::Span::current().record("target", format!("{target:?}"));

    match target {
      // if targeting all, emit to all using emit without filter
      EventTarget::Any => self.emit(event, payload),

      // if targeting any label, emit using emit_filter and filter labels
      EventTarget::AnyLabel {
        label: target_label,
      } => self.emit_filter(event, payload, |t| match t {
        EventTarget::Window { label }
        | EventTarget::Webview { label }
        | EventTarget::WebviewWindow { label }
        | EventTarget::AnyLabel { label } => label == &target_label,
        _ => false,
      }),

      EventTarget::Window {
        label: target_label,
      } => self.emit_filter(event, payload, |t| match t {
        EventTarget::AnyLabel { label } | EventTarget::Window { label } => label == &target_label,
        _ => false,
      }),

      EventTarget::Webview {
        label: target_label,
      } => self.emit_filter(event, payload, |t| match t {
        EventTarget::AnyLabel { label } | EventTarget::Webview { label } => label == &target_label,
        _ => false,
      }),

      EventTarget::WebviewWindow {
        label: target_label,
      } => self.emit_filter(event, payload, |t| match t {
        EventTarget::AnyLabel { label } | EventTarget::WebviewWindow { label } => {
          label == &target_label
        }
        _ => false,
      }),

      // otherwise match same target
      _ => self.emit_filter(event, payload, |t| t == &target),
    }
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
    let window = self.window.windows_lock().remove(label);
    if let Some(window) = window {
      for webview in window.webviews() {
        self.webview.webviews_lock().remove(webview.label());
      }
    }
  }

  #[cfg(desktop)]
  pub(crate) fn on_webview_close(&self, label: &str) {
    self.webview.webviews_lock().remove(label);
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

  pub(crate) fn resources_table(&self) -> MutexGuard<'_, ResourceTable> {
    self
      .resources_table
      .lock()
      .expect("poisoned window manager")
  }

  pub(crate) fn invoke_key(&self) -> &str {
    &self.invoke_key
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
    event::EventTarget,
    generate_context,
    plugin::PluginStore,
    test::{mock_app, MockRuntime},
    webview::WebviewBuilder,
    window::WindowBuilder,
    App, Emitter, Listener, Manager, StateManager, Webview, WebviewWindow, WebviewWindowBuilder,
    Window, Wry,
  };

  use super::AppManager;

  const APP_LISTEN_ID: &str = "App::listen";
  const APP_LISTEN_ANY_ID: &str = "App::listen_any";
  const WINDOW_LISTEN_ID: &str = "Window::listen";
  const WINDOW_LISTEN_ANY_ID: &str = "Window::listen_any";
  const WEBVIEW_LISTEN_ID: &str = "Webview::listen";
  const WEBVIEW_LISTEN_ANY_ID: &str = "Webview::listen_any";
  const WEBVIEW_WINDOW_LISTEN_ID: &str = "WebviewWindow::listen";
  const WEBVIEW_WINDOW_LISTEN_ANY_ID: &str = "WebviewWindow::listen_any";
  const TEST_EVENT_NAME: &str = "event";

  #[test]
  fn check_get_url() {
    let context = generate_context!("test/fixture/src-tauri/tauri.conf.json", crate, test = true);
    let manager: AppManager<Wry> = AppManager::with_handlers(
      context,
      PluginStore::default(),
      Box::new(|_| false),
      None,
      Default::default(),
      StateManager::new(),
      Default::default(),
      #[cfg(all(desktop, feature = "tray-icon"))]
      Default::default(),
      Default::default(),
      Default::default(),
      Default::default(),
      "".into(),
      None,
      crate::generate_invoke_key().unwrap(),
    );

    #[cfg(custom_protocol)]
    {
      assert_eq!(
        manager.get_url(false).to_string(),
        if cfg!(windows) || cfg!(target_os = "android") {
          "http://tauri.localhost/"
        } else {
          "tauri://localhost"
        }
      );
      assert_eq!(
        manager.get_url(true).to_string(),
        if cfg!(windows) || cfg!(target_os = "android") {
          "https://tauri.localhost/"
        } else {
          "tauri://localhost"
        }
      );
    }

    #[cfg(dev)]
    assert_eq!(manager.get_url(false).to_string(), "http://localhost:4000/");
  }

  struct EventSetup {
    app: App<MockRuntime>,
    window: Window<MockRuntime>,
    webview: Webview<MockRuntime>,
    webview_window: WebviewWindow<MockRuntime>,
    tx: Sender<(&'static str, String)>,
    rx: Receiver<(&'static str, String)>,
  }

  fn setup_events(setup_any: bool) -> EventSetup {
    let app = mock_app();

    let window = WindowBuilder::new(&app, "main-window").build().unwrap();

    let webview = window
      .add_child(
        WebviewBuilder::new("main-webview", Default::default()),
        crate::LogicalPosition::new(0, 0),
        window.inner_size().unwrap(),
      )
      .unwrap();

    let webview_window = WebviewWindowBuilder::new(&app, "main-webview-window", Default::default())
      .build()
      .unwrap();

    let (tx, rx) = channel();

    macro_rules! setup_listener {
      ($type:ident, $id:ident, $any_id:ident) => {
        let tx_ = tx.clone();
        $type.listen(TEST_EVENT_NAME, move |evt| {
          tx_
            .send(($id, serde_json::from_str::<String>(evt.payload()).unwrap()))
            .unwrap();
        });

        if setup_any {
          let tx_ = tx.clone();
          $type.listen_any(TEST_EVENT_NAME, move |evt| {
            tx_
              .send((
                $any_id,
                serde_json::from_str::<String>(evt.payload()).unwrap(),
              ))
              .unwrap();
          });
        }
      };
    }

    setup_listener!(app, APP_LISTEN_ID, APP_LISTEN_ANY_ID);
    setup_listener!(window, WINDOW_LISTEN_ID, WINDOW_LISTEN_ANY_ID);
    setup_listener!(webview, WEBVIEW_LISTEN_ID, WEBVIEW_LISTEN_ANY_ID);
    setup_listener!(
      webview_window,
      WEBVIEW_WINDOW_LISTEN_ID,
      WEBVIEW_WINDOW_LISTEN_ANY_ID
    );

    EventSetup {
      app,
      window,
      webview,
      webview_window,
      tx,
      rx,
    }
  }

  fn assert_events(kind: &str, received: &[&str], expected: &[&str]) {
    for e in expected {
      assert!(received.contains(e), "{e} did not receive `{kind}` event");
    }
    assert_eq!(
      received.len(),
      expected.len(),
      "received {received:?} `{kind}` events but expected {expected:?}"
    );
  }

  #[test]
  fn emit() {
    let EventSetup {
      app,
      window,
      webview,
      webview_window,
      tx: _,
      rx,
    } = setup_events(true);

    run_emit_test("emit (app)", app, &rx);
    run_emit_test("emit (window)", window, &rx);
    run_emit_test("emit (webview)", webview, &rx);
    run_emit_test("emit (webview_window)", webview_window, &rx);
  }

  fn run_emit_test<M: Manager<MockRuntime> + Emitter<MockRuntime>>(
    kind: &str,
    m: M,
    rx: &Receiver<(&str, String)>,
  ) {
    let mut received = Vec::new();
    let payload = "global-payload";
    m.emit(TEST_EVENT_NAME, payload).unwrap();
    while let Ok((source, p)) = rx.recv_timeout(Duration::from_secs(1)) {
      assert_eq!(p, payload);
      received.push(source);
    }
    assert_events(
      kind,
      &received,
      &[
        APP_LISTEN_ID,
        APP_LISTEN_ANY_ID,
        WINDOW_LISTEN_ID,
        WINDOW_LISTEN_ANY_ID,
        WEBVIEW_LISTEN_ID,
        WEBVIEW_LISTEN_ANY_ID,
        WEBVIEW_WINDOW_LISTEN_ID,
        WEBVIEW_WINDOW_LISTEN_ANY_ID,
      ],
    );
  }

  #[test]
  fn emit_to() {
    let EventSetup {
      app,
      window,
      webview,
      webview_window,
      tx,
      rx,
    } = setup_events(false);

    run_emit_to_test(
      "emit_to (App)",
      &app,
      &window,
      &webview,
      &webview_window,
      tx.clone(),
      &rx,
    );
    run_emit_to_test(
      "emit_to (window)",
      &window,
      &window,
      &webview,
      &webview_window,
      tx.clone(),
      &rx,
    );
    run_emit_to_test(
      "emit_to (webview)",
      &webview,
      &window,
      &webview,
      &webview_window,
      tx.clone(),
      &rx,
    );
    run_emit_to_test(
      "emit_to (webview_window)",
      &webview_window,
      &window,
      &webview,
      &webview_window,
      tx.clone(),
      &rx,
    );
  }

  fn run_emit_to_test<M: Manager<MockRuntime> + Emitter<MockRuntime>>(
    kind: &str,
    m: &M,
    window: &Window<MockRuntime>,
    webview: &Webview<MockRuntime>,
    webview_window: &WebviewWindow<MockRuntime>,
    tx: Sender<(&'static str, String)>,
    rx: &Receiver<(&'static str, String)>,
  ) {
    let mut received = Vec::new();
    let payload = "global-payload";

    macro_rules! test_target {
      ($target:expr, $id:ident) => {
        m.emit_to($target, TEST_EVENT_NAME, payload).unwrap();
        while let Ok((source, p)) = rx.recv_timeout(Duration::from_secs(1)) {
          assert_eq!(p, payload);
          received.push(source);
        }
        assert_events(kind, &received, &[$id]);

        received.clear();
      };
    }

    test_target!(EventTarget::App, APP_LISTEN_ID);
    test_target!(window.label(), WINDOW_LISTEN_ID);
    test_target!(webview.label(), WEBVIEW_LISTEN_ID);
    test_target!(webview_window.label(), WEBVIEW_WINDOW_LISTEN_ID);

    let other_webview_listen_id = "OtherWebview::listen";
    let other_webview = WebviewWindowBuilder::new(
      window,
      kind.replace(['(', ')', ' '], ""),
      Default::default(),
    )
    .build()
    .unwrap();

    other_webview.listen(TEST_EVENT_NAME, move |evt| {
      tx.send((
        other_webview_listen_id,
        serde_json::from_str::<String>(evt.payload()).unwrap(),
      ))
      .unwrap();
    });
    m.emit_to(other_webview.label(), TEST_EVENT_NAME, payload)
      .unwrap();
    while let Ok((source, p)) = rx.recv_timeout(Duration::from_secs(1)) {
      assert_eq!(p, payload);
      received.push(source);
    }
    assert_events("emit_to", &received, &[other_webview_listen_id]);
  }
}
