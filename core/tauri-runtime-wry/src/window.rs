use std::cell::RefCell;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

use raw_window_handle::HasRawWindowHandle;
use tauri_runtime::dpi::{PhysicalPosition, PhysicalSize, Position, Size};
use tauri_runtime::menu::{Menu, MenuEvent, MenuHash, MenuId, MenuUpdate};
use tauri_runtime::webview::WebviewIpcHandler;
use tauri_runtime::window::{
  CursorIcon, DetachedWindow, FileDropEvent, JsEventListenerKey, PendingWindow, WindowEvent,
};
use tauri_runtime::{Error, ExitRequestedEventAction, Icon, RunEvent, UserEvent};
use tauri_utils::config::WindowConfig;
use tauri_utils::Theme;
#[cfg(windows)]
use webview2_com::FocusChangedEventHandler;
use wry::application::dpi::{
  LogicalPosition as WryLogicalPosition, LogicalSize as WryLogicalSize,
  PhysicalPosition as WryPhysicalPosition, PhysicalSize as WryPhysicalSize,
};
use wry::application::event::{Event, WindowEvent as WryWindowEvent};
use wry::application::event_loop::{ControlFlow, EventLoopWindowTarget};
use wry::application::menu::CustomMenuItem as WryCustomMenuItem;
use wry::application::monitor::MonitorHandle;
#[cfg(target_os = "macos")]
use wry::application::platform::macos::WindowBuilderExtMacOS;
#[cfg(target_os = "macos")]
use wry::application::platform::macos::WindowExtMacOS;
#[cfg(linuxy)]
use wry::application::platform::unix::{WindowBuilderExtUnix, WindowExtUnix};
use wry::application::window::{
  Fullscreen, Icon as WryWindowIcon, Theme as WryTheme, Window as WryWindow,
  WindowBuilder as WryWindowBuilder,
};
use wry::webview::{Url, WebContext, WebView, WebViewBuilder};
#[cfg(windows)]
use wry::{
  application::platform::windows::{WindowBuilderExtWindows, WindowExtWindows},
  webview::{WebViewBuilderExtWindows, WebviewExtWindows},
};

use crate::menu::{to_wry_menu, WindowMenuEventListenerStore};
use crate::{
  on_window_close, Context, FileDropHandler, IpcHandler, MenuEventListenerId, Result,
  WebContextStore, Webview, WebviewEvent, WebviewId, WebviewIdStore, WebviewMessage, Wry,
  WryDispatcher,
};
use crate::{wrappers::*, Message};

#[derive(Debug, Clone, Default)]
pub struct WindowBuilder {
  pub(crate) inner: WryWindowBuilder,
  pub(crate) center: bool,
  #[cfg(target_os = "macos")]
  pub(crate) tabbing_identifier: Option<String>,
  pub(crate) menu: Option<Menu>,
}

// SAFETY: this type is `Send` since `menu_items` are read only here
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for WindowBuilder {}

impl tauri_runtime::webview::WindowBuilderBase for WindowBuilder {}
impl tauri_runtime::webview::WindowBuilder for WindowBuilder {
  fn new() -> Self {
    Self::default().focused(true)
  }

