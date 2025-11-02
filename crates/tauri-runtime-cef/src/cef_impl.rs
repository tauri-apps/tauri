use base64::Engine;
use cef::{rc::*, *};
use sha2::{Digest, Sha256};
use std::{
  cell::RefCell,
  collections::HashMap,
  sync::{
    atomic::{AtomicU32, Ordering},
    mpsc::channel,
    Arc, Mutex,
  },
};
use tauri_runtime::{
  dpi::{PhysicalPosition, PhysicalSize, Position, Rect, Size},
  webview::{InitializationScript, PendingWebview, UriSchemeProtocol},
  window::{PendingWindow, WindowEvent, WindowId},
  ExitRequestedEventAction, RunEvent, UserEvent,
};
use tauri_utils::html::normalize_script_for_csp;

use crate::{AppWindow, BrowserViewWrapper, CefRuntime, Message, WebviewMessage, WindowMessage};

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
    custom_schemes: Vec<String>,
  }

  impl App {
    fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
      Some(AppBrowserProcessHandler::new(self.context.clone()))
    }

    fn on_register_custom_schemes(&self, registrar: Option<&mut SchemeRegistrar>) {
      if let Some(registrar) = registrar {
        let scheme_options = (cef_dll_sys::cef_scheme_options_t::CEF_SCHEME_OPTION_FETCH_ENABLED as i32)
          | (cef_dll_sys::cef_scheme_options_t::CEF_SCHEME_OPTION_SECURE as i32)
          | (cef_dll_sys::cef_scheme_options_t::CEF_SCHEME_OPTION_CORS_ENABLED as i32)
          | (cef_dll_sys::cef_scheme_options_t::CEF_SCHEME_OPTION_STANDARD as i32);

        for scheme in &self.custom_schemes {
          registrar.add_custom_scheme(Some(&(scheme.as_str()).into()), scheme_options);
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
      if http_status_code < 200 || http_status_code >= 300 {
        return;
      }

      let Some(frame) = frame else { return };

      let url = frame.url();
      let url_str = cef::CefString::from(&url).to_string();
      let url_obj = url::Url::parse(&url_str).ok();

      let is_remote_url = url_obj
        .as_ref()
        .map(|u| matches!(u.scheme(), "http" | "https"))
        .unwrap_or(false);

      if !is_remote_url {
        return;
      }

      let is_main_frame = frame.is_main() == 1;

      let scripts_to_execute: Vec<_> = if is_main_frame {
        self.initialization_scripts.clone()
      } else {
        self.initialization_scripts
          .iter()
          .filter(|s| !s.script.for_main_frame_only)
          .cloned()
          .collect()
      };

      for script in scripts_to_execute {
        let script_text = script.script.script.clone();
        let script_url = format!("{}://__tauri_init_script__", url_obj.as_ref().map(|u| u.scheme()).unwrap_or("http"));

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
        // Use Alloy style for multiwebview support
        RuntimeStyle::from(cef_runtime_style_t::CEF_RUNTIME_STYLE_ALLOY)
      } else {
        RuntimeStyle::from(cef_runtime_style_t::CEF_RUNTIME_STYLE_CHROME)
      }
    }
  }
}

wrap_window_delegate! {
  struct AppWindowDelegate<T: UserEvent> {
    window_id: WindowId,
    callback: Arc<RefCell<Box<dyn Fn(RunEvent<T>)>>>,
    windows: Arc<RefCell<HashMap<WindowId, AppWindow>>>,
  }

  impl ViewDelegate {}

  impl PanelDelegate {}

  impl WindowDelegate {
    fn on_window_destroyed(&self, _window: Option<&mut Window>) {
      on_window_destroyed(self.window_id, &self.windows, &self.callback);
    }

    fn with_standard_window_buttons(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      // Check if decorations are enabled (standard buttons only shown when decorated)
      let windows = self.windows.borrow();
      if let Some(app_window) = windows.get(&self.window_id) {
        app_window.attributes.decorations.unwrap_or(true) as i32
      } else {
        1
      }
    }

    fn can_resize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      let windows = self.windows.borrow();
      if let Some(app_window) = windows.get(&self.window_id) {
        app_window.attributes.resizable.unwrap_or(true) as i32
      } else {
        1
      }
    }

    fn can_maximize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      let windows = self.windows.borrow();
      if let Some(app_window) = windows.get(&self.window_id) {
        // Can maximize if maximizable is true and resizable is true (or not set, defaulting to true)
        let resizable = app_window.attributes.resizable.unwrap_or(true);
        let maximizable = app_window.attributes.maximizable.unwrap_or(true);
        (resizable && maximizable) as i32
      } else {
        1
      }
    }

    fn can_minimize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      let windows = self.windows.borrow();
      if let Some(app_window) = windows.get(&self.window_id) {
        app_window.attributes.minimizable.unwrap_or(true) as i32
      } else {
        1
      }
    }

    fn can_close(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      let windows = self.windows.borrow();
      let closable = windows
        .get(&self.window_id)
        .map(|w| w.attributes.closable.unwrap_or(true))
        .unwrap_or(true);

      if !closable {
        return 0;
      }

      let (tx, rx) = channel();
      let event = WindowEvent::CloseRequested { signal_tx: tx };

      send_window_event(self.window_id, &self.windows, &self.callback, event.clone());

      let should_prevent = matches!(rx.try_recv(), Ok(true));

      if should_prevent {
        0
      } else {
        1
      }
    }
  }
}

