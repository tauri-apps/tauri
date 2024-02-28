// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
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
use tauri_runtime::{
  webview::{DetachedWebview, PendingWebview, WebviewAttributes},
  WebviewDispatch,
};
#[cfg(desktop)]
use tauri_runtime::{
  window::dpi::{PhysicalPosition, PhysicalSize, Position, Size},
  WindowDispatch,
};
use tauri_utils::{
  acl::APP_ACL_KEY,
  config::{WebviewUrl, WindowConfig},
};
pub use url::Url;

use crate::{
  app::{UriSchemeResponder, WebviewEvent},
  event::{EmitArgs, EventTarget},
  ipc::{
    CallbackFn, CommandArg, CommandItem, Invoke, InvokeBody, InvokeError, InvokeMessage,
    InvokeResolver, Origin, OwnedInvokeResponder,
  },
  manager::{webview::WebviewLabelDef, AppManager},
  sealed::{ManagerBase, RuntimeOrDispatch},
  AppHandle, Event, EventId, EventLoopMessage, Manager, Runtime, Window,
};

use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  path::PathBuf,
  sync::{Arc, Mutex},
};

pub(crate) type WebResourceRequestHandler =
  dyn Fn(http::Request<Vec<u8>>, &mut http::Response<Cow<'static, [u8]>>) + Send + Sync;
pub(crate) type NavigationHandler = dyn Fn(&Url) -> bool + Send;
pub(crate) type UriSchemeProtocolHandler =
  Box<dyn Fn(http::Request<Vec<u8>>, UriSchemeResponder) + Send + Sync>;
pub(crate) type OnPageLoad<R> = dyn Fn(Webview<R>, PageLoadPayload<'_>) + Send + Sync + 'static;

pub(crate) type DownloadHandler<R> = dyn Fn(Webview<R>, DownloadEvent<'_>) -> bool + Send + Sync;

#[derive(Clone, Serialize)]
struct CreatedEvent {
  label: String,
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
    /// limitations.
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
#[derive(Debug)]
pub struct InvokeRequest {
  /// The invoke command.
  pub cmd: String,
  /// The success callback.
  pub callback: CallbackFn,
  /// The error callback.
  pub error: CallbackFn,
  /// The body of the request.
  pub body: InvokeBody,
  /// The request headers.
  pub headers: HeaderMap,
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
  pub fn inner(&self) -> cocoa::base::id {
    self.0.webview
  }

  /// Returns WKWebView [controller] handle.
  ///
  /// [controller]: https://developer.apple.com/documentation/webkit/wkusercontentcontroller
  #[cfg(any(target_os = "macos", target_os = "ios"))]
  #[cfg_attr(docsrs, doc(cfg(any(target_os = "macos", target_os = "ios"))))]
  pub fn controller(&self) -> cocoa::base::id {
    self.0.manager
  }

  /// Returns [NSWindow] associated with the WKWebView webview.
  ///
  /// [NSWindow]: https://developer.apple.com/documentation/appkit/nswindow
  #[cfg(target_os = "macos")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
  pub fn ns_window(&self) -> cocoa::base::id {
    self.0.ns_window
  }

  /// Returns [UIViewController] used by the WKWebView webview NSWindow.
  ///
  /// [UIViewController]: https://developer.apple.com/documentation/uikit/uiviewcontroller
  #[cfg(target_os = "ios")]
  #[cfg_attr(docsrs, doc(cfg(target_os = "ios")))]
  pub fn view_controller(&self) -> cocoa::base::id {
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
  /// On Windows, this function deadlocks when used in a synchronous command, see [the Webview2 issue].
  /// You should use `async` commands when creating windows.
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
  /// On Windows, this function deadlocks when used in a synchronous command, see [the Webview2 issue].
  /// You should use `async` commands when creating webviews.
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
async fn reopen_window(app: tauri::AppHandle) {
  let window = tauri::window::WindowBuilder::from_config(&app, &app.config().app.windows.get(0).unwrap().clone())
    .unwrap()
    .build()
    .unwrap();
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
    window_labels: &[String],
    webview_labels: &[WebviewLabelDef],
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

    if let Some(on_page_load_handler) = self.on_page_load_handler.take() {
      let label = pending.label.clone();
      let manager = manager.manager_owned();
      pending
        .on_page_load_handler
        .replace(Box::new(move |url, event| {
          if let Some(w) = manager.get_webview(&label) {
            on_page_load_handler(w, PageLoadPayload { url: &url, event });
          }
        }));
    }

    manager.manager().webview.prepare_webview(
      manager,
      pending,
      window_label,
      window_labels,
      webview_labels,
    )
  }

  /// Creates a new webview on the given window.
  #[cfg(desktop)]
  pub(crate) fn build(
    self,
    window: Window<R>,
    position: Position,
    size: Size,
  ) -> crate::Result<Webview<R>> {
    let window_labels = window
      .manager()
      .window
      .labels()
      .into_iter()
      .collect::<Vec<_>>();
    let webview_labels = window
      .manager()
      .webview
      .webviews_lock()
      .values()
      .map(|w| WebviewLabelDef {
        window_label: w.window().label().to_string(),
        label: w.label().to_string(),
      })
      .collect::<Vec<_>>();

    let app_manager = window.manager();

    let mut pending =
      self.into_pending_webview(&window, window.label(), &window_labels, &webview_labels)?;

    pending.webview_attributes.bounds = Some((position, size));

    let webview = match &mut window.runtime() {
      RuntimeOrDispatch::Dispatch(dispatcher) => dispatcher.create_webview(pending),
      _ => unimplemented!(),
    }
    .map(|webview| app_manager.webview.attach_webview(window.clone(), webview))?;

    app_manager.webview.eval_script_all(format!(
      "window.__TAURI_INTERNALS__.metadata.windows = {window_labels_array}.map(function (label) {{ return {{ label: label }} }})",
      window_labels_array = serde_json::to_string(&app_manager.webview.labels())?,
    ))?;

    app_manager.emit_filter(
      "tauri://webview-created",
      Some(CreatedEvent {
        label: webview.label().into(),
      }),
      |s| match s {
        EventTarget::Webview { label } => label == webview.label(),
        _ => false,
      },
    )?;

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

  /// Disables the file drop handler. This is required to use drag and drop APIs on the front end on Windows.
  #[must_use]
  pub fn disable_file_drop_handler(mut self) -> Self {
    self.webview_attributes.file_drop_handler_enabled = false;
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

  /// Enable or disable incognito mode for the WebView..
  ///
  ///  ## Platform-specific:
  ///
  ///  **Android**: Unsupported.
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

  /// Sets the webview to automatically grow and shrink its size and position when the parent window resizes.
  #[must_use]
  pub fn auto_resize(mut self) -> Self {
    self.webview_attributes.auto_resize = true;
    self
  }
}

/// Webview.
#[default_runtime(crate::Wry, wry)]
pub struct Webview<R: Runtime> {
  window_label: Arc<Mutex<String>>,
  /// The manager to associate this webview with.
  pub(crate) manager: Arc<AppManager<R>>,
  pub(crate) app_handle: AppHandle<R>,
  /// The webview created by the runtime.
  pub(crate) webview: DetachedWebview<EventLoopMessage, R>,
}

impl<R: Runtime> std::fmt::Debug for Webview<R> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Window")
      .field("window_label", &self.window_label)
      .field("webview", &self.webview)
      .finish()
  }
}

impl<R: Runtime> Clone for Webview<R> {
  fn clone(&self) -> Self {
    Self {
      window_label: self.window_label.clone(),
      manager: self.manager.clone(),
      app_handle: self.app_handle.clone(),
      webview: self.webview.clone(),
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
  pub(crate) fn new(window: Window<R>, webview: DetachedWebview<EventLoopMessage, R>) -> Self {
    Self {
      window_label: Arc::new(Mutex::new(window.label().into())),
      manager: window.manager.clone(),
      app_handle: window.app_handle.clone(),
      webview,
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

  /// Registers a window event listener.
  pub fn on_webview_event<F: Fn(&WebviewEvent) + Send + 'static>(&self, f: F) {
    self
      .webview
      .dispatcher
      .on_webview_event(move |event| f(&event.clone().into()));
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

  /// Closes this webview.
  pub fn close(&self) -> crate::Result<()> {
    let window = self.window();
    if window.is_webview_window() {
      window.close()
    } else {
      self.webview.dispatcher.close()?;
      self.manager().on_webview_close(self.label());
      Ok(())
    }
  }

  /// Resizes this webview.
  pub fn set_size<S: Into<Size>>(&self, size: S) -> crate::Result<()> {
    let window = self.window();
    if window.is_webview_window() {
      window.set_size(size.into())
    } else {
      self
        .webview
        .dispatcher
        .set_size(size.into())
        .map_err(Into::into)
    }
  }

  /// Sets this webviews's position.
  pub fn set_position<Pos: Into<Position>>(&self, position: Pos) -> crate::Result<()> {
    let window = self.window();
    if window.is_webview_window() {
      window.set_position(position.into())
    } else {
      self
        .webview
        .dispatcher
        .set_position(position.into())
        .map_err(Into::into)
    }
  }

  /// Focus the webview.
  pub fn set_focus(&self) -> crate::Result<()> {
    self.webview.dispatcher.set_focus().map_err(Into::into)
  }

  /// Move the webview to the given window.
  pub fn reparent(&self, window: &Window<R>) -> crate::Result<()> {
    let current_window = self.window();
    if current_window.is_webview_window() || window.is_webview_window() {
      Err(crate::Error::CannotReparentWebviewWindow)
    } else {
      self.webview.dispatcher.reparent(window.window.id)?;
      *self.window_label.lock().unwrap() = window.label().to_string();
      Ok(())
    }
  }

  /// Returns the webview position.
  ///
  /// - For child webviews, returns the position of the top-left hand corner of the webviews's client area relative to the top-left hand corner of the parent window.
  /// - For webview window, returns the inner position of the window.
  pub fn position(&self) -> crate::Result<PhysicalPosition<i32>> {
    let window = self.window();
    if window.is_webview_window() {
      window.inner_position()
    } else {
      self.webview.dispatcher.position().map_err(Into::into)
    }
  }

  /// Returns the physical size of the webviews's client area.
  pub fn size(&self) -> crate::Result<PhysicalSize<u32>> {
    let window = self.window();
    if window.is_webview_window() {
      window.inner_size()
    } else {
      self.webview.dispatcher.size().map_err(Into::into)
    }
  }
}

/// Webview APIs.
impl<R: Runtime> Webview<R> {
  /// The window that is hosting this webview.
  pub fn window(&self) -> Window<R> {
    self
      .manager
      .get_window(&self.window_label.lock().unwrap())
      .expect("could not locate webview parent window")
  }

  pub(crate) fn window_label(&self) -> String {
    self.window_label.lock().unwrap().clone()
  }

  /// Executes a closure, providing it with the webview handle that is specific to the current platform.
  ///
  /// The closure is executed on the main thread.
  ///
  /// # Examples
  ///
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```rust,no_run
#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;
use tauri::Manager;

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let main_webview = app.get_webview("main").unwrap();
      main_webview.with_webview(|webview| {
        #[cfg(target_os = "linux")]
        {
          // see https://docs.rs/webkit2gtk/2.0.0/webkit2gtk/struct.WebView.html
          // and https://docs.rs/webkit2gtk/2.0.0/webkit2gtk/trait.WebViewExt.html
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
          let () = msg_send![webview.inner(), setPageZoom: 4.];
          let () = msg_send![webview.controller(), removeAllUserScripts];
          let bg_color: cocoa::base::id = msg_send![class!(NSColor), colorWithDeviceRed:0.5 green:0.2 blue:0.4 alpha:1.];
          let () = msg_send![webview.ns_window(), setBackgroundColor: bg_color];
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
  // TODO: in v2, change this type to Result
  pub fn url(&self) -> Url {
    self.webview.dispatcher.url().unwrap()
  }

  /// Navigates the webview to the defined url.
  pub fn navigate(&mut self, url: Url) {
    self.webview.dispatcher.navigate(url).unwrap();
  }

  fn is_local_url(&self, current_url: &Url) -> bool {
    // if from `tauri://` custom protocol
    ({
      let protocol_url = self.manager().protocol_url();
      current_url.scheme() == protocol_url.scheme()
      && current_url.domain() == protocol_url.domain()
    }) ||

    // or if relative to `devUrl` or `frontendDist`
      self
          .manager()
          .get_url()
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
          let protocol_url = self.manager().protocol_url();
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
    let current_url = self.url();
    let is_local = self.is_local_url(&current_url);

    let custom_responder = self.manager().webview.invoke_responder.clone();

    let resolver = InvokeResolver::new(
      self.clone(),
      Arc::new(Mutex::new(Some(Box::new(
        #[allow(unused_variables)]
        move |webview: Webview<R>, cmd, response, callback, error| {
          if let Some(responder) = &custom_responder {
            (responder)(&webview, &cmd, &response, callback, error);
          }

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
        url: current_url.to_string(),
      }
    };
    let (resolved_acl, has_app_acl_manifest) = {
      let runtime_authority = manager.runtime_authority.lock().unwrap();
      let acl = runtime_authority
        .resolve_access(
          &request.cmd,
          message.webview.window().label(),
          message.webview.label(),
          &acl_origin,
        )
        .cloned();
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
      && request.cmd != crate::ipc::channel::FETCH_CHANNEL_DATA_COMMAND
      && invoke.acl.is_none()
    {
      #[cfg(debug_assertions)]
      {
        let (key, command_name) = plugin_command
          .clone()
          .unwrap_or_else(|| (APP_ACL_KEY, request.cmd.clone()));
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
                    .map(|id| id.channel_on(webview.clone()));
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
    event: &str,
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
  pub(crate) fn unlisten_js(&self, event: &str, id: EventId) -> crate::Result<()> {
    let listeners = self.manager().listeners();

    self.eval(&crate::event::unlisten_js_script(
      listeners.listeners_object_name(),
      event,
      id,
    ))?;

    listeners.unlisten_js(id);

    Ok(())
  }

  pub(crate) fn emit_js(&self, emit_args: &EmitArgs, target: &EventTarget) -> crate::Result<()> {
    self.eval(&crate::event::emit_js_script(
      self.manager().listeners().function_name(),
      emit_args,
      &serde_json::to_string(target)?,
    )?)?;
    Ok(())
  }

  /// Opens the developer tools window (Web Inspector).
  /// The devtools is only enabled on debug builds or with the `devtools` feature flag.
  ///
  /// ## Platform-specific
  ///
  /// - **macOS:** Only supported on macOS 10.15+.
  /// This is a private API on macOS, so you cannot use this if your application will be published on the App Store.
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
  /// This is a private API on macOS, so you cannot use this if your application will be published on the App Store.
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
  /// This is a private API on macOS, so you cannot use this if your application will be published on the App Store.
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
}

/// Event system APIs.
impl<R: Runtime> Webview<R> {
  /// Listen to an event on this webview.
  ///
  /// # Examples
  #[cfg_attr(
    feature = "unstable",
    doc = r####"
```
use tauri::Manager;

tauri::Builder::default()
  .setup(|app| {
    let webview = app.get_webview("main").unwrap();
    webview.listen("component-loaded", move |event| {
      println!("window just loaded a component");
    });

    Ok(())
  });
```
  "####
  )]
  pub fn listen<F>(&self, event: impl Into<String>, handler: F) -> EventId
  where
    F: Fn(Event) + Send + 'static,
  {
    self.manager.listen(
      event.into(),
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
use tauri::Manager;

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
  pub fn unlisten(&self, id: EventId) {
    self.manager.unlisten(id)
  }

  /// Listen to an event on this webview only once.
  ///
  /// See [`Self::listen`] for more information.
  pub fn once<F>(&self, event: impl Into<String>, handler: F) -> EventId
  where
    F: FnOnce(Event) + Send + 'static,
  {
    self.manager.once(
      event.into(),
      EventTarget::Webview {
        label: self.label().to_string(),
      },
      handler,
    )
  }
}

impl<R: Runtime> Manager<R> for Webview<R> {}

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

#[cfg(test)]
mod tests {
  #[test]
  fn webview_is_send_sync() {
    crate::test_utils::assert_send::<super::Webview>();
    crate::test_utils::assert_sync::<super::Webview>();
  }
}