  fn with_config(config: WindowConfig) -> Self {
    let mut window = WindowBuilder::new();

    #[cfg(target_os = "macos")]
    {
      window = window
        .hidden_title(config.hidden_title)
        .title_bar_style(config.title_bar_style);
      if let Some(identifier) = &config.tabbing_identifier {
        window = window.tabbing_identifier(identifier);
      }
    }

    #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
    {
      window = window.transparent(config.transparent);
    }
    #[cfg(all(
      target_os = "macos",
      not(feature = "macos-private-api"),
      debug_assertions
    ))]
    if config.transparent {
      eprintln!(
        "The window is set to be transparent but the `macos-private-api` is not enabled.
        This can be enabled via the `tauri.macOSPrivateApi` configuration property <https://tauri.app/docs/api/config#tauri.macOSPrivateApi>
      ");
    }

    #[cfg(linuxy)]
    {
      // Mouse event is disabled on Linux since sudden event bursts could block event loop.
      window.inner = window.inner.with_cursor_moved_event(false);
    }

    #[cfg(desktop)]
    {
      window = window
        .title(config.title.to_string())
        .inner_size(config.width, config.height)
        .visible(config.visible)
        .resizable(config.resizable)
        .fullscreen(config.fullscreen)
        .decorations(config.decorations)
        .maximized(config.maximized)
        .always_on_top(config.always_on_top)
        .content_protected(config.content_protected)
        .skip_taskbar(config.skip_taskbar)
        .theme(config.theme)
        .shadow(config.shadow);

      if let (Some(min_width), Some(min_height)) = (config.min_width, config.min_height) {
        window = window.min_inner_size(min_width, min_height);
      }
      if let (Some(max_width), Some(max_height)) = (config.max_width, config.max_height) {
        window = window.max_inner_size(max_width, max_height);
      }
      if let (Some(x), Some(y)) = (config.x, config.y) {
        window = window.position(x, y);
      }

      if config.center {
        window = window.center();
      }
    }

    window
  }

  fn menu(mut self, menu: Menu) -> Self {
    self.menu.replace(menu);
    self
  }

  fn center(mut self) -> Self {
    self.center = true;
    self
  }

  fn position(mut self, x: f64, y: f64) -> Self {
    self.inner = self.inner.with_position(WryLogicalPosition::new(x, y));
    self
  }

  fn inner_size(mut self, width: f64, height: f64) -> Self {
    self.inner = self
      .inner
      .with_inner_size(WryLogicalSize::new(width, height));
    self
  }

  fn min_inner_size(mut self, min_width: f64, min_height: f64) -> Self {
    self.inner = self
      .inner
      .with_min_inner_size(WryLogicalSize::new(min_width, min_height));
    self
  }

  fn max_inner_size(mut self, max_width: f64, max_height: f64) -> Self {
    self.inner = self
      .inner
      .with_max_inner_size(WryLogicalSize::new(max_width, max_height));
    self
  }

  fn resizable(mut self, resizable: bool) -> Self {
    self.inner = self.inner.with_resizable(resizable);
    self
  }

  fn title<S: Into<String>>(mut self, title: S) -> Self {
    self.inner = self.inner.with_title(title.into());
    self
  }

  fn fullscreen(mut self, fullscreen: bool) -> Self {
    self.inner = if fullscreen {
      self
        .inner
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
    } else {
      self.inner.with_fullscreen(None)
    };
    self
  }

  fn focused(mut self, focused: bool) -> Self {
    self.inner = self.inner.with_focused(focused);
    self
  }

  fn maximized(mut self, maximized: bool) -> Self {
    self.inner = self.inner.with_maximized(maximized);
    self
  }

  fn visible(mut self, visible: bool) -> Self {
    self.inner = self.inner.with_visible(visible);
    self
  }

  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  fn transparent(mut self, transparent: bool) -> Self {
    self.inner = self.inner.with_transparent(transparent);
    self
  }

  fn decorations(mut self, decorations: bool) -> Self {
    self.inner = self.inner.with_decorations(decorations);
    self
  }

  fn always_on_top(mut self, always_on_top: bool) -> Self {
    self.inner = self.inner.with_always_on_top(always_on_top);
    self
  }

  fn content_protected(mut self, protected: bool) -> Self {
    self.inner = self.inner.with_content_protection(protected);
    self
  }

  fn shadow(#[allow(unused_mut)] mut self, _enable: bool) -> Self {
    #[cfg(windows)]
    {
      self.inner = self.inner.with_undecorated_shadow(_enable);
    }
    #[cfg(target_os = "macos")]
    {
      self.inner = self.inner.with_has_shadow(_enable);
    }
    self
  }

  #[cfg(windows)]
  fn parent_window(mut self, parent: windows::Win32::Foundation::HWND) -> Self {
    self.inner = self.inner.with_parent_window(parent);
    self
  }

  #[cfg(target_os = "macos")]
  fn parent_window(mut self, parent: *mut std::ffi::c_void) -> Self {
    self.inner = self.inner.with_parent_window(parent);
    self
  }

  #[cfg(windows)]
  fn owner_window(mut self, owner: windows::Win32::Foundation::HWND) -> Self {
    self.inner = self.inner.with_owner_window(owner);
    self
  }

  #[cfg(target_os = "macos")]
  fn title_bar_style(mut self, style: TitleBarStyle) -> Self {
    match style {
      TitleBarStyle::Visible => {
        self.inner = self.inner.with_titlebar_transparent(false);
        // Fixes rendering issue when resizing window with devtools open (https://github.com/tauri-apps/tauri/issues/3914)
        self.inner = self.inner.with_fullsize_content_view(true);
      }
      TitleBarStyle::Transparent => {
        self.inner = self.inner.with_titlebar_transparent(true);
        self.inner = self.inner.with_fullsize_content_view(false);
      }
      TitleBarStyle::Overlay => {
        self.inner = self.inner.with_titlebar_transparent(true);
        self.inner = self.inner.with_fullsize_content_view(true);
      }
    }
    self
  }

  #[cfg(target_os = "macos")]
  fn hidden_title(mut self, hidden: bool) -> Self {
    self.inner = self.inner.with_title_hidden(hidden);
    self
  }

  #[cfg(target_os = "macos")]
  fn tabbing_identifier(mut self, identifier: &str) -> Self {
    self.inner = self.inner.with_tabbing_identifier(identifier);
    self.tabbing_identifier.replace(identifier.into());
    self
  }

  fn icon(mut self, icon: Icon) -> Result<Self> {
    self.inner = self
      .inner
      .with_window_icon(Some(WryIcon::try_from(icon)?.0));
    Ok(self)
  }

  #[cfg(any(windows, linuxy))]
  fn skip_taskbar(mut self, skip: bool) -> Self {
    self.inner = self.inner.with_skip_taskbar(skip);
    self
  }

  #[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
  fn skip_taskbar(self, _skip: bool) -> Self {
    self
  }

  #[allow(unused_variables, unused_mut)]
  fn theme(mut self, theme: Option<Theme>) -> Self {
    self.inner = self.inner.with_theme(if let Some(t) = theme {
      match t {
        Theme::Dark => Some(WryTheme::Dark),
        _ => Some(WryTheme::Light),
      }
    } else {
      None
    });

    self
  }

  fn has_icon(&self) -> bool {
    self.inner.window.window_icon.is_some()
  }

  fn get_menu(&self) -> Option<&Menu> {
    self.menu.as_ref()
  }
}