fn get_browser_view<T: UserEvent>(
  context: &Context<T>,
  window_id: WindowId,
  webview_id: u32,
) -> Option<cef::BrowserView> {
  context
    .windows
    .borrow()
    .get(&window_id)
    .and_then(|app_window| {
      app_window
        .webviews
        .iter()
        .find(|w| w.webview_id == webview_id)
        .map(|w| w.browser_view.clone())
    })
}

fn handle_webview_message<T: UserEvent>(
  context: &Context<T>,
  window_id: WindowId,
  webview_id: u32,
  message: WebviewMessage,
) {
  match message {
    WebviewMessage::AddEventListener(event_id, handler) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        let listeners = app_window.webview_event_listeners.clone();
        let mut listeners_map = listeners.lock().unwrap();
        let webview_listeners = listeners_map
          .entry(webview_id)
          .or_insert_with(|| Arc::new(Mutex::new(HashMap::new())));
        webview_listeners.lock().unwrap().insert(event_id, handler);
      }
    }
    WebviewMessage::EvaluateScript(script) => {
      get_browser_view(context, window_id, webview_id)
        .and_then(|bv| bv.browser())
        .and_then(|b| b.main_frame())
        .map(|frame| {
          frame.execute_java_script(
            Some(&cef::CefString::from(script.as_str())),
            Some(&cef::CefString::from("")),
            0,
          );
        });
    }
    WebviewMessage::Navigate(url) => {
      get_browser_view(context, window_id, webview_id)
        .and_then(|bv| bv.browser())
        .and_then(|b| b.main_frame())
        .map(|frame| frame.load_url(Some(&cef::CefString::from(url.as_str()))));
    }
    WebviewMessage::Reload => {
      get_browser_view(context, window_id, webview_id)
        .and_then(|bv| bv.browser())
        .map(|browser| browser.reload());
    }
    WebviewMessage::Print => {
      get_browser_view(context, window_id, webview_id)
        .and_then(|bv| bv.browser())
        .and_then(|b| b.host())
        .map(|host| host.print());
    }
    WebviewMessage::Close => {
      if let Some(app_window) = context.windows.borrow_mut().get_mut(&window_id) {
        let webview_index = app_window
          .webviews
          .iter()
          .position(|w| w.webview_id == webview_id);

        if let Some(index) = webview_index {
          let browser_view_wrapper = app_window.webviews.remove(index);

          if let Some(overlay) = browser_view_wrapper.overlay {
            overlay.destroy();
          }

          app_window
            .webview_event_listeners
            .lock()
            .unwrap()
            .remove(&webview_id);
        }
      }
    }
    WebviewMessage::Show => {
      context
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|app_window| {
          app_window
            .webviews
            .iter()
            .find(|w| w.webview_id == webview_id)
        })
        .and_then(|wrapper| wrapper.overlay.as_ref())
        .map(|overlay| overlay.set_visible(1));
    }
    WebviewMessage::Hide => {
      context
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|app_window| {
          app_window
            .webviews
            .iter()
            .find(|w| w.webview_id == webview_id)
        })
        .and_then(|wrapper| wrapper.overlay.as_ref())
        .map(|overlay| overlay.set_visible(0));
    }
    WebviewMessage::SetPosition(position) => {
      context.windows.borrow().get(&window_id).map(|app_window| {
        let device_scale_factor = app_window
          .window
          .display()
          .map(|d| d.device_scale_factor() as f64)
          .unwrap_or(1.0);
        let physical_position = position.to_physical::<i32>(device_scale_factor);
        app_window
          .webviews
          .iter()
          .find(|w| w.webview_id == webview_id)
          .and_then(|wrapper| wrapper.overlay.as_ref())
          .map(|overlay| {
            let current_bounds = overlay.bounds();
            let new_bounds = cef::Rect {
              x: physical_position.x,
              y: physical_position.y,
              width: current_bounds.width,
              height: current_bounds.height,
            };
            overlay.set_bounds(Some(&new_bounds));
          });
      });
    }
    WebviewMessage::SetSize(size) => {
      context.windows.borrow().get(&window_id).map(|app_window| {
        let device_scale_factor = app_window
          .window
          .display()
          .map(|d| d.device_scale_factor() as f64)
          .unwrap_or(1.0);
        let physical_size = size.to_physical::<u32>(device_scale_factor);
        app_window
          .webviews
          .iter()
          .find(|w| w.webview_id == webview_id)
          .and_then(|wrapper| wrapper.overlay.as_ref())
          .map(|overlay| {
            let current_bounds = overlay.bounds();
            let new_bounds = cef::Rect {
              x: current_bounds.x,
              y: current_bounds.y,
              width: physical_size.width as i32,
              height: physical_size.height as i32,
            };
            overlay.set_bounds(Some(&new_bounds));
          });
      });
    }
    WebviewMessage::SetBounds(bounds) => {
      context.windows.borrow().get(&window_id).map(|app_window| {
        let device_scale_factor = app_window
          .window
          .display()
          .map(|d| d.device_scale_factor() as f64)
          .unwrap_or(1.0);
        let physical_position = bounds.position.to_physical::<i32>(device_scale_factor);
        let physical_size = bounds.size.to_physical::<u32>(device_scale_factor);
        app_window
          .webviews
          .iter()
          .find(|w| w.webview_id == webview_id)
          .and_then(|wrapper| wrapper.overlay.as_ref())
          .map(|overlay| {
            overlay.set_bounds(Some(&cef::Rect {
              x: physical_position.x,
              y: physical_position.y,
              width: physical_size.width as i32,
              height: physical_size.height as i32,
            }));
          });
      });
    }
    WebviewMessage::SetFocus => {
      get_browser_view(context, window_id, webview_id)
        .and_then(|bv| bv.browser())
        .and_then(|b| b.host())
        .map(|host| host.set_focus(1));
    }
    WebviewMessage::Reparent(target_window_id, tx) => {
      let mut windows = context.windows.borrow_mut();

      if !windows.contains_key(&target_window_id) {
        let _ = tx.send(Err(tauri_runtime::Error::FailedToSendMessage));
        return;
      };

      let Some(mut webview_wrapper) = windows.get_mut(&window_id).and_then(|app_window| {
        app_window
          .webviews
          .iter()
          .position(|w| w.webview_id == webview_id)
          .map(|index| app_window.webviews.remove(index))
      }) else {
        let _ = tx.send(Err(tauri_runtime::Error::FailedToSendMessage));
        return;
      };

      let Some(target_window) = windows.get_mut(&target_window_id) else {
        let _ = tx.send(Err(tauri_runtime::Error::FailedToSendMessage));
        return;
      };

      let bounds = webview_wrapper
        .overlay
        .as_ref()
        .map(|overlay| overlay.bounds())
        .unwrap_or_else(|| {
          // Use default bounds if we don't have existing bounds
          cef::Rect {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
          }
        });

      if let Some(overlay) = &webview_wrapper.overlay {
        overlay.destroy();
      }

      let overlay = target_window.window.add_overlay_view(
        Some(&mut View::from(&webview_wrapper.browser_view)),
        cef::DockingMode::from(cef::sys::cef_docking_mode_t::CEF_DOCKING_MODE_CUSTOM),
        1,
      );

      if let Some(new_overlay) = overlay {
        new_overlay.set_bounds(Some(&bounds));
        new_overlay.set_visible(1);

        webview_wrapper.overlay.replace(new_overlay);

        target_window.webviews.push(webview_wrapper);

        let _ = tx.send(Ok(()));
      } else {
        let _ = tx.send(Err(tauri_runtime::Error::FailedToSendMessage));
      }
    }
    WebviewMessage::SetAutoResize(_auto_resize) => {
      // TODO: Implement auto-resize functionality
    }
    WebviewMessage::SetZoom(scale_factor) => {
      get_browser_view(context, window_id, webview_id)
        .and_then(|bv| bv.browser())
        .and_then(|b| b.host())
        .map(|host| host.set_zoom_level(scale_factor));
    }
    WebviewMessage::SetBackgroundColor(color) => {
      // Convert Color to ARGB format (u32)
      let color_value = color
        .map(|c| {
          let (r, g, b, a) = c.into();
          // Convert to ARGB: (A << 24) | (R << 16) | (G << 8) | B
          ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        })
        .unwrap_or(0xFFFFFFFF);

      context
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|app_window| {
          app_window
            .webviews
            .iter()
            .find(|w| w.webview_id == webview_id)
        })
        .map(|wrapper| wrapper.browser_view.set_background_color(color_value));
    }
    WebviewMessage::ClearAllBrowsingData => {
      // TODO: Implement clear browsing data
    }
    // Getters
    WebviewMessage::Url(tx) => {
      let result = get_browser_view(context, window_id, webview_id)
        .and_then(|bv| bv.browser())
        .and_then(|b| b.main_frame())
        .map(|frame| {
          let url = frame.url();
          cef::CefString::from(&url).to_string()
        })
        .ok_or(tauri_runtime::Error::FailedToSendMessage);
      let _ = tx.send(result);
    }
    WebviewMessage::Bounds(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|app_window| {
          app_window
            .webviews
            .iter()
            .find(|w| w.webview_id == webview_id)
        })
        .and_then(|wrapper| wrapper.overlay.as_ref())
        .map(|overlay| {
          let bounds = overlay.bounds();
          let physical_position = PhysicalPosition::new(bounds.x, bounds.y);
          let physical_size = PhysicalSize::new(bounds.width as u32, bounds.height as u32);
          Rect {
            position: Position::Physical(physical_position),
            size: Size::Physical(physical_size),
          }
        })
        .ok_or(tauri_runtime::Error::FailedToSendMessage);
      let _ = tx.send(result);
    }
    WebviewMessage::Position(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|app_window| {
          app_window
            .webviews
            .iter()
            .find(|w| w.webview_id == webview_id)
        })
        .and_then(|wrapper| wrapper.overlay.as_ref())
        .map(|overlay| {
          let bounds = overlay.bounds();
          PhysicalPosition::new(bounds.x, bounds.y)
        })
        .ok_or(tauri_runtime::Error::FailedToSendMessage);
      let _ = tx.send(result);
    }
    WebviewMessage::Size(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|app_window| {
          app_window
            .webviews
            .iter()
            .find(|w| w.webview_id == webview_id)
        })
        .and_then(|wrapper| wrapper.overlay.as_ref())
        .map(|overlay| {
          let bounds = overlay.bounds();
          PhysicalSize::new(bounds.width as u32, bounds.height as u32)
        })
        .ok_or(tauri_runtime::Error::FailedToSendMessage);
      let _ = tx.send(result);
    }
    WebviewMessage::WithWebview(f) => {
      get_browser_view(context, window_id, webview_id).map(|browser_view| {
        f(Box::new(browser_view));
      });
    }
    // Devtools
    #[cfg(any(debug_assertions, feature = "devtools"))]
    WebviewMessage::OpenDevTools => {
      get_browser_view(context, window_id, webview_id)
        .and_then(|bv| bv.browser())
        .and_then(|b| b.host())
        .map(|host| {
          let window_info = cef::WindowInfo::default();
          let settings = cef::BrowserSettings::default();
          let inspect_at = cef::Point { x: 0, y: 0 };
          host.show_dev_tools(
            Some(&window_info),
            Option::<&mut cef::Client>::None,
            Some(&settings),
            Some(&inspect_at),
          );
        });
    }
    #[cfg(any(debug_assertions, feature = "devtools"))]
    WebviewMessage::CloseDevTools => {
      get_browser_view(context, window_id, webview_id)
        .and_then(|bv| bv.browser())
        .and_then(|b| b.host())
        .map(|host| host.close_dev_tools());
    }
    #[cfg(any(debug_assertions, feature = "devtools"))]
    WebviewMessage::IsDevToolsOpen(tx) => {
      let _ = tx.send(false);
    }
    WebviewMessage::CookiesForUrl(_url, tx) => {
      // TODO: Implement cookie retrieval for URL
      let _ = tx.send(Ok(Vec::new()));
    }
    WebviewMessage::Cookies(tx) => {
      // TODO: Implement cookie retrieval
      let _ = tx.send(Ok(Vec::new()));
    }
    WebviewMessage::SetCookie(_cookie) => {
      // TODO: Implement cookie setting
    }
    WebviewMessage::DeleteCookie(_cookie) => {
      // TODO: Implement cookie deletion
    }
  }
}

