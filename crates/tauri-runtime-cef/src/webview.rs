// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::{
  Mutex,
  atomic::Ordering,
  mpsc::{self, Receiver, Sender},
};

use cef::*;
use sha2::{Digest, Sha256};
use tauri_runtime::{
  Cookie, Error, Result, Runtime, UserEvent, WebviewDispatch, WebviewEventId,
  dpi::{PhysicalPosition, PhysicalSize, Position, Rect, Size},
  webview::{
    DetachedWebview, InitializationScript, PendingWebview, UriSchemeProtocolHandler,
    WebviewAttributes,
  },
  window::{WebviewEvent, WindowId},
};
use tauri_utils::{Theme, config::Color, html::normalize_script_for_csp};
use url::Url;

use crate::cef_impl::{client as browser_client, cookie, request_context, request_handler};
use crate::runtime::{CefRuntime, Message, RuntimeContext, WinitCefApp};
use crate::window::AppWindow;

/// A handle to the native CEF browser backing a Tauri webview.
///
/// This is the runtime-specific webview object exposed through
/// [`tauri_runtime::WebviewDispatch::with_webview`].
#[derive(Clone)]
pub struct Webview {
  browser: cef::Browser,
  snapshot: WebviewSnapshot,
  frame_navigation_state: crate::FrameNavigationState,
  popups: Vec<Webview>,
  opener: Option<crate::FrameNavigationState>,
}

/// Native state sampled on the CEF UI thread immediately before a
/// `with_webview` callback. This does not assert renderer responsiveness or
/// that the view is unobscured on screen.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct WebviewSnapshot {
  /// Native browser identity, distinct for each popup.
  pub browser_id: i32,
  /// Native JavaScript dialog observation. Unknown is distinct from absent.
  pub dialogs: crate::NativeDialogObservation,
  /// All-frame document generation validated against the current native frame
  /// identities and load state. `None` means document admission is unavailable.
  pub document: Option<crate::NativeDocumentToken>,
  /// Runtime window label; CEF-owned popups have no Tauri window label.
  pub window_label: Option<String>,
  /// Opaque lifetime of the runtime window. Labels and native handle values
  /// may be reused after teardown; this token distinguishes their replacements.
  /// `None` means the runtime could not observe a native window lifetime.
  pub window: Option<crate::NativeWindowToken>,
  /// Whether the actual native parent matches the observed native window.
  /// `None` means the platform could not establish the relationship. A
  /// CEF-owned popup always reports `None`, permanently rather than
  /// transiently: CEF owns its native window, so there is no independently
  /// observed parent for the runtime to check it against.
  pub parent_matches: Option<bool>,
  /// Current bounds relative to the native parent, in the indicated DPI units.
  pub bounds: Option<Rect>,
  /// Native view visibility. `None` means native inspection was unavailable.
  /// Visibility is separate from occlusion, minimization, and page lifecycle.
  pub visible: Option<bool>,
}

impl Webview {
  pub(crate) fn new(
    browser: cef::Browser,
    snapshot: WebviewSnapshot,
    frame_navigation_state: crate::FrameNavigationState,
  ) -> Self {
    Self {
      browser,
      snapshot,
      frame_navigation_state,
      popups: Vec::new(),
      opener: None,
    }
  }

  /// Returns the native state sampled for this `with_webview` callback.
  /// Retaining the handle does not refresh this observation.
  pub fn snapshot(&self) -> &WebviewSnapshot {
    &self.snapshot
  }

  /// Returns read-only live navigation state for this native browser lifetime.
  /// Unlike `snapshot`, this handle follows subsequent native frame events.
  pub fn frame_navigation_state(&self) -> &crate::FrameNavigationState {
    &self.frame_navigation_state
  }

  pub(crate) fn set_opener(&mut self, opener: crate::FrameNavigationState) {
    self.opener = Some(opener);
  }

  /// Native CEF-owned popup descendants sampled in this same UI-thread callback.
  /// Popup windows have no Tauri label and retain their actual CEF opener.
  pub fn popups(&self) -> &[Webview] {
    &self.popups
  }

  /// Exact native opener lifetime, if this is a CEF-owned popup.
  pub fn opener(&self) -> Option<&crate::FrameNavigationState> {
    self.opener.as_ref()
  }

  /// Select an observed document within this runtime-owned browser family.
  /// The returned snapshot is valid only for the current native callback.
  pub fn for_document(&self, document: &crate::NativeDocumentToken) -> Option<&Webview> {
    std::iter::once(self)
      .chain(self.popups.iter())
      .find(|view| view.snapshot.document.as_ref() == Some(document))
  }

  /// Returns the [`cef::Browser`] backing this webview.
  ///
  /// From the browser you can reach the rest of the CEF API, such as the
  /// browser host, the main frame or the native window handle.
  pub fn browser(&self) -> cef::Browser {
    self.browser.clone()
  }
}

pub fn webview_version() -> tauri_runtime::Result<String> {
  Ok(format!(
    "{}.{}.{}.{}",
    cef::sys::CHROME_VERSION_MAJOR,
    cef::sys::CHROME_VERSION_MINOR,
    cef::sys::CHROME_VERSION_PATCH,
    cef::sys::CHROME_VERSION_BUILD
  ))
}

#[inline]
fn color_to_argb(color: Color) -> u32 {
  let (r, g, b, a) = color.into();
  ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Maps the subset of [`WebviewAttributes`] that CEF's `BrowserSettings`
/// supports.
///
/// The following Tauri webview attributes have no per-webview equivalent in CEF
/// and are intentionally ignored here:
/// - `user_agent`: CEF only exposes a process-global user agent via
///   `CefSettings.user_agent`, which is fixed before any webview is created.
/// - `additional_browser_args`, `scroll_bar_style`, `general_autofill_enabled`:
///   WebView2 (Windows)-only concepts.
/// - `allow_link_preview`, `accept_first_mouse`: WKWebView (macOS/iOS)-only.
/// - `browser_extensions_enabled`, `extensions_path`: CEF dropped extension
///   support in the Chrome runtime.
/// - `data_store_identifier`: a WKWebView data-store concept with no CEF analog
///   (per-webview isolation is done through the request context cache path).
///
/// `proxy_url` is handled separately via the request context preference, and
/// `zoom_hotkeys_enabled` through the client's command handler, because zoom
/// reaches a browser through Chromium's accelerator table rather than through a
/// browser setting.
fn browser_settings_from_webview_attributes(
  webview_attributes: &WebviewAttributes,
) -> cef::BrowserSettings {
  cef::BrowserSettings {
    javascript: cef::State::from(if webview_attributes.javascript_disabled {
      cef::sys::cef_state_t::STATE_DISABLED
    } else {
      cef::sys::cef_state_t::STATE_ENABLED
    }),
    javascript_access_clipboard: cef::State::from(if webview_attributes.clipboard {
      cef::sys::cef_state_t::STATE_ENABLED
    } else {
      cef::sys::cef_state_t::STATE_DISABLED
    }),
    background_color: webview_attributes
      .background_color
      .map(color_to_argb)
      .unwrap_or(0),
    // Browser chrome a Tauri window has no business showing: the status bubble
    // is the link target that slides in over the bottom-left of the page on
    // hover, and the zoom bubble the popup Chrome anchors to its (absent)
    // toolbar on Ctrl+Plus. Both draw over the app's own UI, neither is
    // something the app asked for, and both are ignored under Alloy style.
    chrome_status_bubble: cef::State::from(cef::sys::cef_state_t::STATE_DISABLED),
    chrome_zoom_bubble: cef::State::from(cef::sys::cef_state_t::STATE_DISABLED),
    ..Default::default()
  }
}

/// A Chrome DevTools Protocol notification observed on a native browser.
///
/// Observers see the whole browser, including requests issued by the runtime
/// itself and by other callers, so nothing here is scoped to one producer.
/// No notification names a browser, and none has to: an observer is registered
/// on one native browser and never receives another's traffic — a CEF-owned
/// popup is a separate browser, observed only by the runtime's own internal
/// observer.
#[derive(Debug, Clone)]
pub enum DevToolsProtocol {
  /// The raw agent message, before it is classified as an event or a result.
  Message(Vec<u8>),
  /// An agent event. Events are unsolicited and carry no request identifier.
  Event { method: String, params: Vec<u8> },
  /// The result of one request.
  ///
  /// `message_id` correlates with the `id` of the request that produced it.
  /// Compare it against an identifier obtained from
  /// [`allocate_devtools_message_id`](crate::allocate_devtools_message_id);
  /// a result whose identifier you did not allocate answers someone else's
  /// request. Numeric correlation does not authorize a browser or document.
  MethodResult {
    message_id: i32,
    success: bool,
    result: Vec<u8>,
  },
}

pub(crate) type DevToolsProtocolHandler = dyn Fn(DevToolsProtocol) + Send + Sync;

/// One message a renderer wrote to the JavaScript console.
///
/// Reported synchronously on CEF's UI thread, before CEF logs it as it normally
/// would; observing a message neither suppresses that logging nor changes what
/// DevTools shows.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ConsoleMessage {
  /// How severe the renderer considers the message.
  pub level: ConsoleMessageLevel,
  /// The message text, already formatted by the renderer the way DevTools shows
  /// it.
  pub message: String,
  /// What wrote the message — a script URL, usually. Empty when CEF names none.
  pub source: String,
  /// The 1-based line in `source`. Zero when CEF names none.
  pub line: i32,
}

