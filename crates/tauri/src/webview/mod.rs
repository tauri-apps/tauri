// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri webview types and functions.

pub(crate) mod plugin;
mod webview_window;

pub use webview_window::{WebviewWindow, WebviewWindowBuilder};

use http::HeaderMap;
use serde::Serialize;
use tauri_macros::default_runtime;
pub use tauri_runtime::webview::PageLoadEvent;
pub use tauri_runtime::Cookie;
#[cfg(desktop)]
use tauri_runtime::{
  dpi::{PhysicalPosition, PhysicalSize, Position, Size},
  WindowDispatch,
};
use tauri_runtime::{
  webview::{DetachedWebview, PendingWebview, WebviewAttributes},
  WebviewDispatch,
};
pub use tauri_utils::config::Color;
use tauri_utils::config::{BackgroundThrottlingPolicy, WebviewUrl, WindowConfig};
pub use url::Url;

use crate::{
  app::{UriSchemeResponder, WebviewEvent},
  event::{EmitArgs, EventTarget},
  ipc::{
    CallbackFn, CommandArg, CommandItem, CommandScope, GlobalScope, Invoke, InvokeBody,
    InvokeError, InvokeMessage, InvokeResolver, Origin, OwnedInvokeResponder, ScopeObject,
  },
  manager::AppManager,
  sealed::{ManagerBase, RuntimeOrDispatch},
  AppHandle, Emitter, Event, EventId, EventLoopMessage, EventName, Listener, Manager,
  ResourceTable, Runtime, Window,
};

use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  path::{Path, PathBuf},
  sync::{Arc, Mutex, MutexGuard},
};

pub(crate) type WebResourceRequestHandler =
  dyn Fn(http::Request<Vec<u8>>, &mut http::Response<Cow<'static, [u8]>>) + Send + Sync;
pub(crate) type NavigationHandler = dyn Fn(&Url) -> bool + Send;
pub(crate) type UriSchemeProtocolHandler =
  Box<dyn Fn(&str, http::Request<Vec<u8>>, UriSchemeResponder) + Send + Sync>;
pub(crate) type OnPageLoad<R> = dyn Fn(Webview<R>, PageLoadPayload<'_>) + Send + Sync + 'static;

pub(crate) type DownloadHandler<R> = dyn Fn(Webview<R>, DownloadEvent<'_>) -> bool + Send + Sync;

#[derive(Clone, Serialize)]
pub(crate) struct CreatedEvent {
  pub(crate) label: String,
}

/// Download event for the [`WebviewBuilder#method.on_download`] hook.
#[non_exhaustive]
pub enum DownloadEvent<'a> {
  /// Download requested.
  Requested {
    /// The url being downloaded.
    url: Url,
    /// Represents where the file will be downloaded to.
    /// Can be used to set the download location by assigning a new path to it.
    /// The assigned path _must_ be absolute.
    destination: &'a mut PathBuf,
  },
  /// Download finished.
  Finished {
    /// The URL of the original download request.
    url: Url,
    /// Potentially representing the filesystem path the file was downloaded to.
    ///
    /// A value of `None` being passed instead of a `PathBuf` does not necessarily indicate that the download
    /// did not succeed, and may instead indicate some other failure - always check the third parameter if you need to
    /// know if the download succeeded.
    ///
    /// ## Platform-specific:
    ///
    /// - **macOS**: The second parameter indicating the path the file was saved to is always empty, due to API
    ///   limitations.
    path: Option<PathBuf>,
    /// Indicates if the download succeeded or not.
    success: bool,
  },
}

/// The payload for the [`WebviewBuilder::on_page_load`] hook.
#[derive(Debug, Clone)]
pub struct PageLoadPayload<'a> {
  pub(crate) url: &'a Url,
  pub(crate) event: PageLoadEvent,
}

impl<'a> PageLoadPayload<'a> {
  /// The page URL.
  pub fn url(&self) -> &'a Url {
    self.url
  }

  /// The page load event.
  pub fn event(&self) -> PageLoadEvent {
    self.event
  }
}

/// The IPC invoke request.
///
/// # Stability
///
/// This struct is **NOT** part of the public stable API and is only meant to be used
/// by internal code and external testing/fuzzing tools or custom invoke systems.
#[derive(Debug)]
pub struct InvokeRequest {
  /// The invoke command.
  pub cmd: String,
  /// The success callback.
  pub callback: CallbackFn,
  /// The error callback.
  pub error: CallbackFn,
  /// URL of the frame that requested this command.
  pub url: Url,
  /// The body of the request.
  pub body: InvokeBody,
  /// The request headers.
  pub headers: HeaderMap,
  /// The invoke key. Must match what was passed to the app manager.
  pub invoke_key: String,
}

/// The platform webview handle. Accessed with [`Webview#method.with_webview`];
#[cfg(feature = "wry")]
#[cfg_attr(docsrs, doc(cfg(feature = "wry")))]
pub struct PlatformWebview(tauri_runtime_wry::Webview);

#[cfg(feature = "wry")]
impl PlatformWebview {
  /// Returns [`webkit2gtk::WebView`] handle.
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  #[cfg_attr(
    docsrs,
    doc(cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "netbsd",
      target_os = "openbsd"
    )))
  )]
  pub fn inner(&self) -> webkit2gtk::WebView {
    self.0.clone()
  }

  /// Returns the WebView2 controller.
  #[cfg(windows)]
  #[cfg_attr(docsrs, doc(cfg(windows)))]
  pub fn controller(
    &self,
  ) -> webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller {
    self.0.controller.clone()
  }

  /// Returns the [WKWebView] handle.
  ///
  /// [WKWebView]: https://developer.apple.com/documentation/webkit/wkwebview
  #[cfg(any(target_os = "macos", target_os = "ios"))]
  #[cfg_attr(docsrs, doc(cfg(any(target_os = "macos", target_os = "ios"))))]
  pub fn inner(&self) -> *mut std::ffi::c_void {
    self.0.webview
  }

  /// Returns WKWebView [controller] handle.
  ///
  /// [controller]: https://developer.apple.com/documentation/webkit/wkusercontentcontroller
  #[cfg(any(target_os = "macos", target_os = "ios"))]
  #[cfg_attr(docsrs, doc(cfg(any(target_os = "macos", target_os = "ios"))))]
  pub fn controller(&self) -> *mut std::ffi::c_void {
    self.0.manager
  }

  /// Returns [NSWindow] associated with the WKWebView webview.
  ///
  /// [NSWindow]: https://developer.apple.com/documentation/appkit/nswindow
  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  pub fn ns_window(&self) -> *mut std::ffi::c_void {
    self.0.ns_window
  }

  /// Returns [UIViewController] used by the WKWebView webview NSWindow.
  ///
  /// [UIViewController]: https://developer.apple.com/documentation/uikit/uiviewcontroller
  #[cfg(target_os = "ios")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "ios")))]
  pub fn view_controller(&self) -> *mut std::ffi::c_void {
    self.0.view_controller
  }

  /// Returns handle for JNI execution.
  #[cfg(target_os = "android")]
  pub fn jni_handle(&self) -> tauri_runtime_wry::wry::JniHandle {
    self.0
  }
}

macro_rules! unstable_struct {
    (#[doc = $doc:expr] $($tokens:tt)*) => {
      #[cfg(any(test, feature = "unstable"))]
      #[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
      #[doc = $doc]
      pub $($tokens)*

      #[cfg(not(any(test, feature = "unstable")))]
      pub(crate) $($tokens)*
    }
}

