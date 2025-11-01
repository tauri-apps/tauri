use base64::Engine;
use cef::{rc::*, *};
use sha2::{Digest, Sha256};
use std::{
  cell::RefCell,
  collections::HashMap,
  sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
  },
};
use tauri_runtime::{
  webview::{InitializationScript, PendingWebview, UriSchemeProtocol},
  window::{PendingWindow, WindowId},
  RunEvent, UserEvent,
};
use tauri_utils::html::normalize_script_for_csp;

use crate::{AppWindow, BrowserViewWrapper, CefRuntime, Message};

mod request_handler;

#[derive(Clone)]
pub struct CefInitScript {
  pub script: InitializationScript,
  pub hash: String,
}

impl CefInitScript {
  pub fn new(script: InitializationScript) -> Self {
    let hash = hash_script(script.script.as_str());
    Self { script, hash }
  }
}

fn hash_script(script: &str) -> String {
  let normalized = normalize_script_for_csp(script.as_bytes());
  let mut hasher = Sha256::new();
  hasher.update(&normalized);
  let hash = hasher.finalize();
  format!(
    "'sha256-{}'",
    base64::engine::general_purpose::STANDARD.encode(hash)
  )
}

// Initialization scripts are now injected into HTML responses via ResourceHandler

#[derive(Clone)]
pub struct Context<T: UserEvent> {
  pub windows: Arc<RefCell<HashMap<WindowId, AppWindow>>>,
  pub callback: Arc<RefCell<Box<dyn Fn(RunEvent<T>)>>>,
  pub next_window_id: Arc<AtomicU32>,
  pub next_webview_id: Arc<AtomicU32>,
  pub next_window_event_id: Arc<AtomicU32>,
  pub next_webview_event_id: Arc<AtomicU32>,
}

impl<T: UserEvent> Context<T> {
  pub fn next_window_id(&self) -> WindowId {
    self.next_window_id.fetch_add(1, Ordering::Relaxed).into()
  }

  pub fn next_webview_id(&self) -> u32 {
    self.next_webview_id.fetch_add(1, Ordering::Relaxed)
  }

  pub fn next_window_event_id(&self) -> u32 {
    self.next_window_event_id.fetch_add(1, Ordering::Relaxed)
  }

  pub fn next_webview_event_id(&self) -> u32 {
    self.next_webview_event_id.fetch_add(1, Ordering::Relaxed)
  }
}

wrap_app! {
  pub struct TauriApp<T: UserEvent> {
    context: Context<T>,
  }

  impl App {
    fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
      Some(AppBrowserProcessHandler::new(self.context.clone()))
    }

    /// Called before the process starts to register custom schemes.
    /// This is where we mark schemes as fetch-enabled, secure, and CORS-enabled.
    fn on_register_custom_schemes(&self, registrar: Option<&mut SchemeRegistrar>) {
      if let Some(registrar) = registrar {
        // Standard CEF scheme options for custom protocols:
        // - FETCH_ENABLED: Allows Fetch API requests
        // - SECURE: Treats as secure like https (no mixed content warnings)
        // - CORS_ENABLED: Allows CORS requests
        // - STANDARD: Standard URL scheme behavior
        let scheme_options = (cef_dll_sys::cef_scheme_options_t::CEF_SCHEME_OPTION_FETCH_ENABLED as i32)
          | (cef_dll_sys::cef_scheme_options_t::CEF_SCHEME_OPTION_SECURE as i32)
          | (cef_dll_sys::cef_scheme_options_t::CEF_SCHEME_OPTION_CORS_ENABLED as i32)
          | (cef_dll_sys::cef_scheme_options_t::CEF_SCHEME_OPTION_STANDARD as i32);

        for scheme in ["ipc", "tauri"] {
          let scheme_name = CefString::from(scheme);
          registrar.add_custom_scheme(Some(&scheme_name), scheme_options);
        }
      }
    }
  }
}