/// The severity of a [`ConsoleMessage`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConsoleMessageLevel {
  /// `console.debug`.
  Verbose,
  /// `console.log` and `console.info`. Also CEF's default severity, which it
  /// documents as INFO.
  Info,
  /// `console.warn`.
  Warning,
  /// `console.error`, and messages the renderer itself reports as errors, such as
  /// an uncaught exception or a blocked subresource.
  Error,
  /// A fatal log severity. No console API produces one.
  Fatal,
  /// A severity this build of the runtime does not name.
  Other,
}

/// Synchronous observer of renderer console output.
pub type ConsoleMessageHandler = dyn Fn(ConsoleMessage) + Send + Sync + 'static;

impl ConsoleMessage {
  pub(crate) fn from_cef(
    level: cef::LogSeverity,
    message: Option<&CefString>,
    source: Option<&CefString>,
    line: i32,
  ) -> Self {
    use cef::sys::cef_log_severity_t;

    Self {
      level: match cef_log_severity_t::from(level) {
        cef_log_severity_t::LOGSEVERITY_VERBOSE => ConsoleMessageLevel::Verbose,
        cef_log_severity_t::LOGSEVERITY_DEFAULT | cef_log_severity_t::LOGSEVERITY_INFO => {
          ConsoleMessageLevel::Info
        }
        cef_log_severity_t::LOGSEVERITY_WARNING => ConsoleMessageLevel::Warning,
        cef_log_severity_t::LOGSEVERITY_ERROR => ConsoleMessageLevel::Error,
        cef_log_severity_t::LOGSEVERITY_FATAL => ConsoleMessageLevel::Fatal,
        _ => ConsoleMessageLevel::Other,
      },
      message: message.map(ToString::to_string).unwrap_or_default(),
      source: source.map(ToString::to_string).unwrap_or_default(),
      line,
    }
  }
}
pub(crate) type WebviewEventHandler = Box<dyn Fn(&WebviewEvent) + Send>;
pub(crate) type WebviewEventListeners = Arc<Mutex<HashMap<WebviewEventId, WebviewEventHandler>>>;

