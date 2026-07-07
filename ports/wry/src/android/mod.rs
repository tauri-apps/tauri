// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{PageLoadEvent, WebViewAttributes, RGBA};
use crate::{
  custom_protocol_workaround, inject_initialization_scripts::inject_scripts_into_html, Error,
  PermissionKind, PermissionResponse, RequestAsyncResponder, Result,
};
use crossbeam_channel::*;

use http::{Request, Response as HttpResponse};
use jni::{
  errors::Result as JniResult,
  objects::{GlobalRef, JClass, JObject},
  JNIEnv,
};
use ndk::looper::ThreadLooper;
use once_cell::sync::{Lazy, OnceCell};
use raw_window_handle::HasWindowHandle;
use std::{
  borrow::Cow,
  collections::HashMap,
  sync::{mpsc::channel, Arc, Mutex},
  time::Duration,
};

pub(crate) mod binding;
mod main_pipe;
use main_pipe::{
  activity_id_for_window_manager, first_activity_id, register_activity_proxy, ActivityId,
  CreateWebViewAttributes, MainPipe, WebViewMessage,
};

use crate::util::Counter;

static COUNTER: Counter = Counter::new();
const MAIN_PIPE_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Context<'a, 'b> {
  pub env: &'a mut JNIEnv<'b>,
  pub activity: &'a JObject<'b>,
  pub webview: &'a JObject<'b>,
}

type WebviewId = String;

macro_rules! define_static_handlers {
  ($($key: ident, $var:ident = $type_name:ident);+ $(;)?) => {
    $(static $var: Lazy<Mutex<HashMap<$key, $type_name>>> = Lazy::new(||Mutex::new(HashMap::new()));)*
  };

  ($($var:ident = $type_name:ident { $($fields:ident:$types:ty),+ $(,)? });+ $(;)?) => {
    $(
    static $var: Lazy<Mutex<HashMap<WebviewId, $type_name>>> = Lazy::new(||Mutex::new(HashMap::new()));
    struct $type_name {
      $($fields: $types,)*
    }
    impl $type_name {
      fn new($($fields: $types,)*) -> Self {
        Self {
          $($fields,)*
        }
      }
    }
    unsafe impl Send for $type_name {}
    unsafe impl Sync for $type_name {})*
  };
}

