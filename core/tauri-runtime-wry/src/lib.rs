// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The [`wry`] Tauri [`Runtime`].

use tauri_runtime::{
  http::{
    Request as HttpRequest, RequestParts as HttpRequestParts, Response as HttpResponse,
    ResponseParts as HttpResponseParts,
  },
  menu::{CustomMenuItem, Menu, MenuEntry, MenuHash, MenuId, MenuItem, MenuUpdate},
  monitor::Monitor,
  webview::{FileDropEvent, FileDropHandler, WebviewIpcHandler, WindowBuilder, WindowBuilderBase},
  window::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size},
    DetachedWindow, JsEventListenerKey, PendingWindow, WindowEvent,
  },
  ClipboardManager, Dispatch, Error, ExitRequestedEventAction, GlobalShortcutManager, Icon, Result,
  RunEvent, RunIteration, Runtime, RuntimeHandle, UserAttentionType,
};

use tauri_runtime::window::MenuEvent;
#[cfg(feature = "system-tray")]
use tauri_runtime::{SystemTray, SystemTrayEvent};
#[cfg(windows)]
use webview2_com::FocusChangedEventHandler;
#[cfg(windows)]
use windows::Win32::{Foundation::HWND, System::WinRT::EventRegistrationToken};
#[cfg(all(feature = "system-tray", target_os = "macos"))]
use wry::application::platform::macos::{SystemTrayBuilderExtMacOS, SystemTrayExtMacOS};
#[cfg(target_os = "linux")]
use wry::application::platform::unix::{WindowBuilderExtUnix, WindowExtUnix};
#[cfg(windows)]
use wry::application::platform::windows::{WindowBuilderExtWindows, WindowExtWindows};

#[cfg(feature = "system-tray")]
use wry::application::system_tray::{SystemTray as WrySystemTray, SystemTrayBuilder};

use tauri_utils::config::WindowConfig;
use uuid::Uuid;
use wry::{
  application::{
    accelerator::{Accelerator, AcceleratorId},
    clipboard::Clipboard,
    dpi::{
      LogicalPosition as WryLogicalPosition, LogicalSize as WryLogicalSize,
      PhysicalPosition as WryPhysicalPosition, PhysicalSize as WryPhysicalSize,
      Position as WryPosition, Size as WrySize,
    },
    event::{Event, StartCause, WindowEvent as WryWindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
    global_shortcut::{GlobalShortcut, ShortcutManager as WryShortcutManager},
    menu::{
      CustomMenuItem as WryCustomMenuItem, MenuBar, MenuId as WryMenuId, MenuItem as WryMenuItem,
      MenuItemAttributes as WryMenuItemAttributes, MenuType,
    },
    monitor::MonitorHandle,
    window::{Fullscreen, Icon as WindowIcon, UserAttentionType as WryUserAttentionType},
  },
  http::{
    Request as WryHttpRequest, RequestParts as WryRequestParts, Response as WryHttpResponse,
    ResponseParts as WryResponseParts,
  },
  webview::{FileDropEvent as WryFileDropEvent, WebContext, WebView, WebViewBuilder},
};

pub use wry::application::window::{Window, WindowBuilder as WryWindowBuilder, WindowId};

#[cfg(target_os = "windows")]
use wry::webview::WebviewExtWindows;

#[cfg(target_os = "macos")]
use tauri_runtime::{menu::NativeImage, ActivationPolicy};
#[cfg(target_os = "macos")]
pub use wry::application::platform::macos::{
  ActivationPolicy as WryActivationPolicy, CustomMenuItemExtMacOS, EventLoopExtMacOS,
  NativeImage as WryNativeImage, WindowExtMacOS,
};

use std::{
  collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap, HashSet,
  },
  fmt,
  fs::read,
  ops::Deref,
  path::PathBuf,
  sync::{
    mpsc::{channel, Sender},
    Arc, Mutex, MutexGuard, Weak,
  },
  thread::{current as current_thread, ThreadId},
};

#[cfg(feature = "system-tray")]
mod system_tray;
#[cfg(feature = "system-tray")]
use system_tray::*;

type WebContextStore = Arc<Mutex<HashMap<Option<PathBuf>, WebContext>>>;
// window
type WindowEventHandler = Box<dyn Fn(&WindowEvent) + Send>;
type WindowEventListenersMap = Arc<Mutex<HashMap<Uuid, WindowEventHandler>>>;
type WindowEventListeners = Arc<Mutex<HashMap<WindowId, WindowEventListenersMap>>>;
// global shortcut
type GlobalShortcutListeners = Arc<Mutex<HashMap<AcceleratorId, Box<dyn Fn() + Send>>>>;
// menu
pub type MenuEventHandler = Box<dyn Fn(&MenuEvent) + Send>;
pub type MenuEventListeners = Arc<Mutex<HashMap<WindowId, WindowMenuEventListeners>>>;
pub type WindowMenuEventListeners = Arc<Mutex<HashMap<Uuid, MenuEventHandler>>>;

macro_rules! getter {
  ($self: ident, $rx: expr, $message: expr) => {{
    send_user_message(&$self.context, $message)?;
    $rx.recv().map_err(|_| Error::FailedToReceiveMessage)
  }};
}

macro_rules! window_getter {
  ($self: ident, $message: expr) => {{
    let (tx, rx) = channel();
    getter!($self, rx, Message::Window($self.window_id, $message(tx)))
  }};
}

fn send_user_message(context: &Context, message: Message) -> Result<()> {
  if current_thread().id() == context.main_thread_id {
    handle_user_message(
      &context.main_thread.window_target,
      message,
      UserMessageContext {
        window_event_listeners: &context.window_event_listeners,
        global_shortcut_manager: context.main_thread.global_shortcut_manager.clone(),
        clipboard_manager: context.main_thread.clipboard_manager.clone(),
        menu_event_listeners: &context.menu_event_listeners,
        windows: context.main_thread.windows.clone(),
        #[cfg(feature = "system-tray")]
        tray_context: &context.main_thread.tray_context,
      },
      &context.main_thread.web_context,
    );
    Ok(())
  } else {
    context
      .proxy
      .send_event(message)
      .map_err(|_| Error::FailedToSendMessage)
  }
}

#[derive(Clone)]
struct Context {
  main_thread_id: ThreadId,
  proxy: EventLoopProxy<Message>,
  window_event_listeners: WindowEventListeners,
  menu_event_listeners: MenuEventListeners,
  main_thread: DispatcherMainThreadContext,
}

#[derive(Debug, Clone)]
struct DispatcherMainThreadContext {
  window_target: EventLoopWindowTarget<Message>,
  web_context: WebContextStore,
  global_shortcut_manager: Arc<Mutex<WryShortcutManager>>,
  clipboard_manager: Arc<Mutex<Clipboard>>,
  windows: Arc<Mutex<HashMap<WindowId, WindowWrapper>>>,
  #[cfg(feature = "system-tray")]
  tray_context: TrayContext,
}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for DispatcherMainThreadContext {}

impl fmt::Debug for Context {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Context")
      .field("main_thread_id", &self.main_thread_id)
      .field("proxy", &self.proxy)
      .field("main_thread", &self.main_thread)
      .finish()
  }
}

struct HttpRequestPartsWrapper(HttpRequestParts);

impl From<HttpRequestPartsWrapper> for HttpRequestParts {
  fn from(parts: HttpRequestPartsWrapper) -> Self {
    Self {
      method: parts.0.method,
      uri: parts.0.uri,
      headers: parts.0.headers,
    }
  }
}

impl From<HttpRequestParts> for HttpRequestPartsWrapper {
  fn from(request: HttpRequestParts) -> Self {
    Self(HttpRequestParts {
      method: request.method,
      uri: request.uri,
      headers: request.headers,
    })
  }
}

impl From<WryRequestParts> for HttpRequestPartsWrapper {
  fn from(request: WryRequestParts) -> Self {
    Self(HttpRequestParts {
      method: request.method,
      uri: request.uri,
      headers: request.headers,
    })
  }
}

struct HttpRequestWrapper(HttpRequest);

impl From<&WryHttpRequest> for HttpRequestWrapper {
  fn from(req: &WryHttpRequest) -> Self {
    Self(HttpRequest {
      body: req.body.clone(),
      head: HttpRequestPartsWrapper::from(req.head.clone()).0,
    })
  }
}

// response
struct HttpResponsePartsWrapper(WryResponseParts);
impl From<HttpResponseParts> for HttpResponsePartsWrapper {
  fn from(response: HttpResponseParts) -> Self {
    Self(WryResponseParts {
      mimetype: response.mimetype,
      status: response.status,
      version: response.version,
      headers: response.headers,
    })
  }
}

struct HttpResponseWrapper(WryHttpResponse);
impl From<HttpResponse> for HttpResponseWrapper {
  fn from(response: HttpResponse) -> Self {
    Self(WryHttpResponse {
      body: response.body,
      head: HttpResponsePartsWrapper::from(response.head).0,
    })
  }
}

pub struct MenuItemAttributesWrapper<'a>(pub WryMenuItemAttributes<'a>);

impl<'a> From<&'a CustomMenuItem> for MenuItemAttributesWrapper<'a> {
  fn from(item: &'a CustomMenuItem) -> Self {
    let mut attributes = WryMenuItemAttributes::new(&item.title)
      .with_enabled(item.enabled)
      .with_selected(item.selected)
      .with_id(WryMenuId(item.id));
    if let Some(accelerator) = item.keyboard_accelerator.as_ref() {
      attributes = attributes.with_accelerators(&accelerator.parse().expect("invalid accelerator"));
    }
    Self(attributes)
  }
}

pub struct MenuItemWrapper(pub WryMenuItem);