pub(crate) enum WebviewMessage {
  AddEventListener(WebviewEventId, Box<dyn Fn(&WebviewEvent) + Send>),
  EvaluateScript(String),
  EvaluateScriptWithCallback(String, Box<dyn Fn(String) + Send + 'static>),
  Navigate(Url),
  Reload,
  GoBack,
  CanGoBack(Sender<Result<bool>>),
  GoForward,
  CanGoForward(Sender<Result<bool>>),
  Print,
  Close,
  Show,
  Hide,
  SetPosition(Position),
  SetSize(Size),
  SetBounds(Rect),
  SetFocus,
  Reparent(WindowId, Sender<Result<()>>),
  SetAutoResize(bool),
  SetZoom(f64),
  SetBackgroundColor(Option<Color>),
  ClearAllBrowsingData,
  Url(Sender<Result<String>>),
  Bounds(Sender<Result<Rect>>),
  Position(Sender<Result<PhysicalPosition<i32>>>),
  Size(Sender<Result<PhysicalSize<u32>>>),
  WithWebview(Box<dyn FnOnce(Webview) + Send>),
  CookiesForUrl(Url, Sender<Result<Vec<Cookie<'static>>>>),
  Cookies(Sender<Result<Vec<Cookie<'static>>>>),
  SetCookie(Cookie<'static>),
  DeleteCookie(Cookie<'static>),
  #[cfg(any(debug_assertions, feature = "devtools"))]
  OpenDevTools,
  #[cfg(any(debug_assertions, feature = "devtools"))]
  CloseDevTools,
  #[cfg(any(debug_assertions, feature = "devtools"))]
  IsDevToolsOpen(Sender<bool>),
  SendDevToolsMessage(Vec<u8>, Sender<Result<()>>),
  OnDevToolsProtocol(Arc<DevToolsProtocolHandler>, Sender<Result<()>>),
}

/// A webview's bounds expressed as a fraction of its parent window, used to
/// reposition/resize auto-resize webviews when the parent window changes size.
#[derive(Clone, Copy)]
pub(crate) struct BoundsRate {
  pub(crate) x: f32,
  pub(crate) y: f32,
  pub(crate) width: f32,
  pub(crate) height: f32,
}

impl Default for BoundsRate {
  fn default() -> Self {
    Self {
      x: 0.,
      y: 0.,
      width: 1.,
      height: 1.,
    }
  }
}

pub(crate) struct AppWebview {
  pub(crate) webview_id: u32,
  pub(crate) label: String,
  pub(crate) browser: cef::Browser,
  pub(crate) browser_id: i32,
  pub(crate) frame_navigation_state: crate::FrameNavigationState,
  pub(crate) popup_family: Arc<crate::popup::PopupFamily>,
  pub(crate) dialogs: crate::dialog::DialogState,
  pub(crate) host: cef::BrowserHost,
  pub(crate) uri_scheme_protocols: Arc<HashMap<String, Arc<Box<UriSchemeProtocolHandler>>>>,
  pub(crate) devtools_protocol_handlers: Arc<Mutex<Vec<Arc<DevToolsProtocolHandler>>>>,
  /// Keeps the DevTools message observer registered. Dropping this unregisters the observer.
  pub(crate) devtools_observer_registration: Arc<Mutex<Option<cef::Registration>>>,
  pub(crate) listeners: WebviewEventListeners,
  pub(crate) bounds_rate: Option<BoundsRate>,
}

impl AppWebview {
  pub(crate) fn set_bounds(&mut self, parent_size: PhysicalSize<u32>, scale: f64, bounds: Rect) {
    let position = bounds.position.to_physical::<i32>(scale);
    let size = bounds.size.to_physical::<u32>(scale);

    let x = position.x;
    let y = position.y;
    let w = size.width as i32;
    let h = size.height as i32;

    if self.bounds_rate.is_some() {
      let win_w = parent_size.width.max(1) as f32;
      let win_h = parent_size.height.max(1) as f32;
      self.bounds_rate = Some(BoundsRate {
        x: x as f32 / win_w,
        y: y as f32 / win_h,
        width: w as f32 / win_w,
        height: h as f32 / win_h,
      });
    }

    self.host.notify_move_or_resize_started();
    self.apply_physical_bounds(scale, x, y, w, h);
    self.host.was_resized();
  }

  pub(crate) fn set_visible(&self, visible: bool) {
    self.host.was_hidden(if visible { 0 } else { 1 });
    self.apply_visible(visible);
  }

  pub fn url(&self) -> Option<String> {
    self
      .browser
      .main_frame()
      .map(|frame| cef::CefString::from(&frame.url()).to_string())
  }
}

impl<T: UserEvent> WinitCefApp<T> {
  pub(crate) fn create_webview(
    &mut self,
    window_id: WindowId,
    webview_id: u32,
    pending: PendingWebview<T, CefRuntime<T>>,
  ) -> Result<()> {
    let Self {
      context,
      scheme_registry,
      state,
      ..
    } = self;
    let Some(appwindow) = state.windows.get_mut(&window_id) else {
      return Err(Error::CreateWebview(
        format!("window {window_id:?} does not exist").into(),
      ));
    };
    Self::build_and_attach_webview(
      context,
      scheme_registry,
      &mut state.live_browsers,
      appwindow,
      webview_id,
      browser_client::DragDropEventTarget::Webview,
      pending,
    )
  }

  /// Builds a webview and attaches it to `appwindow`, bumping `live_browsers`
  /// and relaying it out. Works whether `appwindow` already lives in `state` or
  /// is still being assembled, so window and child creation share one path.
  pub(crate) fn build_and_attach_webview(
    context: &RuntimeContext<T>,
    scheme_registry: &request_handler::SchemeRegistry,
    live_browsers: &mut usize,
    appwindow: &mut AppWindow,
    webview_id: u32,
    drag_drop_event_target: browser_client::DragDropEventTarget,
    pending: PendingWebview<T, CefRuntime<T>>,
  ) -> Result<()> {
    let parent = appwindow.raw_cef_handle();
    let parent_size = appwindow.window.surface_size();
    let scale = appwindow.window.scale_factor();
    let app_wide_theme = *context.app_wide_theme.lock().unwrap();
    let theme = appwindow.resolved_theme(app_wide_theme);
    let Some(child) = Self::build_browser_child(
      context,
      scheme_registry,
      appwindow.id,
      webview_id,
      parent,
      parent_size,
      scale,
      theme,
      drag_drop_event_target,
      pending,
    ) else {
      return Err(Error::CreateWebview(
        "failed to create CEF browser".to_string().into(),
      ));
    };

    // On Windows a window's webviews are sibling child HWNDs. Put each new one
    // on top of the ones already there — the order they were created in — and
    // pin it, so Chromium's focus raise cannot reshuffle them behind our back
    // and bury an overlay webview under the one that fills the window.
    #[cfg(windows)]
    child.raise_to_top();

    *live_browsers += 1;
    appwindow.children.push(child);
    layout_app_window(appwindow);
    Ok(())
  }

  pub(crate) fn build_browser_child(
    context: &RuntimeContext<T>,
    scheme_registry: &request_handler::SchemeRegistry,
    window_id: WindowId,
    webview_id: u32,
    parent: cef::sys::cef_window_handle_t,
    parent_size: PhysicalSize<u32>,
    scale: f64,
    theme: Option<Theme>,
    drag_drop_event_target: browser_client::DragDropEventTarget,
    mut pending: PendingWebview<T, CefRuntime<T>>,
  ) -> Option<AppWebview> {
    let bounds_rate = compute_child_bounds_rate(
      pending.webview_attributes.bounds.as_ref(),
      pending.webview_attributes.auto_resize,
      parent_size,
      scale,
    );
    let initialization_scripts = initialization_scripts(&mut pending.webview_attributes);
    let uri_scheme_protocols: Arc<HashMap<_, _>> = Arc::new(
      pending
        .uri_scheme_protocols
        .into_iter()
        .map(|(scheme, handler)| (scheme, Arc::new(handler)))
        .collect(),
    );
    let on_page_load_handler = pending.on_page_load_handler.take().map(Arc::from);
    let document_title_changed_handler =
      pending.document_title_changed_handler.take().map(Arc::from);
    let devtools_enabled = (cfg!(debug_assertions) || cfg!(feature = "devtools"))
      && pending.webview_attributes.devtools.unwrap_or(true);
    let zoom_hotkeys_enabled = pending.webview_attributes.zoom_hotkeys_enabled;
    let drag_drop_handler_enabled = pending.webview_attributes.drag_drop_handler_enabled;
    let drag_drop_state = Arc::new(Mutex::new(browser_client::DragDropState::default()));
    let web_content_process_terminate_handler = pending
      .on_web_content_process_terminate_handler
      .take()
      .map(Arc::from);
    let frame_navigation_state = crate::FrameNavigationState::new();
    let popup_family = Arc::new(crate::popup::PopupFamily::new(
      frame_navigation_state.clone(),
    ));
    let dialogs = crate::dialog::DialogState::new(frame_navigation_state.clone());
    let frame_state_for_events = frame_navigation_state.clone();
    let frame_event_handler = pending
      .runtime_specific_attributes
      .frame_event_handler
      .clone();
    let handlers = browser_client::TauriCefBrowserClientHandlers {
      // The internal navigation observer must see every notification this client
      // receives; the app observer is bound to this exact native browser. CEF can
      // route a browser this webview does not own through the same client — a
      // DevTools window is the standing case — and a `FrameEvent` carries the full
      // URL, so those must never reach an observer registered for this webview.
      frame_event_handler: Some(Arc::new(move |event| {
        frame_state_for_events.on_frame_event(&event);
        if frame_state_for_events.has_browser_id(event.browser_id)
          && let Some(handler) = &frame_event_handler
        {
          handler(event);
        }
      })),
      ipc_handler: pending.ipc_handler.map(Arc::from),
      on_page_load_handler,
      document_title_changed_handler,
      navigation_handler: pending.navigation_handler.map(Arc::from),
      new_window_handler: pending.new_window_handler.map(Arc::from),
      download_handler: pending.download_handler.take(),
      console_message_handler: pending
        .runtime_specific_attributes
        .console_message_handler
        .clone(),
      permission_request_handler: pending.permission_request_handler.take().map(Arc::from),
      web_content_process_terminate_handler,
    };

    let mut client =
      browser_client::TauriCefBrowserClient::build(browser_client::TauriCefBrowserClientArgs {
        context: context.clone(),
        window_id,
        webview_id,
        label: pending.label.clone(),
        initial_url: Some(pending.url.as_str().to_string()),
        devtools_enabled,
        zoom_hotkeys_enabled,
        drag_drop_event_target,
        drag_drop_handler_enabled,
        drag_drop_state,
        frame_navigation_state: frame_navigation_state.clone(),
        popup_family: Arc::downgrade(&popup_family),
        opener: None,
        handlers,
        proxy: context.proxy.clone(),
        sender: context.sender.clone(),
      });

    // If the bounds are not specified, default to the parent window's size and position.
    // aka full-window webview.
    let bounds = pending.webview_attributes.bounds.unwrap_or_else(|| Rect {
      position: PhysicalPosition::new(0, 0).into(),
      size: parent_size.into(),
    });
    #[cfg(not(target_os = "macos"))]
    let bounds = bounds.to_physical::<i32, i32>(scale);
    #[cfg(target_os = "macos")]
    let bounds = bounds.to_logical::<i32, i32>(scale);
    let bounds = cef::Rect {
      x: bounds.position.x,
      y: bounds.position.y,
      width: bounds.size.width,
      height: bounds.size.height,
    };

    // Let CEF pick the runtime style unless overridden per-webview.
    let cef_runtime_style = match pending.runtime_specific_attributes.runtime_style {
      Some(RuntimeStyle::Alloy) => cef::RuntimeStyle::ALLOY,
      Some(RuntimeStyle::Chrome) => cef::RuntimeStyle::CHROME,
      None => cef::RuntimeStyle::DEFAULT,
    };

    let mut window_info = cef::WindowInfo::default().set_as_child(parent, &bounds);
    window_info.runtime_style = cef_runtime_style;
    let settings = browser_settings_from_webview_attributes(&pending.webview_attributes);

    let custom_protocol_scheme = if pending.webview_attributes.use_https_scheme {
      "https"
    } else {
      "http"
    }
    .to_string();
    let custom_scheme_domain_names: Vec<String> = uri_scheme_protocols
      .keys()
      .map(|scheme| format!("{scheme}.localhost"))
      .collect();
    let real_initial_url = pending.url.as_str().to_string();
    let (browser_tx, browser_rx) = mpsc::channel();
    let (init_done, on_initialized) = request_context::deferred_init_continuation({
      let scheme_registry = scheme_registry.clone();
      let uri_scheme_protocols = uri_scheme_protocols.clone();
      let initialization_scripts = initialization_scripts.clone();
      let custom_protocol_scheme = custom_protocol_scheme.clone();
      let custom_scheme_domain_names = custom_scheme_domain_names.clone();
      let label = pending.label.clone();
      move |mut request_context| {
        request_context::apply_theme_scheme(request_context.as_ref(), theme);

        // Create with an inert document so the BrowserHost exists before the real
        // navigation; the real URL is loaded once the document-start script is set.
        let initial_url = CefString::from(INITIAL_LOAD_URL);
        let Some(browser) = cef::browser_host_create_browser_sync(
          Some(&window_info),
          Some(&mut client),
          Some(&initial_url),
          Some(&settings),
          None,
          request_context.as_mut(),
        ) else {
          log::error!("failed to create CEF browser for webview {label:?}");
          return;
        };
        let Some(host) = browser.host() else {
          log::error!("CEF browser for webview {label:?} has no host");
          return;
        };
        let browser_id = browser.identifier();

        {
          let mut registry = scheme_registry.lock().unwrap();
          for (scheme, handler) in uri_scheme_protocols.iter() {
            registry.insert(
              (browser_id, scheme.clone()),
              (
                label.clone(),
                handler.clone(),
                initialization_scripts.clone(),
              ),
            );
          }
        }

        // The app observers registered through `on_dev_tools_protocol` live in
        // this list, and it belongs to this one native browser. Every CEF-owned
        // popup registers its own protocol observer against a list of its own,
        // so nothing registered here ever observes a popup: a
        // `DevToolsProtocol` notification carries the page's content, its
        // network activity and its dialog messages with no browser identity to
        // separate them, and a popup navigates wherever its own content goes —
        // an SSO or OAuth window is the standing case.
        let devtools_protocol_handlers: Arc<Mutex<Vec<Arc<DevToolsProtocolHandler>>>> =
          Arc::default();
        let pending_initial_loads: PendingInitialLoads = Arc::new(Mutex::new(HashMap::new()));
        let devtools_observer_registration = Arc::new(Mutex::new(add_dev_tools_observer(
          &browser,
          devtools_protocol_handlers.clone(),
          pending_initial_loads.clone(),
          dialogs.clone(),
        )));
        load_initial_url_after_registering_initialization_scripts(
          &browser,
          &initialization_scripts,
          &custom_protocol_scheme,
          &custom_scheme_domain_names,
          &real_initial_url,
          &pending_initial_loads,
        );

        browser_tx
          .send(AppWebview {
            webview_id,
            label,
            browser,
            browser_id,
            frame_navigation_state,
            popup_family,
            dialogs,
            host,
            uri_scheme_protocols,
            devtools_protocol_handlers,
            devtools_observer_registration,
            listeners: Default::default(),
            bounds_rate,
          })
          .expect("failed to send initialized CEF browser");
      }
    });
    let request_context = request_context::request_context_from_webview_attributes(
      &context.cache_path,
      &pending.webview_attributes,
      uri_scheme_protocols.keys(),
      &custom_protocol_scheme,
      scheme_registry.clone(),
      on_initialized,
    );
    if request_context.is_none() {
      init_done.store(true, Ordering::SeqCst);
    }
    request_context::wait_for_deferred_init(&init_done);

    // `None` here means browser creation failed (or the request context never
    // initialized); the continuation logs the reason. Soft-fail instead of
    // taking down the whole process.
    browser_rx.recv().ok()
  }

  pub(crate) fn handle_webview_message(
    &mut self,
    window_id: WindowId,
    webview_id: u32,
    message: WebviewMessage,
  ) {
    // If the runtime is exiting, don't process any more messages to avoid macOS crash on exit.
    if self.state.exiting {
      return;
    }

    let Some(appwindow) = self.state.windows.get_mut(&window_id) else {
      return;
    };
    let message = match message {
      WebviewMessage::WithWebview(callback) => {
        let Some(child) = appwindow
          .children
          .iter()
          .find(|child| child.webview_id == webview_id)
        else {
          return;
        };
        let document = child
          .frame_navigation_state
          .observe_document(&child.browser);
        let dialogs = child.dialogs.snapshot(document.as_ref());
        let snapshot = WebviewSnapshot {
          browser_id: child.browser_id,
          dialogs,
          document,
          window_label: Some(appwindow.label.clone()),
          window: Some(appwindow.lifetime.clone()),
          parent_matches: child.native_parent_matches(appwindow),
          bounds: child.bounds(),
          visible: child.native_visible(),
        };
        let mut native = Webview::new(
          child.browser.clone(),
          snapshot,
          child.frame_navigation_state.clone(),
        );
        native.popups = child.popup_family.observe();
        callback(native);
        return;
      }
      message => message,
    };
    let Some(child) = appwindow
      .children
      .iter_mut()
      .find(|child| child.webview_id == webview_id)
    else {
      return;
    };

    match message {
      WebviewMessage::EvaluateScript(script) => {
        if let Some(frame) = child.browser.main_frame() {
          let script = cef::CefString::from(script.as_str());
          let url = cef::CefString::from("");
          frame.execute_java_script(Some(&script), Some(&url), 0);
        }
      }
      WebviewMessage::EvaluateScriptWithCallback(script, callback) => {
        let host = &child.host;
        let Ok(message_id) = crate::devtools::allocate_runtime_devtools_message_id() else {
          callback(String::new());
          return;
        };
        let callback = Arc::new(Mutex::new(Some(callback)));
        let registration = Arc::new(Mutex::new(None));
        let mut observer = EvalScriptWithCallbackDevToolsObserver::new(
          message_id,
          callback.clone(),
          registration.clone(),
        );

        if let Some(observer_registration) =
          host.add_dev_tools_message_observer(Some(&mut observer))
        {
          *registration.lock().unwrap() = Some(observer_registration);

          let message = serde_json::json!({
            "id": message_id,
            "method": "Runtime.evaluate",
            "params": {
              "expression": script,
              "returnByValue": true,
            }
          })
          .to_string();

          if host.send_dev_tools_message(Some(message.as_bytes())) != 1 {
            let _ = registration.lock().unwrap().take();
            if let Some(callback) = callback.lock().unwrap().take() {
              callback(String::new());
            }
          }
        } else if let Some(callback) = callback.lock().unwrap().take() {
          callback(String::new());
        }
      }
      WebviewMessage::Navigate(url) => {
        if let Some(frame) = child.browser.main_frame() {
          frame.load_url(Some(&cef::CefString::from(url.as_str())));
        }
      }
      WebviewMessage::Reload => child.browser.reload(),
      WebviewMessage::GoBack => child.browser.go_back(),
      WebviewMessage::CanGoBack(tx) => _ = tx.send(Ok(child.browser.can_go_back() == 1)),
      WebviewMessage::GoForward => child.browser.go_forward(),
      WebviewMessage::CanGoForward(tx) => _ = tx.send(Ok(child.browser.can_go_forward() == 1)),
      WebviewMessage::Close => child.host.close_browser(0),
      WebviewMessage::SetBounds(bounds) => {
        let parent_size = appwindow.window.surface_size();
        let scale = appwindow.window.scale_factor();
        child.set_bounds(parent_size, scale, bounds);
      }
      WebviewMessage::SetSize(size) => {
        let parent_size = appwindow.window.surface_size();
        let scale = appwindow.window.scale_factor();
        let bounds = child.bounds().unwrap_or_default();
        let new_bounds = Rect {
          position: bounds.position,
          size,
        };
        child.set_bounds(parent_size, scale, new_bounds);
      }
      WebviewMessage::SetPosition(position) => {
        let parent_size = appwindow.window.surface_size();
        let scale = appwindow.window.scale_factor();
        let bounds = child.bounds().unwrap_or_default();
        let new_bounds = Rect {
          position,
          size: bounds.size,
        };
        child.set_bounds(parent_size, scale, new_bounds);
      }
      WebviewMessage::SetFocus => child.host.set_focus(1),
      WebviewMessage::Url(tx) => {
        let url = child.url().unwrap_or_default();
        let _ = tx.send(Ok(url));
      }
      WebviewMessage::Bounds(tx) => {
        let bounds = child.bounds().ok_or(Error::FailedToSendMessage);
        let _ = tx.send(bounds);
      }
      WebviewMessage::Position(tx) => {
        let bounds = child.bounds().ok_or(Error::FailedToSendMessage);
        let position = bounds.map(|b| b.position);
        let position = position.map(|p| p.to_physical::<i32>(appwindow.window.scale_factor()));
        let _ = tx.send(position);
      }
      WebviewMessage::Size(tx) => {
        let bounds = child.bounds().ok_or(Error::FailedToSendMessage);
        let size = bounds.map(|b| b.size.to_physical::<u32>(appwindow.window.scale_factor()));
        let _ = tx.send(size);
      }
      WebviewMessage::WithWebview(_) => unreachable!("native observation dispatched above"),
      WebviewMessage::Print => child.host.print(),
      WebviewMessage::AddEventListener(event_id, handler) => {
        child.listeners.lock().unwrap().insert(event_id, handler);
      }
      WebviewMessage::Show => child.set_visible(true),
      WebviewMessage::Hide => child.set_visible(false),
      WebviewMessage::SetZoom(scale_factor) => {
        // CEF uses a logarithmic zoom level where percentage = 1.2^level
        // (Chromium's kTextSizeMultiplierRatio). Convert from Tauri linear
        // scale factor (1.0 = 100%) to CEF's level (0.0 = 100%)
        const CEF_ZOOM_BASE: f64 = 1.2;
        let zoom_level = if scale_factor > 0.0 {
          scale_factor.ln() / CEF_ZOOM_BASE.ln()
        } else {
          0.0
        };
        child.host.set_zoom_level(zoom_level);
      }
      WebviewMessage::SetAutoResize(auto_resize) => {
        if auto_resize {
          let bounds = child.bounds();
          let parent_size = appwindow.window.surface_size();
          let scale = appwindow.window.scale_factor();
          child.bounds_rate = compute_child_bounds_rate(bounds.as_ref(), true, parent_size, scale);
        } else {
          child.bounds_rate = None;
        }
      }
      WebviewMessage::SetBackgroundColor(color) => child.set_background_color(color),
      WebviewMessage::ClearAllBrowsingData => {
        if let Some(manager) = child.cookie_manager() {
          manager.delete_cookies(None, None, None);
          manager.flush_store(None);
        }
        if let Some(request_context) = child.host.request_context() {
          request_context.clear_http_cache(None);
        }
      }
      WebviewMessage::CookiesForUrl(url, tx) => {
        if let Some(manager) = child.cookie_manager() {
          cookie::visit_url_cookies(manager, url, tx);
        } else {
          let _ = tx.send(Ok(Vec::new()));
        }
      }
      WebviewMessage::Cookies(tx) => {
        if let Some(manager) = child.cookie_manager() {
          cookie::visit_all_cookies(manager, tx);
        } else {
          let _ = tx.send(Ok(Vec::new()));
        }
      }
      WebviewMessage::SetCookie(cookie) => {
        if let Some(manager) = child.cookie_manager() {
          let url = child.url();
          cookie::set_cookie(manager, url, cookie);
        }
      }
      WebviewMessage::DeleteCookie(cookie) => {
        if let Some(manager) = child.cookie_manager() {
          let url = child.url();
          cookie::delete_cookie(manager, url, cookie);
        }
      }
      WebviewMessage::Reparent(target_window_id, tx) => {
        if window_id == target_window_id {
          let _ = tx.send(Ok(()));
          return;
        }

        if !self.state.windows.contains_key(&target_window_id) {
          let _ = tx.send(Err(Error::WindowNotFound));
          return;
        }

        let Some(mut child) = self
          .state
          .windows
          .get_mut(&window_id)
          .and_then(|appwindow| {
            appwindow
              .children
              .iter()
              .position(|child| child.webview_id == webview_id)
              .map(|index| appwindow.children.remove(index))
          })
        else {
          let _ = tx.send(Err(Error::WindowNotFound));
          return;
        };

        let Some(target_appwindow) = self.state.windows.get_mut(&target_window_id) else {
          let _ = tx.send(Err(Error::WindowNotFound));
          return;
        };

        let bounds = child.bounds().unwrap_or_else(|| Rect {
          position: PhysicalPosition::new(0, 0).into(),
          size: target_appwindow.window.surface_size().into(),
        });
        child.reparent(target_appwindow);
        child.set_bounds(
          target_appwindow.window.surface_size(),
          target_appwindow.window.scale_factor(),
          bounds,
        );
        // Re-parenting does not preserve z-order: a view docked back into a
        // window that already owns a full-window main webview must be put back
        // on top, or it lands behind it and renders nothing.
        #[cfg(windows)]
        child.raise_to_top();

        target_appwindow.children.push(child);
        let _ = tx.send(Ok(()));
      }
      #[cfg(any(debug_assertions, feature = "devtools"))]
      WebviewMessage::OpenDevTools => child.host.show_dev_tools(None, None, None, None),
      #[cfg(any(debug_assertions, feature = "devtools"))]
      WebviewMessage::CloseDevTools => child.host.close_dev_tools(),
      #[cfg(any(debug_assertions, feature = "devtools"))]
      WebviewMessage::IsDevToolsOpen(tx) => _ = tx.send(child.host.has_dev_tools() == 1),
      WebviewMessage::SendDevToolsMessage(message, tx) => {
        let result = child.host.send_dev_tools_message(Some(&message));
        let _ = tx.send(if result == 1 {
          Ok(())
        } else {
          Err(Error::FailedToSendMessage)
        });
      }
      WebviewMessage::OnDevToolsProtocol(handler, tx) => {
        child
          .devtools_protocol_handlers
          .lock()
          .unwrap()
          .push(handler);

        let needs_devtools_observer = child
          .devtools_observer_registration
          .lock()
          .unwrap()
          .is_none();
        if needs_devtools_observer {
          if let Some(registration) = add_dev_tools_observer(
            &child.browser,
            child.devtools_protocol_handlers.clone(),
            Arc::new(Mutex::new(HashMap::new())),
            child.dialogs.clone(),
          ) {
            *child.devtools_observer_registration.lock().unwrap() = Some(registration);
            let _ = tx.send(Ok(()));
          } else {
            let _ = tx.send(Err(Error::FailedToSendMessage));
          }
        } else {
          let _ = tx.send(Ok(()));
        }
      }
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub enum RuntimeStyle {
  Alloy,
  Chrome,
}

/// The CEF-specific webview attributes, set through
/// [`WebviewWindowBuilderCefExt`](crate::WebviewWindowBuilderCefExt).
///
/// # Permission requests on CEF
///
/// The runtime honors `WebviewAttributes::on_permission_request` — an `Allow`
/// grants without showing Chrome's prompt, a `Deny` refuses without one — with
/// three things worth knowing before relying on it.
///
/// ## Most decisions are made once per origin and then persist
///
/// Chromium consults a permission prompt only while the stored content setting
/// for that (origin, permission) still says "ask", and answering the prompt
/// persists the decision to the on-disk profile. Everything routed through the
/// prompt therefore reaches the handler **once per origin and permission, ever** —
/// including across restarts of the app.
///
/// A handler whose answer depends on application state is not re-consulted when
/// that state changes; its first answer stands. Nothing calls back to say the app
/// changed its mind, so an app that has to revoke a grant must rewrite the content
/// setting through the request context itself.
///
/// Camera and microphone are the exception. Chromium routes *every*
/// `getUserMedia()` call through the media path, so those two do reach the handler
/// on each call and a changing answer is honored.
///
/// ## Permissions Tauri has no kind for arrive as `PermissionKind::Other`
///
/// Chromium has more request types than Tauri has kinds. Storage Access and Top
/// Level Storage Access, FedCM, protocol handler registration, idle detection,
/// local and loopback network access, web app installation, the AR and VR sessions
/// behind WebXR, hand tracking, keyboard lock and disk quota all arrive as
/// `PermissionKind::Other`, as does any request type a future CEF build adds.
///
/// Failing closed is deliberate — a request type this runtime does not recognise
/// must never be granted behind the app's back — but it means a handler written
/// elsewhere as `match kind { Camera => Allow, _ => Deny }` hard-denies all of
/// them here, and denying Storage Access or FedCM breaks third-party SSO flows
/// outright. Return `PermissionResponse::Default` for the kinds you did not mean
/// to answer about, and CEF's own handling runs for them unchanged.
///
/// ## `PermissionKind::DisplayCapture` is never granted by an `Allow`
///
/// A `getDisplayMedia()` request that the handler answers `Allow` is *not*
/// granted here; it is handed back to CEF, which shows Chromium's desktop media
/// picker under Chrome style and refuses under Alloy style. A `Deny` still
/// refuses it outright.
///
/// The reason is that granting it from the handler would grant *everything*: CEF
/// builds the stream from the permission mask, and a desktop video bit with no
/// requested source synthesises the full desktop and returns it with no picker at
/// all. `PermissionKind::DisplayCapture` names no screen, window or tab, so a
/// blanket rule such as `.on_permission_request(|_| PermissionResponse::Allow)`
/// would silently give any page in the webview — including remote content reached
/// through a redirect — a full-desktop stream. The picker is the only thing that
/// can say what is actually shared, so it stays.
///
/// # Chrome accelerators an app window does not get
///
/// A Chrome style browser keeps its whole accelerator table live even hosted as a
/// child view with no browser UI, so this runtime swallows the commands that have
/// no meaning in an app window (new window and tab, the tab strip, history and
/// downloads and settings, print, save page, view source, the omnibox focus
/// commands). Two of those exclusions are worth calling out, because they take
/// away keystrokes users expect:
///
/// - **Zoom.** `WebviewAttributes::zoom_hotkeys_enabled` is honored, and it
///   **defaults to `false`**, so Ctrl+Plus, Ctrl+Minus and Ctrl+0 do not zoom
///   unless the webview opted in with `.zoom_hotkeys_enabled(true)`. Ctrl+mouse
///   wheel zoom is unaffected either way: Chromium applies it in the render
///   widget rather than through the command controller. Note that on Linux and
///   macOS Tauri injects a JavaScript zoom polyfill when the flag is true, which
///   coexists with Chrome's own accelerator, so a keyboard zoom steps twice
///   there. `WebviewDispatch::set_zoom` is untouched.
///
/// - **History.** Alt+Left and Alt+Right do not navigate the session history.
///   The browser is created at an internal placeholder URL and then navigated to
///   the app's own, so the app's first screen already sits on a second history
///   entry and going back from it lands on a blank page with no way forward. The
///   page context menu drops Back and Forward for the same reason.
///   `WebviewDispatch::go_back` and `go_forward` are untouched, and an app's own
///   routing is what an app's "back" should mean anyway.
#[derive(Default, Clone)]
pub struct CefWebviewAttributes {
  /// The browser runtime style, see [`RuntimeStyle`]. CEF picks one when not set.
  pub runtime_style: Option<RuntimeStyle>,
  /// Observer of the native lifecycle events of every frame of the webview.
  ///
  /// Scoped to this webview's own native browser — its main frame and its child
  /// frames. Every notification carries that one
  /// [`browser_id`](crate::FrameEvent::browser_id). A CEF-owned popup is a
  /// separate browser that navigates wherever its own content goes, and a
  /// [`FrameEvent`](crate::FrameEvent) carries the full URL, so popups are never
  /// reported here. Observe them through [`Webview::popups`], whose
  /// [`FrameNavigationState`](crate::FrameNavigationState) follows a popup's
  /// native lifecycle without exposing its URLs.
  pub frame_event_handler: Option<Arc<crate::FrameEventHandler>>,
  /// Observer of the messages the renderer writes to the JavaScript console.
  ///
  /// Scoped to this webview's own native browser. A CEF-owned popup is a separate
  /// browser running its own scripts, and so is a DevTools window opened on this
  /// webview, so neither one's console output is reported here.
  pub console_message_handler: Option<Arc<ConsoleMessageHandler>>,
}

impl std::fmt::Debug for CefWebviewAttributes {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter
      .debug_struct("CefWebviewAttributes")
      .field("runtime_style", &self.runtime_style)
      .field("frame_event_handler", &self.frame_event_handler.is_some())
      .field(
        "console_message_handler",
        &self.console_message_handler.is_some(),
      )
      .finish()
  }
}

#[derive(Debug, Clone)]
pub struct CefInitScript {
  pub(crate) script: String,
  pub(crate) hash: String,
  for_main_frame_only: bool,
}

impl CefInitScript {
  /// Whether this script must run in a document loaded in the given frame.
  pub(crate) fn runs_in_frame(&self, is_main_frame: bool) -> bool {
    is_main_frame || !self.for_main_frame_only
  }

  fn new(script: InitializationScript) -> Self {
    let mut hasher = Sha256::new();
    hasher.update(normalize_script_for_csp(script.script.as_bytes()));
    let hash = format!(
      "'sha256-{}'",
      base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        hasher.finalize()
      )
    );
    Self {
      script: script.script,
      hash,
      for_main_frame_only: script.for_main_frame_only,
    }
  }
}

pub(crate) fn initialization_scripts(attrs: &mut WebviewAttributes) -> Arc<Vec<CefInitScript>> {
  let mut initialization_scripts = Vec::new();

  if attrs.drag_drop_handler_enabled {
    let drag_script = browser_client::drag_drop_initialization_script();
    initialization_scripts.push(CefInitScript::new(drag_script));
  }

  initialization_scripts.extend(
    std::mem::take(&mut attrs.initialization_scripts)
      .into_iter()
      .map(CefInitScript::new),
  );

  Arc::new(initialization_scripts)
}

#[derive(Debug, Clone)]
pub struct CefWebviewDispatcher<T: UserEvent> {
  pub(crate) window_id: Arc<Mutex<WindowId>>,
  pub(crate) webview_id: u32,
  pub(crate) context: RuntimeContext<T>,
}

impl<T: UserEvent> CefWebviewDispatcher<T> {
  /// Sends a UTF-8 encoded Chrome DevTools Protocol message to the DevTools agent.
  ///
  /// The message's `id` must come from
  /// [`allocate_devtools_message_id`](crate::allocate_devtools_message_id), the
  /// allocator every caller on this browser shares. A hardcoded or
  /// self-incremented `id` can consume another caller's
  /// [`DevToolsProtocol::MethodResult`]. The runtime's own requests use
  /// identifiers reserved above that allocator's range, so they cannot be
  /// answered by a caller's message.
  pub fn send_dev_tools_message(&self, message: &[u8]) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SendDevToolsMessage(message.to_vec(), tx),
    })?;
    rx.recv().map_err(|_| Error::FailedToReceiveMessage)?
  }

  /// Observes the [`DevToolsProtocol`] traffic of this browser.
  ///
  /// The observer receives every message on the browser, including the runtime's
  /// own requests, so results must be matched against an identifier obtained from
  /// [`allocate_devtools_message_id`](crate::allocate_devtools_message_id).
  ///
  /// Scoped to this webview's own native browser. A CEF-owned popup is a
  /// separate browser whose protocol traffic — its page content, its network
  /// activity and its dialog messages — is never reported here, the way a
  /// [`FrameEvent`](crate::FrameEvent) of a popup is not. Observe popups
  /// through [`Webview::popups`], whose
  /// [`FrameNavigationState`](crate::FrameNavigationState) follows a popup's
  /// native lifecycle without exposing what it loaded.
  pub fn on_dev_tools_protocol<F: Fn(DevToolsProtocol) + Send + Sync + 'static>(
    &self,
    f: F,
  ) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    let handler =
      Arc::new(move |protocol: DevToolsProtocol| f(protocol)) as Arc<DevToolsProtocolHandler>;
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::OnDevToolsProtocol(handler, tx),
    })?;
    rx.recv().map_err(|_| Error::FailedToReceiveMessage)?
  }
}

