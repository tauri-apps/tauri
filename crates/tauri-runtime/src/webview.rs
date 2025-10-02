// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A layer between raw [`Runtime`] webviews and Tauri.
//!
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::window::WindowId;
use crate::{window::is_label_valid, Rect, Runtime, UserEvent};

use http::Request;
use tauri_utils::config::{
  BackgroundThrottlingPolicy, Color, ScrollBarStyle as ConfigScrollBarStyle, WebviewUrl,
  WindowConfig, WindowEffectsConfig,
};
use url::Url;

use std::{
  borrow::Cow,
  collections::HashMap,
  hash::{Hash, Hasher},
  path::PathBuf,
  sync::Arc,
};

type UriSchemeProtocol = dyn Fn(&str, http::Request<Vec<u8>>, Box<dyn FnOnce(http::Response<Cow<'static, [u8]>>) + Send>)
  + Send
  + Sync
  + 'static;

type WebResourceRequestHandler =
  dyn Fn(http::Request<Vec<u8>>, &mut http::Response<Cow<'static, [u8]>>) + Send + Sync;

type NavigationHandler = dyn Fn(&Url) -> bool + Send;

type NewWindowHandler = dyn Fn(Url, NewWindowFeatures) -> NewWindowResponse + Send + Sync;

type OnPageLoadHandler = dyn Fn(Url, PageLoadEvent) + Send;

type DocumentTitleChangedHandler = dyn Fn(String) + Send + 'static;

type DownloadHandler = dyn Fn(DownloadEvent) -> bool + Send + Sync;

#[cfg(target_os = "ios")]
type InputAccessoryViewBuilderFn = dyn Fn(&objc2_ui_kit::UIView) -> Option<objc2::rc::Retained<objc2_ui_kit::UIView>>
  + Send
  + Sync
  + 'static;

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

/// Information about the webview that initiated a new window request.
#[derive(Debug)]
pub struct NewWindowOpener {
  /// The instance of the webview that initiated the new window request.
  ///
  /// This must be set as the related view of the new webview. See [`WebviewAttributes::related_view`].
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
  ))]
  pub webview: webkit2gtk::WebView,
  /// The instance of the webview that initiated the new window request.
  ///
  /// The target webview environment **MUST** match the environment of the opener webview. See [`WebviewAttributes::environment`].
  #[cfg(windows)]
  pub webview: webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2,
  #[cfg(windows)]
  pub environment: webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment,
  /// The instance of the webview that initiated the new window request.
  #[cfg(target_os = "macos")]
  pub webview: objc2::rc::Retained<objc2_web_kit::WKWebView>,
  /// Configuration of the target webview.
  ///
  /// This **MUST** be used when creating the target webview. See [`WebviewAttributes::webview_configuration`].
  #[cfg(target_os = "macos")]
  pub target_configuration: objc2::rc::Retained<objc2_web_kit::WKWebViewConfiguration>,
}

/// Window features of a window requested to open.
#[derive(Debug)]
pub struct NewWindowFeatures {
  pub(crate) size: Option<crate::dpi::LogicalSize<f64>>,
  pub(crate) position: Option<crate::dpi::LogicalPosition<f64>>,
  pub(crate) opener: NewWindowOpener,
}

impl NewWindowFeatures {
  pub fn new(
    size: Option<crate::dpi::LogicalSize<f64>>,
    position: Option<crate::dpi::LogicalPosition<f64>>,
    opener: NewWindowOpener,
  ) -> Self {
    Self {
      size,
      position,
      opener,
    }
  }

  /// Specifies the size of the content area
  /// as defined by the user's operating system where the new window will be generated.
  pub fn size(&self) -> Option<crate::dpi::LogicalSize<f64>> {
    self.size
  }

  /// Specifies the position of the window relative to the work area
  /// as defined by the user's operating system where the new window will be generated.
  pub fn position(&self) -> Option<crate::dpi::LogicalPosition<f64>> {
    self.position
  }

  /// Returns information about the webview that initiated a new window request.
  pub fn opener(&self) -> &NewWindowOpener {
    &self.opener
  }
}