wrap_browser_process_handler! {
  struct AppBrowserProcessHandler<T: UserEvent> {
    context: Context<T>,
  }

  impl BrowserProcessHandler {
    // The real lifespan of cef starts from `on_context_initialized`, so all the cef objects should be manipulated after that.
    fn on_context_initialized(&self) {
      (self.context.callback.borrow_mut())(RunEvent::Ready);
    }
  }
}

wrap_load_handler! {
  struct BrowserLoadHandler {
    initialization_scripts: Vec<CefInitScript>,
  }

  impl LoadHandler {
    fn on_load_end(
      &self,
      browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      http_status_code: ::std::os::raw::c_int,
    ) {
      // Only execute scripts for successful loads (200-299)
      if http_status_code < 200 || http_status_code >= 300 {
        return;
      }

      let Some(frame) = frame else { return };

      // Get the URL to check if it's a remote URL
      let url = frame.url();
      let url_str = cef::CefString::from(&url).to_string();
      let url_obj = url::Url::parse(&url_str).ok();

      // Only execute scripts for remote URLs (http/https)
      // Custom schemes use HTML injection
      let is_remote_url = url_obj
        .as_ref()
        .map(|u| matches!(u.scheme(), "http" | "https"))
        .unwrap_or(false);

      if !is_remote_url {
        return;
      }

      let is_main_frame = frame.is_main() == 1;

      // Filter scripts based on frame type
      let scripts_to_execute: Vec<_> = if is_main_frame {
        self.initialization_scripts.clone()
      } else {
        self.initialization_scripts
          .iter()
          .filter(|s| !s.script.for_main_frame_only)
          .cloned()
          .collect()
      };

      // Execute each script via frame.execute_java_script
      for script in scripts_to_execute {
        let script_text = script.script.script.clone();
        let script_url = format!("{}://__tauri_init_script__", url_obj.as_ref().map(|u| u.scheme()).unwrap_or("http"));

        // Execute JavaScript in the frame
        frame.execute_java_script(
          Some(&cef::CefString::from(script_text.as_str())),
          Some(&cef::CefString::from(script_url.as_str())),
          0,
        );
      }
    }
  }
}

wrap_client! {
  struct BrowserClient {
    initialization_scripts: Vec<CefInitScript>,
  }

  impl Client {
    fn request_handler(&self) -> Option<RequestHandler> {
      // Use pre-computed script hashes (computed once at webview creation)
      Some(request_handler::WebRequestHandler::new(
        self.initialization_scripts.clone(),
      ))
    }

    fn load_handler(&self) -> Option<LoadHandler> {
      Some(BrowserLoadHandler::new(
        self.initialization_scripts.clone(),
      ))
    }
  }
}

wrap_browser_view_delegate! {
  struct BrowserViewDelegateImpl {
    use_alloy_style: bool,
  }

  impl ViewDelegate {}

  impl BrowserViewDelegate {
    fn browser_runtime_style(&self) -> RuntimeStyle {
      use cef::sys::cef_runtime_style_t;

      if self.use_alloy_style {
        // Use Alloy style for additional webviews (multiwebview support)
        RuntimeStyle::from(cef_runtime_style_t::CEF_RUNTIME_STYLE_ALLOY)
      } else {
        // Use Chrome style (default) for the first webview
        RuntimeStyle::from(cef_runtime_style_t::CEF_RUNTIME_STYLE_CHROME)
      }
    }
  }
}