unstable_struct!(
  #[doc = "A builder for a webview."]
  struct WebviewBuilder<R: Runtime> {
    pub(crate) label: String,
    pub(crate) webview_attributes: WebviewAttributes,
    pub(crate) web_resource_request_handler: Option<Box<WebResourceRequestHandler>>,
    pub(crate) navigation_handler: Option<Box<NavigationHandler>>,
    pub(crate) on_page_load_handler: Option<Box<OnPageLoad<R>>>,
    pub(crate) download_handler: Option<Arc<DownloadHandler<R>>>,
  }
);

#[cfg_attr(not(feature = "unstable"), allow(dead_code))]
impl<R: Runtime> WebviewBuilder<R> {
  /// Initializes a webview builder with the given webview label and URL to load.
  ///
  /// # Known issues
  ///
  /// On Windows, this function deadlocks when used in a synchronous command or event handlers, see [the Webview2 issue].
  /// You should use `async` commands and separate threads when creating webviews.
  ///
  /// # Examples
  ///
  /// - Create a webview in the setup hook:
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
tauri::Builder::default()
  .setup(|app| {
    let window = tauri::window::WindowBuilder::new(app, "label").build()?;
    let webview_builder = tauri::webview::WebviewBuilder::new("label", tauri::WebviewUrl::App("index.html".into()));
    let webview = window.add_child(webview_builder, tauri::LogicalPosition::new(0, 0), window.inner_size().unwrap());
    Ok(())
  });
```
  "####
  )]
  ///
  /// - Create a webview in a separate thread:
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
tauri::Builder::default()
  .setup(|app| {
    let handle = app.handle().clone();
    std::thread::spawn(move || {
      let window = tauri::window::WindowBuilder::new(&handle, "label").build().unwrap();
      let webview_builder = tauri::webview::WebviewBuilder::new("label", tauri::WebviewUrl::App("index.html".into()));
      window.add_child(webview_builder, tauri::LogicalPosition::new(0, 0), window.inner_size().unwrap());
    });
    Ok(())
  });
```
   "####
  )]
  ///
  /// - Create a webview in a command:
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
#[tauri::command]
async fn create_window(app: tauri::AppHandle) {
  let window = tauri::window::WindowBuilder::new(&app, "label").build().unwrap();
  let webview_builder = tauri::webview::WebviewBuilder::new("label", tauri::WebviewUrl::External("https://tauri.app/".parse().unwrap()));
  window.add_child(webview_builder, tauri::LogicalPosition::new(0, 0), window.inner_size().unwrap());
}
```
  "####
  )]
  ///
  /// [the Webview2 issue]: https://github.com/tauri-apps/wry/issues/583
  pub fn new<L: Into<String>>(label: L, url: WebviewUrl) -> Self {
    Self {
      label: label.into(),
      webview_attributes: WebviewAttributes::new(url),
      web_resource_request_handler: None,
      navigation_handler: None,
      on_page_load_handler: None,
      download_handler: None,
    }
  }

  /// Initializes a webview builder from a [`WindowConfig`] from tauri.conf.json.
  /// Keep in mind that you can't create 2 webviews with the same `label` so make sure
  /// that the initial webview was closed or change the label of the new [`WebviewBuilder`].
  ///
  /// # Known issues
  ///
  /// On Windows, this function deadlocks when used in a synchronous command or event handlers, see [the Webview2 issue].
  /// You should use `async` commands and separate threads when creating webviews.
  ///
  /// # Examples
  ///
  /// - Create a webview in a command:
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
#[tauri::command]
async fn create_window(app: tauri::AppHandle) {
  let window = tauri::window::WindowBuilder::new(&app, "label").build().unwrap();
  let webview_builder = tauri::webview::WebviewBuilder::from_config(&app.config().app.windows.get(0).unwrap().clone());
  window.add_child(webview_builder, tauri::LogicalPosition::new(0, 0), window.inner_size().unwrap());
}
```
  "####
  )]
  ///
  /// [the Webview2 issue]: https://github.com/tauri-apps/wry/issues/583
  pub fn from_config(config: &WindowConfig) -> Self {
    Self {
      label: config.label.clone(),
      webview_attributes: WebviewAttributes::from(config),
      web_resource_request_handler: None,
      navigation_handler: None,
      on_page_load_handler: None,
      download_handler: None,
    }
  }

  /// Defines a closure to be executed when the webview makes an HTTP request for a web resource, allowing you to modify the response.
  ///
  /// Currently only implemented for the `tauri` URI protocol.
  ///
  /// **NOTE:** Currently this is **not** executed when using external URLs such as a development server,
  /// but it might be implemented in the future. **Always** check the request URL.
  ///
  /// # Examples
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```rust,no_run
use tauri::{
  utils::config::{Csp, CspDirectiveSources, WebviewUrl},
  window::WindowBuilder,
  webview::WebviewBuilder,
};
use http::header::HeaderValue;
use std::collections::HashMap;
tauri::Builder::default()
  .setup(|app| {
    let window = tauri::window::WindowBuilder::new(app, "label").build()?;

    let webview_builder = WebviewBuilder::new("core", WebviewUrl::App("index.html".into()))
      .on_web_resource_request(|request, response| {
        if request.uri().scheme_str() == Some("tauri") {
          // if we have a CSP header, Tauri is loading an HTML file
          //  for this example, let's dynamically change the CSP
          if let Some(csp) = response.headers_mut().get_mut("Content-Security-Policy") {
            // use the tauri helper to parse the CSP policy to a map
            let mut csp_map: HashMap<String, CspDirectiveSources> = Csp::Policy(csp.to_str().unwrap().to_string()).into();
            csp_map.entry("script-src".to_string()).or_insert_with(Default::default).push("'unsafe-inline'");
            // use the tauri helper to get a CSP string from the map
            let csp_string = Csp::from(csp_map).to_string();
            *csp = HeaderValue::from_str(&csp_string).unwrap();
          }
        }
      });

    let webview = window.add_child(webview_builder, tauri::LogicalPosition::new(0, 0), window.inner_size().unwrap())?;

    Ok(())
  });
```
  "####
  )]
  pub fn on_web_resource_request<
    F: Fn(http::Request<Vec<u8>>, &mut http::Response<Cow<'static, [u8]>>) + Send + Sync + 'static,
  >(
    mut self,
    f: F,
  ) -> Self {
    self.web_resource_request_handler.replace(Box::new(f));
    self
  }

  /// Defines a closure to be executed when the webview navigates to a URL. Returning `false` cancels the navigation.
  ///
  /// # Examples
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```rust,no_run
use tauri::{
  utils::config::{Csp, CspDirectiveSources, WebviewUrl},
  window::WindowBuilder,
  webview::WebviewBuilder,
};
use http::header::HeaderValue;
use std::collections::HashMap;
tauri::Builder::default()
  .setup(|app| {
    let window = tauri::window::WindowBuilder::new(app, "label").build()?;

    let webview_builder = WebviewBuilder::new("core", WebviewUrl::App("index.html".into()))
      .on_navigation(|url| {
        // allow the production URL or localhost on dev
        url.scheme() == "tauri" || (cfg!(dev) && url.host_str() == Some("localhost"))
      });

    let webview = window.add_child(webview_builder, tauri::LogicalPosition::new(0, 0), window.inner_size().unwrap())?;
    Ok(())
  });
```
  "####
  )]
  pub fn on_navigation<F: Fn(&Url) -> bool + Send + 'static>(mut self, f: F) -> Self {
    self.navigation_handler.replace(Box::new(f));
    self
  }

  /// Set a download event handler to be notified when a download is requested or finished.
  ///
  /// Returning `false` prevents the download from happening on a [`DownloadEvent::Requested`] event.
  ///
  /// # Examples
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```rust,no_run
use tauri::{
  utils::config::{Csp, CspDirectiveSources, WebviewUrl},
  window::WindowBuilder,
  webview::{DownloadEvent, WebviewBuilder},
};

tauri::Builder::default()
  .setup(|app| {
    let window = WindowBuilder::new(app, "label").build()?;
    let webview_builder = WebviewBuilder::new("core", WebviewUrl::App("index.html".into()))
      .on_download(|webview, event| {
        match event {
          DownloadEvent::Requested { url, destination } => {
            println!("downloading {}", url);
            *destination = "/home/tauri/target/path".into();
          }
          DownloadEvent::Finished { url, path, success } => {
            println!("downloaded {} to {:?}, success: {}", url, path, success);
          }
          _ => (),
        }
        // let the download start
        true
      });

    let webview = window.add_child(webview_builder, tauri::LogicalPosition::new(0, 0), window.inner_size().unwrap())?;
    Ok(())
  });
```
  "####
  )]
  pub fn on_download<F: Fn(Webview<R>, DownloadEvent<'_>) -> bool + Send + Sync + 'static>(
    mut self,
    f: F,
  ) -> Self {
    self.download_handler.replace(Arc::new(f));
    self
  }

  /// Defines a closure to be executed when a page load event is triggered.
  /// The event can be either [`PageLoadEvent::Started`] if the page has started loading
  /// or [`PageLoadEvent::Finished`] when the page finishes loading.
  ///
  /// # Examples
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```rust,no_run
use tauri::{
  utils::config::{Csp, CspDirectiveSources, WebviewUrl},
  window::WindowBuilder,
  webview::{PageLoadEvent, WebviewBuilder},
};
use http::header::HeaderValue;
use std::collections::HashMap;
tauri::Builder::default()
  .setup(|app| {
    let window = tauri::window::WindowBuilder::new(app, "label").build()?;
    let webview_builder = WebviewBuilder::new("core", WebviewUrl::App("index.html".into()))
      .on_page_load(|webview, payload| {
        match payload.event() {
          PageLoadEvent::Started => {
            println!("{} finished loading", payload.url());
          }
          PageLoadEvent::Finished => {
            println!("{} finished loading", payload.url());
          }
        }
      });
    let webview = window.add_child(webview_builder, tauri::LogicalPosition::new(0, 0), window.inner_size().unwrap())?;
    Ok(())
  });
```
  "####
  )]
  pub fn on_page_load<F: Fn(Webview<R>, PageLoadPayload<'_>) + Send + Sync + 'static>(
    mut self,
    f: F,
  ) -> Self {
    self.on_page_load_handler.replace(Box::new(f));
    self
  }

  pub(crate) fn into_pending_webview<M: Manager<R>>(
    mut self,
    manager: &M,
    window_label: &str,
  ) -> crate::Result<PendingWebview<EventLoopMessage, R>> {
    let mut pending = PendingWebview::new(self.webview_attributes, self.label.clone())?;
    pending.navigation_handler = self.navigation_handler.take();
    pending.web_resource_request_handler = self.web_resource_request_handler.take();

    if let Some(download_handler) = self.download_handler.take() {
      let label = pending.label.clone();
      let manager = manager.manager_owned();
      pending.download_handler.replace(Arc::new(move |event| {
        if let Some(w) = manager.get_webview(&label) {
          download_handler(
            w,
            match event {
              tauri_runtime::webview::DownloadEvent::Requested { url, destination } => {
                DownloadEvent::Requested { url, destination }
              }
              tauri_runtime::webview::DownloadEvent::Finished { url, path, success } => {
                DownloadEvent::Finished { url, path, success }
              }
            },
          )
        } else {
          false
        }
      }));
    }

    let label_ = pending.label.clone();
    let manager_ = manager.manager_owned();
    pending
      .on_page_load_handler
      .replace(Box::new(move |url, event| {
        if let Some(w) = manager_.get_webview(&label_) {
          if let Some(handler) = self.on_page_load_handler.as_ref() {
            handler(w, PageLoadPayload { url: &url, event });
          }
        }
      }));

    manager
      .manager()
      .webview
      .prepare_webview(manager, pending, window_label)
  }

  /// Creates a new webview on the given window.
  #[cfg(desktop)]
  pub(crate) fn build(
    self,
    window: Window<R>,
    position: Position,
    size: Size,
  ) -> crate::Result<Webview<R>> {
    let app_manager = window.manager();

    let mut pending = self.into_pending_webview(&window, window.label())?;

    pending.webview_attributes.bounds = Some(tauri_runtime::Rect { size, position });

    let use_https_scheme = pending.webview_attributes.use_https_scheme;

    let webview = match &mut window.runtime() {
      RuntimeOrDispatch::Dispatch(dispatcher) => dispatcher.create_webview(pending),
      _ => unimplemented!(),
    }
    .map(|webview| {
      app_manager
        .webview
        .attach_webview(window.clone(), webview, use_https_scheme)
    })?;

    Ok(webview)
  }
}