impl From<MenuItem> for MenuItemWrapper {
  fn from(item: MenuItem) -> Self {
    match item {
      MenuItem::About(v) => Self(WryMenuItem::About(v)),
      MenuItem::Hide => Self(WryMenuItem::Hide),
      MenuItem::Services => Self(WryMenuItem::Services),
      MenuItem::HideOthers => Self(WryMenuItem::HideOthers),
      MenuItem::ShowAll => Self(WryMenuItem::ShowAll),
      MenuItem::CloseWindow => Self(WryMenuItem::CloseWindow),
      MenuItem::Quit => Self(WryMenuItem::Quit),
      MenuItem::Copy => Self(WryMenuItem::Copy),
      MenuItem::Cut => Self(WryMenuItem::Cut),
      MenuItem::Undo => Self(WryMenuItem::Undo),
      MenuItem::Redo => Self(WryMenuItem::Redo),
      MenuItem::SelectAll => Self(WryMenuItem::SelectAll),
      MenuItem::Paste => Self(WryMenuItem::Paste),
      MenuItem::EnterFullScreen => Self(WryMenuItem::EnterFullScreen),
      MenuItem::Minimize => Self(WryMenuItem::Minimize),
      MenuItem::Zoom => Self(WryMenuItem::Zoom),
      MenuItem::Separator => Self(WryMenuItem::Separator),
      _ => unimplemented!(),
    }
  }
}

#[cfg(target_os = "macos")]
pub struct NativeImageWrapper(pub WryNativeImage);

#[cfg(target_os = "macos")]
impl std::fmt::Debug for NativeImageWrapper {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("NativeImageWrapper").finish()
  }
}

#[cfg(target_os = "macos")]
impl From<NativeImage> for NativeImageWrapper {
  fn from(image: NativeImage) -> NativeImageWrapper {
    let wry_image = match image {
      NativeImage::Add => WryNativeImage::Add,
      NativeImage::Advanced => WryNativeImage::Advanced,
      NativeImage::Bluetooth => WryNativeImage::Bluetooth,
      NativeImage::Bookmarks => WryNativeImage::Bookmarks,
      NativeImage::Caution => WryNativeImage::Caution,
      NativeImage::ColorPanel => WryNativeImage::ColorPanel,
      NativeImage::ColumnView => WryNativeImage::ColumnView,
      NativeImage::Computer => WryNativeImage::Computer,
      NativeImage::EnterFullScreen => WryNativeImage::EnterFullScreen,
      NativeImage::Everyone => WryNativeImage::Everyone,
      NativeImage::ExitFullScreen => WryNativeImage::ExitFullScreen,
      NativeImage::FlowView => WryNativeImage::FlowView,
      NativeImage::Folder => WryNativeImage::Folder,
      NativeImage::FolderBurnable => WryNativeImage::FolderBurnable,
      NativeImage::FolderSmart => WryNativeImage::FolderSmart,
      NativeImage::FollowLinkFreestanding => WryNativeImage::FollowLinkFreestanding,
      NativeImage::FontPanel => WryNativeImage::FontPanel,
      NativeImage::GoLeft => WryNativeImage::GoLeft,
      NativeImage::GoRight => WryNativeImage::GoRight,
      NativeImage::Home => WryNativeImage::Home,
      NativeImage::IChatTheater => WryNativeImage::IChatTheater,
      NativeImage::IconView => WryNativeImage::IconView,
      NativeImage::Info => WryNativeImage::Info,
      NativeImage::InvalidDataFreestanding => WryNativeImage::InvalidDataFreestanding,
      NativeImage::LeftFacingTriangle => WryNativeImage::LeftFacingTriangle,
      NativeImage::ListView => WryNativeImage::ListView,
      NativeImage::LockLocked => WryNativeImage::LockLocked,
      NativeImage::LockUnlocked => WryNativeImage::LockUnlocked,
      NativeImage::MenuMixedState => WryNativeImage::MenuMixedState,
      NativeImage::MenuOnState => WryNativeImage::MenuOnState,
      NativeImage::MobileMe => WryNativeImage::MobileMe,
      NativeImage::MultipleDocuments => WryNativeImage::MultipleDocuments,
      NativeImage::Network => WryNativeImage::Network,
      NativeImage::Path => WryNativeImage::Path,
      NativeImage::PreferencesGeneral => WryNativeImage::PreferencesGeneral,
      NativeImage::QuickLook => WryNativeImage::QuickLook,
      NativeImage::RefreshFreestanding => WryNativeImage::RefreshFreestanding,
      NativeImage::Refresh => WryNativeImage::Refresh,
      NativeImage::Remove => WryNativeImage::Remove,
      NativeImage::RevealFreestanding => WryNativeImage::RevealFreestanding,
      NativeImage::RightFacingTriangle => WryNativeImage::RightFacingTriangle,
      NativeImage::Share => WryNativeImage::Share,
      NativeImage::Slideshow => WryNativeImage::Slideshow,
      NativeImage::SmartBadge => WryNativeImage::SmartBadge,
      NativeImage::StatusAvailable => WryNativeImage::StatusAvailable,
      NativeImage::StatusNone => WryNativeImage::StatusNone,
      NativeImage::StatusPartiallyAvailable => WryNativeImage::StatusPartiallyAvailable,
      NativeImage::StatusUnavailable => WryNativeImage::StatusUnavailable,
      NativeImage::StopProgressFreestanding => WryNativeImage::StopProgressFreestanding,
      NativeImage::StopProgress => WryNativeImage::StopProgress,

      NativeImage::TrashEmpty => WryNativeImage::TrashEmpty,
      NativeImage::TrashFull => WryNativeImage::TrashFull,
      NativeImage::User => WryNativeImage::User,
      NativeImage::UserAccounts => WryNativeImage::UserAccounts,
      NativeImage::UserGroup => WryNativeImage::UserGroup,
      NativeImage::UserGuest => WryNativeImage::UserGuest,
    };
    Self(wry_image)
  }
}

#[derive(Debug, Clone)]
pub struct GlobalShortcutWrapper(GlobalShortcut);

// SAFETY: usage outside of main thread is guarded, we use the event loop on such cases.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for GlobalShortcutWrapper {}

/// Wrapper around [`WryShortcutManager`].
#[derive(Clone)]
pub struct GlobalShortcutManagerHandle {
  context: Context,
  shortcuts: Arc<Mutex<HashMap<String, (AcceleratorId, GlobalShortcutWrapper)>>>,
  listeners: GlobalShortcutListeners,
}

// SAFETY: this is safe since the `Context` usage is guarded on `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for GlobalShortcutManagerHandle {}

impl fmt::Debug for GlobalShortcutManagerHandle {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("GlobalShortcutManagerHandle")
      .field("context", &self.context)
      .field("shortcuts", &self.shortcuts)
      .finish()
  }
}

impl GlobalShortcutManager for GlobalShortcutManagerHandle {
  fn is_registered(&self, accelerator: &str) -> Result<bool> {
    let (tx, rx) = channel();
    getter!(
      self,
      rx,
      Message::GlobalShortcut(GlobalShortcutMessage::IsRegistered(
        accelerator.parse().expect("invalid accelerator"),
        tx
      ))
    )
  }

  fn register<F: Fn() + Send + 'static>(&mut self, accelerator: &str, handler: F) -> Result<()> {
    let wry_accelerator: Accelerator = accelerator.parse().expect("invalid accelerator");
    let id = wry_accelerator.clone().id();
    let (tx, rx) = channel();
    let shortcut = getter!(
      self,
      rx,
      Message::GlobalShortcut(GlobalShortcutMessage::Register(wry_accelerator, tx))
    )??;

    self.listeners.lock().unwrap().insert(id, Box::new(handler));
    self
      .shortcuts
      .lock()
      .unwrap()
      .insert(accelerator.into(), (id, shortcut));

    Ok(())
  }

  fn unregister_all(&mut self) -> Result<()> {
    let (tx, rx) = channel();
    getter!(
      self,
      rx,
      Message::GlobalShortcut(GlobalShortcutMessage::UnregisterAll(tx))
    )??;
    self.listeners.lock().unwrap().clear();
    self.shortcuts.lock().unwrap().clear();
    Ok(())
  }

  fn unregister(&mut self, accelerator: &str) -> Result<()> {
    if let Some((accelerator_id, shortcut)) = self.shortcuts.lock().unwrap().remove(accelerator) {
      let (tx, rx) = channel();
      getter!(
        self,
        rx,
        Message::GlobalShortcut(GlobalShortcutMessage::Unregister(shortcut, tx))
      )??;
      self.listeners.lock().unwrap().remove(&accelerator_id);
    }
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct ClipboardManagerWrapper {
  context: Context,
}

// SAFETY: this is safe since the `Context` usage is guarded on `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for ClipboardManagerWrapper {}

impl ClipboardManager for ClipboardManagerWrapper {
  fn read_text(&self) -> Result<Option<String>> {
    let (tx, rx) = channel();
    getter!(self, rx, Message::Clipboard(ClipboardMessage::ReadText(tx)))
  }

  fn write_text<T: Into<String>>(&mut self, text: T) -> Result<()> {
    let (tx, rx) = channel();
    getter!(
      self,
      rx,
      Message::Clipboard(ClipboardMessage::WriteText(text.into(), tx))
    )?;
    Ok(())
  }
}

/// Wrapper around a [`wry::application::window::Icon`] that can be created from an [`Icon`].
pub struct WryIcon(WindowIcon);

fn icon_err<E: std::error::Error + Send + 'static>(e: E) -> Error {
  Error::InvalidIcon(Box::new(e))
}

impl TryFrom<Icon> for WryIcon {
  type Error = Error;
  fn try_from(icon: Icon) -> std::result::Result<Self, Self::Error> {
    let image_bytes = match icon {
      Icon::File(path) => read(path).map_err(icon_err)?,
      Icon::Raw(raw) => raw,
      _ => unimplemented!(),
    };
    let extension = infer::get(&image_bytes)
      .expect("could not determine icon extension")
      .extension();
    match extension {
      #[cfg(windows)]
      "ico" => {
        let icon_dir = ico::IconDir::read(std::io::Cursor::new(image_bytes)).map_err(icon_err)?;
        let entry = &icon_dir.entries()[0];
        let icon = WindowIcon::from_rgba(
          entry.decode().map_err(icon_err)?.rgba_data().to_vec(),
          entry.width(),
          entry.height(),
        )
        .map_err(icon_err)?;
        Ok(Self(icon))
      }
      #[cfg(target_os = "linux")]
      "png" => {
        let decoder = png::Decoder::new(std::io::Cursor::new(image_bytes));
        let (info, mut reader) = decoder.read_info().map_err(icon_err)?;
        let mut buffer = Vec::new();
        while let Ok(Some(row)) = reader.next_row() {
          buffer.extend(row);
        }
        let icon = WindowIcon::from_rgba(buffer, info.width, info.height).map_err(icon_err)?;
        Ok(Self(icon))
      }
      _ => panic!(
        "image `{}` extension not supported; please file a Tauri feature request",
        extension
      ),
    }
  }
}