wrap_window_delegate! {
  struct AppWindowDelegate {
    initial_browser_view: Option<BrowserView>,
  }

  impl ViewDelegate {
    fn on_child_view_changed(
      &self,
      _view: Option<&mut View>,
      _added: ::std::os::raw::c_int,
      _child: Option<&mut View>,
    ) {
      // view.as_panel().map(|x| x.as_window().map(|w| w.close()));
    }
  }

  impl PanelDelegate {}

  impl WindowDelegate {
    fn on_window_created(&self, window: Option<&mut Window>) {
      if let Some(window) = window {
        // If we have an initial browser view, add it
        if let Some(ref browser_view) = self.initial_browser_view {
          let mut view = View::from(browser_view);
          window.add_child_view(Some(&mut view));
        }
        window.show();
      }
    }

    fn on_window_destroyed(&self, _window: Option<&mut Window>) {
      // TODO: send destroyed event
    }

    fn with_standard_window_buttons(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      1
    }

    fn can_resize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      1
    }

    fn can_maximize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      1
    }

    fn can_minimize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      1
    }

    fn can_close(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      1
    }
  }
}

pub fn handle_message<T: UserEvent>(context: &Context<T>, message: Message<T>) {
  match message {
    Message::CreateWindow {
      window_id,
      webview_id,
      pending,
      after_window_creation: _todo,
    } => create_window(context, window_id, webview_id, pending),
    Message::CreateWebview {
      window_id,
      webview_id,
      pending,
    } => create_webview(
      WebviewKind::WindowChild,
      context,
      window_id,
      webview_id,
      pending,
    ),
    #[cfg(any(debug_assertions, feature = "devtools"))]
    Message::OpenDevTools {
      window_id,
      webview_id,
    } => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        if let Some(browser_view_wrapper) = app_window
          .webviews
          .iter()
          .find(|w| w.webview_id == webview_id)
        {
          if let Some(browser) = browser_view_wrapper.browser_view.browser() {
            if let Some(host) = browser.host() {
              // ShowDevTools(window_info, client, settings, inspect_element_at)
              // Using None for client and default settings, inspect at (0,0)
              let window_info = cef::WindowInfo::default();
              let settings = cef::BrowserSettings::default();
              let inspect_at = cef::Point { x: 0, y: 0 };
              host.show_dev_tools(
                Some(&window_info),
                Option::<&mut cef::Client>::None,
                Some(&settings),
                Some(&inspect_at),
              );
            }
          }
        }
      }
    }
    #[cfg(any(debug_assertions, feature = "devtools"))]
    Message::CloseDevTools {
      window_id,
      webview_id,
    } => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        if let Some(browser_view_wrapper) = app_window
          .webviews
          .iter()
          .find(|w| w.webview_id == webview_id)
        {
          if let Some(browser) = browser_view_wrapper.browser_view.browser() {
            if let Some(host) = browser.host() {
              host.close_dev_tools();
            }
          }
        }
      }
    }
    Message::Task(t) => t(),
    Message::UserEvent(evt) => {
      (context.callback.borrow_mut())(RunEvent::UserEvent(evt));
    }
    Message::Noop => {}
  }
}

wrap_task! {
  pub struct SendMessageTask<T: UserEvent>  {
    context: Context<T>,
    message: Arc<RefCell<Message<T>>>,
  }

  impl Task {
    fn execute(&self) {
      handle_message(&self.context, self.message.replace(Message::Noop));
    }
  }
}