/// Webview attributes.
impl<R: Runtime> WebviewBuilder<R> {
  /// Sets whether clicking an inactive window also clicks through to the webview.
  #[must_use]
  pub fn accept_first_mouse(mut self, accept: bool) -> Self {
    self.webview_attributes.accept_first_mouse = accept;
    self
  }

  /// Adds the provided JavaScript to a list of scripts that should be run after the global object has been created,
  /// but before the HTML document has been parsed and before any other script included by the HTML document is run.
  ///
  /// Since it runs on all top-level document and child frame page navigations,
  /// it's recommended to check the `window.location` to guard your script from running on unexpected origins.
  ///
  /// # Examples
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```rust
use tauri::{WindowBuilder, Runtime};

const INIT_SCRIPT: &str = r#"
  if (window.location.origin === 'https://tauri.app') {
    console.log("hello world from js init script");

    window.__MY_CUSTOM_PROPERTY__ = { foo: 'bar' };
  }
"#;

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let window = tauri::window::WindowBuilder::new(app, "label").build()?;
      let webview_builder = tauri::webview::WebviewBuilder::new("label", tauri::WebviewUrl::App("index.html".into()))
        .initialization_script(INIT_SCRIPT);
      let webview = window.add_child(webview_builder, tauri::LogicalPosition::new(0, 0), window.inner_size().unwrap())?;
      Ok(())
    });
}
```
  "####
  )]
  #[must_use]
  pub fn initialization_script(mut self, script: &str) -> Self {
    self
      .webview_attributes
      .initialization_scripts
      .push(script.to_string());
    self
  }

  /// Set the user agent for the webview
  #[must_use]
  pub fn user_agent(mut self, user_agent: &str) -> Self {
    self.webview_attributes.user_agent = Some(user_agent.to_string());
    self
  }

  /// Set additional arguments for the webview.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS / Linux / Android / iOS**: Unsupported.
  ///
  /// ## Warning
  ///
  /// By default wry passes `--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection`
  /// so if you use this method, you also need to disable these components by yourself if you want.
  #[must_use]
  pub fn additional_browser_args(mut self, additional_args: &str) -> Self {
    self.webview_attributes.additional_browser_args = Some(additional_args.to_string());
    self
  }

  /// Data directory for the webview.
  #[must_use]
  pub fn data_directory(mut self, data_directory: PathBuf) -> Self {
    self
      .webview_attributes
      .data_directory
      .replace(data_directory);
    self
  }

  /// Disables the drag and drop handler. This is required to use HTML5 drag and drop APIs on the frontend on Windows.
  #[must_use]
  pub fn disable_drag_drop_handler(mut self) -> Self {
    self.webview_attributes.drag_drop_handler_enabled = false;
    self
  }

  /// Enables clipboard access for the page rendered on **Linux** and **Windows**.
  ///
  /// **macOS** doesn't provide such method and is always enabled by default,
  /// but you still need to add menu item accelerators to use shortcuts.
  #[must_use]
  pub fn enable_clipboard_access(mut self) -> Self {
    self.webview_attributes.clipboard = true;
    self
  }

  /// Enable or disable incognito mode for the WebView.
  ///
  ///  ## Platform-specific:
  ///
  ///  - **Windows**: Requires WebView2 Runtime version 101.0.1210.39 or higher, does nothing on older versions,
  ///    see https://learn.microsoft.com/en-us/microsoft-edge/webview2/release-notes/archive?tabs=dotnetcsharp#10121039
  ///  - **Android**: Unsupported.
  ///  - **macOS / iOS**: Uses the nonPersistent DataStore
  #[must_use]
  pub fn incognito(mut self, incognito: bool) -> Self {
    self.webview_attributes.incognito = incognito;
    self
  }

  /// Set a proxy URL for the WebView for all network requests.
  ///
  /// Must be either a `http://` or a `socks5://` URL.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS**: Requires the `macos-proxy` feature flag and only compiles for macOS 14+.
  #[must_use]
  pub fn proxy_url(mut self, url: Url) -> Self {
    self.webview_attributes.proxy_url = Some(url);
    self
  }

  /// Enable or disable transparency for the WebView.
  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  #[cfg_attr(
    docsrs,
    doc(cfg(any(not(target_os = "macos"), feature = "macos-private-api")))
  )]
  #[must_use]
  pub fn transparent(mut self, transparent: bool) -> Self {
    self.webview_attributes.transparent = transparent;
    self
  }

  /// Whether the webview should be focused or not.
  #[must_use]
  pub fn focused(mut self, focus: bool) -> Self {
    self.webview_attributes.focus = focus;
    self
  }

  /// Sets the webview to automatically grow and shrink its size and position when the parent window resizes.
  #[must_use]
  pub fn auto_resize(mut self) -> Self {
    self.webview_attributes.auto_resize = true;
    self
  }

  /// Whether page zooming by hotkeys and mousewheel should be enabled or not.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: Controls WebView2's [`IsZoomControlEnabled`](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2settings?view=webview2-winrt-1.0.2420.47#iszoomcontrolenabled) setting.
  /// - **MacOS / Linux**: Injects a polyfill that zooms in and out with `Ctrl/Cmd + [- = +]` hotkeys or mousewheel events,
  ///   20% in each step, ranging from 20% to 1000%. Requires `core:webview:allow-set-webview-zoom` permission
  ///
  /// - **Android / iOS**: Unsupported.
  #[must_use]
  pub fn zoom_hotkeys_enabled(mut self, enabled: bool) -> Self {
    self.webview_attributes.zoom_hotkeys_enabled = enabled;
    self
  }

  /// Whether browser extensions can be installed for the webview process
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: Enables the WebView2 environment's [`AreBrowserExtensionsEnabled`](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2environmentoptions?view=webview2-winrt-1.0.2739.15#arebrowserextensionsenabled)
  /// - **MacOS / Linux / iOS / Android** - Unsupported.
  #[must_use]
  pub fn browser_extensions_enabled(mut self, enabled: bool) -> Self {
    self.webview_attributes.browser_extensions_enabled = enabled;
    self
  }

  /// Set the path from which to load extensions from. Extensions stored in this path should be unpacked Chrome extensions on Windows, and compiled `.so` extensions on Linux.
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: Browser extensions must first be enabled. See [`browser_extensions_enabled`](Self::browser_extensions_enabled)
  /// - **MacOS / iOS / Android** - Unsupported.
  #[must_use]
  pub fn extensions_path(mut self, path: impl AsRef<Path>) -> Self {
    self.webview_attributes.extensions_path = Some(path.as_ref().to_path_buf());
    self
  }

  /// Initialize the WebView with a custom data store identifier.
  /// Can be used as a replacement for data_directory not being available in WKWebView.
  ///
  /// - **macOS / iOS**: Available on macOS >= 14 and iOS >= 17
  /// - **Windows / Linux / Android**: Unsupported.
  ///
  /// Note: Enable incognito mode to use the `nonPersistent` DataStore.
  #[must_use]
  pub fn data_store_identifier(mut self, data_store_identifier: [u8; 16]) -> Self {
    self.webview_attributes.data_store_identifier = Some(data_store_identifier);
    self
  }

  /// Sets whether the custom protocols should use `https://<scheme>.localhost` instead of the default `http://<scheme>.localhost` on Windows and Android. Defaults to `false`.
  ///
  /// ## Note
  ///
  /// Using a `https` scheme will NOT allow mixed content when trying to fetch `http` endpoints and therefore will not match the behavior of the `<scheme>://localhost` protocols used on macOS and Linux.
  ///
  /// ## Warning
  ///
  /// Changing this value between releases will change the IndexedDB, cookies and localstorage location and your app will not be able to access the old data.
  #[must_use]
  pub fn use_https_scheme(mut self, enabled: bool) -> Self {
    self.webview_attributes.use_https_scheme = enabled;
    self
  }

  /// Whether web inspector, which is usually called browser devtools, is enabled or not. Enabled by default.
  ///
  /// This API works in **debug** builds, but requires `devtools` feature flag to enable it in **release** builds.
  ///
  /// ## Platform-specific
  ///
  /// - macOS: This will call private functions on **macOS**
  /// - Android: Open `chrome://inspect/#devices` in Chrome to get the devtools window. Wry's `WebView` devtools API isn't supported on Android.
  /// - iOS: Open Safari > Develop > [Your Device Name] > [Your WebView] to get the devtools window.
  #[must_use]
  pub fn devtools(mut self, enabled: bool) -> Self {
    self.webview_attributes.devtools.replace(enabled);
    self
  }

  /// Set the webview background color.
  ///
  /// ## Platform-specific:
  ///
  /// - **macOS / iOS**: Not implemented.
  /// - **Windows**: On Windows 7, alpha channel is ignored.
  /// - **Windows**: On Windows 8 and newer, if alpha channel is not `0`, it will be ignored.
  #[must_use]
  pub fn background_color(mut self, color: Color) -> Self {
    self.webview_attributes.background_color = Some(color);
    self
  }

  /// Change the default background throttling behaviour.
  ///
  /// By default, browsers use a suspend policy that will throttle timers and even unload
  /// the whole tab (view) to free resources after roughly 5 minutes when a view became
  /// minimized or hidden. This will pause all tasks until the documents visibility state
  /// changes back from hidden to visible by bringing the view back to the foreground.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / Windows / Android**: Unsupported. Workarounds like a pending WebLock transaction might suffice.
  /// - **iOS**: Supported since version 17.0+.
  /// - **macOS**: Supported since version 14.0+.
  ///
  /// see https://github.com/tauri-apps/tauri/issues/5250#issuecomment-2569380578
  #[must_use]
  pub fn background_throttling(mut self, policy: BackgroundThrottlingPolicy) -> Self {
    self.webview_attributes.background_throttling = Some(policy);
    self
  }

  /// Whether JavaScript should be disabled.
  #[must_use]
  pub fn disable_javascript(mut self) -> Self {
    self.webview_attributes.javascript_disabled = true;
    self
  }
}