pub type WindowEventListenerId = u64;
pub type WindowEventListener = Box<dyn Fn(&WindowEvent) + Send>;
pub type WindowEventListenerStore = Arc<Mutex<HashMap<WindowEventListenerId, WindowEventListener>>>;

#[derive(Clone)]
pub enum WindowHandle {
  Webview {
    inner: Arc<WebView>,
    context_store: WebContextStore,
    // the key of the WebContext if it's not shared
    context_key: Option<PathBuf>,
  },
  Window(Arc<WryWindow>),
}

impl Drop for WindowHandle {
  fn drop(&mut self) {
    if let Self::Webview {
      inner,
      context_store,
      context_key,
    } = self
    {
      if Arc::get_mut(inner).is_some() {
        context_store.lock().unwrap().remove(context_key);
      }
    }
  }
}

impl fmt::Debug for WindowHandle {
  fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
    Ok(())
  }
}

impl Deref for WindowHandle {
  type Target = WryWindow;

  #[inline(always)]
  fn deref(&self) -> &WryWindow {
    match self {
      Self::Webview { inner, .. } => inner.window(),
      Self::Window(w) => w,
    }
  }
}

impl WindowHandle {
  fn inner_size(&self) -> WryPhysicalSize<u32> {
    match self {
      WindowHandle::Window(w) => w.inner_size(),
      WindowHandle::Webview { inner, .. } => inner.inner_size(),
    }
  }
}

pub struct Window {
  pub(crate) label: String,
  pub(crate) inner: Option<WindowHandle>,
  pub(crate) menu_items: Option<HashMap<u16, WryCustomMenuItem>>,
  pub(crate) window_event_listeners_store: WindowEventListenerStore,
  pub(crate) menu_event_listeners_store: WindowMenuEventListenerStore,
}

impl fmt::Debug for Window {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("WindowWrapper")
      .field("label", &self.label)
      .field("inner", &self.inner)
      .field("menu_items", &self.menu_items)
      .finish()
  }
}