pub(crate) fn create_webview_detached<T: UserEvent>(
  context: &RuntimeContext<T>,
  window_id: WindowId,
  pending: PendingWebview<T, CefRuntime<T>>,
) -> Result<DetachedWebview<T, CefRuntime<T>>> {
  let label = pending.label.clone();
  let webview_id = context.next_webview_id();
  let (result_tx, result_rx) = mpsc::channel();
  context.send_message(Message::CreateWebview {
    window_id,
    webview_id,
    pending: Box::new(pending),
    result_tx,
  })?;
  // Block until the event loop has created the browser so a creation failure
  // is surfaced to the caller instead of leaving a detached, dead webview.
  result_rx
    .recv()
    .map_err(|_| Error::FailedToReceiveMessage)??;
  Ok(DetachedWebview {
    label,
    dispatcher: CefWebviewDispatcher {
      window_id: Arc::new(Mutex::new(window_id)),
      webview_id,
      context: context.clone(),
    },
  })
}

fn getter<T: UserEvent, R>(
  context: &RuntimeContext<T>,
  message: Message<T>,
  receiver: Receiver<Result<R>>,
) -> Result<R> {
  context.send_message(message)?;
  receiver.recv().map_err(|_| Error::FailedToReceiveMessage)?
}

macro_rules! webview_getter {
  ($self:ident, $variant:ident) => {{
    let (tx, rx) = mpsc::channel();
    // Drop the guard before waiting: CEF page-load callbacks on the UI thread
    // may need this same lock to dispatch work before servicing the getter.
    let window_id = *$self.window_id.lock().unwrap();
    getter(
      &$self.context,
      Message::Webview {
        window_id,
        webview_id: $self.webview_id,
        message: WebviewMessage::$variant(tx),
      },
      rx,
    )
  }};
}