struct WindowEventWrapper(Option<WindowEvent>);

impl WindowEventWrapper {
  fn parse(webview: &WindowHandle, event: &WryWindowEvent<'_>) -> Self {
    match event {
      // resized event from tao doesn't include a reliable size on macOS
      // because wry replaces the NSView
      WryWindowEvent::Resized(_) => Self(Some(WindowEvent::Resized(
        PhysicalSizeWrapper(webview.inner_size()).into(),
      ))),
      e => e.into(),
    }
  }
}

impl<'a> From<&WryWindowEvent<'a>> for WindowEventWrapper {
  fn from(event: &WryWindowEvent<'a>) -> Self {
    let event = match event {
      WryWindowEvent::Resized(size) => WindowEvent::Resized(PhysicalSizeWrapper(*size).into()),
      WryWindowEvent::Moved(position) => {
        WindowEvent::Moved(PhysicalPositionWrapper(*position).into())
      }
      WryWindowEvent::Destroyed => WindowEvent::Destroyed,
      WryWindowEvent::ScaleFactorChanged {
        scale_factor,
        new_inner_size,
      } => WindowEvent::ScaleFactorChanged {
        scale_factor: *scale_factor,
        new_inner_size: PhysicalSizeWrapper(**new_inner_size).into(),
      },
      #[cfg(any(target_os = "linux", target_os = "macos"))]
      WryWindowEvent::Focused(focused) => WindowEvent::Focused(*focused),
      _ => return Self(None),
    };
    Self(Some(event))
  }
}

impl From<&WebviewEvent> for WindowEventWrapper {
  fn from(event: &WebviewEvent) -> Self {
    let event = match event {
      WebviewEvent::Focused(focused) => WindowEvent::Focused(*focused),
    };
    Self(Some(event))
  }
}

pub struct MonitorHandleWrapper(MonitorHandle);

impl From<MonitorHandleWrapper> for Monitor {
  fn from(monitor: MonitorHandleWrapper) -> Monitor {
    Self {
      name: monitor.0.name(),
      position: PhysicalPositionWrapper(monitor.0.position()).into(),
      size: PhysicalSizeWrapper(monitor.0.size()).into(),
      scale_factor: monitor.0.scale_factor(),
    }
  }
}

struct PhysicalPositionWrapper<T>(WryPhysicalPosition<T>);

impl<T> From<PhysicalPositionWrapper<T>> for PhysicalPosition<T> {
  fn from(position: PhysicalPositionWrapper<T>) -> Self {
    Self {
      x: position.0.x,
      y: position.0.y,
    }
  }
}

impl<T> From<PhysicalPosition<T>> for PhysicalPositionWrapper<T> {
  fn from(position: PhysicalPosition<T>) -> Self {
    Self(WryPhysicalPosition {
      x: position.x,
      y: position.y,
    })
  }
}

struct LogicalPositionWrapper<T>(WryLogicalPosition<T>);

impl<T> From<LogicalPosition<T>> for LogicalPositionWrapper<T> {
  fn from(position: LogicalPosition<T>) -> Self {
    Self(WryLogicalPosition {
      x: position.x,
      y: position.y,
    })
  }
}

struct PhysicalSizeWrapper<T>(WryPhysicalSize<T>);

impl<T> From<PhysicalSizeWrapper<T>> for PhysicalSize<T> {
  fn from(size: PhysicalSizeWrapper<T>) -> Self {
    Self {
      width: size.0.width,
      height: size.0.height,
    }
  }
}

impl<T> From<PhysicalSize<T>> for PhysicalSizeWrapper<T> {
  fn from(size: PhysicalSize<T>) -> Self {
    Self(WryPhysicalSize {
      width: size.width,
      height: size.height,
    })
  }
}

struct LogicalSizeWrapper<T>(WryLogicalSize<T>);

impl<T> From<LogicalSize<T>> for LogicalSizeWrapper<T> {
  fn from(size: LogicalSize<T>) -> Self {
    Self(WryLogicalSize {
      width: size.width,
      height: size.height,
    })
  }
}

struct SizeWrapper(WrySize);

impl From<Size> for SizeWrapper {
  fn from(size: Size) -> Self {
    match size {
      Size::Logical(s) => Self(WrySize::Logical(LogicalSizeWrapper::from(s).0)),
      Size::Physical(s) => Self(WrySize::Physical(PhysicalSizeWrapper::from(s).0)),
    }
  }
}

struct PositionWrapper(WryPosition);

impl From<Position> for PositionWrapper {
  fn from(position: Position) -> Self {
    match position {
      Position::Logical(s) => Self(WryPosition::Logical(LogicalPositionWrapper::from(s).0)),
      Position::Physical(s) => Self(WryPosition::Physical(PhysicalPositionWrapper::from(s).0)),
    }
  }
}

#[derive(Debug, Clone)]
pub struct UserAttentionTypeWrapper(WryUserAttentionType);

impl From<UserAttentionType> for UserAttentionTypeWrapper {
  fn from(request_type: UserAttentionType) -> UserAttentionTypeWrapper {
    let o = match request_type {
      UserAttentionType::Critical => WryUserAttentionType::Critical,
      UserAttentionType::Informational => WryUserAttentionType::Informational,
    };
    Self(o)
  }
}

#[derive(Debug, Clone, Default)]
pub struct WindowBuilderWrapper {
  inner: WryWindowBuilder,
  center: bool,
  menu: Option<Menu>,
}

// SAFETY: this type is `Send` since `menu_items` are read only here
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for WindowBuilderWrapper {}

impl WindowBuilderBase for WindowBuilderWrapper {}
impl WindowBuilder for WindowBuilderWrapper {
  fn new() -> Self {
    Default::default()
  }