impl Window {
  pub fn new<T: UserEvent>(
    window_id: WebviewId,
    event_loop: &EventLoopWindowTarget<Message<T>>,
    web_context_store: &WebContextStore,
    context: Context<T>,
    pending: PendingWindow<T, Wry<T>>,
  ) -> Result<Self> {
    #[allow(unused_mut)]
    let PendingWindow {
      webview_attributes,
      uri_scheme_protocols,
      mut window_builder,
      ipc_handler,
      label,
      url,
      menu_ids,
      js_event_listeners,
      #[cfg(target_os = "android")]
      on_webview_created,
      ..
    } = pending;
    let webview_id_map = context.webview_id_store.clone();
    #[cfg(windows)]
    let proxy = context.proxy.clone();

    let window_event_listeners = WindowEventListenerStore::default();

    #[cfg(windows)]
    {
      window_builder.inner = window_builder
        .inner
        .with_drag_and_drop(webview_attributes.file_drop_handler_enabled);
    }

    #[cfg(windows)]
    let window_theme = window_builder.inner.window.preferred_theme;

    #[cfg(target_os = "macos")]
    {
      if window_builder.tabbing_identifier.is_none()
        || window_builder.inner.window.transparent
        || !window_builder.inner.window.decorations
      {
        window_builder.inner = window_builder.inner.with_automatic_window_tabbing(false);
      }
    }

    let is_window_transparent = window_builder.inner.window.transparent;
    let menu_items = if let Some(menu) = window_builder.menu {
      let mut menu_items = HashMap::new();
      let menu = to_wry_menu(&mut menu_items, menu);
      window_builder.inner = window_builder.inner.with_menu(menu);
      Some(menu_items)
    } else {
      None
    };
    let window = window_builder.inner.build(event_loop).unwrap();

    webview_id_map.insert(window.id(), window_id);

    if window_builder.center {
      let _ = center_window(&window, window.inner_size());
    }
    let mut webview_builder = WebViewBuilder::new(window)
      .map_err(|e| Error::CreateWebview(Box::new(e)))?
      .with_url(&url)
      .unwrap() // safe to unwrap because we validate the URL beforehand
      .with_transparent(is_window_transparent)
      .with_accept_first_mouse(webview_attributes.accept_first_mouse);
    if webview_attributes.file_drop_handler_enabled {
      webview_builder = webview_builder
        .with_file_drop_handler(create_file_drop_handler(window_event_listeners.clone()));
    }
    if let Some(navigation_handler) = pending.navigation_handler {
      webview_builder = webview_builder.with_navigation_handler(move |url| {
        Url::parse(&url).map(&navigation_handler).unwrap_or(true)
      });
    }
    if let Some(user_agent) = webview_attributes.user_agent {
      webview_builder = webview_builder.with_user_agent(&user_agent);
    }

    #[cfg(windows)]
    if let Some(additional_browser_args) = webview_attributes.additional_browser_args {
      webview_builder = webview_builder.with_additional_browser_args(&additional_browser_args);
    }

    #[cfg(windows)]
    if let Some(theme) = window_theme {
      webview_builder = webview_builder.with_theme(match theme {
        WryTheme::Dark => wry::webview::Theme::Dark,
        WryTheme::Light => wry::webview::Theme::Light,
        _ => wry::webview::Theme::Light,
      });
    }

    let mut web_context = web_context_store.lock().expect("poisoned WebContext store");
    let is_first_context = web_context.is_empty();
    let automation_enabled = std::env::var("TAURI_AUTOMATION").as_deref() == Ok("true");
    let web_context_key = // force a unique WebContext when automation is false;
    // the context must be stored on the HashMap because it must outlive the WebView on macOS
    if automation_enabled {
      webview_attributes.data_directory.clone()
    } else {
      // random unique key
      Some(context.next_id().to_string().into())
    };
    let entry = web_context.entry(web_context_key.clone());
    let web_context = match entry {
      Occupied(occupied) => occupied.into_mut(),
      Vacant(vacant) => {
        let mut web_context = WebContext::new(webview_attributes.data_directory);
        web_context.set_allows_automation(if automation_enabled {
          is_first_context
        } else {
          false
        });
        vacant.insert(web_context)
      }
    };

    if let Some(handler) = ipc_handler {
      webview_builder = webview_builder.with_ipc_handler(create_ipc_handler(
        context,
        label.clone(),
        menu_ids,
        js_event_listeners,
        handler,
      ));
    }
    for (scheme, protocol) in uri_scheme_protocols {
      webview_builder = webview_builder.with_custom_protocol(scheme, move |wry_request| -> std::result::Result<wry::http::Response<std::borrow::Cow<[u8]>>, wry::Error> {
      protocol(&HttpRequestWrapper::from(wry_request).0)
        .map(|tauri_response| HttpResponseWrapper::from(tauri_response).0)
        .map_err(|_| wry::Error::InitScriptError)
    });
    }

    for script in webview_attributes.initialization_scripts {
      webview_builder = webview_builder.with_initialization_script(&script);
    }

    if webview_attributes.clipboard {
      webview_builder.webview.clipboard = true;
    }

    #[cfg(any(debug_assertions, feature = "devtools"))]
    {
      webview_builder = webview_builder.with_devtools(true);
    }

    #[cfg(target_os = "android")]
    {
      if let Some(on_webview_created) = on_webview_created {
        webview_builder = webview_builder.on_webview_created(move |ctx| {
          on_webview_created(tauri_runtime::window::CreationContext {
            env: ctx.env,
            activity: ctx.activity,
            webview: ctx.webview,
          })
        });
      }
    }

    let webview = webview_builder
      .with_web_context(web_context)
      .build()
      .map_err(|e| Error::CreateWebview(Box::new(e)))?;

    #[cfg(windows)]
    {
      let controller = webview.controller();
      let proxy_ = proxy.clone();
      let mut token = windows::Win32::System::WinRT::EventRegistrationToken::default();
      unsafe {
        controller.add_GotFocus(
          &FocusChangedEventHandler::create(Box::new(move |_, _| {
            let _ = proxy_.send_event(Message::Webview(
              window_id,
              WebviewMessage::WebviewEvent(WebviewEvent::Focused(true)),
            ));
            Ok(())
          })),
          &mut token,
        )
      }
      .unwrap();
      unsafe {
        controller.add_LostFocus(
          &FocusChangedEventHandler::create(Box::new(move |_, _| {
            let _ = proxy.send_event(Message::Webview(
              window_id,
              WebviewMessage::WebviewEvent(WebviewEvent::Focused(false)),
            ));
            Ok(())
          })),
          &mut token,
        )
      }
      .unwrap();
    }

    Ok(Self {
      label,
      inner: Some(WindowHandle::Webview {
        inner: Arc::new(webview),
        context_store: web_context_store.clone(),
        context_key: if automation_enabled {
          None
        } else {
          web_context_key
        },
      }),
      menu_items,
      window_event_listeners_store: window_event_listeners,
      menu_event_listeners_store: Default::default(),
    })
  }
}

