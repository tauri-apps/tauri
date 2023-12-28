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

use serde::Serialize;
use serialize_to_javascript::{default_template, DefaultTemplate, Template};
use tauri_runtime::{
  webview::WindowBuilder,
  window::{
    dpi::{PhysicalPosition, PhysicalSize},
    DetachedWindow, FileDropEvent, PendingWindow,
  },
};
use tauri_utils::config::WindowUrl;
use url::Url;

use crate::{
  app::{GlobalWindowEventListener, OnPageLoad, UriSchemeResponder},
  ipc::{InvokeHandler, InvokeResponder},
  pattern::PatternJavascript,
  window::PageLoadPayload,
  AppHandle, EventLoopMessage, Icon, Manager, Runtime, Scopes, Window, WindowEvent,
};

use super::AppManager;

// we need to proxy the dev server on mobile because we can't use `localhost`, so we use the local IP address
// and we do not get a secure context without the custom protocol that proxies to the dev server
// additionally, we need the custom protocol to inject the initialization scripts on Android
// must also keep in sync with the `let mut response` assignment in prepare_uri_scheme_protocol
pub(crate) const PROXY_DEV_SERVER: bool = cfg!(all(dev, mobile));

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
  include_str!("../../scripts/process-ipc-message-fn.js");

#[cfg(feature = "isolation")]
#[derive(Template)]
#[default_template("../../scripts/isolation.js")]
pub(crate) struct IsolationJavascript<'a> {
  pub(crate) isolation_src: &'a str,
  pub(crate) style: &'a str,
}

#[derive(Template)]
#[default_template("../../scripts/ipc.js")]
pub(crate) struct IpcJavascript<'a> {
  pub(crate) isolation_origin: &'a str,
}

/// Uses a custom URI scheme handler to resolve file requests
pub struct UriSchemeProtocol<R: Runtime> {
  /// Handler for protocol
  #[allow(clippy::type_complexity)]
  pub protocol:
    Box<dyn Fn(&AppHandle<R>, http::Request<Vec<u8>>, UriSchemeResponder) + Send + Sync>,
}

pub struct WindowManager<R: Runtime> {
  pub windows: Mutex<HashMap<String, Window<R>>>,
  /// The JS message handler.
  pub invoke_handler: Box<InvokeHandler<R>>,
  /// The page load hook, invoked when the webview performs a navigation.
  pub on_page_load: Option<Arc<OnPageLoad<R>>>,
  pub default_icon: Option<Icon>,
  /// The webview protocols available to all windows.
  pub uri_scheme_protocols: Mutex<HashMap<String, Arc<UriSchemeProtocol<R>>>>,

  /// Window event listeners to all windows.
  pub event_listeners: Arc<Vec<GlobalWindowEventListener<R>>>,

  /// Responder for invoke calls.
  pub invoke_responder: Option<Arc<InvokeResponder<R>>>,
  /// The script that initializes the invoke system.
  pub invoke_initialization_script: String,
}

impl<R: Runtime> fmt::Debug for WindowManager<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("WindowManager")
      .field("default_window_icon", &self.default_icon)
      .field(
        "invoke_initialization_script",
        &self.invoke_initialization_script,
      )
      .finish()
  }
}

impl<R: Runtime> WindowManager<R> {
  pub(crate) fn register_uri_scheme_protocol<N: Into<String>>(
    &self,
    uri_scheme: N,
    protocol: Arc<UriSchemeProtocol<R>>,
  ) {
    let uri_scheme = uri_scheme.into();
    self
      .uri_scheme_protocols
      .lock()
      .unwrap()
      .insert(uri_scheme, protocol);
  }