  fn with_config(config: WindowConfig) -> Self {
    let mut window = WindowBuilderWrapper::new()
      .title(config.title.to_string())
      .inner_size(config.width, config.height)
      .visible(config.visible)
      .resizable(config.resizable)
      .fullscreen(config.fullscreen)
      .decorations(config.decorations)
      .maximized(config.maximized)
      .always_on_top(config.always_on_top)
      .skip_taskbar(config.skip_taskbar);

    #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
    {
      window = window.transparent(config.transparent);
    }

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

  /// Deprecated since 0.1.4 (noop)
  /// Windows is automatically focused when created.
  fn focus(self) -> Self {
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

  #[cfg(windows)]
  fn parent_window(mut self, parent: HWND) -> Self {
    self.inner = self.inner.with_parent_window(parent);
    self
  }

  #[cfg(windows)]
  fn owner_window(mut self, owner: HWND) -> Self {
    self.inner = self.inner.with_owner_window(owner);
    self
  }

  fn icon(mut self, icon: Icon) -> Result<Self> {
    self.inner = self
      .inner
      .with_window_icon(Some(WryIcon::try_from(icon)?.0));
    Ok(self)
  }

  #[cfg(any(target_os = "windows", target_os = "linux"))]
  fn skip_taskbar(mut self, skip: bool) -> Self {
    self.inner = self.inner.with_skip_taskbar(skip);
    self
  }

  #[cfg(target_os = "macos")]
  fn skip_taskbar(self, _skip: bool) -> Self {
    self
  }

  fn has_icon(&self) -> bool {
    self.inner.window.window_icon.is_some()
  }

  fn get_menu(&self) -> Option<&Menu> {
    self.menu.as_ref()
  }
}

pub struct FileDropEventWrapper(WryFileDropEvent);

impl From<FileDropEventWrapper> for FileDropEvent {
  fn from(event: FileDropEventWrapper) -> Self {
    match event.0 {
      WryFileDropEvent::Hovered(paths) => FileDropEvent::Hovered(paths),
      WryFileDropEvent::Dropped(paths) => FileDropEvent::Dropped(paths),
      // default to cancelled
      // FIXME(maybe): Add `FileDropEvent::Unknown` event?
      _ => FileDropEvent::Cancelled,
    }
  }
}

#[cfg(target_os = "macos")]
pub struct NSWindow(*mut std::ffi::c_void);
#[cfg(target_os = "macos")]
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for NSWindow {}

#[cfg(windows)]
pub struct Hwnd(HWND);
#[cfg(windows)]
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for Hwnd {}

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
pub struct GtkWindow(gtk::ApplicationWindow);
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for GtkWindow {}

#[derive(Debug, Clone)]
pub enum WindowMessage {
  #[cfg(any(debug_assertions, feature = "devtools"))]
  OpenDevTools,
  // Getters
  ScaleFactor(Sender<f64>),
  InnerPosition(Sender<Result<PhysicalPosition<i32>>>),
  OuterPosition(Sender<Result<PhysicalPosition<i32>>>),
  InnerSize(Sender<PhysicalSize<u32>>),
  OuterSize(Sender<PhysicalSize<u32>>),
  IsFullscreen(Sender<bool>),
  IsMaximized(Sender<bool>),
  IsDecorated(Sender<bool>),
  IsResizable(Sender<bool>),
  IsVisible(Sender<bool>),
  IsMenuVisible(Sender<bool>),
  CurrentMonitor(Sender<Option<MonitorHandle>>),
  PrimaryMonitor(Sender<Option<MonitorHandle>>),
  AvailableMonitors(Sender<Vec<MonitorHandle>>),
  #[cfg(target_os = "macos")]
  NSWindow(Sender<NSWindow>),
  #[cfg(windows)]
  Hwnd(Sender<Hwnd>),
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  GtkWindow(Sender<GtkWindow>),
  // Setters
  Center(Sender<Result<()>>),
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
  SetAlwaysOnTop(bool),
  SetSize(Size),
  SetMinSize(Option<Size>),
  SetMaxSize(Option<Size>),
  SetPosition(Position),
  SetFullscreen(bool),
  SetFocus,
  SetIcon(WindowIcon),
  SetSkipTaskbar(bool),
  DragWindow,
  UpdateMenuItem(u16, MenuUpdate),
  RequestRedraw,
}

#[derive(Debug, Clone)]
pub enum WebviewMessage {
  EvaluateScript(String),
  #[allow(dead_code)]
  WebviewEvent(WebviewEvent),
  Print,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum WebviewEvent {
  Focused(bool),
}

#[cfg(feature = "system-tray")]
#[derive(Debug, Clone)]
pub enum TrayMessage {
  UpdateItem(u16, MenuUpdate),
  UpdateMenu(SystemTrayMenu),
  UpdateIcon(Icon),
  #[cfg(target_os = "macos")]
  UpdateIconAsTemplate(bool),
  Close,
}

#[derive(Debug, Clone)]
pub enum GlobalShortcutMessage {
  IsRegistered(Accelerator, Sender<bool>),
  Register(Accelerator, Sender<Result<GlobalShortcutWrapper>>),
  Unregister(GlobalShortcutWrapper, Sender<Result<()>>),
  UnregisterAll(Sender<Result<()>>),
}

#[derive(Debug, Clone)]
pub enum ClipboardMessage {
  WriteText(String, Sender<()>),
  ReadText(Sender<Option<String>>),
}

pub enum Message {
  Task(Box<dyn FnOnce() + Send>),
  Window(WindowId, WindowMessage),
  Webview(WindowId, WebviewMessage),
  #[cfg(feature = "system-tray")]
  Tray(TrayMessage),
  CreateWebview(
    Box<
      dyn FnOnce(&EventLoopWindowTarget<Message>, &WebContextStore) -> Result<WindowWrapper> + Send,
    >,
    Sender<WindowId>,
  ),
  CreateWindow(
    Box<dyn FnOnce() -> (String, WryWindowBuilder) + Send>,
    Sender<Result<Weak<Window>>>,
  ),
  GlobalShortcut(GlobalShortcutMessage),
  Clipboard(ClipboardMessage),
}

impl Clone for Message {
  fn clone(&self) -> Self {
    match self {
      Self::Window(i, m) => Self::Window(*i, m.clone()),
      Self::Webview(i, m) => Self::Webview(*i, m.clone()),
      #[cfg(feature = "system-tray")]
      Self::Tray(m) => Self::Tray(m.clone()),
      Self::GlobalShortcut(m) => Self::GlobalShortcut(m.clone()),
      Self::Clipboard(m) => Self::Clipboard(m.clone()),
      _ => unimplemented!(),
    }
  }
}

/// The Tauri [`Dispatch`] for [`Wry`].
#[derive(Debug, Clone)]
pub struct WryDispatcher {
  window_id: WindowId,
  context: Context,
}

// SAFETY: this is safe since the `Context` usage is guarded on `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for WryDispatcher {}

impl Dispatch for WryDispatcher {
  type Runtime = Wry;
  type WindowBuilder = WindowBuilderWrapper;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    send_user_message(&self.context, Message::Task(Box::new(f)))
  }

  fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) -> Uuid {
    let id = Uuid::new_v4();
    self
      .context
      .window_event_listeners
      .lock()
      .unwrap()
      .get(&self.window_id)
      .unwrap()
      .lock()
      .unwrap()
      .insert(id, Box::new(f));
    id
  }

  fn on_menu_event<F: Fn(&MenuEvent) + Send + 'static>(&self, f: F) -> Uuid {
    let id = Uuid::new_v4();
    self
      .context
      .menu_event_listeners
      .lock()
      .unwrap()
      .get(&self.window_id)
      .unwrap()
      .lock()
      .unwrap()
      .insert(id, Box::new(f));
    id
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn open_devtools(&self) {
    let _ = send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::OpenDevTools),
    );
  }

  // Getters

  fn scale_factor(&self) -> Result<f64> {
    window_getter!(self, WindowMessage::ScaleFactor)
  }

  fn inner_position(&self) -> Result<PhysicalPosition<i32>> {
    window_getter!(self, WindowMessage::InnerPosition)?
  }

  fn outer_position(&self) -> Result<PhysicalPosition<i32>> {
    window_getter!(self, WindowMessage::OuterPosition)?
  }

  fn inner_size(&self) -> Result<PhysicalSize<u32>> {
    window_getter!(self, WindowMessage::InnerSize)
  }

  fn outer_size(&self) -> Result<PhysicalSize<u32>> {
    window_getter!(self, WindowMessage::OuterSize)
  }

  fn is_fullscreen(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsFullscreen)
  }

  fn is_maximized(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsMaximized)
  }

  /// Gets the window’s current decoration state.
  fn is_decorated(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsDecorated)
  }

  /// Gets the window’s current resizable state.
  fn is_resizable(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsResizable)
  }

  fn is_visible(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsVisible)
  }

  fn is_menu_visible(&self) -> Result<bool> {
    window_getter!(self, WindowMessage::IsMenuVisible)
  }

  fn current_monitor(&self) -> Result<Option<Monitor>> {
    Ok(window_getter!(self, WindowMessage::CurrentMonitor)?.map(|m| MonitorHandleWrapper(m).into()))
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    Ok(window_getter!(self, WindowMessage::PrimaryMonitor)?.map(|m| MonitorHandleWrapper(m).into()))
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    Ok(
      window_getter!(self, WindowMessage::AvailableMonitors)?
        .into_iter()
        .map(|m| MonitorHandleWrapper(m).into())
        .collect(),
    )
  }

  #[cfg(target_os = "macos")]
  fn ns_window(&self) -> Result<*mut std::ffi::c_void> {
    window_getter!(self, WindowMessage::NSWindow).map(|w| w.0)
  }

  #[cfg(windows)]
  fn hwnd(&self) -> Result<HWND> {
    window_getter!(self, WindowMessage::Hwnd).map(|w| w.0)
  }

  /// Returns the `ApplicatonWindow` from gtk crate that is used by this window.
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn gtk_window(&self) -> Result<gtk::ApplicationWindow> {
    window_getter!(self, WindowMessage::GtkWindow).map(|w| w.0)
  }

  // Setters

  fn center(&self) -> Result<()> {
    window_getter!(self, WindowMessage::Center)?
  }

  fn print(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Webview(self.window_id, WebviewMessage::Print),
    )
  }

  fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.window_id,
        WindowMessage::RequestUserAttention(request_type.map(Into::into)),
      ),
    )
  }

  // Creates a window by dispatching a message to the event loop.
  // Note that this must be called from a separate thread, otherwise the channel will introduce a deadlock.
  fn create_window(
    &mut self,
    pending: PendingWindow<Self::Runtime>,
  ) -> Result<DetachedWindow<Self::Runtime>> {
    let (tx, rx) = channel();
    let label = pending.label.clone();
    let menu_ids = pending.menu_ids.clone();
    let js_event_listeners = pending.js_event_listeners.clone();
    let context = self.context.clone();

    send_user_message(
      &self.context,
      Message::CreateWebview(
        Box::new(move |event_loop, web_context| {
          create_webview(event_loop, web_context, context, pending)
        }),
        tx,
      ),
    )?;
    let window_id = rx.recv().unwrap();

    let dispatcher = WryDispatcher {
      window_id,
      context: self.context.clone(),
    };
    Ok(DetachedWindow {
      label,
      dispatcher,
      menu_ids,
      js_event_listeners,
    })
  }

  fn set_resizable(&self, resizable: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetResizable(resizable)),
    )
  }

  fn set_title<S: Into<String>>(&self, title: S) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetTitle(title.into())),
    )
  }

  fn maximize(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::Maximize),
    )
  }

  fn unmaximize(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::Unmaximize),
    )
  }

  fn minimize(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::Minimize),
    )
  }

  fn unminimize(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::Unminimize),
    )
  }

  fn show_menu(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::ShowMenu),
    )
  }

  fn hide_menu(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::HideMenu),
    )
  }

  fn show(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::Show),
    )
  }

  fn hide(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::Hide),
    )
  }

  fn close(&self) -> Result<()> {
    // NOTE: close cannot use the `send_user_message` function because it accesses the event loop callback
    self
      .context
      .proxy
      .send_event(Message::Window(self.window_id, WindowMessage::Close))
      .map_err(|_| Error::FailedToSendMessage)
  }

  fn set_decorations(&self, decorations: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetDecorations(decorations)),
    )
  }

  fn set_always_on_top(&self, always_on_top: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetAlwaysOnTop(always_on_top)),
    )
  }

  fn set_size(&self, size: Size) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetSize(size)),
    )
  }

  fn set_min_size(&self, size: Option<Size>) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetMinSize(size)),
    )
  }

  fn set_max_size(&self, size: Option<Size>) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetMaxSize(size)),
    )
  }

  fn set_position(&self, position: Position) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetPosition(position)),
    )
  }

  fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetFullscreen(fullscreen)),
    )
  }

  fn set_focus(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetFocus),
    )
  }

  fn set_icon(&self, icon: Icon) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(
        self.window_id,
        WindowMessage::SetIcon(WryIcon::try_from(icon)?.0),
      ),
    )
  }

  fn set_skip_taskbar(&self, skip: bool) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::SetSkipTaskbar(skip)),
    )
  }

  fn start_dragging(&self) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::DragWindow),
    )
  }

  fn eval_script<S: Into<String>>(&self, script: S) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Webview(
        self.window_id,
        WebviewMessage::EvaluateScript(script.into()),
      ),
    )
  }

  fn update_menu_item(&self, id: u16, update: MenuUpdate) -> Result<()> {
    send_user_message(
      &self.context,
      Message::Window(self.window_id, WindowMessage::UpdateMenuItem(id, update)),
    )
  }
}