/// Webview.
#[default_runtime(crate::Wry, wry)]
pub struct Webview<R: Runtime> {
  pub(crate) window: Arc<Mutex<Window<R>>>,
  /// The webview created by the runtime.
  pub(crate) webview: DetachedWebview<EventLoopMessage, R>,
  /// The manager to associate this webview with.
  pub(crate) manager: Arc<AppManager<R>>,
  pub(crate) app_handle: AppHandle<R>,
  pub(crate) resources_table: Arc<Mutex<ResourceTable>>,
  use_https_scheme: bool,
}

impl<R: Runtime> std::fmt::Debug for Webview<R> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Window")
      .field("window", &self.window.lock().unwrap())
      .field("webview", &self.webview)
      .field("use_https_scheme", &self.use_https_scheme)
      .finish()
  }
}

impl<R: Runtime> Clone for Webview<R> {
  fn clone(&self) -> Self {
    Self {
      window: self.window.clone(),
      webview: self.webview.clone(),
      manager: self.manager.clone(),
      app_handle: self.app_handle.clone(),
      resources_table: self.resources_table.clone(),
      use_https_scheme: self.use_https_scheme,
    }
  }
}

impl<R: Runtime> Hash for Webview<R> {
  /// Only use the [`Webview`]'s label to represent its hash.
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.webview.label.hash(state)
  }
}