fn handle_window_message<T: UserEvent>(
  context: &Context<T>,
  window_id: WindowId,
  message: WindowMessage,
) {
  match message {
    WindowMessage::Close => {
      on_close_requested(window_id, &context.windows, &context.callback);
    }
    WindowMessage::Destroy => {
      on_window_close(window_id, &context.windows);
    }
    WindowMessage::AddEventListener(event_id, handler) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window
          .window_event_listeners
          .lock()
          .unwrap()
          .insert(event_id, handler);
      }
    }
    // Getters
    WindowMessage::ScaleFactor(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|w| w.window.display())
        .map(|d| Ok(d.device_scale_factor() as f64))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::InnerPosition(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| {
          let bounds = w.window.bounds();
          Ok(PhysicalPosition::new(bounds.x, bounds.y))
        })
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::OuterPosition(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| {
          let bounds = w.window.bounds();
          Ok(PhysicalPosition::new(bounds.x, bounds.y))
        })
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::InnerSize(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| {
          let bounds = w.window.bounds();
          Ok(PhysicalSize::new(bounds.width as u32, bounds.height as u32))
        })
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::OuterSize(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| {
          let bounds = w.window.bounds();
          Ok(PhysicalSize::new(bounds.width as u32, bounds.height as u32))
        })
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::IsFullscreen(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| Ok(w.window.is_fullscreen() == 1))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::IsMinimized(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| Ok(w.window.is_minimized() == 1))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::IsMaximized(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| Ok(w.window.is_maximized() == 1))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::IsFocused(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| Ok(w.window.has_focus() == 1))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::IsDecorated(_tx) => {
      // TODO: Implement decorations getter
      let _ = _tx.send(Ok(true));
    }
    WindowMessage::IsResizable(_tx) => {
      // TODO: Implement resizable getter
      let _ = _tx.send(Ok(true));
    }
    WindowMessage::IsMaximizable(_tx) => {
      let _ = _tx.send(Ok(true));
    }
    WindowMessage::IsMinimizable(_tx) => {
      let _ = _tx.send(Ok(true));
    }
    WindowMessage::IsClosable(_tx) => {
      let _ = _tx.send(Ok(true));
    }
    WindowMessage::IsVisible(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| Ok(w.window.is_visible() == 1))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::Title(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|w| {
          let title = w.window.title();
          Some(Ok(cef::CefString::from(&title).to_string()))
        })
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::CurrentMonitor(_tx) => {
      // TODO: Implement monitor getter
      let _ = _tx.send(Ok(None));
    }
    WindowMessage::PrimaryMonitor(_tx) => {
      // TODO: Implement monitor getter
      let _ = _tx.send(Ok(None));
    }
    WindowMessage::MonitorFromPoint(_tx, _x, _y) => {
      // TODO: Implement monitor getter
      let _ = _tx.send(Ok(None));
    }
    WindowMessage::AvailableMonitors(_tx) => {
      // TODO: Implement monitor getter
      let _ = _tx.send(Ok(Vec::new()));
    }
    WindowMessage::Theme(_tx) => {
      // TODO: Implement theme getter
      let _ = _tx.send(Ok(tauri_utils::Theme::Light));
    }
    WindowMessage::IsEnabled(_tx) => {
      let _ = _tx.send(Ok(true));
    }
    WindowMessage::IsAlwaysOnTop(_tx) => {
      // TODO: Implement always on top getter
      let _ = _tx.send(Ok(false));
    }
    WindowMessage::RawWindowHandle(_tx) => {
      // TODO: Implement raw window handle
      #[cfg(target_os = "linux")]
      {
        let _ = _tx.send(Ok(unsafe {
          raw_window_handle::WindowHandle::borrow_raw(raw_window_handle::RawWindowHandle::Xlib(
            raw_window_handle::XlibWindowHandle::new(0),
          ))
        }));
      }
      #[cfg(target_os = "macos")]
      {
        let _ = _tx.send(Ok(unsafe {
          raw_window_handle::WindowHandle::borrow_raw(raw_window_handle::RawWindowHandle::AppKit(
            raw_window_handle::AppKitWindowHandle::new(std::ptr::NonNull::from(&()).cast()),
          ))
        }));
      }
      #[cfg(windows)]
      {
        let _ = _tx.send(Ok(unsafe {
          raw_window_handle::WindowHandle::borrow_raw(raw_window_handle::RawWindowHandle::Win32(
            raw_window_handle::Win32WindowHandle::new(std::num::NonZeroIsize::MIN),
          ))
        }));
      }
    }
    // Setters
    WindowMessage::Center => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        if let Some(display) = app_window.window.display() {
          let work_area = display.work_area();
          let current_bounds = app_window.window.bounds();
          let center_x = work_area.x + (work_area.width - current_bounds.width) / 2;
          let center_y = work_area.y + (work_area.height - current_bounds.height) / 2;
          app_window.window.set_bounds(Some(&cef::Rect {
            x: center_x,
            y: center_y,
            width: current_bounds.width,
            height: current_bounds.height,
          }));
        }
      }
    }
    WindowMessage::RequestUserAttention(_attention_type) => {
      // TODO: Implement user attention
    }
    WindowMessage::SetEnabled(_enabled) => {
      // TODO: Implement enabled
    }
    WindowMessage::SetResizable(resizable) => {
      if let Some(app_window) = context.windows.borrow_mut().get_mut(&window_id) {
        app_window.attributes.resizable = Some(resizable);
      }
      // CEF delegate's can_resize will use this value
      // Note: CEF will automatically re-evaluate can_resize when needed
    }
    WindowMessage::SetMaximizable(maximizable) => {
      if let Some(app_window) = context.windows.borrow_mut().get_mut(&window_id) {
        app_window.attributes.maximizable = Some(maximizable);
      }
      // CEF delegate's can_maximize will use this value
      // Note: CEF will automatically re-evaluate can_maximize when needed
    }
    WindowMessage::SetMinimizable(minimizable) => {
      if let Some(app_window) = context.windows.borrow_mut().get_mut(&window_id) {
        app_window.attributes.minimizable = Some(minimizable);
      }
      // CEF delegate's can_minimize will use this value
      // Note: CEF will automatically re-evaluate can_minimize when needed
    }
    WindowMessage::SetClosable(closable) => {
      if let Some(app_window) = context.windows.borrow_mut().get_mut(&window_id) {
        app_window.attributes.closable = Some(closable);
      }
      // CEF delegate's can_close will use this value
      // Note: CEF will automatically re-evaluate can_close when needed
    }
    WindowMessage::SetTitle(title) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window
          .window
          .set_title(Some(&cef::CefString::from(title.as_str())));
      }
    }
    WindowMessage::Maximize => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.window.maximize();
      }
    }
    WindowMessage::Unmaximize => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.window.restore();
      }
    }
    WindowMessage::Minimize => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.window.minimize();
      }
    }
    WindowMessage::Unminimize => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.window.restore();
      }
    }
    WindowMessage::Show => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.window.show();
      }
    }
    WindowMessage::Hide => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.window.hide();
      }
    }
    WindowMessage::SetDecorations(decorations) => {
      if let Some(app_window) = context.windows.borrow_mut().get_mut(&window_id) {
        app_window.attributes.decorations = Some(decorations);
      }
      // CEF delegate's with_standard_window_buttons will use this value
      // Note: CEF may not support changing decorations at runtime, this updates the stored state
    }
    WindowMessage::SetShadow(_shadow) => {
      // TODO: Implement shadow
    }
    WindowMessage::SetAlwaysOnBottom(always_on_bottom) => {
      if let Some(app_window) = context.windows.borrow_mut().get_mut(&window_id) {
        app_window.attributes.always_on_bottom = Some(always_on_bottom);
      }
      // TODO: Apply always on bottom via platform-specific CEF APIs if available
    }
    WindowMessage::SetAlwaysOnTop(always_on_top) => {
      if let Some(app_window) = context.windows.borrow_mut().get_mut(&window_id) {
        app_window.attributes.always_on_top = Some(always_on_top);
      }
      // TODO: Apply always on top via platform-specific CEF APIs if available
    }
    WindowMessage::SetVisibleOnAllWorkspaces(visible_on_all_workspaces) => {
      if let Some(app_window) = context.windows.borrow_mut().get_mut(&window_id) {
        app_window.attributes.visible_on_all_workspaces = Some(visible_on_all_workspaces);
      }
      // TODO: Apply visible on all workspaces via platform-specific CEF APIs if available
    }
    WindowMessage::SetContentProtected(protected) => {
      if let Some(app_window) = context.windows.borrow_mut().get_mut(&window_id) {
        app_window.attributes.content_protected = Some(protected);
      }
      // TODO: Apply content protection via platform-specific CEF APIs if available
    }
    WindowMessage::SetSize(size) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        if let Some(display) = app_window.window.display() {
          let device_scale_factor = display.device_scale_factor() as f64;
          let physical_size = size.to_physical::<u32>(device_scale_factor);
          let current_bounds = app_window.window.bounds();
          app_window.window.set_bounds(Some(&cef::Rect {
            x: current_bounds.x,
            y: current_bounds.y,
            width: physical_size.width as i32,
            height: physical_size.height as i32,
          }));
        }
      }
    }
    WindowMessage::SetMinSize(size) => {
      if let Some(app_window) = context.windows.borrow_mut().get_mut(&window_id) {
        app_window.attributes.min_inner_size = size;
      }
      // CEF doesn't have direct min size API, but we store it for potential enforcement
    }
    WindowMessage::SetMaxSize(size) => {
      if let Some(app_window) = context.windows.borrow_mut().get_mut(&window_id) {
        app_window.attributes.max_inner_size = size;
      }
      // CEF doesn't have direct max size API, but we store it for potential enforcement
    }
    WindowMessage::SetSizeConstraints(constraints) => {
      if let Some(app_window) = context.windows.borrow_mut().get_mut(&window_id) {
        app_window.attributes.inner_size_constraints = Some(constraints);
      }
      // CEF doesn't have direct size constraints API, but we store it for potential enforcement
    }
    WindowMessage::SetPosition(position) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        if let Some(display) = app_window.window.display() {
          let device_scale_factor = display.device_scale_factor() as f64;
          let physical_position = position.to_physical::<i32>(device_scale_factor);
          let current_bounds = app_window.window.bounds();
          app_window.window.set_bounds(Some(&cef::Rect {
            x: physical_position.x,
            y: physical_position.y,
            width: current_bounds.width,
            height: current_bounds.height,
          }));
        }
      }
    }
    WindowMessage::SetFullscreen(fullscreen) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window
          .window
          .set_fullscreen(if fullscreen { 1 } else { 0 });
      }
    }
    #[cfg(target_os = "macos")]
    WindowMessage::SetSimpleFullscreen(_fullscreen) => {
      // TODO: Implement simple fullscreen
    }
    WindowMessage::SetFocus => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.window.show();
        // Focus is typically set when window is shown
      }
    }
    WindowMessage::SetFocusable(_focusable) => {
      // TODO: Implement focusable
    }
    WindowMessage::SetIcon(_icon) => {
      // TODO: Implement icon
    }
    WindowMessage::SetSkipTaskbar(_skip) => {
      // TODO: Implement skip taskbar
    }
    WindowMessage::SetCursorGrab(_grab) => {
      // TODO: Implement cursor grab
    }
    WindowMessage::SetCursorVisible(_visible) => {
      // TODO: Implement cursor visible
    }
    WindowMessage::SetCursorIcon(_icon) => {
      // TODO: Implement cursor icon
    }
    WindowMessage::SetCursorPosition(_position) => {
      // TODO: Implement cursor position
    }
    WindowMessage::SetIgnoreCursorEvents(_ignore) => {
      // TODO: Implement ignore cursor events
    }
    WindowMessage::SetProgressBar(_progress_state) => {
      // TODO: Implement progress bar
    }
    WindowMessage::SetBadgeCount(_count, _desktop_filename) => {
      // TODO: Implement badge count
    }
    WindowMessage::SetBadgeLabel(_label) => {
      // TODO: Implement badge label
    }
    WindowMessage::SetOverlayIcon(_icon) => {
      // TODO: Implement overlay icon
    }
    WindowMessage::SetTitleBarStyle(_style) => {
      // TODO: Implement title bar style
    }
    WindowMessage::SetTrafficLightPosition(_position) => {
      // TODO: Implement traffic light position
    }
    WindowMessage::SetTheme(_theme) => {
      // TODO: Implement theme
    }
    WindowMessage::SetBackgroundColor(_color) => {
      // TODO: Implement background color
    }
    WindowMessage::StartDragging => {
      // TODO: Implement start dragging
    }
    WindowMessage::StartResizeDragging(_direction) => {
      // TODO: Implement start resize dragging
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
    Message::Window { window_id, message } => {
      handle_window_message(context, window_id, message);
    }
    Message::Webview {
      window_id,
      webview_id,
      message,
    } => handle_webview_message(context, window_id, webview_id, message),
    Message::RequestExit(code) => {
      let (tx, rx) = channel();
      (context.callback.borrow_mut())(RunEvent::ExitRequested {
        code: Some(code),
        tx,
      });

      let recv = rx.try_recv();
      let should_prevent = matches!(recv, Ok(ExitRequestedEventAction::Prevent));

      if !should_prevent {
        cef::quit_message_loop();
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
  webview_id: u32,
  pending: PendingWindow<T, CefRuntime<T>>,
) {
  let label = pending.label.clone();
  let attributes = pending.window_builder;

  let mut delegate =
    AppWindowDelegate::<T>::new(window_id, context.callback.clone(), context.windows.clone());

  let window = window_create_top_level(Some(&mut delegate)).expect("Failed to create window");

  if let Some(title) = &attributes.title {
    window.set_title(Some(&CefString::from(title.as_str())));
  }

  if let Some(inner_size) = &attributes.inner_size {
    if let Some(display) = window.display() {
      let device_scale_factor = display.device_scale_factor() as f64;
      let physical_size = inner_size.to_physical::<u32>(device_scale_factor);
      window.set_bounds(Some(&cef::Rect {
        x: window.bounds().x,
        y: window.bounds().y,
        width: physical_size.width as i32,
        height: physical_size.height as i32,
      }));
    }
  }

  if let Some(position) = &attributes.position {
    if let Some(display) = window.display() {
      let device_scale_factor = display.device_scale_factor() as f64;
      let physical_position = position.to_physical::<i32>(device_scale_factor);
      let current_bounds = window.bounds();
      window.set_bounds(Some(&cef::Rect {
        x: physical_position.x,
        y: physical_position.y,
        width: current_bounds.width,
        height: current_bounds.height,
      }));
    }
  }

  if attributes.center {
    // Center window - calculate center position from display size
    if let Some(display) = window.display() {
      let work_area = display.work_area();
      let current_bounds = window.bounds();
      let center_x = work_area.x + (work_area.width - current_bounds.width) / 2;
      let center_y = work_area.y + (work_area.height - current_bounds.height) / 2;
      window.set_bounds(Some(&cef::Rect {
        x: center_x,
        y: center_y,
        width: current_bounds.width,
        height: current_bounds.height,
      }));
    }
  }

  if attributes.visible.unwrap_or(true) {
    window.show();
  }

  if let Some(focused) = attributes.focused {
    if focused {
      // Focus is set when window is shown
    }
  }

  if let Some(maximized) = attributes.maximized {
    if maximized {
      window.maximize();
    }
  }

  if let Some(fullscreen) = attributes.fullscreen {
    if fullscreen {
      window.set_fullscreen(1);
    }
  }

  // Apply size constraints
  // CEF doesn't have direct min/max size APIs, but we store them in attributes
  // They can be enforced via delegate methods if needed in the future
  if attributes.inner_size_constraints.is_some()
    || attributes.min_inner_size.is_some()
    || attributes.max_inner_size.is_some()
  {
    // Size constraints are stored in attributes and can be checked/enforced via delegate
    // when resizing if needed
  }

  // Apply min/max size if set directly
  if let Some(min_size) = &attributes.min_inner_size {
    if let Some(display) = window.display() {
      let device_scale_factor = display.device_scale_factor() as f64;
      let _physical_min_size = min_size.to_physical::<u32>(device_scale_factor);
      // TODO: Apply min size constraint
    }
  }

  if let Some(max_size) = &attributes.max_inner_size {
    if let Some(display) = window.display() {
      let device_scale_factor = display.device_scale_factor() as f64;
      let _physical_max_size = max_size.to_physical::<u32>(device_scale_factor);
      // TODO: Apply max size constraint
    }
  }

  // Apply always_on_top and always_on_bottom
  // Note: CEF Window might not have direct APIs for these, but we store them in attributes
  // for potential platform-specific implementations or future use
  if let Some(always_on_top) = attributes.always_on_top {
    if always_on_top {
      // TODO: Implement always on top for CEF
      // This may require platform-specific implementation
    }
  }

  if let Some(always_on_bottom) = attributes.always_on_bottom {
    if always_on_bottom {
      // TODO: Implement always on bottom for CEF
      // This may require platform-specific implementation
    }
  }

  // Apply visible_on_all_workspaces
  if let Some(visible_on_all_workspaces) = attributes.visible_on_all_workspaces {
    if visible_on_all_workspaces {
      // TODO: Implement visible on all workspaces for CEF
      // This may require platform-specific implementation
    }
  }

  // Apply content_protected
  if let Some(content_protected) = attributes.content_protected {
    if content_protected {
      // TODO: Implement content protection for CEF
      // This may require platform-specific implementation
    }
  }

  // Apply skip_taskbar
  if let Some(skip_taskbar) = attributes.skip_taskbar {
    if skip_taskbar {
      // TODO: Implement skip taskbar for CEF
      // This may require platform-specific implementation
    }
  }

  // Apply shadow
  if let Some(shadow) = attributes.shadow {
    if !shadow {
      // TODO: Implement shadow control for CEF
      // This may require platform-specific implementation
    }
  }

  // Apply transparent
  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  if let Some(transparent) = attributes.transparent {
    if transparent {
      // TODO: Implement transparency for CEF
      // This may require platform-specific implementation or window initialization flags
    }
  }

  // Apply theme
  if let Some(_theme) = attributes.theme {
    // TODO: Implement theme for CEF
    // Theme handling may need to be done at window creation time
  }

  // Apply focusable
  if let Some(focusable) = attributes.focusable {
    if !focusable {
      // TODO: Implement focusable control for CEF
      // This may require platform-specific implementation
    }
  }

  context.windows.borrow_mut().insert(
    window_id,
    AppWindow {
      label,
      window,
      webviews: Vec::new(),
      content_panel: None,
      window_event_listeners: Arc::new(Mutex::new(HashMap::new())),
      webview_event_listeners: Arc::new(Mutex::new(HashMap::new())),
      attributes,
    },
  );

  if let Some(webview) = pending.webview {
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

fn send_window_event<T: UserEvent>(
  window_id: WindowId,
  windows: &Arc<RefCell<HashMap<WindowId, AppWindow>>>,
  callback: &Arc<RefCell<Box<dyn Fn(RunEvent<T>)>>>,
  event: WindowEvent,
) {
  let windows_ref = windows.borrow();
  if let Some(w) = windows_ref.get(&window_id) {
    let label = w.label.clone();
    let window_event_listeners = w.window_event_listeners.clone();

    drop(windows_ref);

    {
      let listeners = window_event_listeners.lock().unwrap();
      let handlers: Vec<_> = listeners.values().collect();
      for handler in handlers.iter() {
        handler(&event);
      }
    }

    (callback.borrow_mut())(RunEvent::WindowEvent { label, event });
  }
}

fn on_close_requested<T: UserEvent>(
  window_id: WindowId,
  windows: &Arc<RefCell<HashMap<WindowId, AppWindow>>>,
  callback: &Arc<RefCell<Box<dyn Fn(RunEvent<T>)>>>,
) {
  let (tx, rx) = channel();
  let event = WindowEvent::CloseRequested { signal_tx: tx };

  send_window_event(window_id, windows, callback, event.clone());

  let prevent = rx.try_recv().unwrap_or_default();

  if !prevent {
    on_window_close(window_id, windows);
  }
}

fn on_window_close(window_id: WindowId, windows: &Arc<RefCell<HashMap<WindowId, AppWindow>>>) {
  if let Some(window) = windows.borrow().get(&window_id) {
    window.window.close();
  }
}

fn on_window_destroyed<T: UserEvent>(
  window_id: WindowId,
  windows: &Arc<RefCell<HashMap<WindowId, AppWindow>>>,
  callback: &Arc<RefCell<Box<dyn Fn(RunEvent<T>)>>>,
) {
  let event = WindowEvent::Destroyed;
  send_window_event(window_id, windows, callback, event);

  let removed = windows.borrow_mut().remove(&window_id).is_some();

  if removed {
    let is_empty = windows.borrow().is_empty();
    if is_empty {
      let (tx, rx) = channel();
      (callback.borrow_mut())(RunEvent::ExitRequested { code: None, tx });

      let recv = rx.try_recv();
      let should_prevent = matches!(recv, Ok(ExitRequestedEventAction::Prevent));

      if !should_prevent {
        cef::quit_message_loop();
      }
    }
  }
}

fn create_webview<T: UserEvent>(
  kind: WebviewKind,
  context: &Context<T>,
  window_id: WindowId,
  webview_id: u32,
  pending: PendingWebview<T, CefRuntime<T>>,
) {
  let window = match context
    .windows
    .borrow()
    .get(&window_id)
    .map(|app_window| app_window.window.clone())
  {
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
    for (scheme, handler) in pending.uri_scheme_protocols {
      let webview_label = pending.label.clone();
      request_context.register_scheme_handler_factory(
        Some(&scheme.as_str().into()),
        None,
        Some(&mut request_handler::UriSchemeHandlerFactory::new(
          webview_label,
          Arc::new(handler) as Arc<UriSchemeProtocol>,
          initialization_scripts.clone(),
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

  let bounds = pending.webview_attributes.bounds.map(|bounds| {
    let device_scale_factor = window
      .display()
      .map(|d| d.device_scale_factor() as f64)
      .unwrap_or(1.0);
    let physical_position = bounds.position.to_physical::<i32>(device_scale_factor);
    let physical_size = bounds.size.to_physical::<u32>(device_scale_factor);
    cef::Rect {
      x: physical_position.x,
      y: physical_position.y,
      width: physical_size.width as i32,
      height: physical_size.height as i32,
    }
  });

  if kind == WebviewKind::WindowChild {
    let overlay = window
      .add_overlay_view(
        Some(&mut View::from(&browser_view)),
        cef::DockingMode::from(cef::sys::cef_docking_mode_t::CEF_DOCKING_MODE_CUSTOM),
        1,
      )
      .expect("Failed to add overlay view");

    if let Some(bounds) = &bounds {
      overlay.set_bounds(Some(bounds));
    }
    overlay.set_visible(1);

    context
      .windows
      .borrow_mut()
      .get_mut(&window_id)
      .unwrap()
      .webviews
      .push(BrowserViewWrapper {
        webview_id,
        browser_view,
        overlay: Some(overlay),
      });
  } else {
    window.add_child_view(Some(&mut View::from(&browser_view)));
    if let Some(bounds) = &bounds {
      browser_view.set_bounds(Some(bounds));
    }

    context
      .windows
      .borrow_mut()
      .get_mut(&window_id)
      .unwrap()
      .webviews
      .push(BrowserViewWrapper {
        webview_id,
        browser_view,
        overlay: None,
      });
  }
}