/// Create a wry ipc handler from a tauri ipc handler.
fn create_ipc_handler<T: UserEvent>(
  context: Context<T>,
  label: String,
  menu_ids: Arc<Mutex<HashMap<MenuHash, MenuId>>>,
  js_event_listeners: Arc<Mutex<HashMap<JsEventListenerKey, HashSet<u64>>>>,
  handler: WebviewIpcHandler<T, Wry<T>>,
) -> Box<IpcHandler> {
  Box::new(move |window, request| {
    let webview_id = context.webview_id_store.get(&window.id()).unwrap();
    handler(
      DetachedWindow {
        dispatcher: WryDispatcher {
          webview_id,
          context: context.clone(),
        },
        label: label.clone(),
        menu_ids: menu_ids.clone(),
        js_event_listeners: js_event_listeners.clone(),
      },
      request,
    );
  })
}

/// Create a wry file drop handler.
fn create_file_drop_handler(
  window_event_listeners: WindowEventListenerStore,
) -> Box<FileDropHandler> {
  Box::new(move |_window, event| {
    let event: FileDropEvent = FileDropEventWrapper(event).into();
    let window_event = WindowEvent::FileDrop(event);
    let listeners_map = window_event_listeners.lock().unwrap();
    let has_listener = !listeners_map.is_empty();
    let handlers = listeners_map.values();
    for listener in handlers {
      listener(&window_event);
    }
    // block the default OS action on file drop if we had a listener
    has_listener
  })
}

pub enum WindowMessage {
  WithWebview(Box<dyn FnOnce(Webview) + Send>),
  AddWindowEventListener(WindowEventListenerId, Box<dyn Fn(&WindowEvent) + Send>),
  AddMenuEventListener(MenuEventListenerId, Box<dyn Fn(&MenuEvent) + Send>),
  // Devtools
  #[cfg(any(debug_assertions, feature = "devtools"))]
  OpenDevTools,
  #[cfg(any(debug_assertions, feature = "devtools"))]
  CloseDevTools,
  #[cfg(any(debug_assertions, feature = "devtools"))]
  IsDevToolsOpen(Sender<bool>),
  // Getters
  Url(Sender<Url>),
  ScaleFactor(Sender<f64>),
  InnerPosition(Sender<Result<PhysicalPosition<i32>>>),
  OuterPosition(Sender<Result<PhysicalPosition<i32>>>),
  InnerSize(Sender<PhysicalSize<u32>>),
  OuterSize(Sender<PhysicalSize<u32>>),
  IsFullscreen(Sender<bool>),
  IsMinimized(Sender<bool>),
  IsMaximized(Sender<bool>),
  IsDecorated(Sender<bool>),
  IsResizable(Sender<bool>),
  IsVisible(Sender<bool>),
  Title(Sender<String>),
  IsMenuVisible(Sender<bool>),
  CurrentMonitor(Sender<Option<MonitorHandle>>),
  PrimaryMonitor(Sender<Option<MonitorHandle>>),
  AvailableMonitors(Sender<Vec<MonitorHandle>>),
  #[cfg(linuxy)]
  GtkWindow(Sender<GtkWindow>),
  RawWindowHandle(Sender<RawWindowHandle>),
  Theme(Sender<Theme>),
  // Setters
  Center,
  RequestUserAttention(Option<UserAttentionTypeWrapper>),
  SetResizable(bool),
  SetTitle(String),
  Maximize,
  Unmaximize,
  Minimize,
  Unminimize,
  ShowMenu,
  HideMenu,
  Show,
  Hide,
  Close,
  SetDecorations(bool),
  SetShadow(bool),
  SetAlwaysOnTop(bool),
  SetContentProtected(bool),
  SetSize(Size),
  SetMinSize(Option<Size>),
  SetMaxSize(Option<Size>),
  SetPosition(Position),
  SetFullscreen(bool),
  SetFocus,
  SetIcon(WryWindowIcon),
  SetSkipTaskbar(bool),
  SetCursorGrab(bool),
  SetCursorVisible(bool),
  SetCursorIcon(CursorIcon),
  SetCursorPosition(Position),
  SetIgnoreCursorEvents(bool),
  DragWindow,
  UpdateMenuItem(u16, MenuUpdate),
  RequestRedraw,
}

