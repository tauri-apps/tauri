// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use base64::Engine;
use cef::{rc::*, *};
use cef_dll_sys::cef_runtime_style_t;
use dioxus_debug_cell::RefCell;
use sha2::{Digest, Sha256};
use std::{
  collections::HashMap,
  path::{Component, Path, PathBuf},
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering},
    mpsc::channel,
  },
};
use tauri_runtime::{
  ExitRequestedEventAction, RunEvent, UserEvent,
  dpi::{
    LogicalPosition, LogicalSize, PhysicalPosition, PhysicalRect, PhysicalSize, Position, Rect,
    Size,
  },
  webview::{InitializationScript, PendingWebview, UriSchemeProtocolHandler, WebviewAttributes},
  window::{DragDropEvent, PendingWindow, WebviewEvent, WindowEvent, WindowId},
};
#[cfg(target_os = "macos")]
use tauri_utils::TitleBarStyle;
use tauri_utils::html::normalize_script_for_csp;

use crate::{
  AppWebview, AppWindow, CefRuntime, CefWebviewDispatcher, CefWindowBuilder,
  DevToolsProtocolHandler, Message, RuntimeContext, RuntimeStyle as CefRuntimeStyle, Webview,
  WebviewAtribute, WebviewMessage, WindowMessage, cef_webview::CefWebview,
};

use std::cell::Cell;

// Tracks whether we're inside a user event callback. When set, `post_message`
// defers through the CEF task runner instead of executing synchronously, to
// avoid Win32 message-pump re-entrancy from APIs like ShowWindow/SetFocus
// or locking a mutex while already locked on the same thread.
thread_local! {
  static IN_EVENT_CALLBACK: Cell<bool> = const { Cell::new(false) };
}

/// Returns true if we're currently inside a user event callback.
pub fn is_in_event_callback() -> bool {
  IN_EVENT_CALLBACK.get()
}

/// Run a function within the context of an event callback, ensuring that [`is_in_event_callback`] returns true for the duration of the callback.
fn in_callback<R>(f: impl FnOnce() -> R) -> R {
  struct Guard;
  impl Drop for Guard {
    fn drop(&mut self) {
      IN_EVENT_CALLBACK.set(false);
    }
  }

  IN_EVENT_CALLBACK.set(true);
  let _guard = Guard;
  f()
}

mod cookie;
mod drag_window;
pub mod request_handler;

use cookie::{CollectAllCookiesVisitor, CollectUrlCookiesVisitor};

#[cfg(target_os = "linux")]
type CefOsEvent<'a> = Option<&'a mut sys::XEvent>;
#[cfg(target_os = "macos")]
type CefOsEvent<'a> = *mut u8;
#[cfg(windows)]
type CefOsEvent<'a> = Option<&'a mut sys::MSG>;
type AddressChangedHandler = dyn Fn(&url::Url) + Send + Sync;
type IpcHandler<T> =
  dyn Fn(tauri_runtime::webview::DetachedWebview<T, CefRuntime<T>>, http::Request<String>) + Send;
type PendingInitialLoad = (Browser, String);
type PendingInitialLoads = Arc<Mutex<HashMap<i32, PendingInitialLoad>>>;

const DRAG_DROP_BRIDGE_PATH: &str = "/__tauri_cef_drag_drop__";
const IPC_MESSAGE_NAME: &str = "tauri:ipc";
const IPC_POST_MESSAGE_FUNCTION: &str = "postMessage";
const INITIAL_LOAD_URL: &str = concat!(
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
static NEXT_INIT_SCRIPT_DEVTOOLS_MESSAGE_ID: AtomicI32 = AtomicI32::new(1_000_000);
const DRAG_DROP_INIT_SCRIPT: &str = r#"
(() => {
  if (window.__TAURI_CEF_DRAG_DROP__) {
    return;
  }

  Object.defineProperty(window, "__TAURI_CEF_DRAG_DROP__", {
    value: true,
    configurable: false,
  });

  const PATH = "/__tauri_cef_drag_drop__";
  let entered = false;

  const position = (event) => ({
    x: event.clientX * window.devicePixelRatio,
    y: event.clientY * window.devicePixelRatio,
  });

  const send = (type, event) => {
    const pos = position(event);
    const url = new URL(PATH, window.location.href);
    url.searchParams.set("payload", JSON.stringify({ type, x: pos.x, y: pos.y }));
    fetch(url.href, {
      method: "GET",
      cache: "no-store",
      credentials: "omit",
    }).catch(() => {});
  };

  const listen = (eventName, handler) => {
    window.addEventListener(eventName, handler, { capture: true });
  };

  listen("dragenter", (event) => {
    if (!entered) {
      entered = true;
      send("enter", event);
    }
  });

  listen("dragover", (event) => {
    if (!entered) {
      entered = true;
      send("enter", event);
    }
    send("over", event);
  });

  listen("drop", (event) => {
    if (!entered) {
      send("enter", event);
    }
    entered = false;
    send("drop", event);
  });

  listen("dragleave", (event) => {
    const x = event.clientX;
    const y = event.clientY;
    if (entered && (x <= 0 || y <= 0 || x >= window.innerWidth || y >= window.innerHeight)) {
      entered = false;
      send("leave", event);
    }
  });
})();
"#;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DragDropEventTarget {
  Window,
  Webview,
}

#[derive(Default)]
struct DragDropState {
  paths: Option<Vec<PathBuf>>,
  native_entered: bool,
  entered: bool,
}

#[derive(Clone, serde::Deserialize)]
struct DragDropScriptEvent {
  #[serde(rename = "type")]
  kind: String,
  x: f64,
  y: f64,
}

/// CEF transparent color value (ARGB)
#[allow(dead_code)]
const TRANSPARENT: u32 = 0x00000000;

#[inline]
fn color_to_cef_argb(color: tauri_utils::config::Color) -> u32 {
  let (r, g, b, a) = color.into();
  ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Convert position to the coordinate space expected by CEF.
/// On Windows, CEF uses physical coordinates; on other platforms, logical.
#[inline]
fn position_to_cef(position: Position, scale_factor: f64) -> cef::Point {
  #[cfg(windows)]
  let p = position.to_physical::<i32>(scale_factor);
  #[cfg(not(windows))]
  let p = position.to_logical::<i32>(scale_factor);
  cef::Point { x: p.x, y: p.y }
}

/// Convert size to the coordinate space expected by CEF.
/// On Windows, CEF uses physical coordinates; on other platforms, logical.
#[inline]
fn size_to_cef(size: Size, scale_factor: f64) -> cef::Size {
  #[cfg(windows)]
  let s = size.to_physical::<i32>(scale_factor);
  #[cfg(not(windows))]
  let s = size.to_logical::<i32>(scale_factor);
  cef::Size {
    width: s.width,
    height: s.height,
  }
}

/// Convert rect to the coordinate space expected by CEF.
/// On Windows, CEF uses physical coordinates; on other platforms, logical.
#[inline]
fn rect_to_cef(rect: Rect, scale_factor: f64) -> cef::Rect {
  let p = position_to_cef(rect.position, scale_factor);
  let s = size_to_cef(rect.size, scale_factor);
  cef::Rect {
    x: p.x,
    y: p.y,
    width: s.width,
    height: s.height,
  }
}

#[inline]
fn window_scale_factor(window: &Window) -> f64 {
  window
    .display()
    .map(|d| d.device_scale_factor() as f64)
    .unwrap_or(1.0)
}

#[inline]
fn theme_to_color_variant(theme: Option<tauri_utils::Theme>) -> ColorVariant {
  match theme {
    Some(tauri_utils::Theme::Dark) => ColorVariant::DARK,
    Some(tauri_utils::Theme::Light) => ColorVariant::LIGHT,
    _ => ColorVariant::SYSTEM,
  }
}

#[inline]
fn color_variant_to_theme(variant: ColorVariant) -> Option<tauri_utils::Theme> {
  if variant == ColorVariant::DARK {
    Some(tauri_utils::Theme::Dark)
  } else if variant == ColorVariant::LIGHT {
    Some(tauri_utils::Theme::Light)
  } else {
    None
  }
}

fn set_window_theme_scheme(app_window: &AppWindow, theme: Option<tauri_utils::Theme>) {
  let variant = theme_to_color_variant(theme);
  for webview in &app_window.webviews {
    if let Some(browser) = webview.inner.browser()
      && let Some(host) = browser.host()
      && let Some(request_context) = host.request_context()
    {
      request_context.set_chrome_color_scheme(variant, 0);
    }
  }
}

fn apply_window_theme_scheme(app_window: &AppWindow, theme: Option<tauri_utils::Theme>) {
  set_window_theme_scheme(app_window, theme);
  // Ask CEF Views to refresh themed colors immediately.
  app_window.window.theme_changed();
}

/// Applies a theme at the runtime/event-loop level, mirroring `tao`'s
/// `EventLoop::set_theme`.
///
/// Unlike the per-window [`WindowMessage::SetTheme`], this has no target window:
/// the theme is stored as the application-wide default (so windows created
/// afterwards inherit it via [`resolve_window_theme`]), pushed to every existing
/// window, and — on macOS — applied app-wide through `NSApp`'s appearance
/// (matching tao's `set_ns_theme`) rather than window by window.
pub fn set_runtime_theme<T: UserEvent>(context: &Context<T>, theme: Option<tauri_utils::Theme>) {
  *context.theme.borrow_mut() = theme;

  // macOS applies the theme at the application level, which covers the native
  // chrome of existing and future windows in a single call.
  #[cfg(target_os = "macos")]
  crate::platform::set_app_theme(theme);

  for app_window in context.windows.borrow().values() {
    app_window.attributes.borrow_mut().theme = theme;
    // Update the per-request-context Chromium color scheme and refresh the
    // native window's themed colors.
    apply_window_theme_scheme(app_window, theme);
  }
}

/// Resolves the theme to use for a window: its explicitly configured theme, or
/// otherwise the application-wide theme set via the runtime-level `set_theme`.
pub fn resolve_window_theme<T: UserEvent>(
  context: &Context<T>,
  window_theme: Option<tauri_utils::Theme>,
) -> Option<tauri_utils::Theme> {
  window_theme.or_else(|| *context.theme.borrow())
}

fn apply_request_context_theme_scheme(
  request_context: Option<&RequestContext>,
  theme: Option<tauri_utils::Theme>,
) {
  if let Some(request_context) = request_context {
    request_context.set_chrome_color_scheme(theme_to_color_variant(theme), 0);
  }
}

#[cfg(target_os = "macos")]
fn apply_macos_window_theme(window: Option<&cef::Window>, theme: Option<tauri_utils::Theme>) {
  use objc2::rc::Retained;
  use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSView,
  };

  let Some(window) = window else {
    return;
  };
  let ns_view = unsafe { Retained::<NSView>::retain(window.window_handle() as _) };
  let Some(ns_view) = ns_view else {
    return;
  };
  let Some(ns_window) = ns_view.window() else {
    return;
  };
  let appearance = match theme {
    Some(tauri_utils::Theme::Dark) => unsafe {
      NSAppearance::appearanceNamed(NSAppearanceNameDarkAqua)
    },
    Some(tauri_utils::Theme::Light) => unsafe {
      NSAppearance::appearanceNamed(NSAppearanceNameAqua)
    },
    _ => None,
  };
  unsafe { ns_window.setAppearance(appearance.as_deref()) };
}

fn native_window_theme(app_window: &AppWindow) -> Option<tauri_utils::Theme> {
  app_window.webviews.iter().find_map(|webview| {
    webview
      .inner
      .browser()
      .and_then(|browser| browser.host())
      .and_then(|host| host.request_context())
      .and_then(|request_context| {
        color_variant_to_theme(request_context.chrome_color_scheme_mode())
          .or_else(|| color_variant_to_theme(request_context.chrome_color_scheme_variant()))
      })
  })
}