#[cfg(feature = "system-tray")]
#[derive(Clone, Default)]
struct TrayContext {
  tray: Arc<Mutex<Option<Arc<Mutex<WrySystemTray>>>>>,
  listeners: SystemTrayEventListeners,
  items: SystemTrayItems,
}

#[cfg(feature = "system-tray")]
impl fmt::Debug for TrayContext {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("TrayContext")
      .field("items", &self.items)
      .finish()
  }
}

enum WindowHandle {
  Webview(WebView),
  Window(Arc<Window>),
}

impl fmt::Debug for WindowHandle {
  fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
    Ok(())
  }
}

impl WindowHandle {
  fn window(&self) -> &Window {
    match self {
      Self::Webview(w) => w.window(),
      Self::Window(w) => w,
    }
  }

  fn inner_size(&self) -> WryPhysicalSize<u32> {
    match self {
      WindowHandle::Window(w) => w.inner_size(),
      WindowHandle::Webview(w) => w.inner_size(),
    }
  }
}

#[derive(Debug)]
pub struct WindowWrapper {
  label: String,
  inner: WindowHandle,
  menu_items: Option<HashMap<u16, WryCustomMenuItem>>,
}

/// A Tauri [`Runtime`] wrapper around wry.
pub struct Wry {
  main_thread_id: ThreadId,
  global_shortcut_manager: Arc<Mutex<WryShortcutManager>>,
  global_shortcut_manager_handle: GlobalShortcutManagerHandle,
  clipboard_manager: Arc<Mutex<Clipboard>>,
  clipboard_manager_handle: ClipboardManagerWrapper,
  event_loop: EventLoop<Message>,
  windows: Arc<Mutex<HashMap<WindowId, WindowWrapper>>>,
  web_context: WebContextStore,
  window_event_listeners: WindowEventListeners,
  menu_event_listeners: MenuEventListeners,
  #[cfg(feature = "system-tray")]
  tray_context: TrayContext,
}

/// A handle to the Wry runtime.
#[derive(Debug, Clone)]
pub struct WryHandle {
  context: Context,
}

// SAFETY: this is safe since the `Context` usage is guarded on `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for WryHandle {}

impl WryHandle {
  /// Creates a new tao window using a callback, and returns its window id.
  pub fn create_tao_window<F: FnOnce() -> (String, WryWindowBuilder) + Send + 'static>(
    &self,
    f: F,
  ) -> Result<Weak<Window>> {
    let (tx, rx) = channel();
    send_user_message(&self.context, Message::CreateWindow(Box::new(f), tx))?;
    rx.recv().unwrap()
  }

  /// Send a message to the event loop.
  pub fn send_event(&self, message: Message) -> Result<()> {
    self
      .context
      .proxy
      .send_event(message)
      .map_err(|_| Error::FailedToSendMessage)?;
    Ok(())
  }
}

impl RuntimeHandle for WryHandle {
  type Runtime = Wry;

  // Creates a window by dispatching a message to the event loop.
  // Note that this must be called from a separate thread, otherwise the channel will introduce a deadlock.
  fn create_window(
    &self,
    pending: PendingWindow<Self::Runtime>,
  ) -> Result<DetachedWindow<Self::Runtime>> {
    let (tx, rx) = channel();
    let label = pending.label.clone();
    let menu_ids = pending.menu_ids.clone();
    let js_event_listeners = pending.js_event_listeners.clone();
    let context = self.context.clone();
    send_user_message(
      &self.context,
      Message::CreateWebview(
        Box::new(move |event_loop, web_context| {
          create_webview(event_loop, web_context, context, pending)
        }),
        tx,
      ),
    )?;
    let window_id = rx.recv().unwrap();

    let dispatcher = WryDispatcher {
      window_id,
      context: self.context.clone(),
    };
    Ok(DetachedWindow {
      label,
      dispatcher,
      menu_ids,
      js_event_listeners,
    })
  }

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    send_user_message(&self.context, Message::Task(Box::new(f)))
  }

  #[cfg(all(windows, feature = "system-tray"))]
  /// Deprecated. (not needed anymore)
  fn remove_system_tray(&self) -> Result<()> {
    send_user_message(&self.context, Message::Tray(TrayMessage::Close))
  }
}

impl Wry {
  fn init(event_loop: EventLoop<Message>) -> Result<Self> {
    let proxy = event_loop.create_proxy();
    let main_thread_id = current_thread().id();
    let web_context = WebContextStore::default();
    let global_shortcut_manager = Arc::new(Mutex::new(WryShortcutManager::new(&event_loop)));
    let clipboard_manager = Arc::new(Mutex::new(Clipboard::new()));
    let windows = Arc::new(Mutex::new(HashMap::default()));
    let window_event_listeners = WindowEventListeners::default();
    let menu_event_listeners = MenuEventListeners::default();

    #[cfg(feature = "system-tray")]
    let tray_context = TrayContext::default();

    let event_loop_context = Context {
      main_thread_id,
      proxy,
      window_event_listeners: window_event_listeners.clone(),
      menu_event_listeners: menu_event_listeners.clone(),
      main_thread: DispatcherMainThreadContext {
        window_target: event_loop.deref().clone(),
        web_context: web_context.clone(),
        global_shortcut_manager: global_shortcut_manager.clone(),
        clipboard_manager: clipboard_manager.clone(),
        windows: windows.clone(),
        #[cfg(feature = "system-tray")]
        tray_context: tray_context.clone(),
      },
    };

    let global_shortcut_listeners = GlobalShortcutListeners::default();
    let clipboard_manager_handle = ClipboardManagerWrapper {
      context: event_loop_context.clone(),
    };

    Ok(Self {
      main_thread_id,
      global_shortcut_manager,
      global_shortcut_manager_handle: GlobalShortcutManagerHandle {
        context: event_loop_context,
        shortcuts: Default::default(),
        listeners: global_shortcut_listeners,
      },
      clipboard_manager,
      clipboard_manager_handle,
      event_loop,
      windows,
      web_context,
      window_event_listeners,
      menu_event_listeners,
      #[cfg(feature = "system-tray")]
      tray_context,
    })
  }
}

impl Runtime for Wry {
  type Dispatcher = WryDispatcher;
  type Handle = WryHandle;
  type GlobalShortcutManager = GlobalShortcutManagerHandle;
  type ClipboardManager = ClipboardManagerWrapper;
  #[cfg(feature = "system-tray")]
  type TrayHandler = SystemTrayHandle;

  fn new() -> Result<Self> {
    let event_loop = EventLoop::<Message>::with_user_event();
    Self::init(event_loop)
  }

  #[cfg(any(windows, target_os = "linux"))]
  fn new_any_thread() -> Result<Self> {
    #[cfg(target_os = "linux")]
    use wry::application::platform::unix::EventLoopExtUnix;
    #[cfg(windows)]
    use wry::application::platform::windows::EventLoopExtWindows;
    let event_loop = EventLoop::<Message>::new_any_thread();
    Self::init(event_loop)
  }

  fn handle(&self) -> Self::Handle {
    WryHandle {
      context: Context {
        main_thread_id: self.main_thread_id,
        proxy: self.event_loop.create_proxy(),
        window_event_listeners: self.window_event_listeners.clone(),
        menu_event_listeners: self.menu_event_listeners.clone(),
        main_thread: DispatcherMainThreadContext {
          window_target: self.event_loop.deref().clone(),
          web_context: self.web_context.clone(),
          global_shortcut_manager: self.global_shortcut_manager.clone(),
          clipboard_manager: self.clipboard_manager.clone(),
          windows: self.windows.clone(),
          #[cfg(feature = "system-tray")]
          tray_context: self.tray_context.clone(),
        },
      },
    }
  }

  fn global_shortcut_manager(&self) -> Self::GlobalShortcutManager {
    self.global_shortcut_manager_handle.clone()
  }

  fn clipboard_manager(&self) -> Self::ClipboardManager {
    self.clipboard_manager_handle.clone()
  }

  fn create_window(&self, pending: PendingWindow<Self>) -> Result<DetachedWindow<Self>> {
    let label = pending.label.clone();
    let menu_ids = pending.menu_ids.clone();
    let js_event_listeners = pending.js_event_listeners.clone();
    let proxy = self.event_loop.create_proxy();
    let webview = create_webview(
      &self.event_loop,
      &self.web_context,
      Context {
        main_thread_id: self.main_thread_id,
        proxy: proxy.clone(),
        window_event_listeners: self.window_event_listeners.clone(),
        menu_event_listeners: self.menu_event_listeners.clone(),
        main_thread: DispatcherMainThreadContext {
          window_target: self.event_loop.deref().clone(),
          web_context: self.web_context.clone(),
          global_shortcut_manager: self.global_shortcut_manager.clone(),
          clipboard_manager: self.clipboard_manager.clone(),
          windows: self.windows.clone(),
          #[cfg(feature = "system-tray")]
          tray_context: self.tray_context.clone(),
        },
      },
      pending,
    )?;

    #[cfg(target_os = "windows")]
    {
      let id = webview.inner.window().id();
      if let WindowHandle::Webview(ref webview) = webview.inner {
        if let Some(controller) = webview.controller() {
          let proxy = self.event_loop.create_proxy();
          let mut token = EventRegistrationToken::default();
          unsafe {
            controller.GotFocus(
              FocusChangedEventHandler::create(Box::new(move |_, _| {
                let _ = proxy.send_event(Message::Webview(
                  id,
                  WebviewMessage::WebviewEvent(WebviewEvent::Focused(true)),
                ));
                Ok(())
              })),
              &mut token,
            )
          }
          .unwrap();
          let proxy = self.event_loop.create_proxy();
          unsafe {
            controller.LostFocus(
              FocusChangedEventHandler::create(Box::new(move |_, _| {
                let _ = proxy.send_event(Message::Webview(
                  id,
                  WebviewMessage::WebviewEvent(WebviewEvent::Focused(false)),
                ));
                Ok(())
              })),
              &mut token,
            )
          }
          .unwrap();
        }
      }
    }

    let dispatcher = WryDispatcher {
      window_id: webview.inner.window().id(),
      context: Context {
        main_thread_id: self.main_thread_id,
        proxy,
        window_event_listeners: self.window_event_listeners.clone(),
        menu_event_listeners: self.menu_event_listeners.clone(),
        main_thread: DispatcherMainThreadContext {
          window_target: self.event_loop.deref().clone(),
          web_context: self.web_context.clone(),
          global_shortcut_manager: self.global_shortcut_manager.clone(),
          clipboard_manager: self.clipboard_manager.clone(),
          windows: self.windows.clone(),
          #[cfg(feature = "system-tray")]
          tray_context: self.tray_context.clone(),
        },
      },
    };

    self
      .windows
      .lock()
      .unwrap()
      .insert(webview.inner.window().id(), webview);

    Ok(DetachedWindow {
      label,
      dispatcher,
      menu_ids,
      js_event_listeners,
    })
  }

