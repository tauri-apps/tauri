// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use base64::Engine;
use cef::{rc::*, *};
use sha2::{Digest, Sha256};
use std::{
  cell::RefCell,
  collections::HashMap,
  sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
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

mod cookie;
mod request_handler;
use cookie::{CollectAllCookiesVisitor, CollectUrlCookiesVisitor};

#[inline]
fn color_to_cef_argb(color: tauri_utils::config::Color) -> u32 {
  let (r, g, b, a) = color.into();
  ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[inline]
fn color_opt_to_cef_argb(color: Option<tauri_utils::config::Color>) -> u32 {
  color.map(color_to_cef_argb).unwrap_or(0xFFFFFFFF)
}

/// Convert a CEF Display to a tauri Monitor
pub(crate) fn display_to_monitor(display: &cef::Display) -> tauri_runtime::monitor::Monitor {
  let bounds = display.bounds();
  let work = display.work_area();
  tauri_runtime::monitor::Monitor {
    name: None,
    size: PhysicalSize::new(bounds.width as u32, bounds.height as u32),
    position: PhysicalPosition::new(bounds.x, bounds.y),
    work_area: tauri_runtime::dpi::PhysicalRect {
      position: PhysicalPosition::new(work.x, work.y),
      size: PhysicalSize::new(work.width as u32, work.height as u32),
    },
    scale_factor: display.device_scale_factor() as f64,
  }
}

/// Get the primary monitor
pub(crate) fn get_primary_monitor() -> Option<tauri_runtime::monitor::Monitor> {
  cef::display_get_primary().map(|d| display_to_monitor(&d))
}

/// Get the monitor from a point
pub(crate) fn get_monitor_from_point(x: f64, y: f64) -> Option<tauri_runtime::monitor::Monitor> {
  let rect = cef::Rect {
    x: x as i32,
    y: y as i32,
    width: 1,
    height: 1,
  };
  cef::display_get_matching_bounds(Some(&rect), 1).map(|d| display_to_monitor(&d))
}

/// Get all available monitors
pub(crate) fn get_available_monitors() -> Vec<tauri_runtime::monitor::Monitor> {
  let mut displays: Vec<Option<cef::Display>> = vec![None; cef::display_get_count()];
  cef::display_get_alls(Some(&mut displays));
  displays
    .into_iter()
    .flatten()
    .map(|d| display_to_monitor(&d))
    .collect()
}

/// Convert tauri Icon to CEF Image
fn icon_to_cef_image(icon: tauri_runtime::Icon<'static>) -> Option<cef::Image> {
  let rgba = icon.rgba.to_vec();
  let width = icon.width;
  let height = icon.height;

  // Create a CEF Image
  let image = cef::image_create()?;

  // Add bitmap data to the image
  // RGBA_8888 color type, OPAQUE alpha type (for icons without transparency, or use PREMULTIPLIED for transparency)
  use cef_dll_sys::cef_alpha_type_t;
  let result = image.add_bitmap(
    1.0, // scale_factor
    width as i32,
    height as i32,
    cef::ColorType::default(), // RGBA_8888
    cef::AlphaType::from(cef_alpha_type_t::CEF_ALPHA_TYPE_PREMULTIPLIED), // Use premultiplied for RGBA with alpha
    Some(&rgba),
  );

  if result == 1 {
    Some(image)
  } else {
    None
  }
}

/// Set window icon using CEF native API
fn set_window_icon(window: &cef::Window, icon: tauri_runtime::Icon<'static>) {
  if let Some(mut cef_image) = icon_to_cef_image(icon) {
    window.set_window_icon(Some(&mut cef_image));
  }
}

/// Set overlay icon using CEF native API (set_window_app_icon)
fn set_overlay_icon(window: &cef::Window, icon: Option<tauri_runtime::Icon<'static>>) {
  match icon {
    Some(icon_data) => {
      if let Some(mut cef_image) = icon_to_cef_image(icon_data) {
        window.set_window_app_icon(Some(&mut cef_image));
      }
    }
    None => {
      window.set_window_app_icon(None);
    }
  }
}

#[inline]
fn apply_content_protection(window: &cef::Window, protected: bool) {
  #[cfg(target_os = "linux")]
  {
    let _ = (window, protected);
  }
  #[cfg(windows)]
  {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
      SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
    };
    let hwnd = window.window_handle() as isize;
    unsafe {
      let _ = SetWindowDisplayAffinity(
        HWND(hwnd),
        if protected {
          WDA_EXCLUDEFROMCAPTURE
        } else {
          WDA_NONE
        },
      );
    }
  }

  #[cfg(target_os = "macos")]
  {
    // Set NSWindow sharing type to NSWindowSharingNone/NSWindowSharingReadOnly
    // Safety: must be called on main thread; CEF window APIs run on main thread.
    unsafe {
      use objc2::rc::Retained;
      use objc2_app_kit::{NSView, NSWindow, NSWindowSharingType};
      let ns_view_ptr = window.window_handle() as *mut NSView;
      if let Some(ns_view_ref) = ns_view_ptr.as_ref() {
        if let Ok(ns_view) = Retained::retain(ns_view_ref) {
          if let Some(ns_window) = ns_view.window() {
            let sharing = if protected {
              NSWindowSharingType::None
            } else {
              NSWindowSharingType::ReadOnly
            };
            ns_window.setSharingType(sharing);
          }
        }
      }
    }
  }
}

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
      (self.context.callback.borrow())(RunEvent::Ready);
    }
  }
}