/// Response for the new window request handler.
pub enum NewWindowResponse {
  /// Allow the window to be opened with the default implementation.
  Allow,
  /// Allow the window to be opened, with the given window.
  ///
  /// ## Platform-specific:
  ///
  /// **Linux**: The webview must be related to the caller webview. See [`WebviewAttributes::related_view`].
  /// **Windows**: The webview must use the same environment as the caller webview. See [`WebviewAttributes::environment`].
  #[cfg(not(any(target_os = "android", target_os = "ios")))]
  Create { window_id: WindowId },
  /// Deny the window from being opened.
  Deny,
}

/// The scrollbar style to use in the webview.
///
/// ## Platform-specific
///
/// - **Windows**: This option must be given the same value for all webviews that target the same data directory.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default)]
pub enum ScrollBarStyle {
  #[default]
  /// The default scrollbar style for the webview.
  Default,

  #[cfg(windows)]
  /// Fluent UI style overlay scrollbars. **Windows Only**
  ///
  /// Requires WebView2 Runtime version 125.0.2535.41 or higher, does nothing on older versions,
  /// see https://learn.microsoft.com/en-us/microsoft-edge/webview2/release-notes/?tabs=dotnetcsharp#10253541
  FluentOverlay,
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

  pub new_window_handler: Option<Box<NewWindowHandler>>,

  pub document_title_changed_handler: Option<Box<DocumentTitleChangedHandler>>,

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
        new_window_handler: None,
        document_title_changed_handler: None,
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
    H: Fn(&str, http::Request<Vec<u8>>, Box<dyn FnOnce(http::Response<Cow<'static, [u8]>>) + Send>)
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

  /// The [`crate::WebviewDispatch`] associated with the window.
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
#[derive(Debug)]
pub struct WebviewAttributes {
  pub url: WebviewUrl,
  pub user_agent: Option<String>,
  /// A list of initialization javascript scripts to run when loading new pages.
  /// When webview load a new page, this initialization code will be executed.
  /// It is guaranteed that code is executed before `window.onload`.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows:** scripts are always added to subframes.
  /// - **Android:** When [addDocumentStartJavaScript] is not supported,
  ///   we prepend initialization scripts to each HTML head (implementation only supported on custom protocol URLs).
  ///   For remote URLs, we use [onPageStarted] which is not guaranteed to run before other scripts.
  ///
  /// [addDocumentStartJavaScript]: https://developer.android.com/reference/androidx/webkit/WebViewCompat#addDocumentStartJavaScript(android.webkit.WebView,java.lang.String,java.util.Set%3Cjava.lang.String%3E)
  /// [onPageStarted]: https://developer.android.com/reference/android/webkit/WebViewClient#onPageStarted(android.webkit.WebView,%20java.lang.String,%20android.graphics.Bitmap)
  pub initialization_scripts: Vec<InitializationScript>,
  pub data_directory: Option<PathBuf>,
  pub drag_drop_handler_enabled: bool,
  pub clipboard: bool,
  pub accept_first_mouse: bool,
  pub additional_browser_args: Option<String>,
  pub window_effects: Option<WindowEffectsConfig>,
  pub incognito: bool,
  pub transparent: bool,
  pub focus: bool,
  pub bounds: Option<Rect>,
  pub auto_resize: bool,
  pub proxy_url: Option<Url>,
  pub zoom_hotkeys_enabled: bool,
  pub browser_extensions_enabled: bool,
  pub extensions_path: Option<PathBuf>,
  pub data_store_identifier: Option<[u8; 16]>,
  pub use_https_scheme: bool,
  pub devtools: Option<bool>,
  pub background_color: Option<Color>,
  pub traffic_light_position: Option<dpi::Position>,
  pub background_throttling: Option<BackgroundThrottlingPolicy>,
  pub javascript_disabled: bool,
  /// on macOS and iOS there is a link preview on long pressing links, this is enabled by default.
  /// see https://docs.rs/objc2-web-kit/latest/objc2_web_kit/struct.WKWebView.html#method.allowsLinkPreview
  pub allow_link_preview: bool,
  pub scroll_bar_style: ScrollBarStyle,
  /// Allows overriding the the keyboard accessory view on iOS.
  /// Returning `None` effectively removes the view.
  ///
  /// The closure parameter is the webview instance.
  ///
  /// The accessory view is the view that appears above the keyboard when a text input element is focused.
  /// It usually displays a view with "Done", "Next" buttons.
  ///
  /// # Stability
  ///
  /// This relies on [`objc2_ui_kit`] which does not provide a stable API yet, so it can receive breaking changes in minor releases.
  #[cfg(target_os = "ios")]
  pub input_accessory_view_builder: Option<InputAccessoryViewBuilder>,

  /// Set the environment for the webview.
  /// Useful if you need to share the same environment, for instance when using the [`PendingWebview::new_window_handler`].
  #[cfg(windows)]
  pub environment: Option<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment>,

  /// Creates a new webview sharing the same web process with the provided webview.
  /// Useful if you need to link a webview to another, for instance when using the [`PendingWebview::new_window_handler`].
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
  ))]
  pub related_view: Option<webkit2gtk::WebView>,