  #[cfg(feature = "system-tray")]
  fn system_tray(&self, system_tray: SystemTray) -> Result<Self::TrayHandler> {
    let icon = system_tray
      .icon
      .expect("tray icon not set")
      .into_tray_icon();

    let mut items = HashMap::new();

    #[cfg(target_os = "macos")]
    let tray = SystemTrayBuilder::new(
      icon,
      system_tray
        .menu
        .map(|menu| to_wry_context_menu(&mut items, menu)),
    )
    .with_icon_as_template(system_tray.icon_as_template)
    .build(&self.event_loop)
    .map_err(|e| Error::SystemTray(Box::new(e)))?;

    #[cfg(not(target_os = "macos"))]
    let tray = SystemTrayBuilder::new(
      icon,
      system_tray
        .menu
        .map(|menu| to_wry_context_menu(&mut items, menu)),
    )
    .build(&self.event_loop)
    .map_err(|e| Error::SystemTray(Box::new(e)))?;

    *self.tray_context.items.lock().unwrap() = items;
    *self.tray_context.tray.lock().unwrap() = Some(Arc::new(Mutex::new(tray)));

    Ok(SystemTrayHandle {
      proxy: self.event_loop.create_proxy(),
    })
  }

  #[cfg(feature = "system-tray")]
  fn on_system_tray_event<F: Fn(&SystemTrayEvent) + Send + 'static>(&mut self, f: F) -> Uuid {
    let id = Uuid::new_v4();
    self
      .tray_context
      .listeners
      .lock()
      .unwrap()
      .insert(id, Box::new(f));
    id
  }

  #[cfg(target_os = "macos")]
  fn set_activation_policy(&mut self, activation_policy: ActivationPolicy) {
    self
      .event_loop
      .set_activation_policy(match activation_policy {
        ActivationPolicy::Regular => WryActivationPolicy::Regular,
        ActivationPolicy::Accessory => WryActivationPolicy::Accessory,
        ActivationPolicy::Prohibited => WryActivationPolicy::Prohibited,
        _ => unimplemented!(),
      });
  }

  fn run_iteration<F: FnMut(RunEvent) + 'static>(&mut self, mut callback: F) -> RunIteration {
    use wry::application::platform::run_return::EventLoopExtRunReturn;
    let windows = self.windows.clone();
    let web_context = &self.web_context;
    let window_event_listeners = self.window_event_listeners.clone();
    let menu_event_listeners = self.menu_event_listeners.clone();
    #[cfg(feature = "system-tray")]
    let tray_context = self.tray_context.clone();
    let global_shortcut_manager = self.global_shortcut_manager.clone();
    let global_shortcut_manager_handle = self.global_shortcut_manager_handle.clone();
    let clipboard_manager = self.clipboard_manager.clone();
    let mut iteration = RunIteration::default();

    self
      .event_loop
      .run_return(|event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::MainEventsCleared = &event {
          *control_flow = ControlFlow::Exit;
        }

        iteration = handle_event_loop(
          event,
          event_loop,
          control_flow,
          EventLoopIterationContext {
            callback: &mut callback,
            windows: windows.clone(),
            window_event_listeners: &window_event_listeners,
            global_shortcut_manager: global_shortcut_manager.clone(),
            global_shortcut_manager_handle: &global_shortcut_manager_handle,
            clipboard_manager: clipboard_manager.clone(),
            menu_event_listeners: &menu_event_listeners,
            #[cfg(feature = "system-tray")]
            tray_context: &tray_context,
          },
          web_context,
        );
      });

    iteration
  }

  fn run<F: FnMut(RunEvent) + 'static>(self, mut callback: F) {
    let windows = self.windows.clone();
    let web_context = self.web_context;
    let window_event_listeners = self.window_event_listeners.clone();
    let menu_event_listeners = self.menu_event_listeners.clone();
    #[cfg(feature = "system-tray")]
    let tray_context = self.tray_context;
    let global_shortcut_manager = self.global_shortcut_manager.clone();
    let global_shortcut_manager_handle = self.global_shortcut_manager_handle.clone();
    let clipboard_manager = self.clipboard_manager.clone();

    self.event_loop.run(move |event, event_loop, control_flow| {
      handle_event_loop(
        event,
        event_loop,
        control_flow,
        EventLoopIterationContext {
          callback: &mut callback,
          windows: windows.clone(),
          window_event_listeners: &window_event_listeners,
          global_shortcut_manager: global_shortcut_manager.clone(),
          global_shortcut_manager_handle: &global_shortcut_manager_handle,
          clipboard_manager: clipboard_manager.clone(),
          menu_event_listeners: &menu_event_listeners,
          #[cfg(feature = "system-tray")]
          tray_context: &tray_context,
        },
        &web_context,
      );
    })
  }
}

pub struct EventLoopIterationContext<'a> {
  callback: &'a mut (dyn FnMut(RunEvent) + 'static),
  windows: Arc<Mutex<HashMap<WindowId, WindowWrapper>>>,
  window_event_listeners: &'a WindowEventListeners,
  global_shortcut_manager: Arc<Mutex<WryShortcutManager>>,
  global_shortcut_manager_handle: &'a GlobalShortcutManagerHandle,
  clipboard_manager: Arc<Mutex<Clipboard>>,
  menu_event_listeners: &'a MenuEventListeners,
  #[cfg(feature = "system-tray")]
  tray_context: &'a TrayContext,
}

struct UserMessageContext<'a> {
  window_event_listeners: &'a WindowEventListeners,
  global_shortcut_manager: Arc<Mutex<WryShortcutManager>>,
  clipboard_manager: Arc<Mutex<Clipboard>>,
  menu_event_listeners: &'a MenuEventListeners,
  windows: Arc<Mutex<HashMap<WindowId, WindowWrapper>>>,
  #[cfg(feature = "system-tray")]
  tray_context: &'a TrayContext,
}