wrap_load_handler! {
  struct BrowserLoadHandler {
    initialization_scripts: Vec<CefInitScript>,
    on_page_load_handler: Option<Arc<tauri_runtime::webview::OnPageLoadHandler>>,
  }

  impl LoadHandler {
    fn on_load_start(
      &self,
      _browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      _transition_type: TransitionType,
    ) {
      let Some(handler) = &self.on_page_load_handler else { return };
      let Some(frame) = frame else { return };

      let is_main_frame = frame.is_main() == 1;
      if !is_main_frame {
        return;
      }

      let url = frame.url();
      let url_str = cef::CefString::from(&url).to_string();
      if let Ok(url) = url::Url::parse(&url_str) {
        handler(url, tauri_runtime::webview::PageLoadEvent::Started);
      }
    }

    fn on_load_end(
      &self,
      _browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      http_status_code: ::std::os::raw::c_int,
    ) {
      let Some(frame) = frame else { return };

      if let Some(handler) = &self.on_page_load_handler {
        if frame.is_main() == 1 {
          let url = frame.url();
          let url_str = cef::CefString::from(&url).to_string();
          if let Ok(url) = url::Url::parse(&url_str) {
            handler(url, tauri_runtime::webview::PageLoadEvent::Finished);
          }
        }
      }

      // run init scripts for http/https pages
      // custom schemes are handled by the request handler
      // where we inject scripts directly in the html

      if http_status_code < 200 || http_status_code >= 300 {
        return;
      }

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
    on_page_load_handler: Option<Arc<tauri_runtime::webview::OnPageLoadHandler>>,
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
        self.on_page_load_handler.clone(),
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
    attributes: RefCell<crate::CefWindowBuilder>,
    force_close: Arc<AtomicBool>,
    windows: Arc<RefCell<HashMap<WindowId, AppWindow>>>,
  }

  impl ViewDelegate {
    fn minimum_size(&self, view: Option<&mut View>) -> cef::Size {
      let window = view.and_then(|v| v.window());
      let scale = window
        .and_then(|w| w.display())
        .map(|d| d.device_scale_factor() as f64)
        .unwrap_or(1.0);

      let attributes = self.attributes.borrow();
      let mut min_w: i32 = 0;
      let mut min_h: i32 = 0;

      if let Some(min_size) = attributes.min_inner_size {
        let physical = min_size.to_physical::<u32>(scale);
        min_w = min_w.max(physical.width as i32);
        min_h = min_h.max(physical.height as i32);
      }

      if let Some(constraints) = attributes.inner_size_constraints.as_ref() {
        if let Some(w) = constraints.min_width {
          let w_ph = i32::from(w.to_physical::<u32>(scale));
          min_w = min_w.max(w_ph);
        }
        if let Some(h) = constraints.min_height {
          let h_ph = i32::from(h.to_physical::<u32>(scale));
          min_h = min_h.max(h_ph);
        }
      }

      if min_w != 0 || min_h != 0 {
        cef::Size { width: min_w, height: min_h }
      } else {
        cef::Size { width: 0, height: 0 }
      }
    }

    fn maximum_size(&self, view: Option<&mut View>) -> cef::Size {
      let window = view.and_then(|v| v.window());
      let scale = window
        .and_then(|w| w.display())
        .map(|d| d.device_scale_factor() as f64)
        .unwrap_or(1.0);

      let attributes = self.attributes.borrow();
      let mut max_w: Option<i32> = None;
      let mut max_h: Option<i32> = None;

      if let Some(max_size) = attributes.max_inner_size {
        let physical = max_size.to_physical::<u32>(scale);
        max_w = Some(physical.width as i32);
        max_h = Some(physical.height as i32);
      }

      if let Some(constraints) = attributes.inner_size_constraints.as_ref() {
        if let Some(w) = constraints.max_width {
          let w_ph = i32::from(w.to_physical::<u32>(scale));
          max_w = Some(match max_w { Some(v) => v.min(w_ph), None => w_ph });
        }
        if let Some(h) = constraints.max_height {
          let h_ph = i32::from(h.to_physical::<u32>(scale));
          max_h = Some(match max_h { Some(v) => v.min(h_ph), None => h_ph });
        }
      }

      if max_w.is_some() || max_h.is_some() {
        cef::Size {
          width: max_w.unwrap_or(0),
          height: max_h.unwrap_or(0),
        }
      } else {
        cef::Size { width: 0, height: 0 }
      }
    }
  }

  impl PanelDelegate {}

  impl WindowDelegate {
    fn is_frameless(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      // Map `decorations: false` to frameless window
      let attributes = self.attributes.borrow();
      (!attributes.decorations.unwrap_or(true)) as i32
    }

    fn on_window_destroyed(&self, _window: Option<&mut Window>) {
      on_window_destroyed(self.window_id, &self.windows, &self.callback);
    }

    fn can_resize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      let attributes = self.attributes.borrow();
      attributes.resizable.unwrap_or(true) as i32
    }

    fn can_maximize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      // Can maximize if maximizable is true and resizable is true (or not set, defaulting to true)
      let attributes = self.attributes.borrow();
      let resizable = attributes.resizable.unwrap_or(true);
      let maximizable = attributes.maximizable.unwrap_or(true);
      (resizable && maximizable) as i32
    }

    fn can_minimize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      let attributes = self.attributes.borrow();
      attributes.minimizable.unwrap_or(true) as i32
    }

    fn can_close(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      if self.force_close.load(Ordering::SeqCst) {
        return 1;
      }

      let attributes = self.attributes.borrow();
      let closable = attributes.closable.unwrap_or(true);

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
      let color_value = color_opt_to_cef_argb(color);
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
    WebviewMessage::CookiesForUrl(url, tx) => {
      // Collect cookies for a specific URL
      let url_str = url.as_str().to_string();

      cef::cookie_manager_get_global_manager(None)
        .map(|manager| {
          let collected: Arc<Mutex<Vec<tauri_runtime::Cookie<'static>>>> =
            Arc::new(Mutex::new(Vec::new()));
          let tx_ = tx.clone();

          let mut visitor = CollectUrlCookiesVisitor::new(tx_, collected.clone());
          let url_cef = cef::CefString::from(url_str.as_str());
          manager.visit_url_cookies(Some(&url_cef), 1, Some(&mut visitor));
        })
        .or_else(|| {
          let _ = tx.send(Ok(Vec::new()));
          None
        });
    }
    WebviewMessage::Cookies(tx) => {
      // Collect all cookies
      cef::cookie_manager_get_global_manager(None)
        .map(|manager| {
          let collected: Arc<Mutex<Vec<tauri_runtime::Cookie<'static>>>> =
            Arc::new(Mutex::new(Vec::new()));
          let tx_ = tx.clone();

          let mut visitor = CollectAllCookiesVisitor::new(tx_, collected.clone());
          manager.visit_all_cookies(Some(&mut visitor));
        })
        .or_else(|| {
          let _ = tx.send(Ok(Vec::new()));
          None
        });
    }
    WebviewMessage::SetCookie(cookie) => {
      if let Some(manager) = cef::cookie_manager_get_global_manager(None) {
        // Try to infer a URL for the cookie scope using the currently loaded URL
        let url = get_browser_view(context, window_id, webview_id)
          .and_then(|bv| bv.browser())
          .and_then(|b| b.main_frame())
          .map(|frame| cef::CefString::from(&frame.url()).to_string())
          .unwrap_or_default();

        let mut cef_cookie = cef::Cookie::default();
        cef_cookie.name = cef::CefString::from(cookie.name());
        cef_cookie.value = cef::CefString::from(cookie.value());
        if let Some(d) = cookie.domain() {
          cef_cookie.domain = cef::CefString::from(d);
        }
        if let Some(p) = cookie.path() {
          cef_cookie.path = cef::CefString::from(p);
        }
        if cookie.secure().unwrap_or(false) {
          cef_cookie.secure = 1;
        }
        if cookie.http_only().unwrap_or(false) {
          cef_cookie.httponly = 1;
        }

        let url_cef = if url.is_empty() {
          None
        } else {
          Some(cef::CefString::from(url.as_str()))
        };
        manager.set_cookie(
          url_cef.as_ref(),
          Some(&cef_cookie),
          Option::<&mut cef::SetCookieCallback>::None,
        );
      }
    }
    WebviewMessage::DeleteCookie(cookie) => {
      if let Some(manager) = cef::cookie_manager_get_global_manager(None) {
        // Resolve current URL for targeted deletion
        let url = get_browser_view(context, window_id, webview_id)
          .and_then(|bv| bv.browser())
          .and_then(|b| b.main_frame())
          .map(|frame| cef::CefString::from(&frame.url()).to_string())
          .unwrap_or_default();
        let url_cef = if url.is_empty() {
          None
        } else {
          Some(cef::CefString::from(url.as_str()))
        };
        let name_cef = Some(cef::CefString::from(cookie.name()));
        manager.delete_cookies(
          url_cef.as_ref(),
          name_cef.as_ref(),
          Option::<&mut cef::DeleteCookiesCallback>::None,
        );
      }
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
    WindowMessage::IsDecorated(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| Ok(w.attributes.borrow().decorations.unwrap_or(true)))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::IsResizable(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| Ok(w.attributes.borrow().resizable.unwrap_or(true)))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::IsMaximizable(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| Ok(w.attributes.borrow().maximizable.unwrap_or(true)))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::IsMinimizable(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| Ok(w.attributes.borrow().minimizable.unwrap_or(true)))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::IsClosable(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| Ok(w.attributes.borrow().closable.unwrap_or(true)))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
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
    WindowMessage::CurrentMonitor(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| {
          let b = w.window.bounds();
          cef::display_get_matching_bounds(Some(&b), 1).map(|d| {
            let bounds = d.bounds();
            let work = d.work_area();
            tauri_runtime::monitor::Monitor {
              name: None,
              size: PhysicalSize::new(bounds.width as u32, bounds.height as u32),
              position: PhysicalPosition::new(bounds.x, bounds.y),
              work_area: tauri_runtime::dpi::PhysicalRect {
                position: PhysicalPosition::new(work.x, work.y),
                size: PhysicalSize::new(work.width as u32, work.height as u32),
              },
              scale_factor: d.device_scale_factor() as f64,
            }
          })
        })
        .map(|opt| Ok(opt))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::PrimaryMonitor(tx) => {
      let result = Ok(get_primary_monitor());
      let _ = tx.send(result);
    }
    WindowMessage::MonitorFromPoint(tx, x, y) => {
      let result = Ok(get_monitor_from_point(x, y));
      let _ = tx.send(result);
    }
    WindowMessage::AvailableMonitors(tx) => {
      let monitors = get_available_monitors();
      let _ = tx.send(Ok(monitors));
    }
    WindowMessage::Theme(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| {
          Ok(
            w.attributes
              .borrow()
              .theme
              .unwrap_or(tauri_utils::Theme::Light),
          )
        })
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::IsEnabled(tx) => {
      let _ = tx.send(Ok(true));
    }
    WindowMessage::IsAlwaysOnTop(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| Ok(w.window.is_always_on_top() == 1))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::RawWindowHandle(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| {
          #[cfg(target_os = "linux")]
          unsafe {
            let xid = w.window.window_handle() as u64;
            Ok(raw_window_handle::WindowHandle::borrow_raw(
              raw_window_handle::RawWindowHandle::Xlib(raw_window_handle::XlibWindowHandle::new(
                xid,
              )),
            ))
          }

          #[cfg(target_os = "macos")]
          unsafe {
            let ns_view = w.window.window_handle() as *mut std::ffi::c_void;
            if let Some(nn) = std::ptr::NonNull::new(ns_view) {
              Ok(raw_window_handle::WindowHandle::borrow_raw(
                raw_window_handle::RawWindowHandle::AppKit(
                  raw_window_handle::AppKitWindowHandle::new(nn),
                ),
              ))
            } else {
              Err(raw_window_handle::HandleError::Unavailable)
            }
          }

          #[cfg(windows)]
          unsafe {
            let hwnd = w.window.window_handle() as isize;
            if let Some(nz) = std::num::NonZeroIsize::new(hwnd) {
              Ok(raw_window_handle::WindowHandle::borrow_raw(
                raw_window_handle::RawWindowHandle::Win32(
                  raw_window_handle::Win32WindowHandle::new(nz),
                ),
              ))
            } else {
              Err(raw_window_handle::HandleError::Unavailable)
            }
          }
        })
        .unwrap_or_else(|| Err(raw_window_handle::HandleError::Unavailable));
      let _ = tx.send(result);
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
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().resizable = Some(resizable);
      }
    }
    WindowMessage::SetMaximizable(maximizable) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().maximizable = Some(maximizable);
      }
    }
    WindowMessage::SetMinimizable(minimizable) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().minimizable = Some(minimizable);
      }
    }
    WindowMessage::SetClosable(closable) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().closable = Some(closable);
      }
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
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().decorations = Some(decorations);
      }
    }
    WindowMessage::SetShadow(_shadow) => {
      // TODO: Implement shadow
    }
    WindowMessage::SetAlwaysOnBottom(always_on_bottom) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().always_on_bottom = Some(always_on_bottom);
      }
      // TODO: Apply always on bottom via platform-specific CEF APIs if available
    }
    WindowMessage::SetAlwaysOnTop(always_on_top) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().always_on_top = Some(always_on_top);
        app_window
          .window
          .set_always_on_top(if always_on_top { 1 } else { 0 });
      }
    }
    WindowMessage::SetVisibleOnAllWorkspaces(visible_on_all_workspaces) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().visible_on_all_workspaces =
          Some(visible_on_all_workspaces);
      }
      // TODO: Apply visible on all workspaces via platform-specific CEF APIs if available
    }
    WindowMessage::SetContentProtected(protected) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().content_protected = Some(protected);
        apply_content_protection(&app_window.window, protected);
      }
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
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().min_inner_size = size;
      }
    }
    WindowMessage::SetMaxSize(size) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().max_inner_size = size;
      }
    }
    WindowMessage::SetSizeConstraints(constraints) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().inner_size_constraints = Some(constraints);
      }
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
        app_window.window.request_focus();
      }
    }
    WindowMessage::SetFocusable(focusable) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window
          .window
          .set_focusable(if focusable { 1 } else { 0 });
      }
    }
    WindowMessage::SetIcon(icon) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        set_window_icon(&app_window.window, icon);
      }
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
    WindowMessage::SetOverlayIcon(icon) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        set_overlay_icon(&app_window.window, icon);
      }
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
    WindowMessage::SetBackgroundColor(color) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().background_color = color;
        let color_value = color_opt_to_cef_argb(color);
        app_window.window.set_background_color(color_value);
      }
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
      (context.callback.borrow())(RunEvent::ExitRequested {
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
      (context.callback.borrow())(RunEvent::UserEvent(evt));
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
  let window_builder = pending.window_builder;
  let attributes = RefCell::new(window_builder.clone());
  let force_close = Arc::new(AtomicBool::new(false));

  let mut delegate = AppWindowDelegate::<T>::new(
    window_id,
    context.callback.clone(),
    attributes.clone(),
    force_close.clone(),
    context.windows.clone(),
  );

  let window = window_create_top_level(Some(&mut delegate)).expect("Failed to create window");

  if let Some(icon) = window_builder.icon.clone() {
    set_window_icon(&window, icon);
  }

  if let Some(title) = &window_builder.title {
    window.set_title(Some(&CefString::from(title.as_str())));
  }

  if let Some(inner_size) = &window_builder.inner_size {
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

  if let Some(position) = &window_builder.position {
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

  if window_builder.center {
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

  if let Some(focused) = window_builder.focused {
    if focused {
      window.request_focus();
    }
  }

  if let Some(maximized) = window_builder.maximized {
    if maximized {
      window.maximize();
    }
  }

  if let Some(fullscreen) = window_builder.fullscreen {
    if fullscreen {
      window.set_fullscreen(1);
    }
  }

  if let Some(always_on_top) = window_builder.always_on_top {
    if always_on_top {
      window.set_always_on_top(1);
    }
  }

  if let Some(always_on_bottom) = window_builder.always_on_bottom {
    if always_on_bottom {
      // TODO: Implement always on bottom for CEF
    }
  }

  if let Some(visible_on_all_workspaces) = window_builder.visible_on_all_workspaces {
    if visible_on_all_workspaces {
      // TODO: Implement visible on all workspaces for CEF
    }
  }

  if let Some(content_protected) = window_builder.content_protected {
    apply_content_protection(&window, content_protected);
  }

  if let Some(skip_taskbar) = window_builder.skip_taskbar {
    if skip_taskbar {
      // TODO: Implement skip taskbar for CEF
    }
  }

  if let Some(shadow) = window_builder.shadow {
    if !shadow {
      // TODO: Implement shadow control for CEF
    }
  }

  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  if let Some(transparent) = window_builder.transparent {
    if transparent {
      // TODO: Implement transparency for CEF
    }
  }

  if let Some(_theme) = window_builder.theme {
    // TODO: Implement theme for CEF
  }

  // Apply background color
  if let Some(color) = window_builder.background_color {
    window.set_background_color(color_to_cef_argb(color));
  }

  // Apply focusable
  if let Some(focusable) = window_builder.focusable {
    window.set_focusable(if focusable { 1 } else { 0 });
  }

  if window_builder.visible.unwrap_or(true) {
    window.show();
  }

  context.windows.borrow_mut().insert(
    window_id,
    AppWindow {
      label,
      window,
      force_close,
      webviews: Vec::new(),
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

    (callback.borrow())(RunEvent::WindowEvent { label, event });
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
  let window = windows.borrow().get(&window_id).map(|w| {
    w.force_close.store(true, Ordering::SeqCst);
    w.window.clone()
  });
  if let Some(window) = window {
    window.close();
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
      (callback.borrow())(RunEvent::ExitRequested { code: None, tx });

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
  mut pending: PendingWebview<T, CefRuntime<T>>,
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

  let on_page_load_handler = pending
    .on_page_load_handler
    .take()
    .map(|h| Arc::from(h) as Arc<tauri_runtime::webview::OnPageLoadHandler>);
  let mut client = BrowserClient::new(initialization_scripts.clone(), on_page_load_handler);
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