  /// Get a locked handle to the windows.
  pub(crate) fn windows_lock(&self) -> MutexGuard<'_, HashMap<String, Window<R>>> {
    self.windows.lock().expect("poisoned window manager")
  }

  fn prepare_pending_window(
    &self,
    mut pending: PendingWindow<EventLoopMessage, R>,
    label: &str,
    window_labels: &[String],
    app_handle: AppHandle<R>,
  ) -> crate::Result<PendingWindow<EventLoopMessage, R>> {
    let is_init_global = app_handle.manager.config.build.with_global_tauri;
    let plugin_init = app_handle
      .manager
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .initialization_script();

    let pattern_init = PatternJavascript {
      pattern: (&*app_handle.manager.pattern).into(),
    }
    .render_default(&Default::default())?;

    let mut webview_attributes = pending.webview_attributes;

    let ipc_init = IpcJavascript {
      isolation_origin: &match &*app_handle.manager.pattern {
        #[cfg(feature = "isolation")]
        crate::Pattern::Isolation { schema, .. } => crate::pattern::format_real_schema(schema),
        _ => "".to_string(),
      },
    }
    .render_default(&Default::default())?;

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
      .initialization_script(&self.invoke_initialization_script)
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
        &app_handle.manager,
        &ipc_init.into_string(),
        &pattern_init.into_string(),
        &plugin_init,
        is_init_global,
      )?);

    #[cfg(feature = "isolation")]
    if let crate::Pattern::Isolation { schema, .. } = &*app_handle.manager.pattern {
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

    for (uri_scheme, protocol) in &*self.uri_scheme_protocols.lock().unwrap() {
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
      let protocol = crate::protocol::tauri::get(
        app_handle.manager.clone(),
        &window_origin,
        web_resource_request_handler,
      );
      pending.register_uri_scheme_protocol("tauri", move |request, responder| {
        protocol(request, UriSchemeResponder(responder))
      });
      registered_scheme_protocols.push("tauri".into());
    }

    if !registered_scheme_protocols.contains(&"ipc".into()) {
      let protocol = crate::ipc::protocol::get(app_handle.manager.clone(), pending.label.clone());
      pending.register_uri_scheme_protocol("ipc", move |request, responder| {
        protocol(request, UriSchemeResponder(responder))
      });
      registered_scheme_protocols.push("ipc".into());
    }

    let label = pending.label.clone();
    let manager = app_handle.manager.clone();
    let on_page_load_handler = pending.on_page_load_handler.take();
    pending
      .on_page_load_handler
      .replace(Box::new(move |url, event| {
        let payload = PageLoadPayload { url: &url, event };

        if let Some(w) = manager.get_window(&label) {
          if let Some(on_page_load) = &manager.window.on_page_load {
            on_page_load(&w, &payload);
          }

          manager.plugins.lock().unwrap().on_page_load(&w, &payload);
        }

        if let Some(handler) = &on_page_load_handler {
          handler(url, event);
        }
      }));

    #[cfg(feature = "protocol-asset")]
    if !registered_scheme_protocols.contains(&"asset".into()) {
      let asset_scope = app_handle
        .manager
        .state()
        .get::<crate::Scopes>()
        .asset_protocol
        .clone();
      let protocol = crate::protocol::asset::get(asset_scope.clone(), window_origin.clone());
      pending.register_uri_scheme_protocol("asset", move |request, responder| {
        protocol(request, UriSchemeResponder(responder))
      });
    }

    #[cfg(feature = "isolation")]
    if let crate::Pattern::Isolation {
      assets,
      schema,
      key: _,
      crypto_keys,
    } = &*app_handle.manager.pattern
    {
      let protocol = crate::protocol::isolation::get(assets.clone(), *crypto_keys.aes_gcm().raw());
      pending.register_uri_scheme_protocol(schema, move |request, responder| {
        protocol(request, UriSchemeResponder(responder))
      });
    }

    Ok(pending)
  }

  fn initialization_script(
    &self,
    app_manager: &AppManager<R>,
    ipc_script: &str,
    pattern_script: &str,
    plugin_initialization_script: &str,
    with_global_tauri: bool,
  ) -> crate::Result<String> {
    #[derive(Template)]
    #[default_template("../../scripts/init.js")]
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
    #[default_template("../../scripts/core.js")]
    struct CoreJavascript<'a> {
      os_name: &'a str,
    }

    let bundle_script = if with_global_tauri {
      include_str!("../../scripts/bundle.global.js")
    } else {
      ""
    };

    let freeze_prototype = if app_manager.config.tauri.security.freeze_prototype {
      include_str!("../../scripts/freeze_prototype.js")
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
      event_initialization_script: &crate::event::event_initialization_script(
        app_manager.listeners().function_name(),
        app_manager.listeners().listeners_object_name(),
      ),
      plugin_initialization_script,
      freeze_prototype,
    }
    .render_default(&Default::default())
    .map(|s| s.into_string())
    .map_err(Into::into)
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
          app_handle.manager.get_url()
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
        let config_url = app_handle.manager.get_url();
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
    if let Some(csp) = app_handle.manager.csp() {
      if url.scheme() == "data" {
        if let Ok(data_url) = data_url::DataUrl::process(url.as_str()) {
          let (body, _) = data_url.decode_to_vec().unwrap();
          let html = String::from_utf8_lossy(&body).into_owned();
          // naive way to check if it's an html
          if html.contains('<') && html.contains('>') {
            let document = tauri_utils::html::parse(html);
            tauri_utils::html::inject_csp(&document, &csp.to_string());
            url.set_path(&format!("{},{}", mime::TEXT_HTML, document.to_string()));
          }
        }
      }
    }

    pending.url = url.to_string();

    if !pending.window_builder.has_icon() {
      if let Some(default_window_icon) = self.default_icon.clone() {
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
      pending.ipc_handler = Some(crate::ipc::protocol::message_handler(
        app_handle.manager.clone(),
      ));
    }

    // in `Windows`, we need to force a data_directory
    // but we do respect user-specification
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    if pending.webview_attributes.data_directory.is_none() {
      let local_app_data = app_handle.path().resolve(
        &app_handle.manager.config.tauri.bundle.identifier,
        crate::path::BaseDirectory::LocalData,
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
    let pattern = app_handle.manager.pattern.clone();
    let navigation_handler = pending.navigation_handler.take();
    let manager = app_handle.manager.clone();
    let label = pending.label.clone();
    pending.navigation_handler = Some(Box::new(move |url| {
      // always allow navigation events for the isolation iframe and do not emit them for consumers
      #[cfg(feature = "isolation")]
      if let crate::Pattern::Isolation { schema, .. } = &*pattern {
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
      let window = manager.window.windows_lock().get(&label).cloned();
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
      app_handle.manager.clone(),
      window,
      app_handle,
      #[cfg(desktop)]
      menu,
    );

    let window_ = window.clone();
    let window_event_listeners = self.event_listeners.clone();
    let manager = window.manager.clone();
    window.on_window_event(move |event| {
      let _ = on_window_event(&window_, &manager, event);
      for handler in window_event_listeners.iter() {
        handler(&window_, event);
      }
    });

    // insert the window into our manager
    {
      self
        .windows_lock()
        .insert(window.label().to_string(), window.clone());
    }

    // let plugins know that a new window has been added to the manager
    let manager = window.manager.clone();
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
}

#[derive(Serialize, Clone)]
struct FileDropPayload<'a> {
  paths: &'a Vec<PathBuf>,
  position: &'a PhysicalPosition<f64>,
}

fn on_window_event<R: Runtime>(
  window: &Window<R>,
  manager: &AppManager<R>,
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
      let windows_map = manager.window.windows_lock();
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