#[cfg(test)]
mod getter_tests {
  use super::{Message, WebviewMessage};
  use std::sync::{Arc, Mutex, mpsc};
  use tauri_runtime::{Result, window::WindowId};

  // Exercise the production macro with a bounded UI-reply probe. A real CEF
  // event loop cannot run in a unit-test worker; the probe checks the lock
  // before replying so the regression fails instead of hanging the suite.
  struct Dispatcher {
    context: Arc<Mutex<WindowId>>,
    window_id: Arc<Mutex<WindowId>>,
    webview_id: u32,
  }

  fn getter(
    window_id: &Arc<Mutex<WindowId>>,
    message: Message<()>,
    receiver: mpsc::Receiver<Result<String>>,
  ) -> Result<String> {
    let Message::Webview {
      window_id: requested_window,
      message: WebviewMessage::Url(reply),
      ..
    } = message
    else {
      panic!("expected a URL request");
    };
    let callback_window = window_id
      .try_lock()
      .expect("the UI callback must acquire the window lock before replying");
    assert_eq!(*callback_window, requested_window);
    reply.send(Ok("https://example.test/".into())).unwrap();
    receiver.recv().unwrap()
  }

  #[test]
  fn url_getter_does_not_hold_the_window_lock_while_waiting_for_ui() {
    let window_id = Arc::new(Mutex::new(WindowId::from(1)));
    let dispatcher = Dispatcher {
      context: Arc::clone(&window_id),
      window_id,
      webview_id: 1,
    };
    assert_eq!(
      webview_getter!(dispatcher, Url).unwrap(),
      "https://example.test/"
    );
  }
}

