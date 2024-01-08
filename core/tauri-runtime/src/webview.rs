// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A layer between raw [`Runtime`] webviews and Tauri.
//!
use crate::{
  window::{
    dpi::{Position, Size},
    is_label_valid, DetachedWindow,
  },
  Runtime, UserEvent,
};

use tauri_utils::config::{WebviewUrl, WindowConfig, WindowEffectsConfig};
use url::Url;

use std::{
  borrow::Cow,
  collections::HashMap,
  hash::{Hash, Hasher},
  path::PathBuf,
  sync::Arc,
};

type UriSchemeProtocol = dyn Fn(http::Request<Vec<u8>>, Box<dyn FnOnce(http::Response<Cow<'static, [u8]>>) + Send>)
  + Send
  + Sync
  + 'static;

type WebResourceRequestHandler =
  dyn Fn(http::Request<Vec<u8>>, &mut http::Response<Cow<'static, [u8]>>) + Send + Sync;

type NavigationHandler = dyn Fn(&Url) -> bool + Send;

type OnPageLoadHandler = dyn Fn(Url, PageLoadEvent) + Send;

type DownloadHandler = dyn Fn(DownloadEvent) -> bool + Send + Sync;

/// Download event.
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
    path: Option<PathBuf>,
    /// Indicates if the download succeeded or not.
    success: bool,
  },
}

#[cfg(target_os = "android")]
pub struct CreationContext<'a, 'b> {
  pub env: &'a mut jni::JNIEnv<'b>,
  pub activity: &'a jni::objects::JObject<'b>,
  pub webview: &'a jni::objects::JObject<'b>,
}

/// Kind of event for the page load handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLoadEvent {
  /// Page started to load.
  Started,
  /// Page finished loading.
  Finished,
}

/// A webview that has yet to be built.
pub struct PendingWebview<T: UserEvent, R: Runtime<T>> {
  /// The label that the webview will be named.
  pub label: String,

  /// The [`WebviewAttributes`] that the webview will be created with.
  pub webview_attributes: WebviewAttributes,

  pub uri_scheme_protocols: HashMap<String, Box<UriSchemeProtocol>>,

  /// How to handle IPC calls on the webview.
  pub ipc_handler: Option<WebviewIpcHandler<T, R>>,

  /// A handler to decide if incoming url is allowed to navigate.
  pub navigation_handler: Option<Box<NavigationHandler>>,

  /// The resolved URL to load on the webview.
  pub url: String,

  #[cfg(target_os = "android")]
  #[allow(clippy::type_complexity)]
  pub on_webview_created:
    Option<Box<dyn Fn(CreationContext<'_, '_>) -> Result<(), jni::errors::Error> + Send>>,

  pub web_resource_request_handler: Option<Box<WebResourceRequestHandler>>,

  pub on_page_load_handler: Option<Box<OnPageLoadHandler>>,

  pub download_handler: Option<Arc<DownloadHandler>>,
}

impl<T: UserEvent, R: Runtime<T>> PendingWebview<T, R> {
  /// Create a new [`PendingWebview`] with a label from the given [`WebviewAttributes`].
  pub fn new(
    webview_attributes: WebviewAttributes,
    label: impl Into<String>,
  ) -> crate::Result<Self> {
    let label = label.into();
    if !is_label_valid(&label) {
      Err(crate::Error::InvalidWindowLabel)
    } else {
      Ok(Self {
        webview_attributes,
        uri_scheme_protocols: Default::default(),
        label,
        ipc_handler: None,
        navigation_handler: None,
        url: "tauri://localhost".to_string(),
        #[cfg(target_os = "android")]
        on_webview_created: None,
        web_resource_request_handler: None,
        on_page_load_handler: None,
        download_handler: None,
      })
    }
  }