/// Convert a CEF Display to a tauri Monitor
pub(crate) fn display_to_monitor(display: &cef::Display) -> tauri_runtime::monitor::Monitor {
  let bounds = display.bounds();
  let work = display.work_area();
  let scale = display.device_scale_factor() as f64;
  let physical_size =
    LogicalSize::new(bounds.width as u32, bounds.height as u32).to_physical::<u32>(scale);
  let physical_position = LogicalPosition::new(bounds.x, bounds.y).to_physical::<i32>(scale);
  let work_physical_size =
    LogicalSize::new(work.width as u32, work.height as u32).to_physical::<u32>(scale);
  let work_physical_position = LogicalPosition::new(work.x, work.y).to_physical::<i32>(scale);
  tauri_runtime::monitor::Monitor {
    name: None,
    size: PhysicalSize::new(physical_size.width, physical_size.height),
    position: PhysicalPosition::new(physical_position.x, physical_position.y),
    work_area: PhysicalRect {
      position: PhysicalPosition::new(work_physical_position.x, work_physical_position.y),
      size: PhysicalSize::new(work_physical_size.width, work_physical_size.height),
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
  use sys::cef_alpha_type_t;
  let result = image.add_bitmap(
    1.0, // scale_factor
    width as i32,
    height as i32,
    cef::ColorType::default(), // RGBA_8888
    cef::AlphaType::from(cef_alpha_type_t::CEF_ALPHA_TYPE_PREMULTIPLIED), // Use premultiplied for RGBA with alpha
    Some(&rgba),
  );

  if result == 1 { Some(image) } else { None }
}

/// Set window icon using CEF native API
fn set_window_icon(window: &cef::Window, icon: tauri_runtime::Icon<'static>) {
  if let Some(mut cef_image) = icon_to_cef_image(icon) {
    window.set_window_app_icon(Some(&mut cef_image));
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
    let hwnd = window.window_handle();
    unsafe {
      let _ = SetWindowDisplayAffinity(
        HWND(hwnd.0 as _),
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
      use objc2_app_kit::{NSView, NSWindowSharingType};
      let ns_view = Retained::<NSView>::retain(window.window_handle() as _);
      let ns_window = ns_view.as_ref().and_then(|v| v.window());
      let sharing = if protected {
        NSWindowSharingType::None
      } else {
        NSWindowSharingType::ReadOnly
      };
      if let Some(ns_window) = ns_window {
        ns_window.setSharingType(sharing);
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

fn initialization_scripts_from_webview_attributes(
  webview_attributes: &mut WebviewAttributes,
) -> Arc<Vec<CefInitScript>> {
  let mut initialization_scripts = Vec::new();

  if webview_attributes.drag_drop_handler_enabled {
    initialization_scripts.push(CefInitScript::new(InitializationScript {
      script: DRAG_DROP_INIT_SCRIPT.to_string(),
      for_main_frame_only: false,
    }));
  }

  initialization_scripts.extend(
    std::mem::take(&mut webview_attributes.initialization_scripts)
      .into_iter()
      .map(CefInitScript::new),
  );

  Arc::new(initialization_scripts)
}

fn collect_drag_data_paths(drag_data: &mut DragData) -> Vec<PathBuf> {
  let mut paths = CefStringList::new();
  if drag_data.file_paths(Some(&mut paths)) != 0 {
    let paths = paths
      .into_iter()
      .filter(|path| !path.is_empty())
      .map(PathBuf::from)
      .collect::<Vec<_>>();

    if !paths.is_empty() {
      return paths;
    }
  }

  let file_name = CefStringUtf16::from(&drag_data.file_name()).to_string();
  if file_name.is_empty() {
    Vec::new()
  } else {
    vec![PathBuf::from(file_name)]
  }
}

pub type SchemeHandlerRegistry = Arc<
  Mutex<
    HashMap<
      (i32, String),
      (
        String,
        Arc<Box<tauri_runtime::webview::UriSchemeProtocolHandler>>,
        Arc<Vec<CefInitScript>>,
      ),
    >,
  >,
>;

pub type RunEventCallback<T> = Arc<RefCell<Box<dyn Fn(RunEvent<T>)>>>;

#[derive(Clone)]
pub struct Context<T: UserEvent> {
  pub windows: Arc<RefCell<HashMap<WindowId, AppWindow>>>,
  pub callback: RunEventCallback<T>,
  pub next_window_id: Arc<AtomicU32>,
  pub next_webview_id: Arc<AtomicU32>,
  pub next_window_event_id: Arc<AtomicU32>,
  pub next_webview_event_id: Arc<AtomicU32>,
  pub scheme_handler_registry: SchemeHandlerRegistry,
  /// Root cache path passed to [`cef::Settings::cache_path`] during
  /// [`cef::initialize`]. Per-webview request context cache paths must be
  /// equal to or a child of this directory.
  pub cache_path: Arc<PathBuf>,
  /// Application-wide theme set via the runtime-level `set_theme`. Mirrors
  /// `tao`'s event-loop-level theme: it is applied to every existing window and
  /// is inherited by windows created afterwards that don't specify their own
  /// theme. `None` means "follow the system theme".
  pub theme: Arc<RefCell<Option<tauri_utils::Theme>>>,
  /// Set once an `ExitRequested` has been approved and the runtime is in the
  /// asynchronous tear-down phase. While set, per-window close events
  /// (`CloseRequested`, `Destroyed`) and any further `ExitRequested`/`Exit`
  /// emissions are suppressed so the public event sequence stays at
  /// `ExitRequested -> Exit` for direct exits.
  pub is_shutting_down: Arc<AtomicBool>,
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

fn runtime_context<T: UserEvent>(context: &Context<T>) -> RuntimeContext<T> {
  RuntimeContext {
    main_thread_task_runner: cef::task_runner_get_for_current_thread().expect("null task runner"),
    main_thread_id: std::thread::current().id(),
    cef_context: context.clone(),
  }
}

wrap_app! {
  pub struct TauriApp<T: UserEvent> {
    context: Context<T>,
    custom_schemes: Vec<String>,
    deep_link_schemes: Vec<String>,
    command_line_args: Vec<(String, Option<String>)>,
  }

  impl App {
    fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
      Some(AppBrowserProcessHandler::new(
        self.context.clone(),
        self.deep_link_schemes.clone(),
      ))
    }

    fn render_process_handler(&self) -> Option<RenderProcessHandler> {
      Some(TauriRenderProcessHandler::new())
    }

    fn on_before_command_line_processing(
      &self,
      _process_type: Option<&CefString>,
      command_line: Option<&mut CommandLine>,
    ) {
      if let Some(command_line) = command_line {
        for (arg, value) in &self.command_line_args {
          if let Some(value) = value {
            command_line.append_switch_with_value(
              Some(&CefString::from(arg.as_str())),
              Some(&CefString::from(value.as_str())),
            );
          } else if arg.starts_with("-") {
            command_line.append_switch(Some(&CefString::from(arg.as_str())));
          } else {
            command_line.append_argument(Some(&CefString::from(arg.as_str())));
          }
        }
      }
    }
  }
}

wrap_browser_process_handler! {
  struct AppBrowserProcessHandler<T: UserEvent> {
    context: Context<T>,
    deep_link_schemes: Vec<String>,
  }

  impl BrowserProcessHandler {
    fn on_context_initialized(&self) {
      (self.context.callback.borrow())(RunEvent::Ready);
    }

    fn on_already_running_app_relaunch(
      &self,
      command_line: Option<&mut CommandLine>,
      _current_directory: Option<&CefString>,
    ) -> std::os::raw::c_int {
      let Some(command_line) = command_line else {
        return 0;
      };
      let mut list = CefStringList::new();
      command_line.arguments(Some(&mut list));
      let args: Vec<String> = list.into_iter().collect();
      if let Ok(url) = url::Url::parse(&args[0]) {
        let scheme = url.scheme().to_string();
        if self.deep_link_schemes.iter().any(|s| s == &scheme) {
          (self.context.callback.borrow())(RunEvent::Opened {
            urls: vec![url],
          });
          return 1;
        }
      }
      // TODO: add event
      1
    }
  }
}

wrap_v8_handler! {
  struct IpcPostMessageV8Handler;

  impl V8Handler {
    fn execute(
      &self,
      name: Option<&CefString>,
      _object: Option<&mut V8Value>,
      arguments: Option<&[Option<V8Value>]>,
      retval: Option<&mut Option<V8Value>>,
      exception: Option<&mut CefString>,
    ) -> std::os::raw::c_int {
      let Some(name) = name else {
        return 0;
      };
      if name.to_string() != IPC_POST_MESSAGE_FUNCTION {
        return 0;
      }

      let Some(message) = arguments
        .filter(|arguments| arguments.len() == 1)
        .and_then(|arguments| arguments[0].as_ref())
        .filter(|argument| argument.is_string() != 0)
      else {
        if let Some(exception) = exception {
          *exception = CefString::from("window.ipc.postMessage expects a string argument");
        }
        return 1;
      };

      let Some(context) = v8_context_get_current_context() else {
        return 1;
      };
      let Some(frame) = context.frame() else {
        return 1;
      };

      let body = CefString::from(&message.string_value()).to_string();
      let url = CefString::from(&frame.url()).to_string();
      let mut process_message = process_message_create(Some(&CefString::from(IPC_MESSAGE_NAME)));
      if let Some(args) = process_message.as_ref().and_then(ProcessMessage::argument_list) {
        args.set_string(0, Some(&CefString::from(url.as_str())));
        args.set_string(1, Some(&CefString::from(body.as_str())));
        frame.send_process_message(ProcessId::BROWSER, process_message.as_mut());
      }

      if let Some(retval) = retval {
        *retval = v8_value_create_undefined();
      }
      1
    }
  }
}

fn install_ipc_post_message(context: Option<&mut V8Context>) {
  let Some(window) = context.and_then(|context| context.global()) else {
    return;
  };

  let attributes = sys::cef_v8_propertyattribute_t(
    [
      sys::cef_v8_propertyattribute_t::V8_PROPERTY_ATTRIBUTE_READONLY,
      sys::cef_v8_propertyattribute_t::V8_PROPERTY_ATTRIBUTE_DONTENUM,
      sys::cef_v8_propertyattribute_t::V8_PROPERTY_ATTRIBUTE_DONTDELETE,
    ]
    .into_iter()
    .fold(0, |acc, attr| acc | attr.0),
  )
  .into();

  let Some(mut ipc) = v8_value_create_object(None, None) else {
    return;
  };
  let mut handler = IpcPostMessageV8Handler::new();
  let post_message_name = CefString::from(IPC_POST_MESSAGE_FUNCTION);
  let Some(mut post_message) =
    v8_value_create_function(Some(&post_message_name), Some(&mut handler))
  else {
    return;
  };

  ipc.set_value_bykey(
    Some(&post_message_name),
    Some(&mut post_message),
    attributes,
  );
  window.set_value_bykey(Some(&CefString::from("ipc")), Some(&mut ipc), attributes);
}

wrap_render_process_handler! {
  struct TauriRenderProcessHandler;

  impl RenderProcessHandler {
    fn on_context_created(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      context: Option<&mut V8Context>,
    ) {
      install_ipc_post_message(context);
    }
  }
}

wrap_app! {
  pub struct TauriRenderApp;

  impl App {
    fn render_process_handler(&self) -> Option<RenderProcessHandler> {
      Some(TauriRenderProcessHandler::new())
    }
  }
}

wrap_load_handler! {
  struct BrowserLoadHandler {
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
      _http_status_code: ::std::os::raw::c_int,
    ) {
      let Some(frame) = frame else { return };

      if let Some(handler) = &self.on_page_load_handler
        && frame.is_main() == 1 {
          let url = frame.url();
          let url_str = cef::CefString::from(&url).to_string();
          if let Ok(url) = url::Url::parse(&url_str) {
            handler(url, tauri_runtime::webview::PageLoadEvent::Finished);
          }
        }
    }
  }
}

wrap_drag_handler! {
  struct BrowserDragHandler {
    drag_drop_state: Arc<Mutex<DragDropState>>,
  }

  impl DragHandler {
    fn on_drag_enter(
      &self,
      _browser: Option<&mut Browser>,
      drag_data: Option<&mut DragData>,
      _mask: DragOperationsMask,
    ) -> ::std::os::raw::c_int {
      let mut state = self.drag_drop_state.lock().unwrap();
      state.entered = false;
      state.paths = drag_data
        .map(collect_drag_data_paths)
        .filter(|paths| !paths.is_empty());
      state.native_entered = state.paths.is_some();

      // Let Chromium continue with the drag operation so the injected script can
      // report over/drop/leave with accurate viewport positions.
      0
    }
  }
}

wrap_display_handler! {
  struct BrowserDisplayHandler {
    document_title_changed_handler: Option<Arc<tauri_runtime::webview::DocumentTitleChangedHandler>>,
    address_changed_handler: Option<Arc<AddressChangedHandler>>,
  }

  impl DisplayHandler {
    fn on_title_change(
      &self,
      _browser: Option<&mut Browser>,
      title: Option<&CefString>,
    ) {
      let Some(handler) = &self.document_title_changed_handler else { return };
      let Some(title) = title else { return };
      let title_str = title.to_string();
      handler(title_str);
    }

    fn on_address_change(
      &self,
      _browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      url: Option<&CefString>,
    ) {
      // Only fire for main frame URL changes (matches on_before_browse behavior)
      if let Some(frame) = frame
        && frame.is_main() == 0 {
          return;
        }
      let Some(handler) = &self.address_changed_handler else { return };
      let Some(url) = url else { return };
      let url_str = url.to_string();

      if url_str == INITIAL_LOAD_URL {
        return;
      }

      let Ok(parsed) = url::Url::parse(&url_str) else { return };
      handler(&parsed);
    }
  }
}

wrap_context_menu_handler! {
  struct BrowserContextMenuHandler {
    devtools_enabled: bool,
  }

  impl ContextMenuHandler {
    fn on_before_context_menu(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      _params: Option<&mut ContextMenuParams>,
      model: Option<&mut MenuModel>,
    ) {
      if !self.devtools_enabled
        && let Some(model) = model {
          model.remove_at(model.count() - 1);
        }
    }
  }
}

cef::wrap_dev_tools_message_observer! {
  struct TauriDevToolsProtocolObserver {
    handlers: Arc<Mutex<Vec<Arc<DevToolsProtocolHandler>>>>,
    pending_initial_loads: PendingInitialLoads,
  }

  impl DevToolsMessageObserver {
    fn on_dev_tools_message(
      &self,
      _browser: Option<&mut cef::Browser>,
      message: Option<&[u8]>,
    ) -> std::os::raw::c_int {
      if let Some(msg) = message {
        let protocol = crate::DevToolsProtocol::Message(msg.to_vec());
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
      if let Some((browser, initial_url)) =
        self.pending_initial_loads.lock().unwrap().remove(&message_id)
      {
        post_load_initial_url(browser, initial_url);
      }

      let protocol = crate::DevToolsProtocol::MethodResult {
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
      _browser: Option<&mut Browser>,
      method: Option<&CefString>,
      params: Option<&[u8]>,
    ) {
      let protocol = crate::DevToolsProtocol::Event {
        method: method
          .map(|m| format!("{m}"))
          .unwrap_or_default(),
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

cef::wrap_dev_tools_message_observer! {
  struct EvalScriptWithCallbackDevToolsObserver {
    message_id: Arc<AtomicI32>,
    callback: Arc<Mutex<Option<Box<dyn Fn(String) + Send + 'static>>>>,
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
      if message_id != self.message_id.load(Ordering::Relaxed) {
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
fn add_dev_tools_observer(
  browser: &cef::Browser,
  handlers: Arc<Mutex<Vec<Arc<DevToolsProtocolHandler>>>>,
  pending_initial_loads: PendingInitialLoads,
) -> Option<cef::Registration> {
  browser.host().and_then(|host| {
    let mut observer = TauriDevToolsProtocolObserver::new(handlers, pending_initial_loads);
    host.add_dev_tools_message_observer(Some(&mut observer))
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
    if init_script.script.for_main_frame_only {
      source.push_str(" && __TAURI_CEF_INIT_IS_MAIN_FRAME__");
    }
    source.push_str(") {\n");
    source.push_str(init_script.script.script.as_str());
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
) -> bool {
  let Some(source) = devtools_initialization_script_source(
    initialization_scripts,
    custom_protocol_scheme,
    custom_scheme_domain_names,
  ) else {
    return false;
  };
  let Some(host) = browser.host() else {
    return false;
  };

  let page_enable_message_id = NEXT_INIT_SCRIPT_DEVTOOLS_MESSAGE_ID.fetch_add(1, Ordering::Relaxed);
  let page_enable_message = serde_json::json!({
    "id": page_enable_message_id,
    "method": "Page.enable",
    "params": {}
  })
  .to_string();
  let _ = host.send_dev_tools_message(Some(page_enable_message.as_bytes()));

  let message_id = NEXT_INIT_SCRIPT_DEVTOOLS_MESSAGE_ID.fetch_add(1, Ordering::Relaxed);
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
    true
  } else {
    pending_initial_loads.lock().unwrap().remove(&message_id);
    false
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
fn load_initial_url_after_registering_initialization_scripts(
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

  if !is_waiting_for_initialization_scripts {
    post_load_initial_url(browser_for_callback, initial_url);
  }
}

fn load_initial_url(browser: &Browser, initial_url: &str) {
  if let Some(frame) = browser.main_frame() {
    frame.load_url(Some(&CefString::from(initial_url)));
  }
}

wrap_keyboard_handler! {
  struct BrowserKeyboardHandler {
    devtools_enabled: bool,
  }

  impl KeyboardHandler {
    fn on_pre_key_event(
      &self,
      _browser: Option<&mut Browser>,
      event: Option<&KeyEvent>,
      _os_event: CefOsEvent,
      _is_keyboard_shortcut: Option<&mut ::std::os::raw::c_int>,
    ) -> ::std::os::raw::c_int {
      // If devtools is disabled, block devtools keyboard shortcuts
      if !self.devtools_enabled {
        let Some(event) = event else { return 0; };

        // Check if this is a keydown event
        use cef::sys::cef_key_event_type_t;
        let keydown_type: cef::KeyEventType = cef_key_event_type_t::KEYEVENT_RAWKEYDOWN.into();
        if event.type_ != keydown_type {
          return 0;
        }

        // Get modifier keys
        use cef::sys::cef_event_flags_t;
        #[cfg(windows)]
        let modifiers = event.modifiers as i32;
        #[cfg(not(windows))]
        let modifiers = event.modifiers;

        #[cfg(not(target_os = "macos"))]
        let ctrl = (modifiers & (cef_event_flags_t::EVENTFLAG_CONTROL_DOWN.0)) != 0;
        #[cfg(not(target_os = "macos"))]
        let shift = (modifiers & (cef_event_flags_t::EVENTFLAG_SHIFT_DOWN.0)) != 0;

        let key_code = event.windows_key_code;

        // Block F12 (key code 123)
        if key_code == 123 {
          if let Some(is_keyboard_shortcut) = _is_keyboard_shortcut {
            *is_keyboard_shortcut = 1;
          }
          return 1;
        }

        // Block Ctrl+Shift+I (key code 73 = 'I') on Linux/Windows
        #[cfg(not(target_os = "macos"))]
        if key_code == 73 && ctrl && shift {
          if let Some(is_keyboard_shortcut) = _is_keyboard_shortcut {
            *is_keyboard_shortcut = 1;
          }
          return 1;
        }

        // Block Cmd+Opt+I on macOS
        #[cfg(target_os = "macos")]
        {
          let meta = (modifiers & cef_event_flags_t::EVENTFLAG_COMMAND_DOWN.0) != 0;
          let alt = (modifiers & cef_event_flags_t::EVENTFLAG_ALT_DOWN.0) != 0;
          if key_code == 73 && meta && alt {
            if let Some(is_keyboard_shortcut) = _is_keyboard_shortcut {
              *is_keyboard_shortcut = 1;
            }
            return 1;
          }
        }
      }

      0
    }
  }
}

wrap_permission_handler! {
  struct BrowserPermissionHandler {}

  impl PermissionHandler {
    fn on_request_media_access_permission(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      _requesting_origin: Option<&CefString>,
      requested_permissions: u32,
      callback: Option<&mut MediaAccessCallback>,
    ) -> ::std::os::raw::c_int {
      let Some(callback) = callback else {
        return 0;
      };
      // Allow microphone and camera when requested
      let allowed = requested_permissions & (sys::cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_AUDIO_CAPTURE as u32 | sys::cef_media_access_permission_types_t::CEF_MEDIA_PERMISSION_DEVICE_VIDEO_CAPTURE as u32);
      if allowed != 0 {
        callback.cont(requested_permissions);
        return 1;
      }
      0
    }

    fn on_show_permission_prompt(
      &self,
      _browser: Option<&mut Browser>,
      _prompt_id: u64,
      _requesting_origin: Option<&CefString>,
      requested_permissions: u32,
      callback: Option<&mut PermissionPromptCallback>,
    ) -> ::std::os::raw::c_int {
      let Some(callback) = callback else {
        return 0;
      };
      // Allow permission prompt (e.g. microphone/camera)
      callback.cont(PermissionRequestResult::from(
        cef::sys::cef_permission_request_result_t::CEF_PERMISSION_RESULT_ACCEPT,
      ));
      1
    }
  }
}

wrap_download_handler! {
  struct BrowserDownloadHandler {
    download_handler: Arc<tauri_runtime::webview::DownloadHandler>,
  }

  impl DownloadHandler {
    fn can_download(
      &self,
      _browser: Option<&mut Browser>,
      _url: Option<&CefStringUtf16>,
      _request_method: Option<&CefStringUtf16>,
    ) -> ::std::os::raw::c_int {
      // on_before_download is the one that actually validates the download
      // so we return 1 to allow the download here
      1
    }

    fn on_before_download(
      &self,
      _browser: Option<&mut Browser>,
      download_item: Option<&mut DownloadItem>,
      suggested_name: Option<&CefStringUtf16>,
      callback: Option<&mut BeforeDownloadCallback>,
    ) -> ::std::os::raw::c_int {
      let Some(download_item) = download_item else { return 0; };
      let Some(callback) = callback else { return 0; };

      let url_str = CefString::from(&download_item.url()).to_string();
      let Ok(url) = url::Url::parse(&url_str) else { return 0; };

      let suggested_path = suggested_name
        .map(|s| s.to_string())
        .map(std::path::PathBuf::from)
        .unwrap_or_default();

      let mut destination = suggested_path.clone();

      // Call handler with Requested event
      let should_allow = (self.download_handler)(tauri_runtime::webview::DownloadEvent::Requested {
        url: url.clone(),
        destination: &mut destination,
      });

      if should_allow {
        // Set the download path
        let destination_cef = CefStringUtf16::from(destination.to_string_lossy().as_ref());

        // if the user callback did not modify the destination, show the dialog
        let show_dialog = destination == suggested_path;
        callback.cont(Some(&destination_cef), show_dialog as ::std::os::raw::c_int);
      }
      1
    }

    fn on_download_updated(
      &self,
      _browser: Option<&mut Browser>,
      download_item: Option<&mut DownloadItem>,
      _callback: Option<&mut DownloadItemCallback>,
    ) {
      let Some(download_item) = download_item else { return; };

      // Get download URL
      let url_str = CefString::from(&download_item.url()).to_string();
      let Ok(url) = url::Url::parse(&url_str) else { return; };

      // Check download state - CEF returns i32 where 0 is false, non-zero is true
      let is_complete = download_item.is_complete() != 0;
      let is_canceled = download_item.is_canceled() != 0;
      let success = is_complete && !is_canceled;

      // Get full path if available - full_path() returns CefStringUserfreeUtf16
      let full_path = if is_complete || is_canceled {
        let path_cef = download_item.full_path();
        let path_str = CefString::from(&path_cef).to_string();
        if !path_str.is_empty() {
          Some(std::path::PathBuf::from(path_str))
        } else {
          None
        }
      } else {
        None
      };

      // Only call handler when download is finished (complete or canceled)
      if is_complete || is_canceled {
        // Call handler with Finished event
        (self.download_handler)(tauri_runtime::webview::DownloadEvent::Finished {
          url,
          path: full_path,
          success,
        });
      }
    }
  }
}

wrap_life_span_handler! {
  struct BrowserLifeSpanHandler<T: UserEvent> {
    window_id: WindowId,
    context: Context<T>,
    new_window_handler: Option<Arc<tauri_runtime::webview::NewWindowHandler<T, crate::CefRuntime<T>>>>,
    initial_url: Option<String>,
  }

  impl LifeSpanHandler {
    fn on_after_created(&self, browser: Option<&mut Browser>) {
      if let Some(browser) = browser
        && let Some(initial_url) = &self.initial_url {
          check_and_reload_if_blank(browser.clone(), initial_url.clone());
        }
    }

    fn on_before_close(&self, browser: Option<&mut Browser>) {
      let Some(browser) = browser else {
        return;
      };
      let browser_id = browser.identifier();

      let (webview, is_last_in_window) = {
        let mut windows = self.context.windows.borrow_mut();
        let Some(app_window) = windows.get_mut(&self.window_id) else {
          return;
        };
        let webview_index = app_window
          .webviews
          .iter()
          .position(|w| *w.browser_id.borrow() == browser_id);
        let Some(index) = webview_index else {
          return;
        };
        let webview = app_window.webviews.remove(index);
        let webview_id = webview.webview_id;
        app_window
          .webview_event_listeners
          .lock()
          .unwrap()
          .remove(&webview_id);
        let is_last = app_window.webviews.is_empty();
        (webview, is_last)
      };

      {
        let mut registry = self.context.scheme_handler_registry.lock().unwrap();
        let schemes: Vec<_> = webview
          .uri_scheme_protocols
          .keys()
          .cloned()
          .collect();
        for scheme in schemes {
          registry.remove(&(browser_id, scheme));
        }
      }

      // safe to drop - CEF callbacks can borrow windows
      drop(webview);

      if is_last_in_window {
          on_window_destroyed(self.window_id, &self.context);
      }
    }

    fn on_before_popup(
      &self,
      _browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      _popup_id: std::os::raw::c_int,
      target_url: Option<&CefString>,
      _target_frame_name: Option<&CefString>,
      _target_disposition: WindowOpenDisposition,
      _user_gesture: std::os::raw::c_int,
      popup_features: Option<&PopupFeatures>,
      _window_info: Option<&mut WindowInfo>,
      _client: Option<&mut Option<Client>>,
      _settings: Option<&mut BrowserSettings>,
      _extra_info: Option<&mut Option<DictionaryValue>>,
      _no_javascript_access: Option<&mut i32>,
    ) -> std::os::raw::c_int {
      let Some(handler) = &self.new_window_handler else {
        // No handler, allow default behavior
        return 0;
      };

      let Some(target_url) = target_url else {
        // No URL, deny
        return 1;
      };

      let url_str = target_url.to_string();
      let Ok(url) = url::Url::parse(&url_str) else {
        // Invalid URL, deny
        return 1;
      };

      // Extract size and position from popup_features
      // Note: PopupFeatures fields may vary by CEF version, so we handle them defensively
      let size = popup_features.and({
        // Try to access width/height fields - structure may vary
        // For now, we'll use None if we can't determine the size
        None // TODO: Implement proper PopupFeatures field access when CEF API is available
      });

      let position = popup_features.and({
        // Try to access x/y fields - structure may vary
        // For now, we'll use None if we can't determine the position
        None // TODO: Implement proper PopupFeatures field access when CEF API is available
      });

      let features = tauri_runtime::webview::NewWindowFeatures::new(
        size,
        position,
        crate::NewWindowOpener {},
      );

      let response = handler(url, features);

      match response {
        tauri_runtime::webview::NewWindowResponse::Allow => {
          // Allow CEF to handle the popup with default behavior
          0
        }
        tauri_runtime::webview::NewWindowResponse::Create { window_id: _window_id } => {
          // We need to create a window and associate it with the popup
          // For now, we'll deny the popup and let the handler create the window
          // The window creation should happen via the message system
          // This is a limitation - CEF doesn't easily support creating a window
          // and associating it with a popup in the callback
          // We return 1 to cancel the popup, and the handler should create the window
          1
        }
        tauri_runtime::webview::NewWindowResponse::Deny => {
          // Deny the popup
          1
        }
      }
    }
  }
}

wrap_client! {
  struct BrowserClient<T: UserEvent> {
    window_id: WindowId,
    webview_id: u32,
    label: String,
    drag_drop_event_target: DragDropEventTarget,
    drag_drop_handler_enabled: bool,
    drag_drop_state: Arc<Mutex<DragDropState>>,
    ipc_handler: Option<Arc<IpcHandler<T>>>,
    on_page_load_handler: Option<Arc<tauri_runtime::webview::OnPageLoadHandler>>,
    document_title_changed_handler: Option<Arc<tauri_runtime::webview::DocumentTitleChangedHandler>>,
    navigation_handler: Option<Arc<tauri_runtime::webview::NavigationHandler>>,
    address_changed_handler: Option<Arc<AddressChangedHandler>>,
    new_window_handler: Option<Arc<tauri_runtime::webview::NewWindowHandler<T, crate::CefRuntime<T>>>>,
    download_handler: Option<Arc<tauri_runtime::webview::DownloadHandler>>,
    web_content_process_terminate_handler: Option<Arc<dyn Fn() + Send>>,
    devtools_enabled: bool,
    context: Context<T>,
    runtime_context: RuntimeContext<T>,
    initial_url: Option<String>,
  }

  impl Client {
    fn drag_handler(&self) -> Option<DragHandler> {
      self
        .drag_drop_handler_enabled
        .then(|| BrowserDragHandler::new(self.drag_drop_state.clone()))
    }

    fn request_handler(&self) -> Option<RequestHandler> {
      Some(request_handler::WebRequestHandler::new(
        self.navigation_handler.clone(),
        self.context.clone(),
        self.window_id,
        self.webview_id,
        self.drag_drop_event_target,
        self.drag_drop_handler_enabled,
        self.drag_drop_state.clone(),
        self.web_content_process_terminate_handler.clone(),
      ))
    }

    fn life_span_handler(&self) -> Option<LifeSpanHandler> {
      Some(BrowserLifeSpanHandler::new(
        self.window_id,
        self.context.clone(),
        self.new_window_handler.clone(),
        self.initial_url.clone(),
      ))
    }

    fn load_handler(&self) -> Option<LoadHandler> {
      Some(BrowserLoadHandler::new(self.on_page_load_handler.clone()))
    }

    fn display_handler(&self) -> Option<DisplayHandler> {
      Some(BrowserDisplayHandler::new(
        self.document_title_changed_handler.clone(),
        self.address_changed_handler.clone(),
      ))
    }

    fn download_handler(&self) -> Option<DownloadHandler> {
      self.download_handler.clone().map(|handler| BrowserDownloadHandler::new(handler))
    }

    fn context_menu_handler(&self) -> Option<ContextMenuHandler> {
      Some(BrowserContextMenuHandler::new(self.devtools_enabled))
    }

    fn keyboard_handler(&self) -> Option<KeyboardHandler> {
      Some(BrowserKeyboardHandler::new(self.devtools_enabled))
    }

    fn permission_handler(&self) -> Option<PermissionHandler> {
      Some(BrowserPermissionHandler::new())
    }

    fn on_process_message_received(
      &self,
      _browser: Option<&mut Browser>,
      frame: Option<&mut Frame>,
      source_process: ProcessId,
      message: Option<&mut ProcessMessage>,
    ) -> std::os::raw::c_int {
      if source_process != ProcessId::RENDERER {
        return 0;
      }

      let Some(message) = message else {
        return 0;
      };
      if CefString::from(&message.name()).to_string() != IPC_MESSAGE_NAME {
        return 0;
      }

      let Some(handler) = &self.ipc_handler else {
        return 1;
      };
      let Some(args) = message.argument_list() else {
        return 1;
      };

      let mut url = CefString::from(&args.string(0)).to_string();
      if url.is_empty()
        && let Some(frame) = frame {
          url = CefString::from(&frame.url()).to_string();
        }
      let body = CefString::from(&args.string(1)).to_string();

      if let Ok(request) = http::Request::builder().uri(url).body(body) {
        handler(
          tauri_runtime::webview::DetachedWebview {
            label: self.label.clone(),
            dispatcher: CefWebviewDispatcher {
              window_id: Arc::new(Mutex::new(self.window_id)),
              webview_id: self.webview_id,
              context: self.runtime_context.clone(),
            },
          },
          request,
        );
      }

      1
    }
  }
}

wrap_browser_view_delegate! {
  struct BrowserViewDelegateImpl {
    browser_id: Arc<RefCell<i32>>,
    browser_runtime_style: CefRuntimeStyle,
    scheme_handler_registry: SchemeHandlerRegistry,
    webview_label: String,
    uri_scheme_protocols: Arc<HashMap<String, Arc<Box<tauri_runtime::webview::UriSchemeProtocolHandler>>>>,
    initialization_scripts: Arc<Vec<CefInitScript>>,
    custom_protocol_scheme: String,
    custom_scheme_domain_names: Vec<String>,
    initial_url: String,
    pending_initial_loads: PendingInitialLoads,
    devtools_protocol_handlers: Arc<Mutex<Vec<Arc<DevToolsProtocolHandler>>>>,
    devtools_observer_registration: Arc<Mutex<Option<cef::Registration>>>,
    webview_attributes: Arc<RefCell<WebviewAttributes>>,
  }

  impl ViewDelegate {
    fn on_theme_changed(&self, view: Option<&mut View>) {
      #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
      {
        let Some(view) = view else { return; };

        let webview_attributes = self.webview_attributes.borrow();

        if webview_attributes.transparent {
          view.set_background_color(TRANSPARENT);
        } else if let Some(color) = webview_attributes.background_color {
          let color = color_to_cef_argb(color);
          view.set_background_color(color);
        }
      }
    }
  }

  impl BrowserViewDelegate {
    fn on_browser_created(&self, _browser_view: Option<&mut BrowserView>, browser: Option<&mut Browser>) {
      if let Some(browser) = browser {
        let real_id = browser.identifier();
        let _ = std::mem::replace(&mut *self.browser_id.borrow_mut(), real_id);

        let mut registry = self.scheme_handler_registry.lock().unwrap();
        for (scheme, handler) in self.uri_scheme_protocols.iter() {
          registry.insert(
            (real_id, scheme.clone()),
            (
              self.webview_label.clone(),
              handler.clone(),
              self.initialization_scripts.clone(),
            ),
          );
        }
        drop(registry);

        {
          let mut devtools_observer_registration = self.devtools_observer_registration.lock().unwrap();
          if devtools_observer_registration.is_none()
            && let Some(registration) =
              add_dev_tools_observer(browser, self.devtools_protocol_handlers.clone(), self.pending_initial_loads.clone())
          {
            devtools_observer_registration.replace(registration);
          }
        }

        load_initial_url_after_registering_initialization_scripts(
          browser,
          &self.initialization_scripts,
          &self.custom_protocol_scheme,
          &self.custom_scheme_domain_names,
          &self.initial_url,
          &self.pending_initial_loads,
        );

      }
    }

    fn browser_runtime_style(&self) -> RuntimeStyle {
      use cef::sys::cef_runtime_style_t;

      match self.browser_runtime_style {
        CefRuntimeStyle::Alloy => RuntimeStyle::from(cef_runtime_style_t::CEF_RUNTIME_STYLE_ALLOY),
        CefRuntimeStyle::Chrome => RuntimeStyle::from(cef_runtime_style_t::CEF_RUNTIME_STYLE_CHROME),
      }
    }
  }
}

wrap_window_delegate! {
  struct AppWindowDelegate<T: UserEvent> {
    window_id: WindowId,
    callback: RunEventCallback<T>,
    force_close: Arc<AtomicBool>,
    windows: Arc<RefCell<HashMap<WindowId, AppWindow>>>,
    attributes: Arc<RefCell<crate::CefWindowBuilder>>,
    last_emitted_position: RefCell<PhysicalPosition<i32>>,
    last_emitted_size: RefCell<PhysicalSize<u32>>,
    last_emitted_scale_factor: RefCell<f64>,
    suppress_next_theme_changed: RefCell<bool>,
    context: Context<T>
  }

  impl ViewDelegate {
    fn minimum_size(&self, view: Option<&mut View>) -> cef::Size {
      let window = view.and_then(|v| v.window());
      let scale = window
        .and_then(|w| w.display())
        .map(|d| d.device_scale_factor() as f64)
        .unwrap_or(1.0);
      let mut min_w: i32 = 0;
      let mut min_h: i32 = 0;
      let Ok(attributes) = self.attributes.try_borrow() else {
        return cef::Size { width: 0, height: 0 };
      };
      if let Some(min_size) = attributes.min_inner_size {
        let logical = min_size.to_logical::<u32>(scale);
        min_w = min_w.max(logical.width as i32);
        min_h = min_h.max(logical.height as i32);
      }
      if let Some(constraints) = attributes.inner_size_constraints.as_ref() {
        if let Some(w) = constraints.min_width {
          let w_lg = i32::from(w.to_logical::<u32>(scale));
          min_w = min_w.max(w_lg);
        }
        if let Some(h) = constraints.min_height {
          let h_lg = i32::from(h.to_logical::<u32>(scale));
          min_h = min_h.max(h_lg);
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
      let mut max_w: Option<i32> = None;
      let mut max_h: Option<i32> = None;
      let Ok(attributes) = self.attributes.try_borrow() else {
        return cef::Size { width: 0, height: 0 };
      };

      if let Some(max_size) = attributes.max_inner_size {
        let logical = max_size.to_logical::<u32>(scale);
        max_w = Some(logical.width as i32);
        max_h = Some(logical.height as i32);
      }
      if let Some(constraints) = attributes.inner_size_constraints.as_ref() {
        if let Some(w) = constraints.max_width {
          let w_lg = i32::from(w.to_logical::<u32>(scale));
          max_w = Some(match max_w { Some(v) => v.min(w_lg), None => w_lg });
        }
        if let Some(h) = constraints.max_height {
          let h_lg = i32::from(h.to_logical::<u32>(scale));
          max_h = Some(match max_h { Some(v) => v.min(h_lg), None => h_lg });
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

    fn on_theme_changed(&self, view: Option<&mut View>) {
      let attrs = self.attributes.borrow();

      #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
      {
        let Some(view) = view else { return; };
          if attrs.transparent.unwrap_or_default() {
          view.set_background_color(TRANSPARENT);
        } else if let Some(color) = attrs.background_color {
          let color = color_to_cef_argb(color);
          view.set_background_color(color);
        }
      }

      // macOS resets traffic light button positions during the layout pass
      // that follows an appearance change, so we must defer the reapply
      // to run after that layout completes.
      #[cfg(target_os = "macos")]
      if let Some(position) = attrs.traffic_light_position {
        send_message_task(
          &self.context,
          Message::Window {
            window_id: self.window_id,
            message: WindowMessage::SetTrafficLightPosition(position),
          },
        );
      }

      if std::mem::take(&mut *self.suppress_next_theme_changed.borrow_mut()) {
        return;
      }

      let (system_theme, explicit_theme) = {
        let windows = self.windows.borrow();
        let Some(app_window) = windows.get(&self.window_id) else {
          return;
        };

        let Some(system_theme) = native_window_theme(app_window) else {
          return;
        };

        let explicit_theme = app_window.attributes.borrow().theme;
        (system_theme, explicit_theme)
      };

      if let Some(explicit_theme) = explicit_theme
        && let Some(app_window) = self.windows.borrow().get(&self.window_id)
      {
        #[cfg(target_os = "macos")]
        {
          *self.suppress_next_theme_changed.borrow_mut() = true;
          send_message_task(
            &self.context,
            Message::Window {
              window_id: self.window_id,
              message: WindowMessage::SetTheme(Some(explicit_theme)),
            },
          );
        }
        set_window_theme_scheme(app_window, Some(explicit_theme));
      }

      send_window_event(
        self.window_id,
        &self.windows,
        &self.callback,
        WindowEvent::ThemeChanged(system_theme),
      );
    }
  }

  impl PanelDelegate {}

  impl WindowDelegate {
    fn on_window_created(&self, window: Option<&mut Window>) {
      if let Some(window) = window {
        *self.last_emitted_scale_factor.borrow_mut() = window_scale_factor(window);

        // Setup necessary handling for `start_window_dragging` to work on Windows
        #[cfg(windows)]
        drag_window::windows::subclass_window_for_dragging(window);

        let a = self.attributes.borrow();

        #[cfg(windows)]
        if let Some(parent) = a.parent {
          crate::platform::set_parent(window, parent);
        } else if let Some(owner) = a.owner {
          crate::platform::set_owner(window, owner);
        }

        #[cfg(target_os = "macos")]
        apply_macos_window_theme(Some(window), a.theme);
        if let Some(icon) = a.icon.clone() {
          set_window_icon(window, icon);
        }

        #[cfg(target_os = "macos")]
        {
          let decorations = a.decorations.unwrap_or(true);

          // default to transparent title bar if decorations are disabled, otherwise use visible title bar
          let default_style = if decorations {
            TitleBarStyle::Visible
          } else {
            TitleBarStyle::Transparent
          };
          let style = a.title_bar_style.unwrap_or(default_style);

          // default to hidden title if decorations are disabled, otherwise show title
          let hidden_title = a.hidden_title.unwrap_or(!decorations);

          apply_titlebar_style(window, style, hidden_title);
        }

        if let Some(title) = &a.title {
          window.set_title(Some(&CefString::from(title.as_str())));
        }

        if let Some(inner_size) = a.inner_size

          && let Some(display) = window.display() {
            let scale = display.device_scale_factor() as f64;
            let size = size_to_cef(inner_size, scale);

            // On Windows, the size set via CEF APIs is the outer size (including borders),
            // so we need to adjust it to set the correct inner size.
            #[cfg(windows)]
            let size = crate::utils::windows::adjust_size(window.window_handle(), size);

            window.set_size(Some(&size));
          }

        if let Some(position) = &a.position
          && let Some(display) = window.display() {
            let device_scale_factor = display.device_scale_factor() as f64;
            let position = position_to_cef(*position, device_scale_factor);
            window.set_position(Some(&position));
          }

        if a.center {
          // Use CEF's native centering API
          window.center_window(Some(&window.size()));
        }

        if let Some(focused) = a.focused
          && focused {
            window.request_focus();
          }

        if let Some(maximized) = a.maximized
          && maximized {
            window.maximize();
          }

        if let Some(fullscreen) = a.fullscreen
          && fullscreen {
            window.set_fullscreen(1);
          }

        if let Some(always_on_top) = a.always_on_top
          && always_on_top {
            window.set_always_on_top(1);
          }

        if let Some(always_on_bottom) = a.always_on_bottom {
          crate::platform::set_always_on_bottom(window, always_on_bottom);
        }

        if let Some(visible_on_all_workspaces) = a.visible_on_all_workspaces
          && visible_on_all_workspaces {
            crate::platform::set_visible_on_all_workspaces(window, visible_on_all_workspaces);
          }

        if let Some(content_protected) = a.content_protected {
          apply_content_protection(window, content_protected);
        }

        if let Some(skip_taskbar) = a.skip_taskbar
          && skip_taskbar {
            crate::platform::set_skip_taskbar(window, skip_taskbar);
          }

        if let Some(shadow) = a.shadow {
          crate::platform::set_shadow(window, shadow);
        }

        if let Some(focusable) = a.focusable {
          window.set_focusable(if focusable { 1 } else { 0 });
        }

        if a.visible.unwrap_or(true) {
          window.show();
        }

        // Set traffic light position on macOS after window is fully created
        // by posting a task to the UI thread to avoid issues with early setting
        #[cfg(target_os = "macos")]
        if let Some(pos) = a.traffic_light_position {
          let window_message = WindowMessage::SetTrafficLightPosition(pos);
          let message = Message::Window {
            window_id: self.window_id,
            message: window_message,
          };

          send_message_task(&self.context, message);
        }
      }
    }

    fn is_frameless(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      // Map `decorations: false` to frameless window
      let decorated = self
        .attributes
        .borrow()
        .decorations
        .unwrap_or(true);
      (!decorated) as i32
    }

    fn with_standard_window_buttons(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      1
    }

    fn on_window_destroyed(&self, _window: Option<&mut Window>) {
      on_window_destroyed(self.window_id, &self.context);
    }

    fn can_resize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      self
        .attributes
        .borrow()
        .resizable
        .unwrap_or(true) as i32
    }

    fn can_maximize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      // Can maximize if maximizable is true and resizable is true (or not set, defaulting to true)
      let a = self.attributes.borrow();
      let resizable = a.resizable.unwrap_or(true);
      let maximizable = a.maximizable.unwrap_or(true);
      (resizable && maximizable) as i32
    }

    fn can_minimize(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      self
        .attributes
        .borrow()
        .minimizable
        .unwrap_or(true) as i32
    }

    fn can_close(&self, _window: Option<&mut Window>) -> ::std::os::raw::c_int {
      // Direct-exit tear-down: behave like cefclient with `force_close = true`
      // — skip the embedder dialog and just drive the cooperative browser
      // close so `OnBeforeClose` fires.
      if self.context.is_shutting_down.load(Ordering::SeqCst)
        || self.force_close.load(Ordering::SeqCst)
      {
        close_window_browsers(self.window_id, &self.windows);
        return 1;
      }
      let closable = self
        .attributes
        .borrow()
        .closable
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
        close_window_browsers(self.window_id, &self.windows) as i32
      }
    }

    fn on_window_bounds_changed(
      &self,
      window: Option<&mut Window>,
      bounds: Option<&cef::Rect>,
    ) {
      let (Some(window), Some(bounds)) = (window, bounds) else { return; };

      #[cfg(target_os = "macos")]
      if let Some(pos) = &self.attributes.borrow().traffic_light_position {
        apply_traffic_light_position(window.window_handle(), pos);
      }

      #[cfg(not(windows))]
      let size = LogicalSize::new(bounds.width as u32, bounds.height as u32);

      // On Windows, we need to get the inner size because the bounds include the window borders.
      #[cfg(windows)]
      let size = crate::utils::windows::inner_size(window.window_handle());

      // Update autoresize overlay bounds
      let bounds_updates: Vec<(CefWebview, cef::Rect)> =
        if let Ok(windows_ref) = self.windows.try_borrow() {
          if let Some(app_window) = windows_ref.get(&self.window_id) {
            app_window
              .webviews
              .iter()
              .filter_map(|wrapper| {
                if wrapper.inner.is_browser() {
                  wrapper.bounds.lock().unwrap().as_ref().map(|b| {
                    let new_rect = cef::Rect {
                      x: (size.width as f32 * b.x_rate) as i32,
                      y: (size.height as f32 * b.y_rate) as i32,
                      width: (size.width as f32 * b.width_rate) as i32,
                      height: (size.height as f32 * b.height_rate) as i32,
                    };
                    (wrapper.inner.clone(), new_rect)
                  })
                } else {
                  None
                }
              })
              .collect()
          } else {
            Vec::new()
          }
        } else {
          Vec::new()
        };

      for (inner, rect) in bounds_updates {
        inner.set_bounds(Some(&rect));
      }

      let scale = window_scale_factor(window);

      #[cfg(not(windows))]
      let physical_size = size.to_physical::<u32>(scale);

      #[cfg(windows)]
      let physical_size = size;

      let scale_factor_changed = {
        let mut emitted_scale_factor = self.last_emitted_scale_factor.borrow_mut();
        let changed = *emitted_scale_factor != scale;
        if changed {
          *emitted_scale_factor = scale;
        }
        changed
      };
      if scale_factor_changed {
        send_window_event(
          self.window_id,
          &self.windows,
          &self.callback,
          WindowEvent::ScaleFactorChanged {
            scale_factor: scale,
            new_inner_size: physical_size,
          },
        );
      }

      let physical_position = LogicalPosition::new(bounds.x, bounds.y)
        .to_physical::<i32>(scale);
      let position_changed = {
        let mut emitted_pos = self.last_emitted_position.borrow_mut();
        let changed = *emitted_pos != physical_position;
        if changed {
          *emitted_pos = physical_position;
        }
        changed
      };
      if position_changed {
        send_window_event(
          self.window_id,
          &self.windows,
          &self.callback,
          WindowEvent::Moved(physical_position),
        );
      }

      let size_changed = {
        let mut emitted_size = self.last_emitted_size.borrow_mut();
        let changed = *emitted_size != physical_size;
        if changed {
          *emitted_size = physical_size;
        }
        changed
      };
      if size_changed {
        send_window_event(
          self.window_id,
          &self.windows,
          &self.callback,
          WindowEvent::Resized(physical_size),
        );
      }
    }

    fn on_window_activation_changed(
      &self,
      _window: Option<&mut Window>,
      active: ::std::os::raw::c_int,
    ) {
      send_window_event(
        self.window_id,
        &self.windows,
        &self.callback,
        WindowEvent::Focused(active == 1),
      );
    }
  }
}

fn get_webview<T: UserEvent>(
  context: &Context<T>,
  window_id: WindowId,
  webview_id: u32,
) -> Option<AppWebview> {
  context
    .windows
    .borrow()
    .get(&window_id)
    .and_then(|app_window| {
      app_window
        .webviews
        .iter()
        .find(|w| w.webview_id == webview_id)
        .cloned()
    })
}

fn get_main_frame<T: UserEvent>(
  context: &Context<T>,
  window_id: WindowId,
  webview_id: u32,
) -> Option<Frame> {
  get_webview(context, window_id, webview_id)
    .and_then(|bv| bv.inner.browser())
    .and_then(|b| b.main_frame())
}

fn get_browser<T: UserEvent>(
  context: &Context<T>,
  window_id: WindowId,
  webview_id: u32,
) -> Option<Browser> {
  get_webview(context, window_id, webview_id).and_then(|bv| bv.inner.browser())
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
      if let Some(frame) = get_main_frame(context, window_id, webview_id) {
        frame.execute_java_script(
          Some(&cef::CefString::from(script.as_str())),
          Some(&cef::CefString::from("")),
          0,
        );
      }
    }
    WebviewMessage::EvaluateScriptWithCallback(script, callback) => {
      if let Some(host) = get_browser(context, window_id, webview_id).and_then(|b| b.host()) {
        let message_id = context.next_webview_event_id() as i32 + 1;
        let message_id = Arc::new(AtomicI32::new(message_id));
        let callback = Arc::new(Mutex::new(Some(callback)));
        let registration = Arc::new(Mutex::new(None));
        let mut observer = EvalScriptWithCallbackDevToolsObserver::new(
          message_id.clone(),
          callback.clone(),
          registration.clone(),
        );

        if let Some(observer_registration) =
          host.add_dev_tools_message_observer(Some(&mut observer))
        {
          *registration.lock().unwrap() = Some(observer_registration);

          let message = serde_json::json!({
            "id": message_id.load(Ordering::Relaxed),
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
    }
    WebviewMessage::Navigate(url) => {
      if let Some(frame) = get_main_frame(context, window_id, webview_id) {
        frame.load_url(Some(&cef::CefString::from(url.as_str())))
      }
    }
    WebviewMessage::Reload => {
      if let Some(browser) = get_browser(context, window_id, webview_id) {
        browser.reload()
      }
    }
    WebviewMessage::GoBack => {
      if let Some(browser) = get_browser(context, window_id, webview_id) {
        browser.go_back()
      }
    }
    WebviewMessage::CanGoBack(tx) => {
      if let Some(browser) = get_browser(context, window_id, webview_id) {
        let _ = tx.send(Ok(browser.can_go_back() != 0));
      } else {
        let _ = tx.send(Err(tauri_runtime::Error::FailedToSendMessage));
      }
    }
    WebviewMessage::GoForward => {
      if let Some(browser) = get_browser(context, window_id, webview_id) {
        browser.go_forward()
      }
    }
    WebviewMessage::CanGoForward(tx) => {
      if let Some(browser) = get_browser(context, window_id, webview_id) {
        let _ = tx.send(Ok(browser.can_go_forward() != 0));
      } else {
        let _ = tx.send(Err(tauri_runtime::Error::FailedToSendMessage));
      }
    }
    WebviewMessage::Print => {
      if let Some(host) = get_browser(context, window_id, webview_id).and_then(|b| b.host()) {
        host.print()
      }
    }
    WebviewMessage::Close => {
      let webview_to_close = {
        let mut windows = context.windows.borrow_mut();
        if let Some(app_window) = windows.get_mut(&window_id) {
          let webview_index = app_window
            .webviews
            .iter()
            .position(|w| w.webview_id == webview_id);

          if let Some(index) = webview_index {
            let wrapper = app_window.webviews.remove(index);
            app_window
              .webview_event_listeners
              .lock()
              .unwrap()
              .remove(&webview_id);
            Some(wrapper)
          } else {
            None
          }
        } else {
          None
        }
      };

      if let Some(wrapper) = webview_to_close {
        let browser_id = *wrapper.browser_id.borrow();
        {
          let mut registry = context.scheme_handler_registry.lock().unwrap();
          for scheme in wrapper.uri_scheme_protocols.keys() {
            registry.remove(&(browser_id, scheme.clone()));
          }
        }
        wrapper.inner.close();
      }
    }
    WebviewMessage::Show => {
      if let Some(wrapper) = get_webview(context, window_id, webview_id) {
        wrapper.inner.set_visible(1)
      }
    }
    WebviewMessage::Hide => {
      if let Some(wrapper) = get_webview(context, window_id, webview_id) {
        wrapper.inner.set_visible(0)
      }
    }
    WebviewMessage::SetPosition(position) => {
      let data = context
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|app_window| {
          let device_scale_factor = app_window
            .window
            .display()
            .map(|d| d.device_scale_factor() as f64)
            .unwrap_or(1.0);

          let position = position_to_cef(position, device_scale_factor);

          app_window
            .webviews
            .iter()
            .find(|w| w.webview_id == webview_id)
            .map(|wrapper| {
              let current_bounds = wrapper.inner.bounds();
              let new_bounds = cef::Rect {
                x: position.x,
                y: position.y,
                width: current_bounds.width,
                height: current_bounds.height,
              };
              let inner = wrapper.inner.clone();
              let bounds_arc = wrapper.bounds.clone();
              let is_browser = wrapper.inner.is_browser();
              let window_bounds = if is_browser {
                Some(app_window.window.bounds())
              } else {
                None
              };
              (inner, new_bounds, is_browser, bounds_arc, window_bounds)
            })
        });

      if let Some((inner, new_bounds, is_browser, bounds_arc, window_bounds)) = data {
        inner.set_bounds(Some(&new_bounds));
        if is_browser
          && let Some(b) = &mut *bounds_arc.lock().unwrap()
          && let Some(wb) = window_bounds
        {
          let window_size = LogicalSize::new(wb.width as u32, wb.height as u32);
          b.x_rate = new_bounds.x as f32 / window_size.width as f32;
          b.y_rate = new_bounds.y as f32 / window_size.height as f32;
        }
      }
    }
    WebviewMessage::SetSize(size) => {
      let data = context
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|app_window| {
          let device_scale_factor = app_window
            .window
            .display()
            .map(|d| d.device_scale_factor() as f64)
            .unwrap_or(1.0);

          let size = size_to_cef(size, device_scale_factor);

          app_window
            .webviews
            .iter()
            .find(|w| w.webview_id == webview_id)
            .map(|wrapper| {
              let current_bounds = wrapper.inner.bounds();
              let new_bounds = cef::Rect {
                x: current_bounds.x,
                y: current_bounds.y,
                width: size.width,
                height: size.height,
              };
              let inner = wrapper.inner.clone();
              let bounds_arc = wrapper.bounds.clone();
              let is_browser = wrapper.inner.is_browser();
              let window_bounds = if is_browser {
                Some(app_window.window.bounds())
              } else {
                None
              };
              (inner, new_bounds, is_browser, bounds_arc, window_bounds)
            })
        });

      if let Some((inner, new_bounds, is_browser, bounds_arc, window_bounds)) = data {
        inner.set_bounds(Some(&new_bounds));
        if is_browser
          && let Some(b) = &mut *bounds_arc.lock().unwrap()
          && let Some(wb) = window_bounds
        {
          let window_size = LogicalSize::new(wb.width as u32, wb.height as u32);
          b.width_rate = new_bounds.width as f32 / window_size.width as f32;
          b.height_rate = new_bounds.height as f32 / window_size.height as f32;
        }
      }
    }
    WebviewMessage::SetBounds(bounds) => {
      let data = context
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|app_window| {
          let device_scale_factor = app_window
            .window
            .display()
            .map(|d| d.device_scale_factor() as f64)
            .unwrap_or(1.0);

          let new_bounds = rect_to_cef(bounds, device_scale_factor);
          app_window
            .webviews
            .iter()
            .find(|w| w.webview_id == webview_id)
            .map(|wrapper| {
              let inner = wrapper.inner.clone();
              let bounds_arc = wrapper.bounds.clone();
              let is_browser = wrapper.inner.is_browser();
              let window_bounds = if is_browser {
                Some(app_window.window.bounds())
              } else {
                None
              };
              (inner, new_bounds, is_browser, bounds_arc, window_bounds)
            })
        });

      if let Some((inner, new_bounds, is_browser, bounds_arc, window_bounds)) = data {
        inner.set_bounds(Some(&new_bounds));
        if is_browser
          && let Some(b) = &mut *bounds_arc.lock().unwrap()
          && let Some(wb) = window_bounds
        {
          let window_size = LogicalSize::new(wb.width as u32, wb.height as u32);
          b.x_rate = new_bounds.x as f32 / window_size.width as f32;
          b.y_rate = new_bounds.y as f32 / window_size.height as f32;
          b.width_rate = new_bounds.width as f32 / window_size.width as f32;
          b.height_rate = new_bounds.height as f32 / window_size.height as f32;
        }
      }
    }
    WebviewMessage::SetFocus => {
      if let Some(host) = get_webview(context, window_id, webview_id)
        .and_then(|bv| bv.inner.browser())
        .and_then(|b| b.host())
      {
        host.set_focus(1)
      }
    }
    WebviewMessage::Reparent(target_window_id, tx) => {
      let reparent_data = {
        let mut windows = context.windows.borrow_mut();

        if !windows.contains_key(&target_window_id) {
          let _ = tx.send(Err(tauri_runtime::Error::FailedToSendMessage));
          return;
        };

        let Some(webview_wrapper) = windows.get_mut(&window_id).and_then(|app_window| {
          app_window
            .webviews
            .iter()
            .position(|w| w.webview_id == webview_id)
            .map(|index| app_window.webviews.remove(index))
        }) else {
          let _ = tx.send(Err(tauri_runtime::Error::FailedToSendMessage));
          return;
        };

        let target_cef_window = match windows.get(&target_window_id) {
          Some(target_window) => target_window.window.clone(),
          None => {
            let _ = tx.send(Err(tauri_runtime::Error::FailedToSendMessage));
            return;
          }
        };

        (webview_wrapper, target_cef_window)
      };

      let (webview_wrapper, target_cef_window) = reparent_data;

      let bounds = webview_wrapper.inner.bounds();
      webview_wrapper.inner.set_parent(&target_cef_window);
      webview_wrapper.inner.set_bounds(Some(&bounds));

      {
        let mut windows = context.windows.borrow_mut();
        if let Some(target_window) = windows.get_mut(&target_window_id) {
          target_window.webviews.push(webview_wrapper);
          let _ = tx.send(Ok(()));
        } else {
          let _ = tx.send(Err(tauri_runtime::Error::FailedToSendMessage));
        }
      }
    }
    WebviewMessage::SetAutoResize(auto_resize) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id)
        && let Some(wrapper) = app_window
          .webviews
          .iter()
          .find(|w| w.webview_id == webview_id)
        && wrapper.inner.is_browser()
      {
        if auto_resize {
          let window_bounds = app_window.window.bounds();
          let window_size =
            LogicalSize::new(window_bounds.width as u32, window_bounds.height as u32);

          let ob = wrapper.inner.bounds();
          let pos = LogicalPosition::new(ob.x, ob.y);
          let size = LogicalSize::new(ob.width as u32, ob.height as u32);

          *wrapper.bounds.lock().unwrap() = Some(crate::WebviewBounds {
            x_rate: pos.x as f32 / window_size.width as f32,
            y_rate: pos.y as f32 / window_size.height as f32,
            width_rate: size.width as f32 / window_size.width as f32,
            height_rate: size.height as f32 / window_size.height as f32,
          });
        } else {
          *wrapper.bounds.lock().unwrap() = None;
        }
      }
    }
    WebviewMessage::SetZoom(scale_factor) => {
      if let Some(host) = get_webview(context, window_id, webview_id)
        .and_then(|bv| bv.inner.browser())
        .and_then(|b| b.host())
      {
        // CEF uses a logarithmic zoom level where percentage = 1.2^level
        // (Chromium's kTextSizeMultiplierRatio). Convert from Tauri linear
        // scale factor (1.0 = 100%) to CEF's level (0.0 = 100%)
        const CEF_ZOOM_BASE: f64 = 1.2;
        let zoom_level = if scale_factor > 0.0 {
          scale_factor.ln() / CEF_ZOOM_BASE.ln()
        } else {
          0.0
        };
        host.set_zoom_level(zoom_level)
      }
    }
    WebviewMessage::SetBackgroundColor(color) => {
      if let Some(bv) = context
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|app_window| {
          app_window
            .webviews
            .iter()
            .find(|w| w.webview_id == webview_id)
        })
      {
        bv.webview_attributes.borrow_mut().background_color = color;

        bv.inner.set_background_color(color.map(color_to_cef_argb));
      }
    }
    WebviewMessage::ClearAllBrowsingData => {
      // CEF has no single "clear all browsing data" call, so we clear
      // the cookies (including session cookies) and the HTTP cache for this
      // context.
      if let Some(request_context) = get_browser(context, window_id, webview_id)
        .and_then(|b| b.host())
        .and_then(|host| host.request_context())
      {
        if let Some(manager) = request_context.cookie_manager(None) {
          manager.delete_cookies(None, None, None);
          manager.flush_store(None);
        }
        request_context.clear_http_cache(None);
      }
    }
    // Getters
    WebviewMessage::Url(tx) => {
      let result = get_main_frame(context, window_id, webview_id)
        .map(|frame| cef::CefString::from(&frame.url()).to_string())
        .ok_or(tauri_runtime::Error::FailedToSendMessage);
      let _ = tx.send(result);
    }
    WebviewMessage::Bounds(tx) => {
      let result = get_webview(context, window_id, webview_id)
        .map(|webview| {
          let bounds = webview.inner.bounds();
          let scale = webview.inner.scale_factor();
          let logical_position = LogicalPosition::new(bounds.x, bounds.y);
          let logical_size = LogicalSize::new(bounds.width as u32, bounds.height as u32);
          let physical_position = logical_position.to_physical::<i32>(scale);
          let physical_size = logical_size.to_physical::<u32>(scale);
          tauri_runtime::dpi::Rect {
            position: Position::Physical(physical_position),
            size: Size::Physical(physical_size),
          }
        })
        .ok_or(tauri_runtime::Error::FailedToSendMessage);
      let _ = tx.send(result);
    }
    WebviewMessage::Position(tx) => {
      let result = get_webview(context, window_id, webview_id)
        .map(|webview| {
          let bounds = webview.inner.bounds();
          let scale = webview.inner.scale_factor();
          LogicalPosition::new(bounds.x, bounds.y).to_physical::<i32>(scale)
        })
        .ok_or(tauri_runtime::Error::FailedToSendMessage);
      let _ = tx.send(result);
    }
    WebviewMessage::Size(tx) => {
      let result = get_webview(context, window_id, webview_id)
        .map(|webview| {
          let bounds = webview.inner.bounds();
          let scale = webview.inner.scale_factor();
          let size = LogicalSize::new(bounds.width as u32, bounds.height as u32);
          size.to_physical::<u32>(scale)
        })
        .ok_or(tauri_runtime::Error::FailedToSendMessage);
      let _ = tx.send(result);
    }
    WebviewMessage::WithWebview(f) => {
      if let Some(browser) = get_browser(context, window_id, webview_id) {
        f(Webview::new(browser));
      }
    }
    // Devtools
    #[cfg(any(debug_assertions, feature = "devtools"))]
    WebviewMessage::OpenDevTools => {
      if let Some(host) = get_browser(context, window_id, webview_id).and_then(|b| b.host()) {
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
    #[cfg(any(debug_assertions, feature = "devtools"))]
    WebviewMessage::CloseDevTools => {
      if let Some(host) = get_browser(context, window_id, webview_id).and_then(|b| b.host()) {
        host.close_dev_tools()
      }
    }
    #[cfg(any(debug_assertions, feature = "devtools"))]
    WebviewMessage::IsDevToolsOpen(tx) => {
      let result = get_browser(context, window_id, webview_id)
        .and_then(|b| b.host())
        .map(|host| host.has_dev_tools() != 0)
        .unwrap_or(false);
      let _ = tx.send(result);
    }
    WebviewMessage::SendDevToolsMessage(message, tx) => {
      let result = get_browser(context, window_id, webview_id)
        .and_then(|b| b.host())
        .map(|host| {
          let result = host.send_dev_tools_message(Some(&message));
          if result == 1 {
            Ok(())
          } else {
            Err(tauri_runtime::Error::FailedToSendMessage)
          }
        })
        .unwrap_or(Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WebviewMessage::OnDevToolsProtocol(handler, tx) => {
      let result = match get_webview(context, window_id, webview_id) {
        Some(webview) => {
          webview
            .devtools_protocol_handlers
            .lock()
            .unwrap()
            .push(handler);

          let needs_devtools_observer = webview
            .devtools_observer_registration
            .lock()
            .unwrap()
            .is_none();
          if needs_devtools_observer
            && let Some(browser) = get_browser(context, window_id, webview_id)
            && let Some(registration) = add_dev_tools_observer(
              &browser,
              webview.devtools_protocol_handlers.clone(),
              Arc::new(Mutex::new(HashMap::new())),
            )
          {
            *webview.devtools_observer_registration.lock().unwrap() = Some(registration);
          }
          Ok(())
        }
        None => Err(tauri_runtime::Error::FailedToSendMessage),
      };
      let _ = tx.send(result);
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
        let url = get_main_frame(context, window_id, webview_id)
          .map(|frame| cef::CefString::from(&frame.url()).to_string())
          .unwrap_or_default();

        let mut cef_cookie = cef::Cookie {
          name: cef::CefString::from(cookie.name()),
          value: cef::CefString::from(cookie.value()),
          ..Default::default()
        };
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
        let url = get_main_frame(context, window_id, webview_id)
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

#[cfg(target_os = "macos")]
fn start_window_dragging(window: &cef::Window) {
  use objc2::rc::Retained;
  use objc2_app_kit::{NSEvent, NSEventModifierFlags, NSEventType, NSView};

  unsafe {
    let ns_view = Retained::<NSView>::retain(window.window_handle() as _);
    if let Some(ns_view) = ns_view
      && let Some(ns_window) = ns_view.window()
    {
      // Get current mouse location
      let mouse_location = NSEvent::mouseLocation();

      // Try to get the current event from NSApp
      let mut event = None;
      if let Some(mtm) = objc2::MainThreadMarker::new() {
        let ns_app = objc2_app_kit::NSApp(mtm);
        event = ns_app.currentEvent();
      }

      // Create a mouse event for dragging
      // If we have a current event, try to use its properties
      let drag_event = if let Some(current_event) = event {
        let event_modifier_flags = current_event.modifierFlags();
        let event_timestamp = current_event.timestamp();
        let event_window_number = current_event.windowNumber();

        NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
            NSEventType::LeftMouseDown,
            mouse_location,
            event_modifier_flags,
            event_timestamp,
            event_window_number,
            None,
            0,
            1,
            1.0,
          )
      } else {
        NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
            NSEventType::LeftMouseDown,
            mouse_location,
            NSEventModifierFlags::empty(),
            0.0,
            ns_window.windowNumber(),
            None,
            0,
            1,
            1.0,
          )
      };

      if let Some(event) = drag_event {
        ns_window.performWindowDragWithEvent(&event);
      }
    }
  }
}

#[cfg(windows)]
fn start_window_dragging(window: &cef::Window) {
  use windows::Win32::Foundation::*;
  use windows::Win32::UI::Input::KeyboardAndMouse::*;
  use windows::Win32::UI::WindowsAndMessaging::*;

  unsafe {
    let hwnd = window.window_handle();

    let mut pos = std::mem::zeroed();
    let _ = GetCursorPos(&mut pos);

    let points = POINTS {
      x: pos.x as i16,
      y: pos.y as i16,
    };

    let _ = ReleaseCapture();

    let _ = PostMessageW(
      Some(HWND(hwnd.0 as _)),
      WM_NCLBUTTONDOWN,
      WPARAM(HTCAPTION as usize),
      LPARAM(&points as *const _ as isize),
    );
  }
}

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
fn start_window_dragging(window: &cef::Window) {
  use std::ffi::CString;
  use std::os::raw::c_long;
  use x11_dl::xlib;

  let Some(xlib) = xlib::Xlib::open().ok() else {
    return;
  };

  unsafe {
    let display = (xlib.XOpenDisplay)(std::ptr::null());
    if display.is_null() {
      return;
    }

    let win = window.window_handle();

    let mut root_x: std::ffi::c_int = 0;
    let mut root_y: std::ffi::c_int = 0;
    let mut _win_x: std::ffi::c_int = 0;
    let mut _win_y: std::ffi::c_int = 0;
    let mut _mask: std::ffi::c_uint = 0;
    let mut root: xlib::Window = (xlib.XDefaultRootWindow)(display);
    let mut _child_return: xlib::Window = 0;
    let _ = (xlib.XQueryPointer)(
      display,
      win,
      &mut root,
      &mut _child_return,
      &mut root_x,
      &mut root_y,
      &mut _win_x,
      &mut _win_y,
      &mut _mask,
    );

    let net_wm_moveresize = CString::new("_NET_WM_MOVERESIZE").unwrap();
    let atom = (xlib.XInternAtom)(display, net_wm_moveresize.as_ptr(), xlib::False);
    if atom == 0 {
      (xlib.XCloseDisplay)(display);
      return;
    }

    // EWMH _NET_WM_MOVERESIZE: direction 8 = move, button 1 = left, source 1 = application
    const NET_WM_MOVERESIZE_MOVE: c_long = 8;
    const SOURCE_APPLICATION: c_long = 1;

    let mut data: xlib::ClientMessageData = std::mem::zeroed();
    {
      let longs = <xlib::ClientMessageData as std::convert::AsMut<[i64]>>::as_mut(&mut data);
      longs[0] = root_x as i64;
      longs[1] = root_y as i64;
      longs[2] = NET_WM_MOVERESIZE_MOVE;
      longs[3] = 1; // Button 1 (left)
      longs[4] = SOURCE_APPLICATION;
    }

    let xclient = xlib::XClientMessageEvent {
      type_: xlib::ClientMessage,
      serial: 0,
      send_event: xlib::True,
      display,
      window: win,
      message_type: atom,
      format: 32,
      data,
    };

    let mut event: xlib::XEvent = xclient.into();
    let _ = (xlib.XSendEvent)(display, root, xlib::False, 0, &mut event);
    (xlib.XFlush)(display);
    (xlib.XCloseDisplay)(display);
  }
}

fn handle_window_message<T: UserEvent>(
  context: &Context<T>,
  window_id: WindowId,
  message: WindowMessage,
) {
  match message {
    WindowMessage::Close => {
      on_close_requested(window_id, context);
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
        .and_then(|w| {
          w.window
            .display()
            .map(|d| Ok(d.device_scale_factor() as f64))
        })
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
          let scale = w
            .window
            .display()
            .map(|d| d.device_scale_factor() as f64)
            .unwrap_or(1.0);
          Ok(LogicalPosition::new(bounds.x, bounds.y).to_physical::<i32>(scale))
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
          let scale = w
            .window
            .display()
            .map(|d| d.device_scale_factor() as f64)
            .unwrap_or(1.0);
          Ok(LogicalPosition::new(bounds.x, bounds.y).to_physical::<i32>(scale))
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
          #[cfg(not(windows))]
          let size = {
            let scale = w
              .window
              .display()
              .map(|d| d.device_scale_factor() as f64)
              .unwrap_or(1.0);

            let bounds = w.window.bounds();
            LogicalSize::new(bounds.width as u32, bounds.height as u32).to_physical::<u32>(scale)
          };

          // On Windows, window.bounds() is the outer size, not the inner size.
          #[cfg(windows)]
          let size = crate::utils::windows::inner_size(w.window.window_handle());

          Ok(size)
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
          let scale = w
            .window
            .display()
            .map(|d| d.device_scale_factor() as f64)
            .unwrap_or(1.0);
          Ok(LogicalSize::new(bounds.width as u32, bounds.height as u32).to_physical::<u32>(scale))
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
        .map(|w| {
          let title = w.window.title();
          Ok(cef::CefString::from(&title).to_string())
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
            let scale = d.device_scale_factor() as f64;
            let physical_size =
              LogicalSize::new(bounds.width as u32, bounds.height as u32).to_physical::<u32>(scale);
            let physical_position =
              LogicalPosition::new(bounds.x, bounds.y).to_physical::<i32>(scale);
            let work_physical_size =
              LogicalSize::new(work.width as u32, work.height as u32).to_physical::<u32>(scale);
            let work_physical_position =
              LogicalPosition::new(work.x, work.y).to_physical::<i32>(scale);
            tauri_runtime::monitor::Monitor {
              name: None,
              size: PhysicalSize::new(physical_size.width, physical_size.height),
              position: PhysicalPosition::new(physical_position.x, physical_position.y),
              work_area: PhysicalRect {
                position: PhysicalPosition::new(work_physical_position.x, work_physical_position.y),
                size: PhysicalSize::new(work_physical_size.width, work_physical_size.height),
              },
              scale_factor: d.device_scale_factor() as f64,
            }
          })
        })
        .map(Ok)
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
        .map(|w| Ok(native_window_theme(w).unwrap_or(tauri_utils::Theme::Light)))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
    }
    WindowMessage::IsEnabled(tx) => {
      let result = context
        .windows
        .borrow()
        .get(&window_id)
        .map(|w| Ok(crate::platform::is_enabled(&w.window)))
        .unwrap_or_else(|| Err(tauri_runtime::Error::FailedToSendMessage));
      let _ = tx.send(result);
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
            let xid = w.window.window_handle();
            Ok(raw_window_handle::WindowHandle::borrow_raw(
              raw_window_handle::RawWindowHandle::Xlib(raw_window_handle::XlibWindowHandle::new(
                xid,
              )),
            ))
          }

          #[cfg(target_os = "macos")]
          unsafe {
            let ns_view = w.window.window_handle();
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
            let hwnd = w.window.window_handle().0 as isize;
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
        .unwrap_or(Err(raw_window_handle::HandleError::Unavailable));
      let _ = tx.send(result);
    }
    // Setters
    WindowMessage::Center => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window
          .window
          .center_window(Some(&app_window.window.size()));
      }
    }
    WindowMessage::RequestUserAttention(attention_type) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        crate::platform::request_user_attention(&app_window.window, attention_type);
      }
    }
    WindowMessage::SetEnabled(enabled) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        crate::platform::set_enabled(&app_window.window, enabled);
      }
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
    WindowMessage::SetShadow(shadow) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        crate::platform::set_shadow(&app_window.window, shadow);
      }
    }
    WindowMessage::SetAlwaysOnBottom(always_on_bottom) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().always_on_bottom = Some(always_on_bottom);
        crate::platform::set_always_on_bottom(&app_window.window, always_on_bottom);
      }
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
        crate::platform::set_visible_on_all_workspaces(
          &app_window.window,
          visible_on_all_workspaces,
        );
      }
    }
    WindowMessage::SetContentProtected(protected) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().content_protected = Some(protected);
        apply_content_protection(&app_window.window, protected);
      }
    }
    #[allow(unused_mut)]
    WindowMessage::SetSize(mut size) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id)
        && let Some(display) = app_window.window.display()
      {
        let device_scale_factor = display.device_scale_factor() as f64;

        let size = size_to_cef(size, device_scale_factor);

        // On Windows, the size set via CEF APIs is the outer size (including borders),
        // so we need to adjust it to set the correct inner size.
        #[cfg(windows)]
        let size = crate::utils::windows::adjust_size(app_window.window.window_handle(), size);

        app_window.window.set_size(Some(&size));
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
      if let Some(app_window) = context.windows.borrow().get(&window_id)
        && let Some(display) = app_window.window.display()
      {
        let device_scale_factor = display.device_scale_factor() as f64;
        let position = position_to_cef(position, device_scale_factor);
        app_window.window.set_position(Some(&position));
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
    WindowMessage::SetSimpleFullscreen(fullscreen) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        crate::platform::set_simple_fullscreen(&app_window.window, fullscreen);
      }
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
    WindowMessage::SetSkipTaskbar(skip) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().skip_taskbar = Some(skip);
        crate::platform::set_skip_taskbar(&app_window.window, skip);
      }
    }
    WindowMessage::SetCursorGrab(grab) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        crate::platform::set_cursor_grab(&app_window.window, grab);
      }
    }
    WindowMessage::SetCursorVisible(visible) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        crate::platform::set_cursor_visible(&app_window.window, visible);
      }
    }
    WindowMessage::SetCursorIcon(icon) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        crate::platform::set_cursor_icon(&app_window.window, icon);
      }
    }
    WindowMessage::SetCursorPosition(position) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        let scale_factor = app_window
          .window
          .display()
          .map(|d| d.device_scale_factor() as f64)
          .unwrap_or(1.0);
        crate::platform::set_cursor_position(&app_window.window, position, scale_factor);
      }
    }
    WindowMessage::SetIgnoreCursorEvents(ignore) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        crate::platform::set_ignore_cursor_events(&app_window.window, ignore);
      }
    }
    WindowMessage::SetProgressBar(progress_state) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        crate::platform::set_progress_bar(&app_window.window, progress_state);
      }
    }
    WindowMessage::SetBadgeCount(count, desktop_filename) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        crate::platform::set_badge_count(&app_window.window, count, desktop_filename);
      }
    }
    #[cfg(target_os = "macos")]
    WindowMessage::SetBadgeLabel(label) => {
      crate::platform::set_badge_label(label);
    }
    #[cfg(not(target_os = "macos"))]
    WindowMessage::SetBadgeLabel(_label) => {
      // Badge labels are a macOS-only concept.
    }
    WindowMessage::SetOverlayIcon(icon) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        set_overlay_icon(&app_window.window, icon);
      }
    }
    WindowMessage::SetTitleBarStyle(_style) => {
      #[cfg(target_os = "macos")]
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        let hidden_title = {
          let a = app_window.attributes.borrow();
          a.hidden_title.unwrap_or(!a.decorations.unwrap_or(true))
        };
        apply_titlebar_style(&app_window.window, _style, hidden_title);
      }
    }
    WindowMessage::SetTrafficLightPosition(_position) => {
      #[cfg(target_os = "macos")]
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().traffic_light_position = Some(_position);
        apply_traffic_light_position(app_window.window.window_handle(), &_position);
      }
    }
    WindowMessage::SetTheme(theme) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        {
          let mut attributes = app_window.attributes.borrow_mut();
          attributes.theme = theme;
        }
        apply_window_theme_scheme(app_window, theme);
        #[cfg(target_os = "macos")]
        {
          apply_macos_window_theme(Some(&app_window.window), theme);
        }
        // theme changed event is sent by the on_theme_changed handler
      }
    }
    WindowMessage::SetBackgroundColor(color) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        app_window.attributes.borrow_mut().background_color = color;
        let color = color.map(color_to_cef_argb).unwrap_or_else(|| {
          app_window
            .window
            .theme_color(ColorId::COLOR_PRIMARY_BACKGROUND.get_raw() as _)
        });
        app_window.window.set_background_color(color);
      }
    }
    WindowMessage::StartDragging => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        start_window_dragging(&app_window.window);
      }
    }
    WindowMessage::StartResizeDragging(direction) => {
      if let Some(app_window) = context.windows.borrow().get(&window_id) {
        crate::platform::start_resize_dragging(&app_window.window, direction);
      }
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
    } => create_window(context, window_id, webview_id, *pending),
    Message::CreateWebview {
      window_id,
      webview_id,
      pending,
    } => create_webview(
      WebviewKind::WindowChild,
      context,
      window_id,
      webview_id,
      *pending,
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
      // Direct-exit path (e.g. `request_exit`, macOS `-terminate:`): emit
      // only `ExitRequested -> Exit`, matching the cefclient terminate flow
      // where `CloseAllWindows` is initiated only after the embedder agrees
      // to quit. Skip if we're already shutting down to avoid re-prompting.
      if context.is_shutting_down.load(Ordering::SeqCst) {
        return;
      }

      let (tx, rx) = channel();
      in_callback(|| {
        (context.callback.borrow())(RunEvent::ExitRequested {
          code: Some(code),
          tx,
        });
      });

      let recv = rx.try_recv();
      let should_prevent = matches!(recv, Ok(ExitRequestedEventAction::Prevent));

      if !should_prevent {
        context.is_shutting_down.store(true, Ordering::SeqCst);
        in_callback(|| (context.callback.borrow())(RunEvent::Exit));
      }
    }
    Message::Task(t) => t(),
    Message::UserEvent(evt) => {
      in_callback(|| (context.callback.borrow())(RunEvent::UserEvent(evt)));
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
      handle_message(&self.context, std::mem::replace(&mut self.message.borrow_mut(), Message::Noop));
    }
  }
}