impl<R: Runtime> Eq for Webview<R> {}
impl<R: Runtime> PartialEq for Webview<R> {
  /// Only use the [`Webview`]'s label to compare equality.
  fn eq(&self, other: &Self) -> bool {
    self.webview.label.eq(&other.webview.label)
  }
}

/// Base webview functions.
impl<R: Runtime> Webview<R> {
  /// Create a new webview that is attached to the window.
  pub(crate) fn new(
    window: Window<R>,
    webview: DetachedWebview<EventLoopMessage, R>,
    use_https_scheme: bool,
  ) -> Self {
    Self {
      manager: window.manager.clone(),
      app_handle: window.app_handle.clone(),
      window: Arc::new(Mutex::new(window)),
      webview,
      resources_table: Default::default(),
      use_https_scheme,
    }
  }

  /// Initializes a webview builder with the given window label and URL to load on the webview.
  ///
  /// Data URLs are only supported with the `webview-data-url` feature flag.
  #[cfg(feature = "unstable")]
  #[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
  pub fn builder<L: Into<String>>(label: L, url: WebviewUrl) -> WebviewBuilder<R> {
    WebviewBuilder::new(label.into(), url)
  }

  /// Runs the given closure on the main thread.
  pub fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> crate::Result<()> {
    self
      .webview
      .dispatcher
      .run_on_main_thread(f)
      .map_err(Into::into)
  }

  /// The webview label.
  pub fn label(&self) -> &str {
    &self.webview.label
  }

  /// Whether the webview was configured to use the HTTPS scheme or not.
  pub(crate) fn use_https_scheme(&self) -> bool {
    self.use_https_scheme
  }

  /// Registers a window event listener.
  pub fn on_webview_event<F: Fn(&WebviewEvent) + Send + 'static>(&self, f: F) {
    self
      .webview
      .dispatcher
      .on_webview_event(move |event| f(&event.clone().into()));
  }

  /// Resolves the given command scope for this webview on the currently loaded URL.
  ///
  /// If the command is not allowed, returns None.
  ///
  /// If the scope cannot be deserialized to the given type, an error is returned.
  ///
  /// In a command context this can be directly resolved from the command arguments via [CommandScope]:
  ///
  /// ```
  /// use tauri::ipc::CommandScope;
  ///
  /// #[derive(Debug, serde::Deserialize)]
  /// struct ScopeType {
  ///   some_value: String,
  /// }
  /// #[tauri::command]
  /// fn my_command(scope: CommandScope<ScopeType>) {
  ///   // check scope
  /// }
  /// ```
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::Manager;
  ///
  /// #[derive(Debug, serde::Deserialize)]
  /// struct ScopeType {
  ///   some_value: String,
  /// }
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let webview = app.get_webview_window("main").unwrap();
  ///     let scope = webview.resolve_command_scope::<ScopeType>("my-plugin", "read");
  ///     Ok(())
  ///   });
  /// ```
  pub fn resolve_command_scope<T: ScopeObject>(
    &self,
    plugin: &str,
    command: &str,
  ) -> crate::Result<Option<ResolvedScope<T>>> {
    let current_url = self.url()?;
    let is_local = self.is_local_url(&current_url);
    let origin = if is_local {
      Origin::Local
    } else {
      Origin::Remote { url: current_url }
    };

    let cmd_name = format!("plugin:{plugin}|{command}");
    let resolved_access = self
      .manager()
      .runtime_authority
      .lock()
      .unwrap()
      .resolve_access(&cmd_name, self.window().label(), self.label(), &origin);

    if let Some(access) = resolved_access {
      let scope_ids = access
        .iter()
        .filter_map(|cmd| cmd.scope_id)
        .collect::<Vec<_>>();

      let command_scope = CommandScope::resolve(self, scope_ids)?;
      let global_scope = GlobalScope::resolve(self, plugin)?;

      Ok(Some(ResolvedScope {
        global_scope,
        command_scope,
      }))
    } else {
      Ok(None)
    }
  }
}

/// Desktop webview setters and actions.
#[cfg(desktop)]
impl<R: Runtime> Webview<R> {
  /// Opens the dialog to prints the contents of the webview.
  /// Currently only supported on macOS on `wry`.
  /// `window.print()` works on all platforms.
  pub fn print(&self) -> crate::Result<()> {
    self.webview.dispatcher.print().map_err(Into::into)
  }

  /// Get the cursor position relative to the top-left hand corner of the desktop.
  ///
  /// Note that the top-left hand corner of the desktop is not necessarily the same as the screen.
  /// If the user uses a desktop with multiple monitors,
  /// the top-left hand corner of the desktop is the top-left hand corner of the main monitor on Windows and macOS
  /// or the top-left of the leftmost monitor on X11.
  ///
  /// The coordinates can be negative if the top-left hand corner of the window is outside of the visible screen region.
  pub fn cursor_position(&self) -> crate::Result<PhysicalPosition<f64>> {
    self.app_handle.cursor_position()
  }

  /// Closes this webview.
  pub fn close(&self) -> crate::Result<()> {
    self.webview.dispatcher.close()?;
    self.manager().on_webview_close(self.label());
    Ok(())
  }

  /// Resizes this webview.
  pub fn set_bounds(&self, bounds: tauri_runtime::Rect) -> crate::Result<()> {
    self
      .webview
      .dispatcher
      .set_bounds(bounds)
      .map_err(Into::into)
  }

  /// Resizes this webview.
  pub fn set_size<S: Into<Size>>(&self, size: S) -> crate::Result<()> {
    self
      .webview
      .dispatcher
      .set_size(size.into())
      .map_err(Into::into)
  }

  /// Sets this webviews's position.
  pub fn set_position<Pos: Into<Position>>(&self, position: Pos) -> crate::Result<()> {
    self
      .webview
      .dispatcher
      .set_position(position.into())
      .map_err(Into::into)
  }

  /// Focus the webview.
  pub fn set_focus(&self) -> crate::Result<()> {
    self.webview.dispatcher.set_focus().map_err(Into::into)
  }