pub fn handle_window_message(
  message: WindowMessage,
  id: u64,
  windows: &Arc<RefCell<HashMap<u64, Window>>>,
) {
  let w = windows.borrow().get(&id).map(|w| {
    (
      w.inner.clone(),
      w.window_event_listeners_store.clone(),
      w.menu_event_listeners_store.clone(),
    )
  });
  if let Some((Some(window), window_event_listeners, menu_event_listeners)) = w {
    match message {
      WindowMessage::WithWebview(f) => {
        if let WindowHandle::Webview { inner: w, .. } = &window {
          #[cfg(linuxy)]
          {
            use wry::webview::WebviewExtUnix;
            f(w.webview());
          }
          #[cfg(target_os = "macos")]
          {
            use wry::webview::WebviewExtMacOS;
            f(Webview {
              webview: w.webview(),
              manager: w.manager(),
              ns_window: w.ns_window(),
            });
          }
          #[cfg(target_os = "ios")]
          {
            use wry::application::platform::ios::WindowExtIOS;
            use wry::webview::WebviewExtIOS;

            f(Webview {
              webview: w.webview(),
              manager: w.manager(),
              view_controller: w.window().ui_view_controller() as cocoa::base::id,
            });
          }
          #[cfg(windows)]
          {
            f(Webview {
              controller: w.controller(),
            });
          }
          #[cfg(target_os = "android")]
          {
            f(w.handle())
          }
        }
      }

      WindowMessage::AddWindowEventListener(id, listener) => {
        window_event_listeners.lock().unwrap().insert(id, listener);
      }

      WindowMessage::AddMenuEventListener(id, listener) => {
        menu_event_listeners.lock().unwrap().insert(id, listener);
      }

      #[cfg(any(debug_assertions, feature = "devtools"))]
      WindowMessage::OpenDevTools => {
        if let WindowHandle::Webview { inner: w, .. } = &window {
          w.open_devtools();
        }
      }
      #[cfg(any(debug_assertions, feature = "devtools"))]
      WindowMessage::CloseDevTools => {
        if let WindowHandle::Webview { inner: w, .. } = &window {
          w.close_devtools();
        }
      }
      #[cfg(any(debug_assertions, feature = "devtools"))]
      WindowMessage::IsDevToolsOpen(tx) => {
        if let WindowHandle::Webview { inner: w, .. } = &window {
          tx.send(w.is_devtools_open()).unwrap();
        } else {
          tx.send(false).unwrap();
        }
      }
      // Getters
      WindowMessage::Url(tx) => {
        if let WindowHandle::Webview { inner: w, .. } = &window {
          tx.send(w.url()).unwrap();
        }
      }
      WindowMessage::ScaleFactor(tx) => tx.send(window.scale_factor()).unwrap(),
      WindowMessage::InnerPosition(tx) => tx
        .send(
          window
            .inner_position()
            .map(|p| PhysicalPositionWrapper(p).into())
            .map_err(|_| Error::FailedToSendMessage),
        )
        .unwrap(),
      WindowMessage::OuterPosition(tx) => tx
        .send(
          window
            .outer_position()
            .map(|p| PhysicalPositionWrapper(p).into())
            .map_err(|_| Error::FailedToSendMessage),
        )
        .unwrap(),
      WindowMessage::InnerSize(tx) => tx
        .send(PhysicalSizeWrapper(window.inner_size()).into())
        .unwrap(),
      WindowMessage::OuterSize(tx) => tx
        .send(PhysicalSizeWrapper(window.outer_size()).into())
        .unwrap(),
      WindowMessage::IsFullscreen(tx) => tx.send(window.fullscreen().is_some()).unwrap(),
      WindowMessage::IsMinimized(tx) => tx.send(window.is_minimized()).unwrap(),
      WindowMessage::IsMaximized(tx) => tx.send(window.is_maximized()).unwrap(),
      WindowMessage::IsDecorated(tx) => tx.send(window.is_decorated()).unwrap(),
      WindowMessage::IsResizable(tx) => tx.send(window.is_resizable()).unwrap(),
      WindowMessage::IsVisible(tx) => tx.send(window.is_visible()).unwrap(),
      WindowMessage::Title(tx) => tx.send(window.title()).unwrap(),
      WindowMessage::IsMenuVisible(tx) => tx.send(window.is_menu_visible()).unwrap(),
      WindowMessage::CurrentMonitor(tx) => tx.send(window.current_monitor()).unwrap(),
      WindowMessage::PrimaryMonitor(tx) => tx.send(window.primary_monitor()).unwrap(),
      WindowMessage::AvailableMonitors(tx) => {
        tx.send(window.available_monitors().collect()).unwrap()
      }
      #[cfg(linuxy)]
      WindowMessage::GtkWindow(tx) => tx.send(GtkWindow(window.gtk_window().clone())).unwrap(),
      WindowMessage::RawWindowHandle(tx) => tx
        .send(RawWindowHandle(window.raw_window_handle()))
        .unwrap(),
      WindowMessage::Theme(tx) => {
        tx.send(map_theme(&window.theme())).unwrap();
      }
      // Setters
      WindowMessage::Center => {
        let _ = center_window(&window, window.inner_size());
      }
      WindowMessage::RequestUserAttention(request_type) => {
        window.request_user_attention(request_type.map(|r| r.0));
      }
      WindowMessage::SetResizable(resizable) => window.set_resizable(resizable),
      WindowMessage::SetTitle(title) => window.set_title(&title),
      WindowMessage::Maximize => window.set_maximized(true),
      WindowMessage::Unmaximize => window.set_maximized(false),
      WindowMessage::Minimize => window.set_minimized(true),
      WindowMessage::Unminimize => window.set_minimized(false),
      WindowMessage::ShowMenu => window.show_menu(),
      WindowMessage::HideMenu => window.hide_menu(),
      WindowMessage::Show => window.set_visible(true),
      WindowMessage::Hide => window.set_visible(false),
      WindowMessage::Close => {
        panic!("cannot handle `WindowMessage::Close` on the main thread")
      }
      WindowMessage::SetDecorations(decorations) => window.set_decorations(decorations),
      WindowMessage::SetShadow(_enable) => {
        #[cfg(windows)]
        window.set_undecorated_shadow(_enable);
        #[cfg(target_os = "macos")]
        window.set_has_shadow(_enable);
      }
      WindowMessage::SetAlwaysOnTop(always_on_top) => window.set_always_on_top(always_on_top),
      WindowMessage::SetContentProtected(protected) => window.set_content_protection(protected),
      WindowMessage::SetSize(size) => {
        window.set_inner_size(SizeWrapper::from(size).0);
      }
      WindowMessage::SetMinSize(size) => {
        window.set_min_inner_size(size.map(|s| SizeWrapper::from(s).0));
      }
      WindowMessage::SetMaxSize(size) => {
        window.set_max_inner_size(size.map(|s| SizeWrapper::from(s).0));
      }
      WindowMessage::SetPosition(position) => {
        window.set_outer_position(PositionWrapper::from(position).0)
      }
      WindowMessage::SetFullscreen(fullscreen) => {
        if fullscreen {
          window.set_fullscreen(Some(Fullscreen::Borderless(None)))
        } else {
          window.set_fullscreen(None)
        }
      }
      WindowMessage::SetFocus => {
        window.set_focus();
      }
      WindowMessage::SetIcon(icon) => {
        window.set_window_icon(Some(icon));
      }
      #[allow(unused_variables)]
      WindowMessage::SetSkipTaskbar(skip) => {
        #[cfg(any(windows, linuxy))]
        window.set_skip_taskbar(skip);
      }
      WindowMessage::SetCursorGrab(grab) => {
        let _ = window.set_cursor_grab(grab);
      }
      WindowMessage::SetCursorVisible(visible) => {
        window.set_cursor_visible(visible);
      }
      WindowMessage::SetCursorIcon(icon) => {
        window.set_cursor_icon(CursorIconWrapper::from(icon).0);
      }
      WindowMessage::SetCursorPosition(position) => {
        let _ = window.set_cursor_position(PositionWrapper::from(position).0);
      }
      WindowMessage::SetIgnoreCursorEvents(ignore) => {
        let _ = window.set_ignore_cursor_events(ignore);
      }
      WindowMessage::DragWindow => {
        let _ = window.drag_window();
      }
      WindowMessage::UpdateMenuItem(_id, _update) => {
        // already handled
      }
      WindowMessage::RequestRedraw => {
        window.request_redraw();
      }
    }
  }
}