pub(crate) fn create_window<T: UserEvent>(
  context: &Context<T>,
  window_id: WindowId,
  webview_id: u32,
  pending: PendingWindow<T, CefRuntime<T>>,
) {
  let PendingWindow {
    label,
    window_builder,
    webview,
  } = pending;

  let force_close = Arc::new(AtomicBool::new(false));
  let attributes = Arc::new(RefCell::new(window_builder));

  let mut delegate = AppWindowDelegate::<T>::new(
    window_id,
    context.callback.clone(),
    force_close.clone(),
    context.windows.clone(),
    attributes.clone(),
    RefCell::new(Default::default()),
    RefCell::new(Default::default()),
    RefCell::new(1.0),
    RefCell::new(false),
    context.clone(),
  );

  let window = window_create_top_level(Some(&mut delegate)).expect("Failed to create window");

  context.windows.borrow_mut().insert(
    window_id,
    AppWindow {
      label,
      window,
      force_close,
      attributes,
      webviews: Vec::new(),
      window_event_listeners: Arc::new(Mutex::new(HashMap::new())),
      webview_event_listeners: Arc::new(Mutex::new(HashMap::new())),
    },
  );

  if let Some(webview) = webview {
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
pub(crate) enum WebviewKind {
  // webview is the entire window content
  WindowContent,
  // webview is a child of the window, which can contain other webviews too
  WindowChild,
}

wrap_task! {
  struct WindowEventTask<T: UserEvent> {
    window_id: WindowId,
    windows: Arc<RefCell<HashMap<WindowId, AppWindow>>>,
    callback: RunEventCallback<T>,
    event: WindowEvent,
  }

  impl Task {
    fn execute(&self) {
      send_window_event(
        self.window_id,
        &self.windows,
        &self.callback,
        self.event.clone(),
      );
    }
  }
}

wrap_task! {
  struct WebviewEventTask<T: UserEvent> {
    context: Context<T>,
    window_id: WindowId,
    webview_id: u32,
    event: WebviewEvent,
  }

  impl Task {
    fn execute(&self) {
      send_webview_event(
        &self.context,
        self.window_id,
        self.webview_id,
        self.event.clone(),
      );
    }
  }
}

wrap_task! {
  struct DragDropScriptEventTask<T: UserEvent> {
    context: Context<T>,
    window_id: WindowId,
    webview_id: u32,
    target: DragDropEventTarget,
    drag_drop_state: Arc<Mutex<DragDropState>>,
    event: DragDropScriptEvent,
  }

  impl Task {
    fn execute(&self) {
      handle_drag_drop_script_event(
        &self.context,
        self.window_id,
        self.webview_id,
        self.target,
        self.drag_drop_state.clone(),
        self.event.clone(),
      );
    }
  }
}

#[cfg(target_os = "macos")]
fn send_message_task<T: UserEvent>(context: &Context<T>, message: Message<T>) {
  let mut task = SendMessageTask::new(context.clone(), Arc::new(RefCell::new(message)));
  cef::post_task(sys::cef_thread_id_t::TID_UI.into(), Some(&mut task));
}

fn post_drag_drop_script_event<T: UserEvent>(
  context: Context<T>,
  window_id: WindowId,
  webview_id: u32,
  target: DragDropEventTarget,
  drag_drop_state: Arc<Mutex<DragDropState>>,
  event: DragDropScriptEvent,
) {
  let mut task = DragDropScriptEventTask::new(
    context,
    window_id,
    webview_id,
    target,
    drag_drop_state,
    event,
  );
  cef::post_task(sys::cef_thread_id_t::TID_UI.into(), Some(&mut task));
}

fn send_window_event<T: UserEvent>(
  window_id: WindowId,
  windows: &Arc<RefCell<HashMap<WindowId, AppWindow>>>,
  callback: &RunEventCallback<T>,
  event: WindowEvent,
) {
  let Ok(windows_ref) = windows.try_borrow() else {
    // post task to run later - windows currently mutably borrowed
    // happens usually on reparent or destroy when there's a focus change event
    let mut task =
      WindowEventTask::new(window_id, windows.clone(), callback.clone(), event.clone());

    cef::post_task(sys::cef_thread_id_t::TID_UI.into(), Some(&mut task));
    return;
  };

  if let Some(w) = windows_ref.get(&window_id) {
    let label = w.label.clone();
    let window_event_listeners = w.window_event_listeners.clone();

    drop(windows_ref);

    in_callback(|| {
      let listeners = window_event_listeners.lock().unwrap();
      let handlers: Vec<_> = listeners.values().collect();
      for handler in handlers.iter() {
        handler(&event);
      }
    });

    in_callback(|| (callback.borrow())(RunEvent::WindowEvent { label, event }));
  }
}

fn send_webview_event<T: UserEvent>(
  context: &Context<T>,
  window_id: WindowId,
  webview_id: u32,
  event: WebviewEvent,
) {
  let Ok(windows_ref) = context.windows.try_borrow() else {
    let mut task = WebviewEventTask::new(context.clone(), window_id, webview_id, event.clone());
    cef::post_task(sys::cef_thread_id_t::TID_UI.into(), Some(&mut task));
    return;
  };

  let Some(w) = windows_ref.get(&window_id) else {
    return;
  };

  let listeners = w.webview_event_listeners.clone();
  drop(windows_ref);

  let Some(webview_listeners) = listeners.lock().unwrap().get(&webview_id).cloned() else {
    return;
  };

  in_callback(|| {
    let listeners = webview_listeners.lock().unwrap();
    let handlers: Vec<_> = listeners.values().collect();
    for handler in handlers.iter() {
      handler(&event);
    }
  });
}

fn send_drag_drop_event<T: UserEvent>(
  context: &Context<T>,
  window_id: WindowId,
  webview_id: u32,
  target: DragDropEventTarget,
  event: DragDropEvent,
) {
  match target {
    DragDropEventTarget::Window => send_window_event(
      window_id,
      &context.windows,
      &context.callback,
      WindowEvent::DragDrop(event),
    ),
    DragDropEventTarget::Webview => send_webview_event(
      context,
      window_id,
      webview_id,
      WebviewEvent::DragDrop(event),
    ),
  }
}

fn handle_drag_drop_script_event<T: UserEvent>(
  context: &Context<T>,
  window_id: WindowId,
  webview_id: u32,
  target: DragDropEventTarget,
  drag_drop_state: Arc<Mutex<DragDropState>>,
  script_event: DragDropScriptEvent,
) {
  let position = PhysicalPosition::new(script_event.x, script_event.y);
  let event = {
    let mut state = drag_drop_state.lock().unwrap();
    if !state.native_entered {
      return;
    }

    match script_event.kind.as_str() {
      "enter" => {
        if state.entered {
          return;
        }

        let Some(paths) = state.paths.clone() else {
          return;
        };
        state.entered = true;
        Some(DragDropEvent::Enter { paths, position })
      }
      "over" => {
        if state.entered {
          Some(DragDropEvent::Over { position })
        } else {
          None
        }
      }
      "drop" => {
        let paths = state.entered.then(|| state.paths.take()).flatten();
        state.entered = false;
        state.native_entered = false;
        paths.map(|paths| DragDropEvent::Drop { paths, position })
      }
      "leave" => {
        state.native_entered = false;
        state.paths = None;

        if state.entered {
          state.entered = false;
          Some(DragDropEvent::Leave)
        } else {
          None
        }
      }
      _ => None,
    }
  };

  if let Some(event) = event {
    send_drag_drop_event(context, window_id, webview_id, target, event);
  }
}

fn on_close_requested<T: UserEvent>(window_id: WindowId, context: &Context<T>) {
  // Skip `CloseRequested` while tearing down — the embedder has already been
  // told `ExitRequested -> Exit`. We still need to drive the close so that
  // CEF can run its `OnBeforeClose` lifecycle.
  if context.is_shutting_down.load(Ordering::SeqCst) {
    on_window_close(window_id, &context.windows);
    return;
  }

  let (tx, rx) = channel();
  let event = WindowEvent::CloseRequested { signal_tx: tx };

  send_window_event(
    window_id,
    &context.windows,
    &context.callback,
    event.clone(),
  );

  let prevent = rx.try_recv().unwrap_or_default();

  if !prevent {
    on_window_close(window_id, &context.windows);
  }
}

// Collects the browser hosts from the webviews.
fn collect_hosts(webviews: &[AppWebview]) -> Vec<BrowserHost> {
  webviews
    .iter()
    .filter_map(|webview| webview.inner.browser().and_then(|b| b.host()))
    .collect()
}

/// Apply the assistive-technology accessibility state to every live browser
/// host — the runtime-wide equivalent of cefclient's `enableAccessibility:`,
/// which only toggled the active browser.
#[cfg(target_os = "macos")]
pub fn set_browsers_accessibility_state<T: UserEvent>(context: &Context<T>, enabled: bool) {
  let state = if enabled {
    State::ENABLED
  } else {
    State::DISABLED
  };
  for app_window in context.windows.borrow().values() {
    for host in collect_hosts(&app_window.webviews) {
      host.set_accessibility_state(state);
    }
  }
}

/// Tear-down equivalent of cefclient's `RootWindowManager::CloseAllWindows`:
/// drives every remaining window through the normal CEF lifecycle so each
/// browser sees `OnBeforeClose`.
///
/// Each call goes `force_close → can_close → close_window_browsers →
/// on_before_close → on_window_destroyed`. While `Context::is_shutting_down`
/// is set, those callbacks suppress their public events so the embedder only
/// sees the `ExitRequested -> Exit` pair we already emitted for the direct
/// exit.
pub fn close_all_windows(windows: &Arc<RefCell<HashMap<WindowId, AppWindow>>>) {
  let window_ids: Vec<_> = windows.borrow().keys().copied().collect();
  for window_id in window_ids {
    on_window_close(window_id, windows);
  }
}

/// Close all browsers for a specific window.
///
/// Returns true if all browsers were closed.
fn close_window_browsers(
  window_id: WindowId,
  windows: &Arc<RefCell<HashMap<WindowId, AppWindow>>>,
) -> bool {
  let hosts = {
    let windows_ref = windows.borrow();
    let Some(app_window) = windows_ref.get(&window_id) else {
      return true;
    };
    collect_hosts(&app_window.webviews)
  };

  let mut all_closed = true;
  for host in hosts {
    host.close_dev_tools();
    if host.try_close_browser() != 1 {
      all_closed = false;
    }
  }

  all_closed
}

fn on_window_close(window_id: WindowId, windows: &Arc<RefCell<HashMap<WindowId, AppWindow>>>) {
  let cef_window = {
    let windows_ref = windows.borrow();
    let Some(app_window) = windows_ref.get(&window_id) else {
      return;
    };
    app_window.force_close.store(true, Ordering::SeqCst);
    app_window.window.clone()
  };

  cef_window.close();
}

fn on_window_destroyed<T: UserEvent>(window_id: WindowId, context: &Context<T>) {
  if context.windows.borrow().get(&window_id).is_none() {
    return;
  }

  let is_shutting_down = context.is_shutting_down.load(Ordering::SeqCst);

  let event = WindowEvent::Destroyed;
  send_window_event(window_id, &context.windows, &context.callback, event);

  let removed_window = {
    let mut guard = context.windows.borrow_mut();
    guard.remove(&window_id)
  };

  if let Some(ref app_window) = removed_window {
    let mut registry = context.scheme_handler_registry.lock().unwrap();
    for webview in &app_window.webviews {
      let browser_id = *webview.browser_id.borrow();
      for scheme in webview.uri_scheme_protocols.keys() {
        registry.remove(&(browser_id, scheme.clone()));
      }
    }
  }

  drop(removed_window);

  let is_empty = context.windows.borrow().is_empty();
  // Window-close exit path: only emit the terminal `ExitRequested -> Exit`
  // pair when this is the last window being destroyed naturally. If we're
  // already shutting down (direct-exit tear-down or a previously approved
  // exit) the events have already been delivered.
  if is_empty && !is_shutting_down {
    let (tx, rx) = channel();
    (context.callback.borrow())(RunEvent::ExitRequested { code: None, tx });

    let recv = rx.try_recv();
    let should_prevent = matches!(recv, Ok(ExitRequestedEventAction::Prevent));

    if !should_prevent {
      context.is_shutting_down.store(true, Ordering::SeqCst);
      (context.callback.borrow())(RunEvent::Exit);
    }
  }
}

pub(crate) fn create_webview<T: UserEvent>(
  kind: WebviewKind,
  context: &Context<T>,
  window_id: WindowId,
  webview_id: u32,
  pending: PendingWebview<T, CefRuntime<T>>,
) {
  let PendingWebview {
    label,
    opener: _,
    mut webview_attributes,
    platform_specific_attributes,
    uri_scheme_protocols,
    ipc_handler,
    navigation_handler,
    new_window_handler,
    document_title_changed_handler,
    address_changed_handler,
    url,
    // Consumed by tauri core itself (wrapped into the `tauri` URI scheme
    // protocol before the pending webview reaches the runtime), so there is
    // nothing to handle here — tauri-runtime-wry ignores it as well.
    web_resource_request_handler: _,
    mut on_page_load_handler,
    download_handler,
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    on_web_content_process_terminate_handler,
  } = pending;

  #[cfg(target_os = "macos")]
  let web_content_process_terminate_handler = on_web_content_process_terminate_handler
    .map(|handler| Arc::from(handler) as Arc<dyn Fn() + Send>);
  #[cfg(not(target_os = "macos"))]
  let web_content_process_terminate_handler: Option<Arc<dyn Fn() + Send>> = None;

  let address_changed_handler = address_changed_handler
    .map(|h| Arc::new(move |url: &url::Url| h(url)) as Arc<AddressChangedHandler>);

  let window = {
    let windows = context.windows.borrow();
    let Some(app_window) = windows.get(&window_id) else {
      eprintln!("Window {window_id:?} not found when creating webview");
      return;
    };
    app_window.window.clone()
  };

  let drag_drop_handler_enabled = webview_attributes.drag_drop_handler_enabled;
  let initialization_scripts =
    initialization_scripts_from_webview_attributes(&mut webview_attributes);

  let on_page_load_handler = on_page_load_handler.take().map(Arc::from);
  let document_title_changed_handler = document_title_changed_handler.map(Arc::from);
  let navigation_handler = navigation_handler.map(Arc::from);
  let new_window_handler = new_window_handler.map(Arc::from);
  let ipc_handler: Option<Arc<IpcHandler<T>>> = ipc_handler.map(Arc::from);

  let devtools_enabled = (cfg!(debug_assertions) || cfg!(feature = "devtools"))
    && webview_attributes.devtools.unwrap_or(true);

  let custom_protocol_scheme = if webview_attributes.use_https_scheme {
    "https"
  } else {
    "http"
  }
  .to_string();

  let custom_schemes = uri_scheme_protocols.keys().cloned().collect::<Vec<_>>();
  let custom_scheme_domain_names = custom_schemes
    .iter()
    .map(|scheme| format!("{scheme}.localhost"))
    .collect::<Vec<_>>();

  let initial_url = url.clone();
  let url = CefString::from(INITIAL_LOAD_URL);
  let drag_drop_state = Arc::new(Mutex::new(DragDropState::default()));
  let drag_drop_event_target = if kind == WebviewKind::WindowContent {
    DragDropEventTarget::Window
  } else {
    DragDropEventTarget::Webview
  };

  let client = BrowserClient::new(
    window_id,
    webview_id,
    label.clone(),
    drag_drop_event_target,
    drag_drop_handler_enabled,
    drag_drop_state,
    ipc_handler,
    on_page_load_handler,
    document_title_changed_handler,
    navigation_handler,
    address_changed_handler,
    new_window_handler,
    download_handler,
    web_content_process_terminate_handler,
    devtools_enabled,
    context.clone(),
    runtime_context(context),
    None,
  );

  let uri_scheme_protocols: HashMap<String, Arc<Box<UriSchemeProtocolHandler>>> =
    uri_scheme_protocols
      .into_iter()
      .map(|(k, v)| (k, Arc::new(v)))
      .collect();

  let runtime_style = platform_specific_attributes
    .iter()
    .map(|attr| match attr {
      WebviewAtribute::RuntimeStyle { style } => *style,
    })
    .next()
    .unwrap_or(if matches!(kind, WebviewKind::WindowChild) {
      CefRuntimeStyle::Alloy
    } else {
      CefRuntimeStyle::Chrome
    });
  let cef_runtime_style: RuntimeStyle = match runtime_style {
    CefRuntimeStyle::Alloy => cef_runtime_style_t::CEF_RUNTIME_STYLE_ALLOY.into(),
    CefRuntimeStyle::Chrome => cef_runtime_style_t::CEF_RUNTIME_STYLE_CHROME.into(),
  };

  let window_theme = resolve_window_theme(
    context,
    context
      .windows
      .borrow()
      .get(&window_id)
      .and_then(|w| w.attributes.borrow().theme),
  );

  let webview_attributes = Arc::new(RefCell::new(webview_attributes));

  // Browser creation is deferred to this continuation, which runs on the CEF
  // UI thread once the request context's underlying Chromium `Profile` is
  // initialized. Calling `browser_view_create` /
  // `browser_host_create_browser_sync` synchronously after
  // `request_context_create_context` would fail
  // `CefRequestContextImpl::VerifyBrowserContext()` whenever the per-webview
  // cache_path triggers `ChromeBrowserContext`'s asynchronous
  // `CreateProfileAsync` branch (i.e., any non-default `data_directory`).
  //
  // We then pump the message loop after returning from the call below until
  // the continuation has finished, so that the function appears synchronous
  // to the runtime: any operation the caller queues against the new webview
  // (`open_devtools`, `on_dev_tools_protocol`, ...) is guaranteed to find it.
  let (init_done, on_initialized) = deferred_init_continuation({
    let context = context.clone();
    let webview_attributes = webview_attributes.clone();
    let initialization_scripts = initialization_scripts.clone();
    let custom_protocol_scheme = custom_protocol_scheme.clone();
    let mut client = client;
    move |mut request_context| {
      apply_request_context_theme_scheme(request_context.as_ref(), window_theme);

      let browser_settings = browser_settings_from_webview_attributes(&webview_attributes.borrow());

      let bounds = webview_attributes.borrow().bounds.map(|b| {
        let device_scale_factor = window
          .display()
          .map(|d| d.device_scale_factor() as f64)
          .unwrap_or(1.0);
        rect_to_cef(b, device_scale_factor)
      });

      let window_handle = window.window_handle();

      if kind == WebviewKind::WindowChild {
        #[cfg(target_os = "macos")]
        let window_handle = ensure_valid_content_view(window_handle);

        let mut window_info = cef::WindowInfo::default().set_as_child(
          window_handle,
          bounds.as_ref().unwrap_or(&cef::Rect::default()),
        );
        window_info.runtime_style = cef_runtime_style;

        let Some(browser_host) = browser_host_create_browser_sync(
          Some(&window_info),
          Some(&mut client),
          Some(&url),
          Some(&browser_settings),
          Option::<&mut DictionaryValue>::None,
          request_context.as_mut(),
        ) else {
          eprintln!("Failed to create browser");
          return;
        };
        let browser_id_val = browser_host.identifier();
        {
          let mut registry = context.scheme_handler_registry.lock().unwrap();
          for (scheme, handler) in &uri_scheme_protocols {
            registry.insert(
              (browser_id_val, scheme.clone()),
              (
                label.clone(),
                handler.clone(),
                initialization_scripts.clone(),
              ),
            );
          }
        }

        let devtools_protocol_handlers = Arc::new(Mutex::new(Vec::<
          Arc<dyn Fn(crate::DevToolsProtocol) + Send + Sync>,
        >::new()));
        let pending_initial_loads = Arc::new(Mutex::new(HashMap::new()));
        let devtools_observer_registration = Arc::new(Mutex::new(add_dev_tools_observer(
          &browser_host,
          devtools_protocol_handlers.clone(),
          pending_initial_loads.clone(),
        )));

        load_initial_url_after_registering_initialization_scripts(
          &browser_host,
          &initialization_scripts,
          &custom_protocol_scheme,
          &custom_scheme_domain_names,
          &initial_url,
          &pending_initial_loads,
        );

        // On Windows, set the browser window to be topmost to esnure correct z-order
        #[cfg(windows)]
        set_browser_on_top(&browser_host);

        let browser = CefWebview::Browser(browser_host);

        browser.set_bounds(bounds.as_ref());

        // On Linux, explicitly set parent after creation as set_as_child may not work correctly
        #[cfg(target_os = "linux")]
        {
          // Try to set parent - if window handle isn't available yet, this will be a no-op
          // but the browser should become visible once the handle is available
          browser.set_parent(&window);
          // Ensure browser is visible after setting parent
          browser.set_visible(1);
          // Set bounds again after reparenting to ensure correct size
          browser.set_bounds(bounds.as_ref());
        }

        let auto_resize = webview_attributes.borrow().auto_resize;
        let initial_bounds_ratio = if auto_resize {
          Some(webview_bounds_ratio(&window, bounds.clone(), &browser))
        } else {
          None
        };

        context
          .windows
          .borrow_mut()
          .get_mut(&window_id)
          .unwrap()
          .webviews
          .push(AppWebview {
            label,
            webview_id,
            browser_id: Arc::new(RefCell::new(browser_id_val)),
            bounds: Arc::new(Mutex::new(initial_bounds_ratio)),
            inner: browser,
            devtools_enabled,
            uri_scheme_protocols: Arc::new(uri_scheme_protocols),
            initialization_scripts,
            devtools_protocol_handlers,
            devtools_observer_registration,
            webview_attributes,
          });
      } else {
        let browser_id = Arc::new(RefCell::new(0));
        let uri_scheme_protocols = Arc::new(uri_scheme_protocols);
        let devtools_protocol_handlers = Arc::new(Mutex::new(Vec::<
          Arc<dyn Fn(crate::DevToolsProtocol) + Send + Sync>,
        >::new()));
        let devtools_observer_registration = Arc::new(Mutex::new(None));
        let pending_initial_loads = Arc::new(Mutex::new(HashMap::new()));

        #[allow(clippy::unnecessary_find_map)]
        let mut browser_view_delegate = BrowserViewDelegateImpl::new(
          browser_id.clone(),
          runtime_style,
          context.scheme_handler_registry.clone(),
          label.clone(),
          uri_scheme_protocols.clone(),
          initialization_scripts.clone(),
          custom_protocol_scheme.clone(),
          custom_scheme_domain_names.clone(),
          initial_url.clone(),
          pending_initial_loads,
          devtools_protocol_handlers.clone(),
          devtools_observer_registration.clone(),
          webview_attributes.clone(),
        );

        let browser_view = browser_view_create(
          Some(&mut client),
          Some(&url),
          Some(&browser_settings),
          Option::<&mut DictionaryValue>::None,
          request_context.as_mut(),
          Some(&mut browser_view_delegate),
        )
        .expect("Failed to create browser view");

        let browser_webview = CefWebview::BrowserView(browser_view.clone());

        window.add_child_view(Some(&mut View::from(&browser_view)));

        context
          .windows
          .borrow_mut()
          .get_mut(&window_id)
          .unwrap()
          .webviews
          .push(AppWebview {
            inner: browser_webview,
            label,
            webview_id,
            browser_id,
            bounds: Arc::new(Mutex::new(None)),
            devtools_enabled,
            uri_scheme_protocols,
            initialization_scripts,
            devtools_protocol_handlers,
            devtools_observer_registration,
            webview_attributes,
          });
      }
    }
  });

  request_context_from_webview_attributes(
    context,
    &webview_attributes.borrow(),
    &custom_schemes,
    &custom_protocol_scheme,
    on_initialized,
  );

  wait_for_deferred_init(&init_done);
}

#[cfg(windows)]
fn set_browser_on_top(browser: &cef::Browser) {
  use windows::Win32::Foundation::HWND;
  use windows::Win32::UI::WindowsAndMessaging::*;

  let Some(host) = browser.host() else {
    return;
  };

  let hwnd = HWND(host.window_handle().0 as _);

  let _ = unsafe {
    SetWindowPos(
      hwnd,
      Some(HWND_TOP),
      0,
      0,
      0,
      0,
      SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
    )
  };
}

// there is some race condition on CEF that causes the app loading to fail
// when there is a network service crash
// "[85296:47750637:0127/131203.017395:ERROR:content/browser/network_service_instance_impl.cc:610] Network service crashed or was terminated, restarting service."
// we check the app URL for a while until it actually loads the initial URL
fn check_and_reload_if_blank(browser: cef::Browser, initial_url: String) {
  if initial_url == "about:blank" {
    return;
  }

  std::thread::spawn(move || {
    std::thread::sleep(std::time::Duration::from_secs(1));

    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(5);
    let check_interval = std::time::Duration::from_millis(100);

    while start_time.elapsed() < timeout {
      if let Some(frame) = browser.main_frame() {
        let url = frame.url();
        let current_url = cef::CefString::from(&url).to_string();
        if current_url.is_empty() || current_url == "about:blank" {
          frame.load_url(Some(&cef::CefString::from(initial_url.as_str())));
          // Continue checking in case it loads about:blank again
        } else {
          // URL has changed to something else (not about:blank), we can stop checking
          return;
        }
      }
      std::thread::sleep(check_interval);
    }
  });
}

fn webview_bounds_ratio(
  window: &cef::Window,
  webview_bounds: Option<cef::Rect>,
  browser: &CefWebview,
) -> crate::WebviewBounds {
  #[cfg(not(windows))]
  let window_size = {
    let window_bounds = window.bounds();
    LogicalSize::new(window_bounds.width as u32, window_bounds.height as u32)
  };

  // On Windows, CEF's window bounds is the outer size not the inner size.
  #[cfg(windows)]
  let window_size = crate::utils::windows::inner_size(window.window_handle());

  let ob = webview_bounds.unwrap_or_else(|| browser.bounds());

  crate::WebviewBounds {
    x_rate: ob.x as f32 / window_size.width as f32,
    y_rate: ob.y as f32 / window_size.height as f32,
    width_rate: ob.width as f32 / window_size.width as f32,
    height_rate: ob.height as f32 / window_size.height as f32,
  }
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
/// - `zoom_hotkeys_enabled`: handled by Chromium's accelerator table, not a
///   browser setting.
///
/// `proxy_url` is handled separately via the request context preference (see
/// [`apply_proxy`]).
fn browser_settings_from_webview_attributes(
  webview_attributes: &WebviewAttributes,
) -> BrowserSettings {
  BrowserSettings {
    javascript: State::from(if webview_attributes.javascript_disabled {
      sys::cef_state_t::STATE_DISABLED
    } else {
      sys::cef_state_t::STATE_ENABLED
    }),
    javascript_access_clipboard: State::from(if webview_attributes.clipboard {
      sys::cef_state_t::STATE_ENABLED
    } else {
      sys::cef_state_t::STATE_DISABLED
    }),
    background_color: webview_attributes
      .background_color
      .map(color_to_cef_argb)
      .unwrap_or(0),
    ..Default::default()
  }
}

/// Resolves a CEF-compatible cache path for a per-webview request context.
///
/// CEF requires `RequestContextSettings.cache_path` to be either empty (which
/// puts the context in incognito mode) or an absolute path that is equal to,
/// or a child directory of, `Settings.root_cache_path` (which defaults to
/// `Settings.cache_path` when not set explicitly). Any value outside of that
/// root makes `request_context_create_context` (and downstream browser
/// creation) fail.
///
/// To support an arbitrary [`WebviewAttributes::data_directory`] while
/// honoring this constraint we:
///
/// * use the requested path directly when it already lives under the global
///   cache root, so callers that opt in to a path under the app cache get the
///   exact location they asked for;
/// * join relative paths without parent (`..`) components onto the root cache
///   path (typical short labels); and
/// * otherwise derive a stable direct child folder under `<root>/<hash>` from
///   the requested path, preserving isolation between webviews. Distinct
///   `data_directory` values produce distinct profiles, and the same value
///   maps to the same on-disk profile across runs.
fn resolve_request_context_cache_path(global_cache_path: &Path, data_directory: &Path) -> PathBuf {
  if data_directory.is_absolute() {
    if data_directory.starts_with(global_cache_path) {
      return data_directory.to_path_buf();
    } else {
      log::warn!(
        "data directory is not a child of the global cache path, we will derive a profile hash from it"
      );
    }
  } else if !data_directory
    .components()
    .any(|c| matches!(c, Component::ParentDir))
  {
    return global_cache_path.join(data_directory);
  } else {
    log::warn!(
      "data directory is a relative path with parent components, we will derive a profile hash from it"
    );
  }

  let mut hasher = Sha256::new();
  hasher.update(data_directory.as_os_str().as_encoded_bytes());
  let hash = hasher.finalize();
  let suffix = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&hash[..16]);
  let path = global_cache_path.join(format!("Profile-{suffix}"));
  log::info!(
    "derived profile hash from data directory: {suffix}, cache path: {}",
    path.display()
  );
  path
}

/// Continuation invoked on the CEF UI thread once the request context's
/// underlying browser context has finished asynchronous initialization.
///
/// Receives a fresh handle to the same [`RequestContext`] that was created in
/// [`request_context_from_webview_attributes`], so the continuation can pass
/// it to `browser_view_create` / `browser_host_create_browser_sync` knowing
/// that `VerifyBrowserContext()` will succeed.
type RequestContextInitContinuation = Box<dyn FnOnce(Option<RequestContext>) + 'static>;

/// Wraps a deferred-init continuation so that it always flips a shared
/// completion flag when it exits, regardless of how it exits (normal return,
/// early `return` on browser-create failure, or panic).
///
/// Returns the completion flag plus the wrapped continuation.
fn deferred_init_continuation<F>(work: F) -> (Arc<AtomicBool>, RequestContextInitContinuation)
where
  F: FnOnce(Option<RequestContext>) + 'static,
{
  struct Guard(Arc<AtomicBool>);
  impl Drop for Guard {
    fn drop(&mut self) {
      self.0.store(true, Ordering::SeqCst);
    }
  }

  let flag = Arc::new(AtomicBool::new(false));
  let guard = Guard(flag.clone());
  let wrapped: RequestContextInitContinuation = Box::new(move |request_context| {
    let _guard = guard;
    work(request_context);
  });
  (flag, wrapped)
}

/// Block the calling thread until `flag` is `true`.
///
/// Browser creation goes through `RequestContextHandler::on_request_context_initialized`,
/// which CEF always dispatches via `CEF_POST_TASK(CEF_UIT, ...)`. Tauri runs
/// CEF with an external message pump (see `cef::do_message_loop_work` in the
/// runtime's main loop), so the only way for that posted task to actually
/// execute is for someone on the CEF UI thread to keep pumping the loop.
///
/// Two cases:
///
/// 1. We're on the CEF UI thread (typical: app setup, [`SendMessageTask`]
///    dispatched messages, or inside a CEF callback like
///    `LifeSpanHandler::on_after_created` /
///    `RequestHandler::on_open_url_from_tab`). Pump the message loop ourselves
///    so the `OnRequestContextInitialized` task can run.
///
///    We must enable nestable tasks for the duration of the pump because we
///    may already be running inside another CEF task; without
///    `CefSetNestableTasksAllowed(true)` Chromium's `RunLoop::RunUntilIdle`
///    refuses to dispatch any task to the UI thread, the deferred init never
///    fires, and we'd spin here forever.
///
/// 2. We're on some other thread (e.g. a tokio IPC handler that called the
///    Tauri API directly without going through [`RuntimeContext::post_message`]).
///    The CEF UI thread is running its own pump and will pick up our queued
///    init task on its own; we just block here on a sleep loop until the flag
///    flips. We can't call `do_message_loop_work` from this thread - it
///    asserts on the init thread.
///
/// Spinning here keeps `create_webview` synchronous
/// from the caller's perspective: the function does not return until the
/// browser exists in `context.windows`, so any subsequent dispatcher call
/// (e.g. `webview.open_devtools()`, `webview.on_dev_tools_protocol(...)`)
/// can find the webview.
fn wait_for_deferred_init(flag: &Arc<AtomicBool>) {
  let on_ui_thread = cef::currently_on(cef::sys::cef_thread_id_t::TID_UI.into()) != 0;

  if on_ui_thread {
    let _allow = AllowNestableTasks::enter();
    while !flag.load(Ordering::SeqCst) {
      cef::do_message_loop_work();
    }
  } else {
    while !flag.load(Ordering::SeqCst) {
      std::thread::sleep(std::time::Duration::from_millis(1));
    }
  }
}

/// RAII guard that scopes `CefSetNestableTasksAllowed(true)` for the current
/// CEF UI-thread call.
///
/// CEF requires balanced enable/disable calls and explicitly forbids
/// reentrancy at the C++ level (`CHECK(allowed != has_value())`). The guard
/// uses a thread-local depth counter so only the outermost
/// [`wait_for_deferred_init`] on this thread toggles the flag, which makes
/// nesting (e.g. an `on_initialized` continuation that creates another
/// webview) safe.
struct AllowNestableTasks;

impl AllowNestableTasks {
  fn enter() -> Self {
    NESTABLE_TASKS_DEPTH.with(|depth| {
      let current = depth.get();
      if current == 0 {
        cef::set_nestable_tasks_allowed(1);
      }
      depth.set(current + 1);
    });
    Self
  }
}

impl Drop for AllowNestableTasks {
  fn drop(&mut self) {
    NESTABLE_TASKS_DEPTH.with(|depth| {
      let current = depth.get();
      depth.set(current - 1);
      if current == 1 {
        cef::set_nestable_tasks_allowed(0);
      }
    });
  }
}

thread_local! {
  static NESTABLE_TASKS_DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

wrap_request_context_handler! {
  struct WebviewRequestContextHandler {
    on_initialized: Arc<Mutex<Option<RequestContextInitContinuation>>>,
  }

  impl RequestContextHandler {
    fn on_request_context_initialized(&self, request_context: Option<&mut RequestContext>) {
      let Some(callback) = self.on_initialized.lock().unwrap().take() else {
        return;
      };
      let request_context = request_context.map(|rc| rc.clone());
      callback(request_context);
    }
  }
}

/// Creates a per-webview [`RequestContext`], registers Tauri's custom URI
/// scheme handler factories on it, and arranges for `on_initialized` to fire
/// once the underlying Chromium `Profile` is fully created.
///
/// CEF only synchronously initializes the request context when its `cache_path`
/// equals `Settings.root_cache_path` (it then reuses the global "Default"
/// profile via `GetPrimaryUserProfile()`) or when the cache_path is empty
/// (off-the-record profile). Any other path (notably the per-`data_directory`
/// case used by Tauri) takes `ChromeBrowserContext::InitializeAsync`'s
/// `CreateProfileAsync` branch which finishes asynchronously. Calling
/// `browser_view_create` / `browser_host_create_browser_sync` synchronously
/// after `request_context_create_context` would then fail
/// `CefRequestContextImpl::VerifyBrowserContext()` and return a null browser.
///
/// Routing browser creation through `on_initialized` keeps a single code path
/// for every cache_path layout: CEF always dispatches the callback through
/// `CEF_POST_TASK(CEF_UIT, ...)`, so even the synchronous-init cases are
/// handled by the same continuation.
///
/// Scheme handler factories are registered here, synchronously after
/// `request_context_create_context` returns, and *before* the
/// `OnRequestContextInitialized` task that drives browser creation is
/// dispatched. `RegisterSchemeHandlerFactory` internally queues its work
/// behind the request context's initialization (`StoreOrTriggerInitCallback`
/// when the browser context is not yet initialized, or an immediate UI -> IO
/// hop otherwise), so by the time the browser finally issues its first
/// navigation against any of these schemes the factories have been wired up
/// on the IO thread.
/// Applies a fixed-server proxy to a request context via the Chromium `proxy`
/// preference. Must be called after the request context has initialized.
fn apply_proxy(request_context: &RequestContext, proxy_url: &url::Url) {
  use cef::{ImplDictionaryValue, ImplValue};

  let scheme = match proxy_url.scheme() {
    "socks5" | "socks5h" => "socks5",
    "socks4" | "socks4a" => "socks4",
    "https" => "https",
    _ => "http",
  };
  let Some(host) = proxy_url.host_str() else {
    log::warn!("ignoring proxy URL without a host: {proxy_url}");
    return;
  };
  let server = match proxy_url.port_or_known_default() {
    Some(port) => format!("{scheme}://{host}:{port}"),
    None => format!("{scheme}://{host}"),
  };

  let pref_name = "proxy";
  if request_context.can_set_preference(Some(&pref_name.into())) != 1 {
    log::warn!("the CEF request context does not allow setting the proxy preference");
    return;
  }

  // Build `{ "mode": "fixed_servers", "server": "<scheme>://<host>:<port>" }`.
  let Some(dict) = cef::dictionary_value_create() else {
    return;
  };
  dict.set_string(Some(&"mode".into()), Some(&"fixed_servers".into()));
  dict.set_string(Some(&"server".into()), Some(&server.as_str().into()));

  let Some(value) = cef::value_create() else {
    return;
  };
  let mut dict = dict;
  value.set_dictionary(Some(&mut dict));

  let mut value = value;
  if request_context.set_preference(Some(&pref_name.into()), Some(&mut value), None) != 1 {
    log::error!("failed to apply the proxy preference to the CEF request context");
  }
}

fn request_context_from_webview_attributes<T: UserEvent>(
  context: &Context<T>,
  webview_attributes: &WebviewAttributes,
  custom_schemes: &[String],
  custom_protocol_scheme: &str,
  on_initialized: RequestContextInitContinuation,
) -> Option<RequestContext> {
  let cache_path: CefStringUtf16 = if webview_attributes.incognito {
    CefStringUtf16::from("")
  } else if let Some(data_directory) = &webview_attributes.data_directory {
    let resolved = resolve_request_context_cache_path(&context.cache_path, data_directory);
    if let Err(error) = std::fs::create_dir_all(&resolved) {
      log::error!(
        "failed to create request context cache directory {}: {error}",
        resolved.display()
      );
    }
    CefStringUtf16::from(resolved.to_string_lossy().as_ref())
  } else {
    let global_context =
      request_context_get_global_context().expect("Failed to get global request context");
    // context.cache_path does not work here - global_context.cache_path() returns the proper profile path
    (&global_context.cache_path()).into()
  };

  let request_context_settings = RequestContextSettings {
    cache_path,
    ..Default::default()
  };

  // Holds a strong reference to the `RequestContext` until the
  // `on_request_context_initialized` callback fires. CEF keeps the underlying
  // C++ `CefRequestContextImpl` alive during async profile creation through
  // its own bound callbacks, but holding an explicit reference here guarantees
  // we don't race with reference-count releases on shutdown paths.
  let rc_holder: Arc<Mutex<Option<RequestContext>>> = Arc::new(Mutex::new(None));
  let proxy_url = webview_attributes.proxy_url.clone();
  let wrapped_callback: RequestContextInitContinuation = Box::new({
    let rc_holder = rc_holder.clone();
    move |rc| {
      // The proxy preference can only be set once the request context's
      // underlying profile has finished initializing, which is exactly what
      // this continuation signals.
      if let (Some(rc), Some(proxy_url)) = (rc.as_ref(), proxy_url.as_ref()) {
        apply_proxy(rc, proxy_url);
      }
      on_initialized(rc);
      let _released = rc_holder.lock().unwrap().take();
    }
  });

  let mut handler = WebviewRequestContextHandler::new(Arc::new(Mutex::new(Some(wrapped_callback))));
  let request_context =
    request_context_create_context(Some(&request_context_settings), Some(&mut handler));
  *rc_holder.lock().unwrap() = request_context.clone();

  if let Some(rc) = request_context.as_ref() {
    for custom_scheme in custom_schemes {
      rc.register_scheme_handler_factory(
        Some(&custom_protocol_scheme.into()),
        Some(&format!("{custom_scheme}.localhost").as_str().into()),
        Some(&mut request_handler::UriSchemeHandlerFactory::new(
          context.scheme_handler_registry.clone(),
          custom_scheme.clone(),
        )),
      );
    }
  }

  request_context
}

#[cfg(target_os = "macos")]
fn apply_titlebar_style(window: &cef::Window, style: TitleBarStyle, hidden_title: bool) {
  use objc2::rc::Retained;
  use objc2_app_kit::NSWindowTitleVisibility;
  use objc2_app_kit::{NSView, NSWindowStyleMask};

  let content_view = unsafe { Retained::<NSView>::retain(window.window_handle() as _) };
  let Some(content_view) = content_view else {
    return;
  };

  let Some(ns_window) = content_view.window() else {
    return;
  };

  let mut mask = ns_window.styleMask();

  match style {
    TitleBarStyle::Visible => {
      mask &= !NSWindowStyleMask::FullSizeContentView;
      ns_window.setTitlebarAppearsTransparent(false);
      ns_window.setStyleMask(mask);
    }
    TitleBarStyle::Transparent => {
      ns_window.setTitlebarAppearsTransparent(true);
      mask &= !NSWindowStyleMask::FullSizeContentView;
      ns_window.setStyleMask(mask);
    }
    TitleBarStyle::Overlay => {
      ns_window.setTitlebarAppearsTransparent(true);
      mask |= NSWindowStyleMask::FullSizeContentView;
      ns_window.setStyleMask(mask);
    }
    unknown => {
      eprintln!("unknown title bar style applied: {unknown}");
    }
  }

  if hidden_title {
    ns_window.setTitleVisibility(NSWindowTitleVisibility::Hidden);
  }
}

/// On macOS, if the window content view is CEF's default `BridgedContentView`,
/// and does not have the expected subviews, replace it with a generic `NSView`
/// to avoid interactivity issues.
///
/// Returns the new content view pointer, or the original window handle if no replacement was made.
///
/// Subsequent calls to this function are no-ops, since the content view has already
/// been replaced and is no longer a BridgedContentView.
///
/// SAFETY: Only call this function for Windows that are intended to host multiple webviews.
#[cfg(target_os = "macos")]
pub(crate) fn ensure_valid_content_view(
  window_handle: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
  use objc2::rc::Retained;
  use objc2::{MainThreadMarker, MainThreadOnly};
  use objc2_app_kit::NSView;

  let nsview = unsafe { Retained::<NSView>::retain(window_handle as _) };
  let nsview = nsview.expect("NSView is null");

  let class = nsview.class().name().to_string_lossy();
  let subviews = unsafe { nsview.subviews() };

  // Filter subviews to only those that are expected in a valid CEF content view,
  // which can only happen if a WebviewKind::WindowContent webview
  // has been created in it using CEF's window.add_child_view API.
  fn is_cef_view(subview: &Retained<NSView>) -> bool {
    let class = subview.class().name().to_string_lossy();
    class == "ViewsCompositorSuperview" || class == "WebContentsViewCocoa"
  }

  // If it's a BridgedContentView without the expected subviews,
  // replace it with a generic NSView to avoid interactivity issues.
  if class == "BridgedContentView" && subviews.iter().filter(is_cef_view).count() != 2 {
    let mtm = MainThreadMarker::new().expect("Not on main thread");

    // Create a new generic NSView
    let generic_nsview = NSView::alloc(mtm);
    let generic_nsview = unsafe { NSView::init(generic_nsview) };

    // Re-add subviews to the new generic NSView (excluding CEF's views)
    for subview in subviews.iter().filter(|v| !is_cef_view(v)) {
      unsafe { subview.removeFromSuperview() };
      unsafe { generic_nsview.addSubview(&subview) };
    }

    // Set the new generic NSView as the content view of the window
    let nswindow = nsview.window().expect("NSWindow is null");
    nswindow.setContentView(Some(&generic_nsview));

    // Return the new content view pointer
    return Retained::into_raw(generic_nsview) as *mut std::ffi::c_void;
  }

  // No replacement needed; return the original handle
  window_handle
}

#[cfg(target_os = "macos")]
fn apply_traffic_light_position(window: *mut std::ffi::c_void, position: &Position) {
  use objc2::msg_send;
  use objc2::rc::Retained;
  use objc2_app_kit::{NSView, NSWindowButton};

  let nsview = unsafe { Retained::<NSView>::retain(window as _) };
  let Some(nsview) = nsview else {
    return;
  };

  let Some(nswindow) = nsview.window() else {
    return;
  };

  let Some(close) = nswindow.standardWindowButton(NSWindowButton::CloseButton) else {
    return;
  };
  let Some(miniaturize) = nswindow.standardWindowButton(NSWindowButton::MiniaturizeButton) else {
    return;
  };
  let Some(zoom) = nswindow.standardWindowButton(NSWindowButton::ZoomButton) else {
    return;
  };

  let pos = position.to_logical::<f64>(nswindow.backingScaleFactor());
  let (x, y) = (pos.x, pos.y);

  let title_bar_container_view = unsafe { close.superview().unwrap().superview().unwrap() };

  let close_rect = NSView::frame(&close);
  let title_bar_frame_height = close_rect.size.height + y;
  let mut title_bar_rect = NSView::frame(&title_bar_container_view);
  title_bar_rect.size.height = title_bar_frame_height;
  title_bar_rect.origin.y = nswindow.frame().size.height - title_bar_frame_height;
  let _: () = unsafe { msg_send![&title_bar_container_view, setFrame: title_bar_rect] };

  let window_buttons = vec![close, miniaturize.clone(), zoom];
  let space_between = NSView::frame(&miniaturize).origin.x - close_rect.origin.x;

  for (i, button) in window_buttons.into_iter().enumerate() {
    let mut rect = NSView::frame(&button);
    rect.origin.x = x + (i as f64 * space_between);
    unsafe { button.setFrameOrigin(rect.origin) };
  }
}

#[cfg(target_os = "macos")]
pub fn set_application_visibility(visible: bool) {
  use objc2::MainThreadMarker;
  use objc2_app_kit::NSApp;

  let mtm = MainThreadMarker::new().expect("not on main thread");
  let app = NSApp(mtm);

  if visible {
    unsafe { app.unhide(None) };
  } else {
    app.hide(None);
  }
}
