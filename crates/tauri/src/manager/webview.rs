// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
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
use serialize_to_javascript::{default_template, DefaultTemplate, Template};
use tauri_runtime::{
  webview::{DetachedWebview, InitializationScript, PendingWebview},
  window::DragDropEvent,
};
use tauri_utils::config::WebviewUrl;
use url::Url;

use crate::{
  app::{GlobalWebviewEventListener, OnPageLoad, UriSchemeResponder, WebviewEvent},
  ipc::InvokeHandler,
  pattern::PatternJavascript,
  sealed::ManagerBase,
  webview::PageLoadPayload,
  EventLoopMessage, EventTarget, Manager, Runtime, Scopes, UriSchemeContext, Webview, Window,
};

use super::{
  window::{DragDropPayload, DRAG_DROP_EVENT, DRAG_ENTER_EVENT, DRAG_LEAVE_EVENT, DRAG_OVER_EVENT},
  {AppManager, EmitPayload},
};

// we need to proxy the dev server on mobile because we can't use `localhost`, so we use the local IP address
// and we do not get a secure context without the custom protocol that proxies to the dev server
// additionally, we need the custom protocol to inject the initialization scripts on Android
// must also keep in sync with the `let mut response` assignment in prepare_uri_scheme_protocol
pub(crate) const PROXY_DEV_SERVER: bool = cfg!(all(dev, mobile));

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
    Box<dyn Fn(UriSchemeContext<'_, R>, http::Request<Vec<u8>>, UriSchemeResponder) + Send + Sync>,
}

pub struct WebviewManager<R: Runtime> {
  pub webviews: Mutex<HashMap<String, Webview<R>>>,
  /// The JS message handler.
  pub invoke_handler: Box<InvokeHandler<R>>,
  /// The page load hook, invoked when the webview performs a navigation.
  pub on_page_load: Option<Arc<OnPageLoad<R>>>,
  /// The webview protocols available to all webviews.
  pub uri_scheme_protocols: Mutex<HashMap<String, Arc<UriSchemeProtocol<R>>>>,
  /// Webview event listeners to all webviews.
  pub event_listeners: Arc<Vec<GlobalWebviewEventListener<R>>>,

  /// The script that initializes the invoke system.
  pub invoke_initialization_script: String,

  /// A runtime generated invoke key.
  pub(crate) invoke_key: String,
}

impl<R: Runtime> fmt::Debug for WebviewManager<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("WebviewManager")
      .field(
        "invoke_initialization_script",
        &self.invoke_initialization_script,
      )
      .field("invoke_key", &self.invoke_key)
      .finish()
  }
}

impl<R: Runtime> WebviewManager<R> {
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