impl<T: UserEvent> WebviewDispatch<T> for CefWebviewDispatcher<T> {
  type Runtime = CefRuntime<T>;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.context.run_on_main_thread(f)
  }

  fn on_webview_event<F: Fn(&WebviewEvent) + Send + 'static>(&self, f: F) -> WebviewEventId {
    let id = self.context.next_webview_event_id();
    let _ = self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::AddEventListener(id, Box::new(f)),
    });
    id
  }

  fn with_webview<F: FnOnce(<Self::Runtime as Runtime<T>>::Webview) + Send + 'static>(
    &self,
    f: F,
  ) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::WithWebview(Box::new(f)),
    })
  }

  fn open_devtools(&self) {
    #[cfg(any(debug_assertions, feature = "devtools"))]
    {
      let _ = self.context.send_message(Message::Webview {
        window_id: *self.window_id.lock().unwrap(),
        webview_id: self.webview_id,
        message: WebviewMessage::OpenDevTools,
      });
    }
    #[cfg(not(any(debug_assertions, feature = "devtools")))]
    log::warn!("devtools are not available: enable the `devtools` feature of `tauri-runtime-cef`");
  }

  fn close_devtools(&self) {
    #[cfg(any(debug_assertions, feature = "devtools"))]
    {
      let _ = self.context.send_message(Message::Webview {
        window_id: *self.window_id.lock().unwrap(),
        webview_id: self.webview_id,
        message: WebviewMessage::CloseDevTools,
      });
    }
  }

  fn is_devtools_open(&self) -> Result<bool> {
    #[cfg(any(debug_assertions, feature = "devtools"))]
    {
      let (tx, rx) = mpsc::channel();
      self.context.send_message(Message::Webview {
        window_id: *self.window_id.lock().unwrap(),
        webview_id: self.webview_id,
        message: WebviewMessage::IsDevToolsOpen(tx),
      })?;
      rx.recv().map_err(|_| Error::FailedToReceiveMessage)
    }
    #[cfg(not(any(debug_assertions, feature = "devtools")))]
    {
      Ok(false)
    }
  }

  fn url(&self) -> Result<String> {
    webview_getter!(self, Url)
  }

  fn bounds(&self) -> Result<Rect> {
    webview_getter!(self, Bounds)
  }

  fn position(&self) -> Result<PhysicalPosition<i32>> {
    webview_getter!(self, Position)
  }

  fn size(&self) -> Result<PhysicalSize<u32>> {
    webview_getter!(self, Size)
  }

  fn navigate(&self, url: Url) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::Navigate(url),
    })
  }

  fn reload(&self) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::Reload,
    })
  }

  fn go_back(&self) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::GoBack,
    })
  }

  fn can_go_back(&self) -> Result<bool> {
    webview_getter!(self, CanGoBack)
  }

  fn go_forward(&self) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::GoForward,
    })
  }

  fn can_go_forward(&self) -> Result<bool> {
    webview_getter!(self, CanGoForward)
  }

  fn print(&self) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::Print,
    })
  }

  fn close(&self) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::Close,
    })
  }

  fn set_bounds(&self, bounds: Rect) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetBounds(bounds),
    })
  }

  fn set_size(&self, size: Size) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetSize(size),
    })
  }

  fn set_position(&self, position: Position) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetPosition(position),
    })
  }

  fn set_focus(&self) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetFocus,
    })
  }

  fn hide(&self) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::Hide,
    })
  }

  fn show(&self) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::Show,
    })
  }

  fn eval_script<S: Into<String>>(&self, script: S) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::EvaluateScript(script.into()),
    })
  }

  fn eval_script_with_callback<S: Into<String>>(
    &self,
    script: S,
    callback: impl Fn(String) + Send + 'static,
  ) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::EvaluateScriptWithCallback(script.into(), Box::new(callback)),
    })
  }

  fn reparent(&self, window_id: WindowId) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::Reparent(window_id, tx),
    })?;
    let result = rx.recv().map_err(|_| Error::FailedToReceiveMessage)?;
    if result.is_ok() {
      *self.window_id.lock().unwrap() = window_id;
    }
    result
  }

  fn cookies_for_url(&self, url: Url) -> Result<Vec<Cookie<'static>>> {
    let (tx, rx) = mpsc::channel();
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::CookiesForUrl(url, tx),
    })?;
    rx.recv().map_err(|_| Error::FailedToReceiveMessage)?
  }

  fn cookies(&self) -> Result<Vec<Cookie<'static>>> {
    webview_getter!(self, Cookies)
  }

  fn set_cookie(&self, cookie: Cookie<'_>) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetCookie(cookie.into_owned()),
    })
  }

  fn delete_cookie(&self, cookie: Cookie<'_>) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::DeleteCookie(cookie.into_owned()),
    })
  }

  fn set_auto_resize(&self, auto_resize: bool) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetAutoResize(auto_resize),
    })
  }

  fn set_zoom(&self, scale_factor: f64) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetZoom(scale_factor),
    })
  }

  fn set_background_color(&self, color: Option<Color>) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::SetBackgroundColor(color),
    })
  }

  fn clear_all_browsing_data(&self) -> Result<()> {
    self.context.send_message(Message::Webview {
      window_id: *self.window_id.lock().unwrap(),
      webview_id: self.webview_id,
      message: WebviewMessage::ClearAllBrowsingData,
    })
  }
}