  /// Hide the webview.
  pub fn hide(&self) -> crate::Result<()> {
    self.webview.dispatcher.hide().map_err(Into::into)
  }

  /// Show the webview.
  pub fn show(&self) -> crate::Result<()> {
    self.webview.dispatcher.show().map_err(Into::into)
  }

  /// Move the webview to the given window.
  pub fn reparent(&self, window: &Window<R>) -> crate::Result<()> {
    #[cfg(not(feature = "unstable"))]
    {
      if self.window_ref().is_webview_window() || window.is_webview_window() {
        return Err(crate::Error::CannotReparentWebviewWindow);
      }
    }

    *self.window.lock().unwrap() = window.clone();
    self.webview.dispatcher.reparent(window.window.id)?;
    Ok(())
  }

  /// Sets whether the webview should automatically grow and shrink its size and position when the parent window resizes.
  pub fn set_auto_resize(&self, auto_resize: bool) -> crate::Result<()> {
    self
      .webview
      .dispatcher
      .set_auto_resize(auto_resize)
      .map_err(Into::into)
  }

  /// Returns the bounds of the webviews's client area.
  pub fn bounds(&self) -> crate::Result<tauri_runtime::Rect> {
    self.webview.dispatcher.bounds().map_err(Into::into)
  }

  /// Returns the webview position.
  ///
  /// - For child webviews, returns the position of the top-left hand corner of the webviews's client area relative to the top-left hand corner of the parent window.
  /// - For webview window, returns the inner position of the window.
  pub fn position(&self) -> crate::Result<PhysicalPosition<i32>> {
    self.webview.dispatcher.position().map_err(Into::into)
  }

  /// Returns the physical size of the webviews's client area.
  pub fn size(&self) -> crate::Result<PhysicalSize<u32>> {
    self.webview.dispatcher.size().map_err(Into::into)
  }
}

/// Webview APIs.
impl<R: Runtime> Webview<R> {
  /// The window that is hosting this webview.
  pub fn window(&self) -> Window<R> {
    self.window.lock().unwrap().clone()
  }