  /// Get a locked handle to the webviews.
  pub(crate) fn webviews_lock(&self) -> MutexGuard<'_, HashMap<String, Webview<R>>> {
    self.webviews.lock().expect("poisoned webview manager")
  }

  fn prepare_pending_webview<M: Manager<R>>(
    &self,
    mut pending: PendingWebview<EventLoopMessage, R>,
    label: &str,
    window_label: &str,
    manager: &M,
  ) -> crate::Result<PendingWebview<EventLoopMessage, R>> {
    let app_manager = manager.manager();

    let plugin_init_scripts = app_manager
      .plugins
      .lock()
      .expect("poisoned plugin store")
      .initialization_script();

    let pattern_init = PatternJavascript {
      pattern: (&*app_manager.pattern).into(),
    }
    .render_default(&Default::default())?;

    let mut webview_attributes = pending.webview_attributes;

    let use_https_scheme = webview_attributes.use_https_scheme;

    let ipc_init = IpcJavascript {
      isolation_origin: &match &*app_manager.pattern {
        #[cfg(feature = "isolation")]
        crate::Pattern::Isolation { schema, .. } => {
          crate::pattern::format_real_schema(schema, use_https_scheme)
        }
        _ => "".to_owned(),
      },
    }
    .render_default(&Default::default())?;

    let mut all_initialization_scripts: Vec<InitializationScript> = vec![];

    fn main_frame_script(script: String) -> InitializationScript {
      InitializationScript {
        script,
        for_main_frame_only: true,
      }
    }

    all_initialization_scripts.push(main_frame_script(
      r"
        Object.defineProperty(window, 'isTauri', {
          value: true,
        });

        if (!window.__TAURI_INTERNALS__) {
          Object.defineProperty(window, '__TAURI_INTERNALS__', {
            value: {
              plugins: {}
            }
          })
        }
      "
      .to_owned(),
    ));
    all_initialization_scripts.push(main_frame_script(self.invoke_initialization_script.clone()));
    all_initialization_scripts.push(main_frame_script(format!(
      r#"
          Object.defineProperty(window.__TAURI_INTERNALS__, 'metadata', {{
            value: {{
              currentWindow: {{ label: {current_window_label} }},
              currentWebview: {{ label: {current_webview_label} }}
            }}
          }})
        "#,
      current_window_label = serde_json::to_string(window_label)?,
      current_webview_label = serde_json::to_string(&label)?,
    )));
    all_initialization_scripts.push(main_frame_script(self.initialization_script(
      app_manager,
      &ipc_init.into_string(),
      &pattern_init.into_string(),
      use_https_scheme,
    )?));

    all_initialization_scripts.extend(plugin_init_scripts);

    #[cfg(feature = "isolation")]
    if let crate::Pattern::Isolation { schema, .. } = &*app_manager.pattern {
      all_initialization_scripts.push(main_frame_script(
        IsolationJavascript {
          isolation_src: &crate::pattern::format_real_schema(schema, use_https_scheme),
          style: tauri_utils::pattern::isolation::IFRAME_STYLE,
        }
        .render_default(&Default::default())?
        .into_string(),
      ));
    }

    if let Some(plugin_global_api_scripts) = &*app_manager.plugin_global_api_scripts {
      for &script in plugin_global_api_scripts.iter() {
        all_initialization_scripts.push(main_frame_script(script.to_owned()));
      }
    }

    // Prepend `all_initialization_scripts` to `webview_attributes.initialization_scripts`
    all_initialization_scripts.extend(webview_attributes.initialization_scripts);
    webview_attributes.initialization_scripts = all_initialization_scripts;

    pending.webview_attributes = webview_attributes;

    let mut registered_scheme_protocols = Vec::new();

    for (uri_scheme, protocol) in &*self.uri_scheme_protocols.lock().unwrap() {
      registered_scheme_protocols.push(uri_scheme.clone());
      let protocol = protocol.clone();
      let app_handle = manager.app_handle().clone();

      pending.register_uri_scheme_protocol(uri_scheme, move |webview_id, request, responder| {
        let context = UriSchemeContext {
          app_handle: &app_handle,
          webview_label: webview_id,
        };
        (protocol.protocol)(context, request, UriSchemeResponder(responder))
      });
    }

    let window_url = Url::parse(&pending.url).unwrap();
    let window_origin = if window_url.scheme() == "data" {
      "null".into()
    } else if (cfg!(windows) || cfg!(target_os = "android"))
      && window_url.scheme() != "http"
      && window_url.scheme() != "https"
    {
      let https = if use_https_scheme { "https" } else { "http" };
      format!("{https}://{}.localhost", window_url.scheme())
    } else if let Some(host) = window_url.host() {
      format!(
        "{}://{}{}",
        window_url.scheme(),
        host,
        window_url
          .port()
          .map(|p| format!(":{p}"))
          .unwrap_or_default()
      )
    } else {
      "null".into()
    };

    if !registered_scheme_protocols.contains(&"tauri".into()) {
      let web_resource_request_handler = pending.web_resource_request_handler.take();
      let protocol = crate::protocol::tauri::get(
        manager.manager_owned(),
        &window_origin,
        web_resource_request_handler,
      );
      pending.register_uri_scheme_protocol("tauri", move |webview_id, request, responder| {
        protocol(webview_id, request, UriSchemeResponder(responder))
      });
      registered_scheme_protocols.push("tauri".into());
    }

    if !registered_scheme_protocols.contains(&"ipc".into()) {
      let protocol = crate::ipc::protocol::get(manager.manager_owned());
      pending.register_uri_scheme_protocol("ipc", move |webview_id, request, responder| {
        protocol(webview_id, request, UriSchemeResponder(responder))
      });
      registered_scheme_protocols.push("ipc".into());
    }

    let label = pending.label.clone();
    let app_manager_ = manager.manager_owned();
    let on_page_load_handler = pending.on_page_load_handler.take();
    pending
      .on_page_load_handler
      .replace(Box::new(move |url, event| {
        let payload = PageLoadPayload { url: &url, event };

        if let Some(w) = app_manager_.get_webview(&label) {
          if let Some(on_page_load) = &app_manager_.webview.on_page_load {
            on_page_load(&w, &payload);
          }

          app_manager_
            .plugins
            .lock()
            .unwrap()
            .on_page_load(&w, &payload);
        }

        if let Some(handler) = &on_page_load_handler {
          handler(url, event);
        }
      }));

    #[cfg(feature = "protocol-asset")]
    if !registered_scheme_protocols.contains(&"asset".into()) {
      let asset_scope = app_manager
        .state()
        .get::<crate::Scopes>()
        .asset_protocol
        .clone();
      let protocol = crate::protocol::asset::get(asset_scope, window_origin.clone());
      pending.register_uri_scheme_protocol("asset", move |webview_id, request, responder| {
        protocol(webview_id, request, UriSchemeResponder(responder))
      });
    }

    #[cfg(feature = "isolation")]
    if let crate::Pattern::Isolation {
      assets,
      schema,
      key: _,
      crypto_keys,
    } = &*app_manager.pattern
    {
      let protocol = crate::protocol::isolation::get(
        manager.manager_owned(),
        schema,
        assets.clone(),
        *crypto_keys.aes_gcm().raw(),
        window_origin,
        use_https_scheme,
      );
      pending.register_uri_scheme_protocol(schema, move |webview_id, request, responder| {
        protocol(webview_id, request, UriSchemeResponder(responder))
      });
    }

    Ok(pending)
  }

  fn initialization_script(
    &self,
    app_manager: &AppManager<R>,
    ipc_script: &str,
    pattern_script: &str,
    use_https_scheme: bool,
  ) -> crate::Result<String> {
    #[derive(Template)]
    #[default_template("../../scripts/init.js")]
    struct InitJavascript<'a> {
      #[raw]
      pattern_script: &'a str,
      #[raw]
      ipc_script: &'a str,
      #[raw]
      core_script: &'a str,
      #[raw]
      event_initialization_script: &'a str,
      #[raw]
      freeze_prototype: &'a str,
    }

    #[derive(Template)]
    #[default_template("../../scripts/core.js")]
    struct CoreJavascript<'a> {
      os_name: &'a str,
      protocol_scheme: &'a str,
      invoke_key: &'a str,
    }

    let freeze_prototype = if app_manager.config.app.security.freeze_prototype {
      include_str!("../../scripts/freeze_prototype.js")
    } else {
      ""
    };

    InitJavascript {
      pattern_script,
      ipc_script,
      core_script: &CoreJavascript {
        os_name: std::env::consts::OS,
        protocol_scheme: if use_https_scheme { "https" } else { "http" },
        invoke_key: self.invoke_key(),
      }
      .render_default(&Default::default())?
      .into_string(),
      event_initialization_script: &crate::event::event_initialization_script(
        app_manager.listeners().function_name(),
        app_manager.listeners().listeners_object_name(),
      ),
      freeze_prototype,
    }
    .render_default(&Default::default())
    .map(|s| s.into_string())
    .map_err(Into::into)
  }

  pub fn prepare_webview<M: Manager<R>>(
    &self,
    manager: &M,
    mut pending: PendingWebview<EventLoopMessage, R>,
    window_label: &str,
  ) -> crate::Result<PendingWebview<EventLoopMessage, R>> {
    if self.webviews_lock().contains_key(&pending.label) {
      return Err(crate::Error::WebviewLabelAlreadyExists(pending.label));
    }

    let app_manager = manager.manager();

    #[allow(unused_mut)] // mut url only for the data-url parsing
    let mut url = match &pending.webview_attributes.url {
      WebviewUrl::App(path) => {
        let app_url = app_manager.get_url(pending.webview_attributes.use_https_scheme);
        let url = if PROXY_DEV_SERVER && is_local_network_url(&app_url) {
          Cow::Owned(Url::parse("tauri://localhost").unwrap())
        } else {
          app_url
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
      WebviewUrl::External(url) => {
        let config_url = app_manager.get_url(pending.webview_attributes.use_https_scheme);
        let is_app_url = config_url.make_relative(url).is_some();
        let mut url = url.clone();
        if is_app_url && PROXY_DEV_SERVER && is_local_network_url(&url) {
          Url::parse("tauri://localhost").unwrap()
        } else {
          url
        }
      }

      WebviewUrl::CustomProtocol(url) => url.clone(),
      _ => unimplemented!(),
    };

    #[cfg(not(feature = "webview-data-url"))]
    if url.scheme() == "data" {
      return Err(crate::Error::InvalidWebviewUrl(
        "data URLs are not supported without the `webview-data-url` feature.",
      ));
    }

    #[cfg(feature = "webview-data-url")]
    if let Some(csp) = app_manager.csp() {
      if url.scheme() == "data" {
        if let Ok(data_url) = data_url::DataUrl::process(url.as_str()) {
          let (body, _) = data_url.decode_to_vec().unwrap();
          let html = String::from_utf8_lossy(&body).into_owned();
          // naive way to check if it's an html
          if html.contains('<') && html.contains('>') {
            let document = tauri_utils::html::parse(html);
            tauri_utils::html::inject_csp(&document, &csp.to_string());
            url.set_path(&format!("{},{document}", mime::TEXT_HTML));
          }
        }
      }
    }

    pending.url = url.to_string();

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
    pending = self.prepare_pending_webview(pending, &label, window_label, manager)?;

    pending.ipc_handler = Some(crate::ipc::protocol::message_handler(
      manager.manager_owned(),
    ));

    // in `windows`, we need to force a data_directory
    // but we do respect user-specification
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    if pending.webview_attributes.data_directory.is_none() {
      let local_app_data = manager.path().resolve(
        &app_manager.config.identifier,
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

    #[cfg(all(desktop, not(target_os = "windows")))]
    if pending.webview_attributes.zoom_hotkeys_enabled {
      #[derive(Template)]
      #[default_template("../webview/scripts/zoom-hotkey.js")]
      struct HotkeyZoom<'a> {
        os_name: &'a str,
      }

      pending
        .webview_attributes
        .initialization_scripts
        .push(InitializationScript {
          script: HotkeyZoom {
            os_name: std::env::consts::OS,
          }
          .render_default(&Default::default())?
          .into_string(),
          for_main_frame_only: true,
        })
    }

    #[cfg(feature = "isolation")]
    let pattern = app_manager.pattern.clone();
    let navigation_handler = pending.navigation_handler.take();
    let app_manager = manager.manager_owned();
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
      let webview = app_manager.webview.webviews_lock().get(&label).cloned();
      if let Some(w) = webview {
        app_manager
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

  pub(crate) fn attach_webview(
    &self,
    window: Window<R>,
    webview: DetachedWebview<EventLoopMessage, R>,
    use_https_scheme: bool,
  ) -> Webview<R> {
    let webview = Webview::new(window, webview, use_https_scheme);

    let webview_event_listeners = self.event_listeners.clone();
    let webview_ = webview.clone();
    webview.on_webview_event(move |event| {
      let _ = on_webview_event(&webview_, event);
      for handler in webview_event_listeners.iter() {
        handler(&webview_, event);
      }
    });

    // insert the webview into our manager
    {
      self
        .webviews_lock()
        .insert(webview.label().to_string(), webview.clone());
    }

    // let plugins know that a new webview has been added to the manager
    let manager = webview.manager_owned();
    let webview_ = webview.clone();
    // run on main thread so the plugin store doesn't dead lock with the event loop handler in App
    let _ = webview.run_on_main_thread(move || {
      manager
        .plugins
        .lock()
        .expect("poisoned plugin store")
        .webview_created(webview_);
    });

    #[cfg(all(target_os = "ios", feature = "wry"))]
    {
      webview
        .with_webview(|w| {
          unsafe { crate::ios::on_webview_created(w.inner() as _, w.view_controller() as _) };
        })
        .expect("failed to run on_webview_created hook");
    }

    let event = crate::EventName::from_str("tauri://webview-created");
    let payload = Some(crate::webview::CreatedEvent {
      label: webview.label().into(),
    });

    let _ = webview
      .manager
      .emit(event, EmitPayload::Serialize(&payload));

    webview
  }

  pub fn eval_script_all<S: Into<String>>(&self, script: S) -> crate::Result<()> {
    let script = script.into();
    let webviews = self.webviews_lock().values().cloned().collect::<Vec<_>>();
    webviews
      .iter()
      .try_for_each(|webview| webview.eval(&script))
  }

  pub fn labels(&self) -> HashSet<String> {
    self.webviews_lock().keys().cloned().collect()
  }

  pub(crate) fn invoke_key(&self) -> &str {
    &self.invoke_key
  }
}

impl<R: Runtime> Webview<R> {
  /// Emits event to [`EventTarget::Window`] and [`EventTarget::WebviewWindow`]
  fn emit_to_webview<S: Serialize>(
    &self,
    event: crate::EventName<&str>,
    payload: &S,
  ) -> crate::Result<()> {
    let window_label = self.label();
    let payload = EmitPayload::Serialize(payload);
    self
      .manager()
      .emit_filter(event, payload, |target| match target {
        EventTarget::Webview { label } | EventTarget::WebviewWindow { label } => {
          label == window_label
        }
        _ => false,
      })
  }
}

fn on_webview_event<R: Runtime>(webview: &Webview<R>, event: &WebviewEvent) -> crate::Result<()> {
  match event {
    WebviewEvent::DragDrop(event) => match event {
      DragDropEvent::Enter { paths, position } => {
        let payload = DragDropPayload {
          paths: Some(paths),
          position,
        };
        webview.emit_to_webview(DRAG_ENTER_EVENT, &payload)?
      }
      DragDropEvent::Over { position } => {
        let payload = DragDropPayload {
          position,
          paths: None,
        };
        webview.emit_to_webview(DRAG_OVER_EVENT, &payload)?
      }
      DragDropEvent::Drop { paths, position } => {
        let scopes = webview.state::<Scopes>();
        for path in paths {
          if path.is_file() {
            let _ = scopes.allow_file(path);
          } else {
            let _ = scopes.allow_directory(path, false);
          }
        }
        let payload = DragDropPayload {
          paths: Some(paths),
          position,
        };
        webview.emit_to_webview(DRAG_DROP_EVENT, &payload)?
      }
      DragDropEvent::Leave => webview.emit_to_webview(DRAG_LEAVE_EVENT, &())?,
      _ => unimplemented!(),
    },
  }

  Ok(())
}

fn is_local_network_url(url: &url::Url) -> bool {
  match url.host() {
    Some(url::Host::Domain(s)) => s == "localhost",
    Some(url::Host::Ipv4(_)) | Some(url::Host::Ipv6(_)) => true,
    None => false,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn local_network_url() {
    assert!(is_local_network_url(&"http://localhost".parse().unwrap()));
    assert!(is_local_network_url(
      &"http://127.0.0.1:8080".parse().unwrap()
    ));
    assert!(is_local_network_url(
      &"https://192.168.3.17".parse().unwrap()
    ));

    assert!(!is_local_network_url(&"https://tauri.app".parse().unwrap()));
  }
}