  pub fn register_uri_scheme_protocol<
    N: Into<String>,
    H: Fn(http::Request<Vec<u8>>, Box<dyn FnOnce(http::Response<Cow<'static, [u8]>>) + Send>)
      + Send
      + Sync
      + 'static,
  >(
    &mut self,
    uri_scheme: N,
    protocol: H,
  ) {
    let uri_scheme = uri_scheme.into();
    self
      .uri_scheme_protocols
      .insert(uri_scheme, Box::new(protocol));
  }

  #[cfg(target_os = "android")]
  pub fn on_webview_created<
    F: Fn(CreationContext<'_, '_>) -> Result<(), jni::errors::Error> + Send + 'static,
  >(
    mut self,
    f: F,
  ) -> Self {
    self.on_webview_created.replace(Box::new(f));
    self
  }
}

/// A webview that is not yet managed by Tauri.
#[derive(Debug)]
pub struct DetachedWebview<T: UserEvent, R: Runtime<T>> {
  /// Name of the window
  pub label: String,

  /// The [`WebviewDispatch`] associated with the window.
  pub dispatcher: R::WebviewDispatcher,
}

impl<T: UserEvent, R: Runtime<T>> Clone for DetachedWebview<T, R> {
  fn clone(&self) -> Self {
    Self {
      label: self.label.clone(),
      dispatcher: self.dispatcher.clone(),
    }
  }
}

impl<T: UserEvent, R: Runtime<T>> Hash for DetachedWebview<T, R> {
  /// Only use the [`DetachedWebview`]'s label to represent its hash.
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.label.hash(state)
  }
}

impl<T: UserEvent, R: Runtime<T>> Eq for DetachedWebview<T, R> {}
impl<T: UserEvent, R: Runtime<T>> PartialEq for DetachedWebview<T, R> {
  /// Only use the [`DetachedWebview`]'s label to compare equality.
  fn eq(&self, other: &Self) -> bool {
    self.label.eq(&other.label)
  }
}

/// The attributes used to create an webview.
#[derive(Debug, Clone)]
pub struct WebviewAttributes {
  pub url: WebviewUrl,
  pub user_agent: Option<String>,
  pub initialization_scripts: Vec<String>,
  pub data_directory: Option<PathBuf>,
  pub file_drop_handler_enabled: bool,
  pub clipboard: bool,
  pub accept_first_mouse: bool,
  pub additional_browser_args: Option<String>,
  pub window_effects: Option<WindowEffectsConfig>,
  pub incognito: bool,
  pub transparent: bool,
  pub bounds: Option<(Position, Size)>,
  pub auto_resize: bool,
}

impl From<&WindowConfig> for WebviewAttributes {
  fn from(config: &WindowConfig) -> Self {
    let mut builder = Self::new(config.url.clone());
    builder = builder.incognito(config.incognito);
    #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
    {
      builder = builder.transparent(config.transparent);
    }
    builder = builder.accept_first_mouse(config.accept_first_mouse);
    if !config.file_drop_enabled {
      builder = builder.disable_file_drop_handler();
    }
    if let Some(user_agent) = &config.user_agent {
      builder = builder.user_agent(user_agent);
    }
    if let Some(additional_browser_args) = &config.additional_browser_args {
      builder = builder.additional_browser_args(additional_browser_args);
    }
    if let Some(effects) = &config.window_effects {
      builder = builder.window_effects(effects.clone());
    }
    builder
  }
}

impl WebviewAttributes {
  /// Initializes the default attributes for a webview.
  pub fn new(url: WebviewUrl) -> Self {
    Self {
      url,
      user_agent: None,
      initialization_scripts: Vec::new(),
      data_directory: None,
      file_drop_handler_enabled: true,
      clipboard: false,
      accept_first_mouse: false,
      additional_browser_args: None,
      window_effects: None,
      incognito: false,
      transparent: false,
      bounds: None,
      auto_resize: false,
    }
  }

  /// Sets the user agent
  #[must_use]
  pub fn user_agent(mut self, user_agent: &str) -> Self {
    self.user_agent = Some(user_agent.to_string());
    self
  }

  /// Sets the init script.
  #[must_use]
  pub fn initialization_script(mut self, script: &str) -> Self {
    self.initialization_scripts.push(script.to_string());
    self
  }

  /// Data directory for the webview.
  #[must_use]
  pub fn data_directory(mut self, data_directory: PathBuf) -> Self {
    self.data_directory.replace(data_directory);
    self
  }

  /// Disables the file drop handler. This is required to use drag and drop APIs on the front end on Windows.
  #[must_use]
  pub fn disable_file_drop_handler(mut self) -> Self {
    self.file_drop_handler_enabled = false;
    self
  }

  /// Enables clipboard access for the page rendered on **Linux** and **Windows**.
  ///
  /// **macOS** doesn't provide such method and is always enabled by default,
  /// but you still need to add menu item accelerators to use shortcuts.
  #[must_use]
  pub fn enable_clipboard_access(mut self) -> Self {
    self.clipboard = true;
    self
  }

  /// Sets whether clicking an inactive window also clicks through to the webview.
  #[must_use]
  pub fn accept_first_mouse(mut self, accept: bool) -> Self {
    self.accept_first_mouse = accept;
    self
  }

  /// Sets additional browser arguments. **Windows Only**
  #[must_use]
  pub fn additional_browser_args(mut self, additional_args: &str) -> Self {
    self.additional_browser_args = Some(additional_args.to_string());
    self
  }

  /// Sets window effects
  #[must_use]
  pub fn window_effects(mut self, effects: WindowEffectsConfig) -> Self {
    self.window_effects = Some(effects);
    self
  }

  /// Enable or disable incognito mode for the WebView.
  #[must_use]
  pub fn incognito(mut self, incognito: bool) -> Self {
    self.incognito = incognito;
    self
  }

  /// Enable or disable transparency for the WebView.
  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  #[must_use]
  pub fn transparent(mut self, transparent: bool) -> Self {
    self.transparent = transparent;
    self
  }

  /// Sets the webview to automatically grow and shrink its size and position when the parent window resizes.
  #[must_use]
  pub fn auto_resize(mut self) -> Self {
    self.auto_resize = true;
    self
  }
}

/// IPC handler.
pub type WebviewIpcHandler<T, R> = Box<dyn Fn(DetachedWindow<T, R>, String) + Send>;