  /// A reference to the window that is hosting this webview.
  pub fn window_ref(&self) -> MutexGuard<'_, Window<R>> {
    self.window.lock().unwrap()
  }

  pub(crate) fn window_label(&self) -> String {
    self.window_ref().label().to_string()
  }

  /// Executes a closure, providing it with the webview handle that is specific to the current platform.
  ///
  /// The closure is executed on the main thread.
  ///
  /// Note that `webview2-com`, `webkit2gtk`, `objc2_web_kit` and similar crates may be updated in minor releases of Tauri.
  /// Therefore it's recommended to pin Tauri to at least a minor version when you're using `with_webview`.
  ///
  /// # Examples
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```rust,no_run
use tauri::Manager;

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let main_webview = app.get_webview("main").unwrap();
      main_webview.with_webview(|webview| {
        #[cfg(target_os = "linux")]
        {
          // see <https://docs.rs/webkit2gtk/2.0.0/webkit2gtk/struct.WebView.html>
          // and <https://docs.rs/webkit2gtk/2.0.0/webkit2gtk/trait.WebViewExt.html>
          use webkit2gtk::WebViewExt;
          webview.inner().set_zoom_level(4.);
        }

        #[cfg(windows)]
        unsafe {
          // see https://docs.rs/webview2-com/0.19.1/webview2_com/Microsoft/Web/WebView2/Win32/struct.ICoreWebView2Controller.html
          webview.controller().SetZoomFactor(4.).unwrap();
        }

        #[cfg(target_os = "macos")]
        unsafe {
          let view: &objc2_web_kit::WKWebView = &*webview.inner().cast();
          let controller: &objc2_web_kit::WKUserContentController = &*webview.controller().cast();
          let window: &objc2_app_kit::NSWindow = &*webview.ns_window().cast();

          view.setPageZoom(4.);
          controller.removeAllUserScripts();
          let bg_color = objc2_app_kit::NSColor::colorWithDeviceRed_green_blue_alpha(0.5, 0.2, 0.4, 1.);
          window.setBackgroundColor(Some(&bg_color));
        }

        #[cfg(target_os = "android")]
        {
          use jni::objects::JValue;
          webview.jni_handle().exec(|env, _, webview| {
            env.call_method(webview, "zoomBy", "(F)V", &[JValue::Float(4.)]).unwrap();
          })
        }
      });
      Ok(())
  });
}
```
  "####
  )]
  #[cfg(feature = "wry")]
  #[cfg_attr(docsrs, doc(feature = "wry"))]
  pub fn with_webview<F: FnOnce(PlatformWebview) + Send + 'static>(
    &self,
    f: F,
  ) -> crate::Result<()> {
    self
      .webview
      .dispatcher
      .with_webview(|w| f(PlatformWebview(*w.downcast().unwrap())))
      .map_err(Into::into)
  }

  /// Returns the current url of the webview.
  pub fn url(&self) -> crate::Result<Url> {
    self
      .webview
      .dispatcher
      .url()
      .map(|url| url.parse().map_err(crate::Error::InvalidUrl))?
  }

  /// Navigates the webview to the defined url.
  pub fn navigate(&self, url: Url) -> crate::Result<()> {
    self.webview.dispatcher.navigate(url).map_err(Into::into)
  }

  /// Reloads the current page.
  pub fn reload(&self) -> crate::Result<()> {
    self.webview.dispatcher.reload().map_err(Into::into)
  }

  fn is_local_url(&self, current_url: &Url) -> bool {
    let uses_https = current_url.scheme() == "https";

    // if from `tauri://` custom protocol
    ({
      let protocol_url = self.manager().protocol_url(uses_https);
      current_url.scheme() == protocol_url.scheme()
      && current_url.domain() == protocol_url.domain()
    }) ||

    // or if relative to `devUrl` or `frontendDist`
      self
          .manager()
          .get_url(uses_https)
          .make_relative(current_url)
          .is_some()

      // or from a custom protocol registered by the user
      || ({
        let scheme = current_url.scheme();
        let protocols = self.manager().webview.uri_scheme_protocols.lock().unwrap();

        #[cfg(all(not(windows), not(target_os = "android")))]
        let local = protocols.contains_key(scheme);

        // on window and android, custom protocols are `http://<protocol-name>.path/to/route`
        // so we check using the first part of the domain
        #[cfg(any(windows, target_os = "android"))]
        let local = {
          let protocol_url = self.manager().protocol_url(uses_https);
          let maybe_protocol = current_url
            .domain()
            .and_then(|d| d .split_once('.'))
            .unwrap_or_default()
            .0;

          protocols.contains_key(maybe_protocol) && scheme == protocol_url.scheme()
        };

        local
      })
  }

  /// Handles this window receiving an [`InvokeRequest`].
  pub fn on_message(self, request: InvokeRequest, responder: Box<OwnedInvokeResponder<R>>) {
    let manager = self.manager_owned();
    let is_local = self.is_local_url(&request.url);

    // ensure the passed key matches what our manager should have injected
    let expected = manager.invoke_key();
    if request.invoke_key != expected {
      #[cfg(feature = "tracing")]
      tracing::error!(
        "__TAURI_INVOKE_KEY__ expected {expected} but received {}",
        request.invoke_key
      );

      #[cfg(not(feature = "tracing"))]
      eprintln!(
        "__TAURI_INVOKE_KEY__ expected {expected} but received {}",
        request.invoke_key
      );

      return;
    }

    let resolver = InvokeResolver::new(
      self.clone(),
      Arc::new(Mutex::new(Some(Box::new(
        #[allow(unused_variables)]
        move |webview: Webview<R>, cmd, response, callback, error| {
          responder(webview, cmd, response, callback, error);
        },
      )))),
      request.cmd.clone(),
      request.callback,
      request.error,
    );

    #[cfg(mobile)]
    let app_handle = self.app_handle.clone();

    let message = InvokeMessage::new(
      self,
      manager.state(),
      request.cmd.to_string(),
      request.body,
      request.headers,
    );

    let acl_origin = if is_local {
      Origin::Local
    } else {
      Origin::Remote {
        url: request.url.clone(),
      }
    };
    let (resolved_acl, has_app_acl_manifest) = {
      let runtime_authority = manager.runtime_authority.lock().unwrap();
      let acl = runtime_authority.resolve_access(
        &request.cmd,
        message.webview.window_ref().label(),
        message.webview.label(),
        &acl_origin,
      );
      (acl, runtime_authority.has_app_manifest())
    };

    let mut invoke = Invoke {
      message,
      resolver: resolver.clone(),
      acl: resolved_acl,
    };

    let plugin_command = request.cmd.strip_prefix("plugin:").map(|raw_command| {
      let mut tokens = raw_command.split('|');
      // safe to unwrap: split always has a least one item
      let plugin = tokens.next().unwrap();
      let command = tokens.next().map(|c| c.to_string()).unwrap_or_default();
      (plugin, command)
    });

    // we only check ACL on plugin commands or if the app defined its ACL manifest
    if (plugin_command.is_some() || has_app_acl_manifest)
      // TODO: Remove this special check in v3
      && request.cmd != crate::ipc::channel::FETCH_CHANNEL_DATA_COMMAND
      && invoke.acl.is_none()
    {
      #[cfg(debug_assertions)]
      {
        let (key, command_name) = plugin_command
          .clone()
          .unwrap_or_else(|| (tauri_utils::acl::APP_ACL_KEY, request.cmd.clone()));
        invoke.resolver.reject(
          manager
            .runtime_authority
            .lock()
            .unwrap()
            .resolve_access_message(
              key,
              &command_name,
              invoke.message.webview.window().label(),
              invoke.message.webview.label(),
              &acl_origin,
            ),
        );
      }
      #[cfg(not(debug_assertions))]
      invoke
        .resolver
        .reject(format!("Command {} not allowed by ACL", request.cmd));
      return;
    }

    if let Some((plugin, command_name)) = plugin_command {
      invoke.message.command = command_name;

      let command = invoke.message.command.clone();

      #[cfg(mobile)]
      let message = invoke.message.clone();

      #[allow(unused_mut)]
      let mut handled = manager.extend_api(plugin, invoke);

      #[cfg(mobile)]
      {
        if !handled {
          handled = true;

          fn load_channels<R: Runtime>(payload: &serde_json::Value, webview: &Webview<R>) {
            use std::str::FromStr;

            if let serde_json::Value::Object(map) = payload {
              for v in map.values() {
                if let serde_json::Value::String(s) = v {
                  let _ = crate::ipc::JavaScriptChannelId::from_str(s)
                    .map(|id| id.channel_on::<R, ()>(webview.clone()));
                }
              }
            }
          }

          let payload = message.payload.into_json();
          // initialize channels
          load_channels(&payload, &message.webview);

          let resolver_ = resolver.clone();
          if let Err(e) = crate::plugin::mobile::run_command(
            plugin,
            &app_handle,
            heck::AsLowerCamelCase(message.command).to_string(),
            payload,
            move |response| match response {
              Ok(r) => resolver_.resolve(r),
              Err(e) => resolver_.reject(e),
            },
          ) {
            resolver.reject(e.to_string());
            return;
          }
        }
      }

      if !handled {
        resolver.reject(format!("Command {command} not found"));
      }
    } else {
      let command = invoke.message.command.clone();
      let handled = manager.run_invoke_handler(invoke);
      if !handled {
        resolver.reject(format!("Command {command} not found"));
      }
    }
  }

  /// Evaluates JavaScript on this window.
  pub fn eval(&self, js: &str) -> crate::Result<()> {
    self.webview.dispatcher.eval_script(js).map_err(Into::into)
  }

  /// Register a JS event listener and return its identifier.
  pub(crate) fn listen_js(
    &self,
    event: EventName<&str>,
    target: EventTarget,
    handler: CallbackFn,
  ) -> crate::Result<EventId> {
    let listeners = self.manager().listeners();

    let id = listeners.next_event_id();

    self.eval(&crate::event::listen_js_script(
      listeners.listeners_object_name(),
      &serde_json::to_string(&target)?,
      event,
      id,
      &format!("window['_{}']", handler.0),
    ))?;

    listeners.listen_js(event, self.label(), target, id);

    Ok(id)
  }

  /// Unregister a JS event listener.
  pub(crate) fn unlisten_js(&self, event: EventName<&str>, id: EventId) -> crate::Result<()> {
    let listeners = self.manager().listeners();

    self.eval(&crate::event::unlisten_js_script(
      listeners.listeners_object_name(),
      event,
      id,
    ))?;

    listeners.unlisten_js(event, id);

    Ok(())
  }

  pub(crate) fn emit_js(&self, emit_args: &EmitArgs, ids: &[u32]) -> crate::Result<()> {
    self.eval(&crate::event::emit_js_script(
      self.manager().listeners().function_name(),
      emit_args,
      &serde_json::to_string(ids)?,
    )?)?;
    Ok(())
  }

  /// Opens the developer tools window (Web Inspector).
  /// The devtools is only enabled on debug builds or with the `devtools` feature flag.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Only supported on macOS 10.15+.
  ///   This is a private API on macOS, so you cannot use this if your application will be published on the App Store.
  ///
  /// # Examples
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```rust,no_run
use tauri::Manager;
tauri::Builder::default()
  .setup(|app| {
    #[cfg(debug_assertions)]
    app.get_webview("main").unwrap().open_devtools();
    Ok(())
  });
```
  "####
  )]
  #[cfg(any(debug_assertions, feature = "devtools"))]
  #[cfg_attr(docsrs, doc(cfg(any(debug_assertions, feature = "devtools"))))]
  pub fn open_devtools(&self) {
    self.webview.dispatcher.open_devtools();
  }

  /// Closes the developer tools window (Web Inspector).
  /// The devtools is only enabled on debug builds or with the `devtools` feature flag.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Only supported on macOS 10.15+.
  ///   This is a private API on macOS, so you cannot use this if your application will be published on the App Store.
  /// - **Windows:** Unsupported.
  ///
  /// # Examples
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```rust,no_run
use tauri::Manager;
tauri::Builder::default()
  .setup(|app| {
    #[cfg(debug_assertions)]
    {
      let webview = app.get_webview("main").unwrap();
      webview.open_devtools();
      std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(10));
        webview.close_devtools();
      });
    }
    Ok(())
  });
```
  "####
  )]
  #[cfg(any(debug_assertions, feature = "devtools"))]
  #[cfg_attr(docsrs, doc(cfg(any(debug_assertions, feature = "devtools"))))]
  pub fn close_devtools(&self) {
    self.webview.dispatcher.close_devtools();
  }

  /// Checks if the developer tools window (Web Inspector) is opened.
  /// The devtools is only enabled on debug builds or with the `devtools` feature flag.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Only supported on macOS 10.15+.
  ///   This is a private API on macOS, so you cannot use this if your application will be published on the App Store.
  /// - **Windows:** Unsupported.
  ///
  /// # Examples
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```rust,no_run
use tauri::Manager;
tauri::Builder::default()
  .setup(|app| {
    #[cfg(debug_assertions)]
    {
      let webview = app.get_webview("main").unwrap();
      if !webview.is_devtools_open() {
        webview.open_devtools();
      }
    }
    Ok(())
  });
```
  "####
  )]
  #[cfg(any(debug_assertions, feature = "devtools"))]
  #[cfg_attr(docsrs, doc(cfg(any(debug_assertions, feature = "devtools"))))]
  pub fn is_devtools_open(&self) -> bool {
    self
      .webview
      .dispatcher
      .is_devtools_open()
      .unwrap_or_default()
  }

  /// Set the webview zoom level
  ///
  /// ## Platform-specific:
  ///
  /// - **Android**: Not supported.
  /// - **macOS**: available on macOS 11+ only.
  /// - **iOS**: available on iOS 14+ only.
  pub fn set_zoom(&self, scale_factor: f64) -> crate::Result<()> {
    self
      .webview
      .dispatcher
      .set_zoom(scale_factor)
      .map_err(Into::into)
  }

  /// Specify the webview background color.
  ///
  /// ## Platfrom-specific:
  ///
  /// - **macOS / iOS**: Not implemented.
  /// - **Windows**:
  ///   - On Windows 7, transparency is not supported and the alpha value will be ignored.
  ///   - On Windows higher than 7: translucent colors are not supported so any alpha value other than `0` will be replaced by `255`
  pub fn set_background_color(&self, color: Option<Color>) -> crate::Result<()> {
    self
      .webview
      .dispatcher
      .set_background_color(color)
      .map_err(Into::into)
  }

  /// Clear all browsing data for this webview.
  pub fn clear_all_browsing_data(&self) -> crate::Result<()> {
    self
      .webview
      .dispatcher
      .clear_all_browsing_data()
      .map_err(Into::into)
  }

  /// Returns all cookies in the runtime's cookie store including HTTP-only and secure cookies.
  ///
  /// Note that cookies will only be returned for URLs with an http or https scheme.
  /// Cookies set through javascript for local files
  /// (such as those served from the tauri://) protocol are not currently supported.
  ///
  /// # Stability
  ///
  /// The return value of this function leverages [`tauri_runtime::Cookie`] which re-exports the cookie crate.
  /// This dependency might receive updates in minor Tauri releases.
  ///
  /// # Known issues
  ///
  /// On Windows, this function deadlocks when used in a synchronous command or event handlers, see [the Webview2 issue].
  /// You should use `async` commands and separate threads when reading cookies.
  ///
  /// [the Webview2 issue]: https://github.com/tauri-apps/wry/issues/583
  pub fn cookies_for_url(&self, url: Url) -> crate::Result<Vec<Cookie<'static>>> {
    self
      .webview
      .dispatcher
      .cookies_for_url(url)
      .map_err(Into::into)
  }

  /// Returns all cookies in the runtime's cookie store for all URLs including HTTP-only and secure cookies.
  ///
  /// Note that cookies will only be returned for URLs with an http or https scheme.
  /// Cookies set through javascript for local files
  /// (such as those served from the tauri://) protocol are not currently supported.
  ///
  /// # Stability
  ///
  /// The return value of this function leverages [`tauri_runtime::Cookie`] which re-exports the cookie crate.
  /// This dependency might receive updates in minor Tauri releases.
  ///
  /// # Known issues
  ///
  /// On Windows, this function deadlocks when used in a synchronous command or event handlers, see [the Webview2 issue].
  /// You should use `async` commands and separate threads when reading cookies.
  ///
  /// ## Platform-specific
  ///
  /// - **Android**: Unsupported, always returns an empty [`Vec`].
  ///
  /// [the Webview2 issue]: https://github.com/tauri-apps/wry/issues/583
  pub fn cookies(&self) -> crate::Result<Vec<Cookie<'static>>> {
    self.webview.dispatcher.cookies().map_err(Into::into)
  }
}