/// Reposition every child webview to follow the parent window size.
///
/// Children with a bounds rate (auto-resize / window-filling) are recomputed
/// from the current window size; children with fixed bounds keep whatever bounds
/// they were last given.
pub(crate) fn layout_app_window(appwindow: &AppWindow) {
  let parent_size = appwindow.window.surface_size();
  let win_w = parent_size.width as f32;
  let win_h = parent_size.height as f32;
  let scale = appwindow.window.scale_factor();
  for child in &appwindow.children {
    let Some(rate) = child.bounds_rate else {
      continue;
    };
    let x = (rate.x * win_w).round() as i32;
    let y = (rate.y * win_h).round() as i32;
    let w = (rate.width * win_w).round() as i32;
    let h = (rate.height * win_h).round() as i32;
    child.host.notify_move_or_resize_started();
    child.apply_physical_bounds(scale, x, y, w, h);
    child.host.was_resized();
  }
}

/// Compute the bounds rate of a child webview relative to its parent window.
///
/// For webiews filling the window, default rate is used, otherwise the rate is computed from the current bounds and parent size
/// if auto_resize is enabled, otherwise None is returned.
pub(crate) fn compute_child_bounds_rate(
  bounds: Option<&Rect>,
  auto_resize: bool,
  parent_size: PhysicalSize<u32>,
  scale: f64,
) -> Option<BoundsRate> {
  let Some(bounds) = bounds else {
    return Some(BoundsRate::default());
  };

  if !auto_resize {
    return None;
  }

  let min_w = parent_size.width.max(1) as i32;
  let min_h = parent_size.height.max(1) as i32;

  let pos = bounds.position.to_physical::<i32>(scale);
  let size = bounds.size.to_physical::<u32>(scale);

  let x = pos.x;
  let y = pos.y;
  let w = size.width;
  let h = size.height;

  Some(BoundsRate {
    x: x as f32 / min_w as f32,
    y: y as f32 / min_h as f32,
    width: w as f32 / min_w as f32,
    height: h as f32 / min_h as f32,
  })
}

pub(crate) const INITIAL_LOAD_URL: &str = concat!(
  "data:text/html;charset=utf-8,",
  "%3C!doctype%20html%3E",
  "%3Chtml%20data-tauri-cef-internal%3D%22initial-load%22%3E",
  "%3Chead%3E",
  "%3Cmeta%20charset%3D%22utf-8%22%3E",
  "%3Ctitle%3ETauri%20CEF%20Initial%20Load%3C%2Ftitle%3E",
  "%3C%2Fhead%3E",
  "%3Cbody%20data-tauri-cef-internal%3D%22initial-load%22%3E",
  "%3C!--%20Tauri%20CEF%20internal%20initial%20load%20placeholder%20--%3E",
  "%3C%2Fbody%3E",
  "%3C%2Fhtml%3E",
);

/// Maps a pending `Page.addScriptToEvaluateOnNewDocument` CDP message id to the
/// `(browser, real_url)` whose real navigation is deferred until that message is
/// acknowledged.
///
/// The keys only ever come from `allocate_runtime_devtools_message_id`, whose
/// reserved range no caller identifier can reach, so a caller cannot release the
/// deferred navigation early by sending a request with a hardcoded `id`.
pub(crate) type PendingInitialLoads = Arc<Mutex<HashMap<i32, (Browser, String)>>>;