fn create_window<T: UserEvent>(
  context: &Context<T>,
  window_id: WindowId,
  _webview_id: u32,
  pending: PendingWindow<T, CefRuntime<T>>,
) {
  let label = pending.label.clone();

  // Create window delegate - we'll handle webviews separately
  // For windows without webviews, we use a delegate without initial browser view
  let mut delegate = AppWindowDelegate::new(None);

  let window = window_create_top_level(Some(&mut delegate)).expect("Failed to create window");
  window.show();

  // Insert window with empty webviews list
  context.windows.borrow_mut().insert(
    window_id,
    AppWindow {
      label,
      window,
      webviews: Vec::new(),
      content_panel: None,
    },
  );

  // If a webview was provided, create it now
  if let Some(webview) = pending.webview {
    let webview_id = context.next_webview_id();
    create_webview(
      WebviewKind::WindowContent,
      context,
      window_id,
      webview_id,
      webview,
    );
  }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum WebviewKind {
  // webview is the entire window content
  WindowContent,
  // webview is a child of the window, which can contain other webviews too
  WindowChild,
}

fn create_webview<T: UserEvent>(
  kind: WebviewKind,
  context: &Context<T>,
  window_id: WindowId,
  webview_id: u32,
  pending: PendingWebview<T, CefRuntime<T>>,
) {
  // Get the window - return early if not found
  let mut windows = context.windows.borrow_mut();
  let app_window = match windows.get_mut(&window_id) {
    Some(w) => w,
    None => {
      eprintln!("Window {:?} not found when creating webview", window_id);
      return;
    }
  };

  // Get initialization scripts from webview attributes
  // Pre-compute script hashes once at webview creation time
  let initialization_scripts: Vec<_> = pending
    .webview_attributes
    .initialization_scripts
    .into_iter()
    .map(CefInitScript::new)
    .collect();

  let mut client = BrowserClient::new(initialization_scripts.clone());
  let url = CefString::from(pending.url.as_str());

  let global_context =
    request_context_get_global_context().expect("Failed to get global request context");
  let global_cache_path: CefStringUtf16 = (&global_context.cache_path()).into();

  let mut request_context = request_context_create_context(
    Some(&RequestContextSettings {
      cache_path: global_cache_path,
      ..Default::default()
    }),
    Option::<&mut RequestContextHandler>::None,
  );
  if let Some(request_context) = &request_context {
    // Ensure schemes are registered with proper flags (fetch-enabled, secure, etc.)
    for (scheme, handler) in pending.uri_scheme_protocols {
      let label = app_window.label.clone();
      request_context.register_scheme_handler_factory(
        Some(&scheme.as_str().into()),
        None,
        Some(&mut request_handler::UriSchemeHandlerFactory::new(
          request_handler::UriSchemeContext {
            label,
            handler: Arc::new(handler) as Arc<UriSchemeProtocol>,
            response: Arc::new(RefCell::new(None)),
            initialization_scripts: Some(initialization_scripts.clone()),
          },
        )),
      );
    }
  }

  let mut browser_view_delegate =
    BrowserViewDelegateImpl::new(matches!(kind, WebviewKind::WindowChild));

  let browser_view = browser_view_create(
    Some(&mut client),
    Some(&url),
    Some(&Default::default()),
    Option::<&mut DictionaryValue>::None,
    request_context.as_mut(),
    Some(&mut browser_view_delegate),
  )
  .expect("Failed to create browser view");

  let mut view = View::from(&browser_view);

  let bounds = pending.webview_attributes.bounds.map(|bounds| {
    let device_scale_factor = app_window
      .window
      .display()
      .map(|d| d.device_scale_factor() as f64)
      .unwrap_or(1.0);
    let physical_position = bounds.position.to_physical::<i32>(device_scale_factor);
    let physical_size = bounds.size.to_physical::<u32>(device_scale_factor);
    Rect {
      x: physical_position.x,
      y: physical_position.y,
      width: physical_size.width as i32,
      height: physical_size.height as i32,
    }
  });

  if kind == WebviewKind::WindowChild {
    let overlay = app_window
      .window
      .add_overlay_view(
        Some(&mut view),
        cef::DockingMode::from(cef::sys::cef_docking_mode_t::CEF_DOCKING_MODE_CUSTOM),
        1,
      )
      .expect("Failed to add overlay view");

    if let Some(bounds) = &bounds {
      overlay.set_bounds(Some(bounds));
    }
    overlay.set_visible(1);

    app_window.webviews.push(BrowserViewWrapper {
      webview_id,
      browser_view,
      overlay: Some(overlay),
    });
  } else {
    app_window.window.add_child_view(Some(&mut view));
    if let Some(bounds) = &bounds {
      view.set_bounds(Some(bounds));
    }

    app_window.webviews.push(BrowserViewWrapper {
      webview_id,
      browser_view,
      overlay: None,
    });
  }
}