  #[cfg(target_os = "macos")]
  pub webview_configuration: Option<objc2::rc::Retained<objc2_web_kit::WKWebViewConfiguration>>,
}

unsafe impl Send for WebviewAttributes {}
unsafe impl Sync for WebviewAttributes {}

#[cfg(target_os = "ios")]
#[non_exhaustive]
pub struct InputAccessoryViewBuilder(pub Box<InputAccessoryViewBuilderFn>);

#[cfg(target_os = "ios")]
impl std::fmt::Debug for InputAccessoryViewBuilder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
    f.debug_struct("InputAccessoryViewBuilder").finish()
  }
}

#[cfg(target_os = "ios")]
impl InputAccessoryViewBuilder {
  pub fn new(builder: Box<InputAccessoryViewBuilderFn>) -> Self {
    Self(builder)
  }
}

impl From<&WindowConfig> for WebviewAttributes {
  fn from(config: &WindowConfig) -> Self {
    let mut builder = Self::new(config.url.clone())
      .incognito(config.incognito)
      .focused(config.focus)
      .zoom_hotkeys_enabled(config.zoom_hotkeys_enabled)
      .use_https_scheme(config.use_https_scheme)
      .browser_extensions_enabled(config.browser_extensions_enabled)
      .background_throttling(config.background_throttling.clone())
      .devtools(config.devtools)
      .scroll_bar_style(match config.scroll_bar_style {
        ConfigScrollBarStyle::Default => ScrollBarStyle::Default,
        #[cfg(windows)]
        ConfigScrollBarStyle::FluentOverlay => ScrollBarStyle::FluentOverlay,
        _ => ScrollBarStyle::Default,
      });

    #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
    {
      builder = builder.transparent(config.transparent);
    }
    #[cfg(target_os = "macos")]
    {
      if let Some(position) = &config.traffic_light_position {
        builder =
          builder.traffic_light_position(dpi::LogicalPosition::new(position.x, position.y).into());
      }
    }
    builder = builder.accept_first_mouse(config.accept_first_mouse);
    if !config.drag_drop_enabled {
      builder = builder.disable_drag_drop_handler();
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
    if let Some(url) = &config.proxy_url {
      builder = builder.proxy_url(url.to_owned());
    }
    if let Some(color) = config.background_color {
      builder = builder.background_color(color);
    }
    builder.javascript_disabled = config.javascript_disabled;
    builder.allow_link_preview = config.allow_link_preview;
    #[cfg(target_os = "ios")]
    if config.disable_input_accessory_view {
      builder
        .input_accessory_view_builder
        .replace(InputAccessoryViewBuilder::new(Box::new(|_webview| None)));
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
      drag_drop_handler_enabled: true,
      clipboard: false,
      accept_first_mouse: false,
      additional_browser_args: None,
      window_effects: None,
      incognito: false,
      transparent: false,
      focus: true,
      bounds: None,
      auto_resize: false,
      proxy_url: None,
      zoom_hotkeys_enabled: false,
      browser_extensions_enabled: false,
      data_store_identifier: None,
      extensions_path: None,
      use_https_scheme: false,
      devtools: None,
      background_color: None,
      traffic_light_position: None,
      background_throttling: None,
      javascript_disabled: false,
      allow_link_preview: true,
      scroll_bar_style: ScrollBarStyle::Default,
      #[cfg(target_os = "ios")]
      input_accessory_view_builder: None,
      #[cfg(windows)]
      environment: None,
      #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
      ))]
      related_view: None,
      #[cfg(target_os = "macos")]
      webview_configuration: None,
    }
  }

  /// Sets the user agent
  #[must_use]
  pub fn user_agent(mut self, user_agent: &str) -> Self {
    self.user_agent = Some(user_agent.to_string());
    self
  }

  /// Adds an init script for the main frame.
  ///
  /// When webview load a new page, this initialization code will be executed.
  /// It is guaranteed that code is executed before `window.onload`.
  ///
  /// This is executed only on the main frame.
  /// If you only want to run it in all frames, use [`Self::initialization_script_on_all_frames`] instead.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows:** scripts are always added to subframes.
  /// - **Android:** When [addDocumentStartJavaScript] is not supported,
  ///   we prepend initialization scripts to each HTML head (implementation only supported on custom protocol URLs).
  ///   For remote URLs, we use [onPageStarted] which is not guaranteed to run before other scripts.
  ///
  /// [addDocumentStartJavaScript]: https://developer.android.com/reference/androidx/webkit/WebViewCompat#addDocumentStartJavaScript(android.webkit.WebView,java.lang.String,java.util.Set%3Cjava.lang.String%3E)
  /// [onPageStarted]: https://developer.android.com/reference/android/webkit/WebViewClient#onPageStarted(android.webkit.WebView,%20java.lang.String,%20android.graphics.Bitmap)
  #[must_use]
  pub fn initialization_script(mut self, script: impl Into<String>) -> Self {
    self.initialization_scripts.push(InitializationScript {
      script: script.into(),
      for_main_frame_only: true,
    });
    self
  }

  /// Adds an init script for all frames.
  ///
  /// When webview load a new page, this initialization code will be executed.
  /// It is guaranteed that code is executed before `window.onload`.
  ///
  /// This is executed on all frames, main frame and also sub frames.
  /// If you only want to run it in the main frame, use [`Self::initialization_script`] instead.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows:** scripts are always added to subframes.
  /// - **Android:** When [addDocumentStartJavaScript] is not supported,
  ///   we prepend initialization scripts to each HTML head (implementation only supported on custom protocol URLs).
  ///   For remote URLs, we use [onPageStarted] which is not guaranteed to run before other scripts.
  ///
  /// [addDocumentStartJavaScript]: https://developer.android.com/reference/androidx/webkit/WebViewCompat#addDocumentStartJavaScript(android.webkit.WebView,java.lang.String,java.util.Set%3Cjava.lang.String%3E)
  /// [onPageStarted]: https://developer.android.com/reference/android/webkit/WebViewClient#onPageStarted(android.webkit.WebView,%20java.lang.String,%20android.graphics.Bitmap)
  #[must_use]
  pub fn initialization_script_on_all_frames(mut self, script: impl Into<String>) -> Self {
    self.initialization_scripts.push(InitializationScript {
      script: script.into(),
      for_main_frame_only: false,
    });
    self
  }

  /// Data directory for the webview.
  #[must_use]
  pub fn data_directory(mut self, data_directory: PathBuf) -> Self {
    self.data_directory.replace(data_directory);
    self
  }

  /// Disables the drag and drop handler. This is required to use HTML5 drag and drop APIs on the frontend on Windows.
  #[must_use]
  pub fn disable_drag_drop_handler(mut self) -> Self {
    self.drag_drop_handler_enabled = false;
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

  /// Whether the webview should be focused or not.
  #[must_use]
  pub fn focused(mut self, focus: bool) -> Self {
    self.focus = focus;
    self
  }

  /// Sets the webview to automatically grow and shrink its size and position when the parent window resizes.
  #[must_use]
  pub fn auto_resize(mut self) -> Self {
    self.auto_resize = true;
    self
  }

  /// Enable proxy for the WebView
  #[must_use]
  pub fn proxy_url(mut self, url: Url) -> Self {
    self.proxy_url = Some(url);
    self
  }

  /// Whether page zooming by hotkeys is enabled
  ///
  /// ## Platform-specific:
  ///
  /// - **Windows**: Controls WebView2's [`IsZoomControlEnabled`](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2settings?view=webview2-winrt-1.0.2420.47#iszoomcontrolenabled) setting.
  /// - **MacOS / Linux**: Injects a polyfill that zooms in and out with `ctrl/command` + `-/=`,
  ///   20% in each step, ranging from 20% to 1000%. Requires `webview:allow-set-webview-zoom` permission
  ///
  /// - **Android / iOS**: Unsupported.
  #[must_use]
  pub fn zoom_hotkeys_enabled(mut self, enabled: bool) -> Self {
    self.zoom_hotkeys_enabled = enabled;
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
    self.browser_extensions_enabled = enabled;
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
    self.use_https_scheme = enabled;
    self
  }

  /// Whether web inspector, which is usually called browser devtools, is enabled or not. Enabled by default.
  ///
  /// This API works in **debug** builds, but requires `devtools` feature flag to enable it in **release** builds.
  ///
  /// ## Platform-specific
  ///
  /// - macOS: This will call private functions on **macOS**.
  /// - Android: Open `chrome://inspect/#devices` in Chrome to get the devtools window. Wry's `WebView` devtools API isn't supported on Android.
  /// - iOS: Open Safari > Develop > [Your Device Name] > [Your WebView] to get the devtools window.
  #[must_use]
  pub fn devtools(mut self, enabled: Option<bool>) -> Self {
    self.devtools = enabled;
    self
  }

  /// Set the window and webview background color.
  /// ## Platform-specific:
  ///
  /// - **Windows**: On Windows 7, alpha channel is ignored for the webview layer.
  /// - **Windows**: On Windows 8 and newer, if alpha channel is not `0`, it will be ignored.
  #[must_use]
  pub fn background_color(mut self, color: Color) -> Self {
    self.background_color = Some(color);
    self
  }

  /// Change the position of the window controls. Available on macOS only.
  ///
  /// Requires titleBarStyle: Overlay and decorations: true.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / Windows / iOS / Android:** Unsupported.
  #[must_use]
  pub fn traffic_light_position(mut self, position: dpi::Position) -> Self {
    self.traffic_light_position = Some(position);
    self
  }

  /// Whether to show a link preview when long pressing on links. Available on macOS and iOS only.
  ///
  /// Default is true.
  ///
  /// See https://docs.rs/objc2-web-kit/latest/objc2_web_kit/struct.WKWebView.html#method.allowsLinkPreview
  ///
  /// ## Platform-specific
  ///
  /// - **Linux / Windows / Android:** Unsupported.
  #[must_use]
  pub fn allow_link_preview(mut self, allow_link_preview: bool) -> Self {
    self.allow_link_preview = allow_link_preview;
    self
  }

  /// Change the default background throttling behavior.
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
  pub fn background_throttling(mut self, policy: Option<BackgroundThrottlingPolicy>) -> Self {
    self.background_throttling = policy;
    self
  }

  /// Specifies the native scrollbar style to use with the webview.
  /// CSS styles that modify the scrollbar are applied on top of the native appearance configured here.
  ///
  /// Defaults to [`ScrollBarStyle::Default`], which is the browser default.
  ///
  /// ## Platform-specific
  ///
  /// - **Windows**:
  ///   - [`ScrollBarStyle::FluentOverlay`] requires WebView2 Runtime version 125.0.2535.41 or higher,
  ///     and does nothing on older versions.
  ///   - This option must be given the same value for all webviews that target the same data directory. Use
  ///     [`WebviewAttributes::data_directory`] to change data directories if needed.
  /// - **Linux / Android / iOS / macOS**: Unsupported. Only supports `Default` and performs no operation.
  #[must_use]
  pub fn scroll_bar_style(mut self, style: ScrollBarStyle) -> Self {
    self.scroll_bar_style = style;
    self
  }
}

/// IPC handler.
pub type WebviewIpcHandler<T, R> = Box<dyn Fn(DetachedWebview<T, R>, Request<String>) + Send>;

/// An initialization script
#[derive(Debug, Clone)]
pub struct InitializationScript {
  /// The script to run
  pub script: String,
  /// Whether the script should be injected to main frame only
  pub for_main_frame_only: bool,
}