fn handle_user_message(
  event_loop: &EventLoopWindowTarget<Message>,
  message: Message,
  context: UserMessageContext<'_>,
  web_context: &WebContextStore,
) -> RunIteration {
  let UserMessageContext {
    window_event_listeners,
    menu_event_listeners,
    global_shortcut_manager,
    clipboard_manager,
    windows,
    #[cfg(feature = "system-tray")]
    tray_context,
  } = context;
  match message {
    Message::Task(task) => task(),
    Message::Window(id, window_message) => {
      if let Some(webview) = windows
        .lock()
        .expect("poisoned webview collection")
        .get_mut(&id)
      {
        let window = webview.inner.window();
        match window_message {
          #[cfg(any(debug_assertions, feature = "devtools"))]
          WindowMessage::OpenDevTools => {
            if let WindowHandle::Webview(w) = &webview.inner {
              w.devtool();
            }
          }
          // Getters
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
            .send(PhysicalSizeWrapper(webview.inner.inner_size()).into())
            .unwrap(),
          WindowMessage::OuterSize(tx) => tx
            .send(PhysicalSizeWrapper(window.outer_size()).into())
            .unwrap(),
          WindowMessage::IsFullscreen(tx) => tx.send(window.fullscreen().is_some()).unwrap(),
          WindowMessage::IsMaximized(tx) => tx.send(window.is_maximized()).unwrap(),
          WindowMessage::IsDecorated(tx) => tx.send(window.is_decorated()).unwrap(),
          WindowMessage::IsResizable(tx) => tx.send(window.is_resizable()).unwrap(),
          WindowMessage::IsVisible(tx) => tx.send(window.is_visible()).unwrap(),
          WindowMessage::IsMenuVisible(tx) => tx.send(window.is_menu_visible()).unwrap(),
          WindowMessage::CurrentMonitor(tx) => tx.send(window.current_monitor()).unwrap(),
          WindowMessage::PrimaryMonitor(tx) => tx.send(window.primary_monitor()).unwrap(),
          WindowMessage::AvailableMonitors(tx) => {
            tx.send(window.available_monitors().collect()).unwrap()
          }
          #[cfg(target_os = "macos")]
          WindowMessage::NSWindow(tx) => tx.send(NSWindow(window.ns_window())).unwrap(),
          #[cfg(windows)]
          WindowMessage::Hwnd(tx) => tx.send(Hwnd(HWND(window.hwnd() as _))).unwrap(),
          #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
          ))]
          WindowMessage::GtkWindow(tx) => tx.send(GtkWindow(window.gtk_window().clone())).unwrap(),
          // Setters
          WindowMessage::Center(tx) => {
            tx.send(center_window(window, webview.inner.inner_size()))
              .unwrap();
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
          WindowMessage::Close => panic!("cannot handle `WindowMessage::Close` on the main thread"),
          WindowMessage::SetDecorations(decorations) => window.set_decorations(decorations),
          WindowMessage::SetAlwaysOnTop(always_on_top) => window.set_always_on_top(always_on_top),
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
          WindowMessage::SetSkipTaskbar(_skip) => {
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            window.set_skip_taskbar(_skip);
          }
          WindowMessage::DragWindow => {
            let _ = window.drag_window();
          }
          WindowMessage::UpdateMenuItem(id, update) => {
            if let Some(menu_items) = webview.menu_items.as_mut() {
              let item = menu_items.get_mut(&id).expect("menu item not found");
              match update {
                MenuUpdate::SetEnabled(enabled) => item.set_enabled(enabled),
                MenuUpdate::SetTitle(title) => item.set_title(&title),
                MenuUpdate::SetSelected(selected) => item.set_selected(selected),
                #[cfg(target_os = "macos")]
                MenuUpdate::SetNativeImage(image) => {
                  item.set_native_image(NativeImageWrapper::from(image).0)
                }
              }
            }
          }
          WindowMessage::RequestRedraw => {
            window.request_redraw();
          }
        }
      }
    }
    Message::Webview(id, webview_message) => match webview_message {
      WebviewMessage::EvaluateScript(script) => {
        if let Some(WindowHandle::Webview(webview)) = windows
          .lock()
          .expect("poisoned webview collection")
          .get(&id)
          .map(|w| &w.inner)
        {
          if let Err(e) = webview.evaluate_script(&script) {
            #[cfg(debug_assertions)]
            eprintln!("{}", e);
          }
        }
      }
      WebviewMessage::Print => {
        if let Some(WindowHandle::Webview(webview)) = windows
          .lock()
          .expect("poisoned webview collection")
          .get(&id)
          .map(|w| &w.inner)
        {
          let _ = webview.print();
        }
      }
      WebviewMessage::WebviewEvent(event) => {
        if let Some(event) = WindowEventWrapper::from(&event).0 {
          for handler in window_event_listeners
            .lock()
            .unwrap()
            .get(&id)
            .unwrap()
            .lock()
            .unwrap()
            .values()
          {
            handler(&event);
          }
        }
      }
    },
    Message::CreateWebview(handler, sender) => match handler(event_loop, web_context) {
      Ok(webview) => {
        let window_id = webview.inner.window().id();
        windows
          .lock()
          .expect("poisoned webview collection")
          .insert(window_id, webview);
        sender.send(window_id).unwrap();
      }
      Err(e) => {
        #[cfg(debug_assertions)]
        eprintln!("{}", e);
      }
    },
    Message::CreateWindow(handler, sender) => {
      let (label, builder) = handler();
      if let Ok(window) = builder.build(event_loop) {
        let window_id = window.id();

        window_event_listeners
          .lock()
          .unwrap()
          .insert(window.id(), WindowEventListenersMap::default());

        menu_event_listeners
          .lock()
          .unwrap()
          .insert(window.id(), WindowMenuEventListeners::default());

        let w = Arc::new(window);

        windows.lock().expect("poisoned webview collection").insert(
          window_id,
          WindowWrapper {
            label,
            inner: WindowHandle::Window(w.clone()),
            menu_items: Default::default(),
          },
        );
        sender.send(Ok(Arc::downgrade(&w))).unwrap();
      } else {
        sender.send(Err(Error::CreateWindow)).unwrap();
      }
    }

    #[cfg(feature = "system-tray")]
    Message::Tray(tray_message) => match tray_message {
      TrayMessage::UpdateItem(menu_id, update) => {
        let mut tray = tray_context.items.as_ref().lock().unwrap();
        let item = tray.get_mut(&menu_id).expect("menu item not found");
        match update {
          MenuUpdate::SetEnabled(enabled) => item.set_enabled(enabled),
          MenuUpdate::SetTitle(title) => item.set_title(&title),
          MenuUpdate::SetSelected(selected) => item.set_selected(selected),
          #[cfg(target_os = "macos")]
          MenuUpdate::SetNativeImage(image) => {
            item.set_native_image(NativeImageWrapper::from(image).0)
          }
        }
      }
      TrayMessage::UpdateMenu(menu) => {
        if let Some(tray) = &*tray_context.tray.lock().unwrap() {
          let mut items = HashMap::new();
          tray
            .lock()
            .unwrap()
            .set_menu(&to_wry_context_menu(&mut items, menu));
          *tray_context.items.lock().unwrap() = items;
        }
      }
      TrayMessage::UpdateIcon(icon) => {
        if let Some(tray) = &*tray_context.tray.lock().unwrap() {
          tray.lock().unwrap().set_icon(icon.into_tray_icon());
        }
      }
      #[cfg(target_os = "macos")]
      TrayMessage::UpdateIconAsTemplate(is_template) => {
        if let Some(tray) = &*tray_context.tray.lock().unwrap() {
          tray.lock().unwrap().set_icon_as_template(is_template);
        }
      }
      TrayMessage::Close => {
        *tray_context.tray.lock().unwrap() = None;
        tray_context.listeners.lock().unwrap().clear();
        tray_context.items.lock().unwrap().clear();
      }
    },
    Message::GlobalShortcut(message) => match message {
      GlobalShortcutMessage::IsRegistered(accelerator, tx) => tx
        .send(
          global_shortcut_manager
            .lock()
            .unwrap()
            .is_registered(&accelerator),
        )
        .unwrap(),
      GlobalShortcutMessage::Register(accelerator, tx) => tx
        .send(
          global_shortcut_manager
            .lock()
            .unwrap()
            .register(accelerator)
            .map(GlobalShortcutWrapper)
            .map_err(|e| Error::GlobalShortcut(Box::new(e))),
        )
        .unwrap(),
      GlobalShortcutMessage::Unregister(shortcut, tx) => tx
        .send(
          global_shortcut_manager
            .lock()
            .unwrap()
            .unregister(shortcut.0)
            .map_err(|e| Error::GlobalShortcut(Box::new(e))),
        )
        .unwrap(),
      GlobalShortcutMessage::UnregisterAll(tx) => tx
        .send(
          global_shortcut_manager
            .lock()
            .unwrap()
            .unregister_all()
            .map_err(|e| Error::GlobalShortcut(Box::new(e))),
        )
        .unwrap(),
    },
    Message::Clipboard(message) => match message {
      ClipboardMessage::WriteText(text, tx) => {
        clipboard_manager.lock().unwrap().write_text(text);
        tx.send(()).unwrap();
      }
      ClipboardMessage::ReadText(tx) => tx
        .send(clipboard_manager.lock().unwrap().read_text())
        .unwrap(),
    },
  }

  let it = RunIteration {
    window_count: windows.lock().expect("poisoned webview collection").len(),
  };
  it
}

fn handle_event_loop(
  event: Event<'_, Message>,
  event_loop: &EventLoopWindowTarget<Message>,
  control_flow: &mut ControlFlow,
  context: EventLoopIterationContext<'_>,
  web_context: &WebContextStore,
) -> RunIteration {
  let EventLoopIterationContext {
    callback,
    windows,
    window_event_listeners,
    global_shortcut_manager,
    global_shortcut_manager_handle,
    clipboard_manager,
    menu_event_listeners,
    #[cfg(feature = "system-tray")]
    tray_context,
  } = context;
  if *control_flow == ControlFlow::Exit {
    return RunIteration {
      window_count: windows.lock().expect("poisoned webview collection").len(),
    };
  }

  *control_flow = ControlFlow::Wait;

  match event {
    Event::NewEvents(StartCause::Init) => {
      callback(RunEvent::Ready);
    }

    Event::NewEvents(StartCause::Poll) => {
      callback(RunEvent::Resumed);
    }

    Event::MainEventsCleared => {
      callback(RunEvent::MainEventsCleared);
    }

    Event::GlobalShortcutEvent(accelerator_id) => {
      for (id, handler) in &*global_shortcut_manager_handle.listeners.lock().unwrap() {
        if accelerator_id == *id {
          handler();
        }
      }
    }
    Event::MenuEvent {
      window_id,
      menu_id,
      origin: MenuType::MenuBar,
      ..
    } => {
      let window_id = window_id.unwrap(); // always Some on MenuBar event
      let event = MenuEvent {
        menu_item_id: menu_id.0,
      };
      let window_menu_event_listeners = {
        let listeners = menu_event_listeners.lock().unwrap();
        listeners.get(&window_id).cloned().unwrap_or_default()
      };
      for handler in window_menu_event_listeners.lock().unwrap().values() {
        handler(&event);
      }
    }
    #[cfg(feature = "system-tray")]
    Event::MenuEvent {
      window_id: _,
      menu_id,
      origin: MenuType::ContextMenu,
      ..
    } => {
      let event = SystemTrayEvent::MenuItemClick(menu_id.0);
      for handler in tray_context.listeners.lock().unwrap().values() {
        handler(&event);
      }
    }
    #[cfg(feature = "system-tray")]
    Event::TrayEvent {
      bounds,
      event,
      position: _cursor_position,
      ..
    } => {
      let (position, size) = (
        PhysicalPositionWrapper(bounds.position).into(),
        PhysicalSizeWrapper(bounds.size).into(),
      );
      let event = match event {
        TrayEvent::RightClick => SystemTrayEvent::RightClick { position, size },
        TrayEvent::DoubleClick => SystemTrayEvent::DoubleClick { position, size },
        // default to left click
        _ => SystemTrayEvent::LeftClick { position, size },
      };
      for handler in tray_context.listeners.lock().unwrap().values() {
        handler(&event);
      }
    }
    Event::WindowEvent {
      event, window_id, ..
    } => {
      // NOTE(amrbashir): we handle this event here instead of `match` statement below because
      // we want to focus the webview as soon as possible, especially on windows.
      if event == WryWindowEvent::Focused(true) {
        if let Some(WindowHandle::Webview(webview)) = windows
          .lock()
          .expect("poisoned webview collection")
          .get(&window_id)
          .map(|w| &w.inner)
        {
          webview.focus();
        }
      }

      {
        let windows_lock = windows.lock().expect("poisoned webview collection");
        if let Some(window_handle) = windows_lock.get(&window_id).map(|w| &w.inner) {
          if let Some(event) = WindowEventWrapper::parse(window_handle, &event).0 {
            drop(windows_lock);
            for handler in window_event_listeners
              .lock()
              .unwrap()
              .get(&window_id)
              .unwrap()
              .lock()
              .unwrap()
              .values()
            {
              handler(&event);
            }
          }
        }
      }

      match event {
        WryWindowEvent::CloseRequested => {
          on_close_requested(
            callback,
            window_id,
            windows.clone(),
            control_flow,
            window_event_listeners,
            menu_event_listeners.clone(),
          );
        }
        WryWindowEvent::Resized(_) => {
          if let Some(WindowHandle::Webview(webview)) = windows
            .lock()
            .expect("poisoned webview collection")
            .get(&window_id)
            .map(|w| &w.inner)
          {
            if let Err(e) = webview.resize() {
              #[cfg(debug_assertions)]
              eprintln!("{}", e);
            }
          }
        }
        _ => {}
      }
    }
    Event::UserEvent(message) => {
      if let Message::Window(id, WindowMessage::Close) = message {
        on_window_close(
          callback,
          id,
          windows.lock().expect("poisoned webview collection"),
          control_flow,
          #[cfg(target_os = "linux")]
          window_event_listeners,
          menu_event_listeners.clone(),
        );
      } else {
        return handle_user_message(
          event_loop,
          message,
          UserMessageContext {
            window_event_listeners,
            global_shortcut_manager,
            clipboard_manager,
            menu_event_listeners,
            windows,
            #[cfg(feature = "system-tray")]
            tray_context,
          },
          web_context,
        );
      }
    }
    _ => (),
  }

  let it = RunIteration {
    window_count: windows.lock().expect("poisoned webview collection").len(),
  };
  it
}