cef::wrap_dev_tools_message_observer! {
  struct TauriDevToolsProtocolObserver {
    handlers: Arc<Mutex<Vec<Arc<DevToolsProtocolHandler>>>>,
    pending_initial_loads: PendingInitialLoads,
    dialogs: crate::dialog::DialogState,
  }

  impl DevToolsMessageObserver {
    fn on_dev_tools_message(
      &self,
      _browser: Option<&mut cef::Browser>,
      message: Option<&[u8]>,
    ) -> std::os::raw::c_int {
      if let Some(message) = message {
        let protocol = DevToolsProtocol::Message(message.to_vec());
        if let Ok(handlers) = self.handlers.lock() {
          for handler in handlers.iter() {
            handler(protocol.clone());
          }
        }
      }
      0
    }

    fn on_dev_tools_method_result(
      &self,
      _browser: Option<&mut Browser>,
      message_id: std::os::raw::c_int,
      success: std::os::raw::c_int,
      result: Option<&[u8]>,
    ) {
      // The real navigation was deferred until the document-start script was
      // registered; this result acknowledges that, so kick off the real load.
      if let Some((browser, initial_url)) = self
        .pending_initial_loads
        .lock()
        .unwrap()
        .remove(&message_id)
      {
        post_load_initial_url(browser, initial_url);
      }

      let protocol = DevToolsProtocol::MethodResult {
        message_id,
        success: success != 0,
        result: result.map(|r| r.to_vec()).unwrap_or_default(),
      };
      if let Ok(handlers) = self.handlers.lock() {
        for handler in handlers.iter() {
          handler(protocol.clone());
        }
      }
    }

    fn on_dev_tools_event(
      &self,
      browser: Option<&mut Browser>,
      method: Option<&CefString>,
      params: Option<&[u8]>,
    ) {
      if let (Some(browser), Some(method)) = (browser, method)
        && self.dialogs.accepts_browser(browser.identifier()) {
        self.dialogs.on_event(&method.to_string(), params.unwrap_or_default());
      }
      let protocol = DevToolsProtocol::Event {
        method: method.map(|m| format!("{m}")).unwrap_or_default(),
        params: params.map(|p| p.to_vec()).unwrap_or_default(),
      };
      if let Ok(handlers) = self.handlers.lock() {
        for handler in handlers.iter() {
          handler(protocol.clone());
        }
      }
    }
  }
}

fn runtime_evaluate_result_to_json(result: Option<&[u8]>) -> String {
  let Some(result) = result else {
    return String::new();
  };
  let Ok(result) = serde_json::from_slice::<serde_json::Value>(result) else {
    return String::new();
  };

  if result.get("exceptionDetails").is_some() {
    return String::new();
  }

  let remote_object = result.get("result").unwrap_or(&result);
  remote_object
    .get("value")
    .and_then(|value| serde_json::to_string(value).ok())
    .unwrap_or_default()
}

type EvalScriptCallback = Box<dyn Fn(String) + Send + 'static>;

cef::wrap_dev_tools_message_observer! {
  struct EvalScriptWithCallbackDevToolsObserver {
    message_id: i32,
    callback: Arc<Mutex<Option<EvalScriptCallback>>>,
    registration: Arc<Mutex<Option<cef::Registration>>>,
  }

  impl DevToolsMessageObserver {
    fn on_dev_tools_method_result(
      &self,
      _browser: Option<&mut Browser>,
      message_id: std::os::raw::c_int,
      success: std::os::raw::c_int,
      result: Option<&[u8]>,
    ) {
      if message_id != self.message_id {
        return;
      }

      let Some(callback) = self.callback.lock().unwrap().take() else {
        return;
      };

      let result = if success != 0 {
        runtime_evaluate_result_to_json(result)
      } else {
        String::new()
      };
      callback(result);

      let _ = self.registration.lock().unwrap().take();
    }
  }
}

/// Registers a DevTools protocol observer. Returns the [`cef::Registration`] which must be
/// kept alive for the observer to stay registered. The observer is unregistered when
/// the Registration is dropped.
pub(crate) fn add_dev_tools_observer(
  browser: &Browser,
  handlers: Arc<Mutex<Vec<Arc<DevToolsProtocolHandler>>>>,
  pending_initial_loads: PendingInitialLoads,
  dialogs: crate::dialog::DialogState,
) -> Option<cef::Registration> {
  browser.host().and_then(|host| {
    let mut observer = TauriDevToolsProtocolObserver::new(handlers, pending_initial_loads, dialogs);
    let registration = host.add_dev_tools_message_observer(Some(&mut observer))?;
    if let Ok(id) = crate::devtools::allocate_runtime_devtools_message_id() {
      let message = serde_json::json!({"id":id,"method":"Page.enable","params":{}}).to_string();
      let _ = host.send_dev_tools_message(Some(message.as_bytes()));
    }
    Some(registration)
  })
}

fn devtools_initialization_script_source(
  initialization_scripts: &[CefInitScript],
  custom_protocol_scheme: &str,
  custom_scheme_domain_names: &[String],
) -> Option<String> {
  if initialization_scripts.is_empty() {
    return None;
  }

  let custom_protocol = serde_json::to_string(&format!("{custom_protocol_scheme}:")).ok()?;
  let custom_domains = serde_json::to_string(custom_scheme_domain_names).ok()?;
  let mut source = format!(
    r#"{{
  const __TAURI_CEF_INIT_CUSTOM_PROTOCOL__ = {custom_protocol};
  const __TAURI_CEF_INIT_CUSTOM_DOMAINS__ = new Set({custom_domains});
  const __TAURI_CEF_INIT_IS_CUSTOM_PROTOCOL__ =
    location.protocol === __TAURI_CEF_INIT_CUSTOM_PROTOCOL__
    && __TAURI_CEF_INIT_CUSTOM_DOMAINS__.has(location.hostname);
  const __TAURI_CEF_INIT_IS_MAIN_FRAME__ = (() => {{
    try {{
      return window.top === window;
    }} catch (_) {{
      return false;
    }}
  }})();
"#
  );

  for init_script in initialization_scripts {
    source.push_str("  if (!__TAURI_CEF_INIT_IS_CUSTOM_PROTOCOL__");
    if init_script.for_main_frame_only {
      source.push_str(" && __TAURI_CEF_INIT_IS_MAIN_FRAME__");
    }
    source.push_str(") {\n");
    source.push_str(init_script.script.as_str());
    source.push_str("\n  }\n");
  }

  source.push_str("}\n");
  Some(source)
}

fn register_initialization_scripts(
  browser: &Browser,
  initialization_scripts: &[CefInitScript],
  custom_protocol_scheme: &str,
  custom_scheme_domain_names: &[String],
  initial_url: String,
  pending_initial_loads: &PendingInitialLoads,
) -> std::result::Result<bool, crate::DevToolsMessageIdExhausted> {
  let Some(source) = devtools_initialization_script_source(
    initialization_scripts,
    custom_protocol_scheme,
    custom_scheme_domain_names,
  ) else {
    return Ok(false);
  };
  let Some(host) = browser.host() else {
    return Ok(false);
  };

  let message_id = crate::devtools::allocate_runtime_devtools_message_id()?;
  let message = serde_json::json!({
    "id": message_id,
    "method": "Page.addScriptToEvaluateOnNewDocument",
    "params": {
      "source": source,
    }
  })
  .to_string();

  pending_initial_loads
    .lock()
    .unwrap()
    .insert(message_id, (browser.clone(), initial_url));
  if host.send_dev_tools_message(Some(message.as_bytes())) == 1 {
    Ok(true)
  } else {
    pending_initial_loads.lock().unwrap().remove(&message_id);
    Ok(false)
  }
}

wrap_task! {
  struct LoadInitialUrlTask {
    browser: Browser,
    initial_url: String,
  }

  impl Task {
    fn execute(&self) {
      load_initial_url(&self.browser, &self.initial_url);
    }
  }
}

fn post_load_initial_url(browser: Browser, initial_url: String) {
  let mut task = LoadInitialUrlTask::new(browser, initial_url);
  cef::post_task(sys::cef_thread_id_t::TID_UI.into(), Some(&mut task));
}

// Browsers are created with an inert internal document so the BrowserHost exists
// before the app's real first navigation starts. That gives us a chance to
// register the CDP document-start script for remote/cross-site navigations; the
// custom-protocol path still injects into HTML because CEF does not apply this
// CDP hook to those documents reliably.
//
// The real load is posted as a CEF UI task instead of performed inline. This
// keeps the browser creation/CDP setup stack from re-entering navigation.
pub(crate) fn load_initial_url_after_registering_initialization_scripts(
  browser: &Browser,
  initialization_scripts: &[CefInitScript],
  custom_protocol_scheme: &str,
  custom_scheme_domain_names: &[String],
  initial_url: &str,
  pending_initial_loads: &PendingInitialLoads,
) {
  let browser_for_callback = browser.clone();
  let initial_url = initial_url.to_string();
  let is_waiting_for_initialization_scripts = register_initialization_scripts(
    browser,
    initialization_scripts,
    custom_protocol_scheme,
    custom_scheme_domain_names,
    initial_url.clone(),
    pending_initial_loads,
  );

  match is_waiting_for_initialization_scripts {
    Ok(false) => post_load_initial_url(browser_for_callback, initial_url),
    Ok(true) => {}
    Err(error) => {
      // Exhaustion cannot fall through to a navigation without the requested
      // document-start scripts or reuse another operation's acknowledgment.
      log::error!("CEF initialization failed: {error}");
      if let Some(host) = browser.host() {
        host.close_browser(1);
      }
    }
  }
}

fn load_initial_url(browser: &Browser, initial_url: &str) {
  if let Some(frame) = browser.main_frame() {
    frame.load_url(Some(&CefString::from(initial_url)));
  }
}