impl<R: Runtime> Listener<R> for Webview<R> {
  /// Listen to an event on this webview.
  ///
  /// # Examples
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
use tauri::{Manager, Listener};

tauri::Builder::default()
  .setup(|app| {
    let webview = app.get_webview("main").unwrap();
    webview.listen("component-loaded", move |event| {
      println!("webview just loaded a component");
    });

    Ok(())
  });
```
  "####
  )]
  fn listen<F>(&self, event: impl Into<String>, handler: F) -> EventId
  where
    F: Fn(Event) + Send + 'static,
  {
    let event = EventName::new(event.into()).unwrap();
    self.manager.listen(
      event,
      EventTarget::Webview {
        label: self.label().to_string(),
      },
      handler,
    )
  }

  /// Listen to an event on this webview only once.
  ///
  /// See [`Self::listen`] for more information.
  fn once<F>(&self, event: impl Into<String>, handler: F) -> EventId
  where
    F: FnOnce(Event) + Send + 'static,
  {
    let event = EventName::new(event.into()).unwrap();
    self.manager.once(
      event,
      EventTarget::Webview {
        label: self.label().to_string(),
      },
      handler,
    )
  }

  /// Unlisten to an event on this webview.
  ///
  /// # Examples
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
use tauri::{Manager, Listener};

tauri::Builder::default()
  .setup(|app| {
    let webview = app.get_webview("main").unwrap();
    let webview_ = webview.clone();
    let handler = webview.listen("component-loaded", move |event| {
      println!("webview just loaded a component");

      // we no longer need to listen to the event
      // we also could have used `webview.once` instead
      webview_.unlisten(event.id());
    });

    // stop listening to the event when you do not need it anymore
    webview.unlisten(handler);

    Ok(())
  });
```
  "####
  )]
  fn unlisten(&self, id: EventId) {
    self.manager.unlisten(id)
  }
}

impl<R: Runtime> Emitter<R> for Webview<R> {}

impl<R: Runtime> Manager<R> for Webview<R> {
  fn resources_table(&self) -> MutexGuard<'_, ResourceTable> {
    self
      .resources_table
      .lock()
      .expect("poisoned window resources table")
  }
}

impl<R: Runtime> ManagerBase<R> for Webview<R> {
  fn manager(&self) -> &AppManager<R> {
    &self.manager
  }

  fn manager_owned(&self) -> Arc<AppManager<R>> {
    self.manager.clone()
  }

  fn runtime(&self) -> RuntimeOrDispatch<'_, R> {
    self.app_handle.runtime()
  }

  fn managed_app_handle(&self) -> &AppHandle<R> {
    &self.app_handle
  }
}

impl<'de, R: Runtime> CommandArg<'de, R> for Webview<R> {
  /// Grabs the [`Webview`] from the [`CommandItem`]. This will never fail.
  fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
    Ok(command.message.webview())
  }
}

/// Resolved scope that can be obtained via [`Webview::resolve_command_scope`].
pub struct ResolvedScope<T: ScopeObject> {
  command_scope: CommandScope<T>,
  global_scope: GlobalScope<T>,
}

impl<T: ScopeObject> ResolvedScope<T> {
  /// The global plugin scope.
  pub fn global_scope(&self) -> &GlobalScope<T> {
    &self.global_scope
  }

  /// The command-specific scope.
  pub fn command_scope(&self) -> &CommandScope<T> {
    &self.command_scope
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn webview_is_send_sync() {
    crate::test_utils::assert_send::<super::Webview>();
    crate::test_utils::assert_sync::<super::Webview>();
  }
}