fn on_close_requested<'a>(
  callback: &'a mut (dyn FnMut(RunEvent) + 'static),
  window_id: WindowId,
  windows: Arc<Mutex<HashMap<WindowId, WindowWrapper>>>,
  control_flow: &mut ControlFlow,
  window_event_listeners: &WindowEventListeners,
  menu_event_listeners: MenuEventListeners,
) -> Option<WindowWrapper> {
  let (tx, rx) = channel();
  let windows_guard = windows.lock().expect("poisoned webview collection");
  if let Some(w) = windows_guard.get(&window_id) {
    let label = w.label.clone();
    drop(windows_guard);
    for handler in window_event_listeners
      .lock()
      .unwrap()
      .get(&window_id)
      .unwrap()
      .lock()
      .unwrap()
      .values()
    {
      handler(&WindowEvent::CloseRequested {
        label: label.clone(),
        signal_tx: tx.clone(),
      });
    }
    callback(RunEvent::CloseRequested {
      label,
      signal_tx: tx,
    });
    if let Ok(true) = rx.try_recv() {
      None
    } else {
      on_window_close(
        callback,
        window_id,
        windows.lock().expect("poisoned webview collection"),
        control_flow,
        #[cfg(target_os = "linux")]
        window_event_listeners,
        menu_event_listeners,
      )
    }
  } else {
    None
  }
}

fn on_window_close<'a>(
  callback: &'a mut (dyn FnMut(RunEvent) + 'static),
  window_id: WindowId,
  mut windows: MutexGuard<'a, HashMap<WindowId, WindowWrapper>>,
  control_flow: &mut ControlFlow,
  #[cfg(target_os = "linux")] window_event_listeners: &WindowEventListeners,
  menu_event_listeners: MenuEventListeners,
) -> Option<WindowWrapper> {
  #[allow(unused_mut)]
  let w = if let Some(mut webview) = windows.remove(&window_id) {
    let is_empty = windows.is_empty();
    drop(windows);
    menu_event_listeners.lock().unwrap().remove(&window_id);
    callback(RunEvent::WindowClose(webview.label.clone()));

    if is_empty {
      let (tx, rx) = channel();
      callback(RunEvent::ExitRequested {
        window_label: webview.label.clone(),
        tx,
      });

      let recv = rx.try_recv();
      let should_prevent = matches!(recv, Ok(ExitRequestedEventAction::Prevent));

      if !should_prevent {
        *control_flow = ControlFlow::Exit;
        callback(RunEvent::Exit);
      }
    }
    Some(webview)
  } else {
    None
  };
  // TODO: tao does not fire the destroyed event properly
  #[cfg(target_os = "linux")]
  {
    for handler in window_event_listeners
      .lock()
      .unwrap()
      .get(&window_id)
      .unwrap()
      .lock()
      .unwrap()
      .values()
    {
      handler(&WindowEvent::Destroyed);
    }
  }
  w
}

fn center_window(window: &Window, window_size: WryPhysicalSize<u32>) -> Result<()> {
  if let Some(monitor) = window.current_monitor() {
    let screen_size = monitor.size();
    let x = (screen_size.width as i32 - window_size.width as i32) / 2;
    let y = (screen_size.height as i32 - window_size.height as i32) / 2;
    window.set_outer_position(WryPhysicalPosition::new(x, y));
    Ok(())
  } else {
    Err(Error::FailedToGetMonitor)
  }
}

fn to_wry_menu(
  custom_menu_items: &mut HashMap<MenuHash, WryCustomMenuItem>,
  menu: Menu,
) -> MenuBar {
  let mut wry_menu = MenuBar::new();
  for item in menu.items {
    match item {
      MenuEntry::CustomItem(c) => {
        let mut attributes = MenuItemAttributesWrapper::from(&c).0;
        attributes = attributes.with_id(WryMenuId(c.id));
        #[allow(unused_mut)]
        let mut item = wry_menu.add_item(attributes);
        #[cfg(target_os = "macos")]
        if let Some(native_image) = c.native_image {
          item.set_native_image(NativeImageWrapper::from(native_image).0);
        }
        custom_menu_items.insert(c.id, item);
      }
      MenuEntry::NativeItem(i) => {
        wry_menu.add_native_item(MenuItemWrapper::from(i).0);
      }
      MenuEntry::Submenu(submenu) => {
        wry_menu.add_submenu(
          &submenu.title,
          submenu.enabled,
          to_wry_menu(custom_menu_items, submenu.inner),
        );
      }
    }
  }
  wry_menu
}

fn create_webview(
  event_loop: &EventLoopWindowTarget<Message>,
  web_context: &WebContextStore,
  context: Context,
  pending: PendingWindow<Wry>,
) -> Result<WindowWrapper> {
  #[allow(unused_mut)]
  let PendingWindow {
    webview_attributes,
    uri_scheme_protocols,
    mut window_builder,
    ipc_handler,
    file_drop_handler,
    label,
    url,
    menu_ids,
    js_event_listeners,
    ..
  } = pending;

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

  context
    .window_event_listeners
    .lock()
    .unwrap()
    .insert(window.id(), WindowEventListenersMap::default());

  context
    .menu_event_listeners
    .lock()
    .unwrap()
    .insert(window.id(), WindowMenuEventListeners::default());

  if window_builder.center {
    let _ = center_window(&window, window.inner_size());
  }
  let mut webview_builder = WebViewBuilder::new(window)
    .map_err(|e| Error::CreateWebview(Box::new(e)))?
    .with_url(&url)
    .unwrap() // safe to unwrap because we validate the URL beforehand
    .with_transparent(is_window_transparent);
  if let Some(handler) = ipc_handler {
    webview_builder = webview_builder.with_ipc_handler(create_ipc_handler(
      context.clone(),
      label.clone(),
      menu_ids.clone(),
      js_event_listeners.clone(),
      handler,
    ));
  }
  if let Some(handler) = file_drop_handler {
    webview_builder = webview_builder.with_file_drop_handler(create_file_drop_handler(
      context,
      label.clone(),
      menu_ids,
      js_event_listeners,
      handler,
    ));
  }
  for (scheme, protocol) in uri_scheme_protocols {
    webview_builder = webview_builder.with_custom_protocol(scheme, move |wry_request| {
      protocol(&HttpRequestWrapper::from(wry_request).0)
        .map(|tauri_response| HttpResponseWrapper::from(tauri_response).0)
        .map_err(|_| wry::Error::InitScriptError)
    });
  }

  for script in webview_attributes.initialization_scripts {
    webview_builder = webview_builder.with_initialization_script(&script);
  }

  let mut web_context = web_context.lock().expect("poisoned WebContext store");
  let is_first_context = web_context.is_empty();
  let automation_enabled = std::env::var("TAURI_AUTOMATION").as_deref() == Ok("true");
  let web_context = match web_context.entry(
    // force a unique WebContext when automation is false;
    // the context must be stored on the HashMap because it must outlive the WebView on macOS
    if automation_enabled {
      webview_attributes.data_directory.clone()
    } else {
      // random unique key
      Some(Uuid::new_v4().to_hyphenated().to_string().into())
    },
  ) {
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

  if webview_attributes.clipboard {
    webview_builder.webview.clipboard = true;
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  {
    webview_builder = webview_builder.with_dev_tool(true);
  }

  let webview = webview_builder
    .with_web_context(web_context)
    .build()
    .map_err(|e| Error::CreateWebview(Box::new(e)))?;

  Ok(WindowWrapper {
    label,
    inner: WindowHandle::Webview(webview),
    menu_items,
  })
}

/// Create a wry ipc handler from a tauri ipc handler.
fn create_ipc_handler(
  context: Context,
  label: String,
  menu_ids: Arc<Mutex<HashMap<MenuHash, MenuId>>>,
  js_event_listeners: Arc<Mutex<HashMap<JsEventListenerKey, HashSet<u64>>>>,
  handler: WebviewIpcHandler<Wry>,
) -> Box<dyn Fn(&Window, String) + 'static> {
  Box::new(move |window, request| {
    handler(
      DetachedWindow {
        dispatcher: WryDispatcher {
          window_id: window.id(),
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

/// Create a wry file drop handler from a tauri file drop handler.
fn create_file_drop_handler(
  context: Context,
  label: String,
  menu_ids: Arc<Mutex<HashMap<MenuHash, MenuId>>>,
  js_event_listeners: Arc<Mutex<HashMap<JsEventListenerKey, HashSet<u64>>>>,
  handler: FileDropHandler<Wry>,
) -> Box<dyn Fn(&Window, WryFileDropEvent) -> bool + 'static> {
  Box::new(move |window, event| {
    handler(
      FileDropEventWrapper(event).into(),
      DetachedWindow {
        dispatcher: WryDispatcher {
          window_id: window.id(),
          context: context.clone(),
        },
        label: label.clone(),
        menu_ids: menu_ids.clone(),
        js_event_listeners: js_event_listeners.clone(),
      },
    )
  })
}