define_static_handlers! {
  IPC = UnsafeIpc { handler: Box<dyn Fn(Request<String>)> };
  REQUEST_HANDLER = UnsafeRequestHandler { handler: Arc<dyn Fn(&str, Request<Vec<u8>>, bool) -> Option<HttpResponse<Cow<'static, [u8]>>> + Send + Sync> };
  TITLE_CHANGE_HANDLER = UnsafeTitleHandler { handler: Box<dyn Fn(String)> };
  URL_LOADING_OVERRIDE = UnsafeUrlLoadingOverride { handler: Box<dyn Fn(String) -> bool> };
  ON_LOAD_HANDLER = UnsafeOnPageLoadHandler { handler: Box<dyn Fn(PageLoadEvent, String)> };
  PERMISSION_HANDLER = UnsafePermissionHandler { handler: Box<dyn Fn(PermissionKind) -> PermissionResponse> };
}
define_static_handlers! {
  WebviewId, WITH_ASSET_LOADER = bool;
  WebviewId, ASSET_LOADER_DOMAIN = String;
  ActivityId, WEBVIEW_ATTRIBUTES = CreateWebViewAttributes;
}

static PACKAGE: OnceCell<String> = OnceCell::new();

type EvalCallback = Box<dyn Fn(String) + Send + 'static>;

static EVAL_ID_GENERATOR: Counter = Counter::new();
static EVAL_CALLBACKS: OnceCell<Mutex<HashMap<i32, EvalCallback>>> = OnceCell::new();

pub fn destroy_webview(activity_id: ActivityId, webview_id: &WebviewId) {
  WEBVIEW_ATTRIBUTES.lock().unwrap().remove(&activity_id);
  IPC.lock().unwrap().remove(webview_id);
  REQUEST_HANDLER.lock().unwrap().remove(webview_id);
  TITLE_CHANGE_HANDLER.lock().unwrap().remove(webview_id);
  URL_LOADING_OVERRIDE.lock().unwrap().remove(webview_id);
  ON_LOAD_HANDLER.lock().unwrap().remove(webview_id);
  PERMISSION_HANDLER.lock().unwrap().remove(webview_id);
  WITH_ASSET_LOADER.lock().unwrap().remove(webview_id);
  ASSET_LOADER_DOMAIN.lock().unwrap().remove(webview_id);
}

/// Sets up the necessary logic for wry to be able to create the webviews later.
///
/// This function must be run on the thread where the [`JNIEnv`] is registered and the looper is local,
/// hence the requirement for a [`ThreadLooper`].
pub unsafe fn android_setup(
  package: &str,
  mut env: JNIEnv,
  _looper: &ThreadLooper,
  activity: GlobalRef,
) {
  PACKAGE.get_or_init(|| package.to_string());

  let vm = env.get_java_vm().unwrap();

  let activity_id = env
    .call_method(activity.as_obj(), "getId", "()I", &[])
    .unwrap()
    .i()
    .unwrap();

  let window_manager = env
    .call_method(
      &activity,
      "getWindowManager",
      "()Landroid/view/WindowManager;",
      &[],
    )
    .unwrap()
    .l()
    .unwrap();
  let window_manager = env.new_global_ref(window_manager).unwrap();

  register_activity_proxy(vm, activity_id, activity, window_manager);

  if let Some(webview_attributes) = WEBVIEW_ATTRIBUTES.lock().unwrap().get(&activity_id) {
    MainPipe::send(
      activity_id,
      WebViewMessage::CreateWebView(webview_attributes.clone()),
    );
  }
}

pub(crate) struct InnerWebView {
  id: String,
  pub activity_id: ActivityId,
}

impl InnerWebView {
  pub fn new_as_child(
    window: &impl HasWindowHandle,
    attributes: WebViewAttributes,
    pl_attrs: super::PlatformSpecificWebViewAttributes,
  ) -> Result<Self> {
    Self::new(window, attributes, pl_attrs)
  }

  pub fn new(
    window: &impl HasWindowHandle,
    attributes: WebViewAttributes,
    pl_attrs: super::PlatformSpecificWebViewAttributes,
  ) -> Result<Self> {
    let window_manager = match window.window_handle()?.as_raw() {
      raw_window_handle::RawWindowHandle::AndroidNdk(window_manager) => {
        window_manager.a_native_window
      }
      _ => return Err(Error::UnsupportedWindowHandle),
    };
    let window_manager = unsafe { JObject::from_raw(window_manager.as_ptr().cast()) };
    let activity_id =
      activity_id_for_window_manager(window_manager).expect("no available activity");
    let WebViewAttributes {
      url,
      html,
      initialization_scripts,
      ipc_handler,
      #[cfg(any(debug_assertions, feature = "devtools"))]
      devtools,
      custom_protocols,
      background_color,
      transparent,
      headers,
      autoplay,
      user_agent,
      javascript_disabled,
      ..
    } = attributes;

    let super::PlatformSpecificWebViewAttributes {
      on_webview_created,
      with_asset_loader,
      asset_loader_domain,
      https_scheme,
    } = pl_attrs;

    let http_or_https = if https_scheme { "https" } else { "http" };

    let url = if let Some(mut url) = url {
      if let Some((protocol, _)) = url.split_once("://") {
        if custom_protocols.contains_key(protocol) {
          url = custom_protocol_workaround::apply_uri_work_around(&url, http_or_https, protocol)
        }
      }

      Some(url)
    } else {
      None
    };

    let id = attributes
      .id
      .map(|id| id.to_string())
      .unwrap_or_else(|| COUNTER.next().to_string());

    WITH_ASSET_LOADER
      .lock()
      .unwrap()
      .insert(id.clone(), with_asset_loader);
    if let Some(domain) = asset_loader_domain {
      ASSET_LOADER_DOMAIN
        .lock()
        .unwrap()
        .insert(id.clone(), domain);
    }

    let initialization_scripts_ = initialization_scripts.clone();
    REQUEST_HANDLER.lock().unwrap().insert(
      id.clone(),
      UnsafeRequestHandler::new(Arc::new(
        move |webview_id, mut request, is_document_start_script_enabled| {
          let uri = request.uri().to_string();
          let (custom_protocol, custom_protocol_handler) =
            custom_protocols.iter().find(|(protocol, _)| {
              custom_protocol_workaround::is_work_around_uri(&uri, http_or_https, protocol)
            })?;

          let uri_res = custom_protocol_workaround::revert_uri_work_around(
            &uri,
            http_or_https,
            custom_protocol,
          )
          .parse();

          if let Ok(uri) = uri_res {
            *request.uri_mut() = uri;
          }

          let (tx, rx) = channel();
          let initialization_scripts = initialization_scripts_.clone();
          let responder: Box<dyn FnOnce(HttpResponse<Cow<'static, [u8]>>)> = Box::new(
            move |mut response| {
              if !is_document_start_script_enabled {
                #[cfg(feature = "tracing")]
                tracing::info!("`addDocumentStartJavaScript` is not supported; injecting initialization scripts via custom protocol handler");
                response = inject_scripts_into_html(response, &initialization_scripts);
              }
              let _ = tx.send(response);
            },
          );

          (custom_protocol_handler)(webview_id, request, RequestAsyncResponder { responder });
          // 3x the timeout while we monitor https://github.com/tauri-apps/wry/issues/1551
          // TODO: Remove timeout
          rx.recv_timeout(MAIN_PIPE_TIMEOUT * 3)
            .inspect_err(|e| {
              eprintln!("custom protocol timed out: {e}");
            })
            .ok()
        },
      )),
    );

    if let Some(i) = ipc_handler {
      IPC
        .lock()
        .unwrap()
        .insert(id.clone(), UnsafeIpc::new(Box::new(i)));
    }

    if let Some(i) = attributes.document_title_changed_handler {
      TITLE_CHANGE_HANDLER
        .lock()
        .unwrap()
        .insert(id.clone(), UnsafeTitleHandler::new(i));
    }

    if let Some(i) = attributes.navigation_handler {
      URL_LOADING_OVERRIDE
        .lock()
        .unwrap()
        .insert(id.clone(), UnsafeUrlLoadingOverride::new(i));
    }

    if let Some(h) = attributes.on_page_load_handler {
      ON_LOAD_HANDLER
        .lock()
        .unwrap()
        .insert(id.clone(), UnsafeOnPageLoadHandler::new(h));
    }

    if let Some(permission_handler) = attributes.permission_handler {
      let permission_handler: Box<dyn Fn(PermissionKind) -> PermissionResponse> =
        permission_handler;
      PERMISSION_HANDLER
        .lock()
        .unwrap()
        .insert(id.clone(), UnsafePermissionHandler::new(permission_handler));
    }

    let attributes = CreateWebViewAttributes {
      id: id.clone(),
      url,
      html,
      #[cfg(any(debug_assertions, feature = "devtools"))]
      devtools,
      background_color,
      transparent,
      headers,
      on_webview_created,
      autoplay,
      user_agent,
      initialization_scripts,
      javascript_disabled,
    };

    WEBVIEW_ATTRIBUTES
      .lock()
      .unwrap()
      .insert(activity_id, attributes.clone());

    MainPipe::send(activity_id, WebViewMessage::CreateWebView(attributes));

    Ok(Self { id, activity_id })
  }

  pub fn print(&self) -> crate::Result<()> {
    Ok(())
  }

  pub fn id(&self) -> crate::WebViewId<'_> {
    &self.id
  }

  pub fn url(&self) -> crate::Result<String> {
    let (tx, rx) = bounded(1);
    MainPipe::send(self.activity_id, WebViewMessage::GetUrl(tx));
    rx.recv_timeout(MAIN_PIPE_TIMEOUT).map_err(Into::into)
  }

  pub fn eval(&self, js: &str, callback: Option<impl Fn(String) + Send + 'static>) -> Result<()> {
    MainPipe::send(
      self.activity_id,
      WebViewMessage::Eval(
        js.into(),
        callback.map(|c| Box::new(c) as Box<dyn Fn(String) + Send + 'static>),
      ),
    );
    Ok(())
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  pub fn open_devtools(&self) {}

  #[cfg(any(debug_assertions, feature = "devtools"))]
  pub fn close_devtools(&self) {}

  #[cfg(any(debug_assertions, feature = "devtools"))]
  pub fn is_devtools_open(&self) -> bool {
    false
  }

  pub fn zoom(&self, _scale_factor: f64) -> Result<()> {
    Ok(())
  }

  pub fn set_background_color(&self, background_color: RGBA) -> Result<()> {
    MainPipe::send(
      self.activity_id,
      WebViewMessage::SetBackgroundColor(background_color),
    );
    Ok(())
  }

  pub fn load_url(&self, url: &str) -> Result<()> {
    MainPipe::send(
      self.activity_id,
      WebViewMessage::LoadUrl(url.to_string(), None),
    );
    Ok(())
  }

  pub fn load_url_with_headers(&self, url: &str, headers: http::HeaderMap) -> Result<()> {
    MainPipe::send(
      self.activity_id,
      WebViewMessage::LoadUrl(url.to_string(), Some(headers)),
    );
    Ok(())
  }

  pub fn load_html(&self, html: &str) -> Result<()> {
    MainPipe::send(self.activity_id, WebViewMessage::LoadHtml(html.to_string()));
    Ok(())
  }

  pub fn reload(&self) -> Result<()> {
    MainPipe::send(self.activity_id, WebViewMessage::Reload);
    Ok(())
  }

  pub fn go_forward(&self) -> Result<()> {
    MainPipe::send(self.activity_id, WebViewMessage::GoForward);
    Ok(())
  }

  pub fn go_back(&self) -> Result<()> {
    MainPipe::send(self.activity_id, WebViewMessage::GoBack);
    Ok(())
  }

  pub fn can_go_forward(&self) -> Result<bool> {
    let (tx, rx) = bounded(1);
    MainPipe::send(self.activity_id, WebViewMessage::CanGoForward(tx));
    rx.recv_timeout(MAIN_PIPE_TIMEOUT).map_err(Into::into)
  }

  pub fn can_go_back(&self) -> Result<bool> {
    let (tx, rx) = bounded(1);
    MainPipe::send(self.activity_id, WebViewMessage::CanGoBack(tx));
    rx.recv_timeout(MAIN_PIPE_TIMEOUT).map_err(Into::into)
  }

  pub fn clear_all_browsing_data(&self) -> Result<()> {
    MainPipe::send(self.activity_id, WebViewMessage::ClearAllBrowsingData);
    Ok(())
  }

  pub fn cookies_for_url(&self, url: &str) -> Result<Vec<cookie::Cookie<'static>>> {
    let (tx, rx) = bounded(1);
    MainPipe::send(
      self.activity_id,
      WebViewMessage::GetCookies(tx, url.to_string()),
    );
    rx.recv_timeout(MAIN_PIPE_TIMEOUT).map_err(Into::into)
  }

  pub fn set_cookie(&self, #[allow(unused)] cookie: &cookie::Cookie<'_>) -> Result<()> {
    // Unsupported
    Ok(())
  }

  pub fn delete_cookie(&self, #[allow(unused)] cookie: &cookie::Cookie<'_>) -> Result<()> {
    // Unsupported
    Ok(())
  }

  pub fn cookies(&self) -> Result<Vec<cookie::Cookie<'static>>> {
    Ok(Vec::new())
  }

  pub fn bounds(&self) -> Result<crate::Rect> {
    Ok(crate::Rect::default())
  }

  pub fn set_bounds(&self, _bounds: crate::Rect) -> Result<()> {
    // Unsupported
    Ok(())
  }

  pub fn set_visible(&self, _visible: bool) -> Result<()> {
    // Unsupported
    Ok(())
  }

  pub fn focus(&self) -> Result<()> {
    // Unsupported
    Ok(())
  }

  pub fn focus_parent(&self) -> Result<()> {
    // Unsupported
    Ok(())
  }
}