pub fn handle_window_event<'a, T: UserEvent>(
  event: Event<Message<T>>,
  windows: &Arc<RefCell<HashMap<u64, Window>>>,
  webview_id_store: &WebviewIdStore,
  callback: &'a mut (dyn FnMut(RunEvent<T>) + 'static),
  control_flow: &mut ControlFlow,
) {
  let dispatch_window_event = &mut |event: tauri_runtime::window::WindowEvent, id| {
    let windows = windows.borrow();
    let window = windows.get(&id);
    if let Some(window) = window {
      callback(RunEvent::WindowEvent {
        label: window.label.clone(),
        event: event.clone(),
      });

      let listeners = window.window_event_listeners_store.lock().unwrap();
      let handlers = listeners.values();
      for handler in handlers {
        handler(&event);
      }
    }
  };

  match event {
    Event::UserEvent(Message::Webview(id, WebviewMessage::WebviewEvent(event))) => {
      if let Some(event) = WindowEventWrapper::from(&event).0 {
        dispatch_window_event(event, id)
      }
    }
    Event::WindowEvent {
      window_id, event, ..
    } => {
      if let Some(webview_id) = webview_id_store.get(&window_id) {
        let window_event_wrapper = {
          let windows_ref = windows.borrow();
          windows_ref
            .get(&webview_id)
            .and_then(|w| WindowEventWrapper::parse(&w.inner, &event).0)
        };

        if let Some(event) = window_event_wrapper {
          dispatch_window_event(event, webview_id)
        }

        match event {
          #[cfg(windows)]
          WryWindowEvent::ThemeChanged(theme) => {
            if let Some(window) = windows.borrow().get(&webview_id) {
              if let Some(WindowHandle::Webview { inner, .. }) = &window.inner {
                let theme = match theme {
                  WryTheme::Dark => wry::webview::Theme::Dark,
                  WryTheme::Light => wry::webview::Theme::Light,
                  _ => wry::webview::Theme::Light,
                };
                inner.set_theme(theme);
              }
            }
          }
          WryWindowEvent::CloseRequested => {
            let (tx, rx) = channel();
            let windows_ref = windows.borrow();
            if let Some(w) = windows_ref.get(&webview_id) {
              let label = w.label.clone();
              let window_event_listeners = w.window_event_listeners_store.clone();

              drop(windows_ref);

              let listeners = window_event_listeners.lock().unwrap();
              let handlers = listeners.values();
              for handler in handlers {
                handler(&WindowEvent::CloseRequested {
                  signal_tx: tx.clone(),
                });
              }
              callback(RunEvent::WindowEvent {
                label,
                event: WindowEvent::CloseRequested { signal_tx: tx },
              });
              if let Ok(true) = rx.try_recv() {
              } else {
                on_window_close(webview_id, windows.clone());
              }
            }
          }
          WryWindowEvent::Destroyed => {
            let removed = windows.borrow_mut().remove(&webview_id).is_some();
            if removed {
              let is_empty = windows.borrow().is_empty();
              if is_empty {
                let (tx, rx) = channel();
                callback(RunEvent::ExitRequested { tx });

                let recv = rx.try_recv();
                let should_prevent = matches!(recv, Ok(ExitRequestedEventAction::Prevent));

                if !should_prevent {
                  *control_flow = ControlFlow::Exit;
                }
              }
            }
          }
          _ => {}
        }
      }
    }
    _ => unreachable!(),
  }
}

fn center_window(window: &WryWindow, window_size: WryPhysicalSize<u32>) -> Result<()> {
  if let Some(monitor) = window.current_monitor() {
    let screen_size = monitor.size();
    let monitor_pos = monitor.position();
    let x = (screen_size.width as i32 - window_size.width as i32) / 2;
    let y = (screen_size.height as i32 - window_size.height as i32) / 2;
    window.set_outer_position(WryPhysicalPosition::new(
      monitor_pos.x + x,
      monitor_pos.y + y,
    ));
    Ok(())
  } else {
    Err(Error::FailedToGetMonitor)
  }
}