#[derive(Clone, Copy)]
pub struct JniHandle {
  pub(crate) activity_id: ActivityId,
}

impl JniHandle {
  /// Execute jni code on the thread of the webview.
  /// Provided function will be provided with the jni evironment, Android activity and WebView
  pub fn exec<F>(&self, func: F)
  where
    F: FnOnce(&mut JNIEnv, &JObject, &JObject) + Send + 'static,
  {
    MainPipe::send(self.activity_id, WebViewMessage::Jni(Box::new(func)));
  }
}

pub fn platform_webview_version() -> Result<String> {
  let (tx, rx) = bounded(1);
  let activity_id = loop {
    match first_activity_id() {
      Some(id) => break id,
      None => {
        std::thread::sleep(Duration::from_millis(100));
      }
    }
  };
  MainPipe::send(activity_id, WebViewMessage::GetWebViewVersion(tx));
  rx.recv_timeout(MAIN_PIPE_TIMEOUT)?
}

/// Finds a class in the project scope.
pub fn find_class<'a>(
  env: &mut JNIEnv<'a>,
  activity: &JObject<'_>,
  name: String,
) -> JniResult<JClass<'a>> {
  let class_name = env.new_string(name.replace('/', "."))?;
  let my_class = env
    .call_method(
      activity,
      "getAppClass",
      "(Ljava/lang/String;)Ljava/lang/Class;",
      &[(&class_name).into()],
    )?
    .l()?;
  Ok(my_class.into())
}

/// Dispatch a closure to run on the Android context.
///
/// The closure takes the JNI env, the Android activity instance and the possibly null webview.
pub fn dispatch<F>(func: F)
where
  F: FnOnce(&mut JNIEnv, &JObject, &JObject) + Send + 'static,
{
  MainPipe::send(
    first_activity_id().expect("no available activity"),
    WebViewMessage::Jni(Box::new(func)),
  );
}
