// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The [`wry`] Tauri [`Runtime`].

#[cfg(feature = "clipboard")]
mod clipboard;
mod dispatcher;
#[cfg(all(desktop, feature = "global-shortcut"))]
mod global_shortcut;
mod macros;
mod menu;
#[cfg(all(desktop, feature = "system-tray"))]
mod system_tray;
mod webview;
mod window;
mod wrappers;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, Weak};
use std::thread::{current as current_thread, ThreadId};

use raw_window_handle::{HasRawDisplayHandle, RawDisplayHandle};
#[cfg(all(desktop, feature = "system-tray"))]
pub use tauri_runtime::system_tray::SystemTrayId;
#[cfg(all(desktop, feature = "system-tray"))]
use tauri_runtime::system_tray::{SystemTray, SystemTrayEvent};
use tauri_runtime::window::{DetachedWindow, PendingWindow};
#[cfg(target_os = "macos")]
use tauri_runtime::{menu::NativeImage, ActivationPolicy};
use tauri_runtime::{
  DeviceEventFilter, Error, EventLoopProxy, Result, RunEvent, RunIteration, Runtime, RuntimeHandle,
  UserEvent,
};
use tauri_utils::debug_eprintln;

pub use wry;
#[cfg(feature = "clipboard")]
use wry::application::clipboard::Clipboard as WryClipboard;
use wry::application::event::{Event, StartCause};
use wry::application::event_loop::{
  ControlFlow, EventLoop, EventLoopProxy as WryEventLoopProxy, EventLoopWindowTarget,
};
use wry::application::menu::MenuType;
#[cfg(target_os = "macos")]
use wry::application::platform::macos::EventLoopWindowTargetExtMacOS;
#[cfg(target_os = "macos")]
pub use wry::application::platform::macos::{
  ActivationPolicy as WryActivationPolicy, CustomMenuItemExtMacOS, EventLoopExtMacOS,
  NativeImage as WryNativeImage, WindowExtMacOS,
};
use wry::application::window::Window as WryWindow;
pub use wry::application::window::{WindowBuilder as WryWindowBuilder, WindowId};
#[cfg(target_os = "android")]
use wry::webview::{
  prelude::{dispatch, find_class},
  WebViewBuilderExtAndroid, WebviewExtAndroid,
};
use wry::webview::{FileDropEvent as WryFileDropEvent, WebContext};

#[cfg(feature = "clipboard")]
use crate::clipboard::*;
pub use crate::dispatcher::*;
#[cfg(all(desktop, feature = "global-shortcut"))]
use crate::global_shortcut::*;
use crate::menu::*;
#[cfg(all(desktop, feature = "system-tray"))]
use crate::system_tray::*;
pub use crate::webview::Webview;
pub use crate::window::WindowMessage;
use crate::window::*;
use crate::wrappers::*;

pub type WebviewId = u64;
type IpcHandler = dyn Fn(&WryWindow, String) + 'static;
type FileDropHandler = dyn Fn(&WryWindow, WryFileDropEvent) -> bool + 'static;
pub type WebContextStore = Arc<Mutex<HashMap<Option<PathBuf>, WebContext>>>;

#[derive(Debug, Clone, Default)]
pub struct WebviewIdStore(Arc<Mutex<HashMap<WindowId, WebviewId>>>);

impl WebviewIdStore {
  pub fn insert(&self, w: WindowId, id: WebviewId) {
    self.0.lock().unwrap().insert(w, id);
  }

  fn get(&self, w: &WindowId) -> Option<WebviewId> {
    self.0.lock().unwrap().get(w).copied()
  }
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub enum ApplicationMessage {
  Show,
  Hide,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum WebviewEvent {
  Focused(bool),
}

#[derive(Debug, Clone)]
pub enum WebviewMessage {
  EvaluateScript(String),
  #[allow(dead_code)]
  WebviewEvent(WebviewEvent),
  Print,
}

pub type CreateWebviewClosure<T> =
  Box<dyn FnOnce(&EventLoopWindowTarget<Message<T>>, &WebContextStore) -> Result<Window> + Send>;

pub enum Message<T: 'static> {
  Task(Box<dyn FnOnce() + Send>),
  #[cfg(target_os = "macos")]
  Application(ApplicationMessage),
  Window(WebviewId, WindowMessage),
  Webview(WebviewId, WebviewMessage),
  #[cfg(all(desktop, feature = "system-tray"))]
  Tray(SystemTrayId, TrayMessage),
  CreateWebview(WebviewId, CreateWebviewClosure<T>),
  CreateWindow(
    WebviewId,
    Box<dyn FnOnce() -> (String, WryWindowBuilder) + Send>,
    Sender<Result<Weak<WryWindow>>>,
  ),
  #[cfg(all(desktop, feature = "global-shortcut"))]
  GlobalShortcut(GlobalShortcutMessage),
  #[cfg(feature = "clipboard")]
  Clipboard(ClipboardMessage),
  UserEvent(T),
}

impl<T: UserEvent> Clone for Message<T> {
  fn clone(&self) -> Self {
    match self {
      Self::Webview(i, m) => Self::Webview(*i, m.clone()),
      #[cfg(all(desktop, feature = "system-tray"))]
      Self::Tray(i, m) => Self::Tray(*i, m.clone()),
      #[cfg(all(desktop, feature = "global-shortcut"))]
      Self::GlobalShortcut(m) => Self::GlobalShortcut(m.clone()),
      #[cfg(feature = "clipboard")]
      Self::Clipboard(m) => Self::Clipboard(m.clone()),
      Self::UserEvent(t) => Self::UserEvent(t.clone()),
      _ => unimplemented!(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct EventProxy<T: UserEvent>(WryEventLoopProxy<Message<T>>);

#[cfg(target_os = "ios")]
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for EventProxy<T> {}

impl<T: UserEvent> EventLoopProxy<T> for EventProxy<T> {
  fn send_event(&self, event: T) -> Result<()> {
    self
      .0
      .send_event(Message::UserEvent(event))
      .map_err(|_| Error::EventLoopClosed)
  }
}

pub trait PluginBuilder<T: UserEvent> {
  type Plugin: Plugin<T>;
  fn build(self, context: Context<T>) -> Self::Plugin;
}

pub trait Plugin<T: UserEvent> {
  fn on_event(
    &mut self,
    event: &Event<Message<T>>,
    event_loop: &EventLoopWindowTarget<Message<T>>,
    proxy: &WryEventLoopProxy<Message<T>>,
    control_flow: &mut ControlFlow,
    context: EventLoopIterationContext<'_, T>,
    web_context: &WebContextStore,
  ) -> bool;
}

pub(crate) fn send_user_message<T: UserEvent>(
  context: &Context<T>,
  message: Message<T>,
) -> Result<()> {
  if current_thread().id() == context.main_thread_id {
    handle_user_message(
      &context.main_thread_context.window_target,
      message,
      UserMessageContext {
        webview_id_map: context.webview_id_store.clone(),
        #[cfg(all(desktop, feature = "global-shortcut"))]
        shortcut_manager: context.main_thread_context.shortcut_manager.clone(),
        #[cfg(feature = "clipboard")]
        clipboard: context.main_thread_context.clipboard.clone(),
        windows: context.main_thread_context.windows.clone(),
        #[cfg(all(desktop, feature = "system-tray"))]
        system_tray_manager: context.main_thread_context.system_tray_manager.clone(),
      },
      &context.main_thread_context.web_context,
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
pub struct Context<T: UserEvent> {
  pub webview_id_store: WebviewIdStore,
  main_thread_id: ThreadId,
  main_thread_context: DispatcherMainThreadContext<T>,
  pub proxy: WryEventLoopProxy<Message<T>>,
  plugins: Arc<Mutex<Vec<Box<dyn Plugin<T> + Send>>>>,
  next_id: Arc<AtomicU64>,
}

impl<T: UserEvent> fmt::Debug for Context<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Context")
      .field("main_thread_id", &self.main_thread_id)
      .field("proxy", &self.proxy)
      .field("main_thread_context", &self.main_thread_context)
      .field("next_id", &self.next_id)
      .finish()
  }
}

impl<T: UserEvent> Context<T> {
  pub fn run_threaded<R, F>(&self, f: F) -> R
  where
    F: FnOnce(Option<&DispatcherMainThreadContext<T>>) -> R,
  {
    let is_main_thread = current_thread().id() == self.main_thread_id;
    let maybe_context = is_main_thread.then_some(&self.main_thread_context);
    f(maybe_context)
  }

  fn next_id(&self) -> u64 {
    self
      .next_id
      .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
  }

  fn create_webview(&self, pending: PendingWindow<T, Wry<T>>) -> Result<DetachedWindow<T, Wry<T>>> {
    let label = pending.label.clone();
    let menu_ids = pending.menu_ids.clone();
    let js_event_listeners = pending.js_event_listeners.clone();
    let context = self.clone();
    let window_id = self.next_id();

    send_user_message(
      self,
      Message::CreateWebview(
        window_id,
        Box::new(move |event_loop, web_context| {
          Window::new(window_id, event_loop, web_context, context, pending)
        }),
      ),
    )?;

    let dispatcher = WryDispatcher {
      webview_id: window_id,
      context: self.clone(),
    };
    Ok(DetachedWindow {
      label,
      dispatcher,
      menu_ids,
      js_event_listeners,
    })
  }
}

#[derive(Debug, Clone)]
pub struct DispatcherMainThreadContext<T: UserEvent> {
  pub window_target: EventLoopWindowTarget<Message<T>>,
  pub web_context: WebContextStore,
  #[cfg(all(desktop, feature = "global-shortcut"))]
  pub shortcut_manager: Arc<Mutex<WryShortcutManager>>,
  #[cfg(feature = "clipboard")]
  pub clipboard: Arc<Mutex<WryClipboard>>,
  pub windows: Arc<RefCell<HashMap<WebviewId, Window>>>,
  #[cfg(all(desktop, feature = "system-tray"))]
  system_tray_manager: SystemTrayManager,
}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Send for DispatcherMainThreadContext<T> {}

// SAFETY: we ensure this type is only used on the main thread.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for DispatcherMainThreadContext<T> {}

pub struct EventLoopIterationContext<'a, T: UserEvent> {
  pub callback: &'a mut (dyn FnMut(RunEvent<T>) + 'static),
  pub webview_id_store: WebviewIdStore,
  pub windows: Arc<RefCell<HashMap<WebviewId, Window>>>,
  #[cfg(all(desktop, feature = "global-shortcut"))]
  pub shortcut_manager: Arc<Mutex<WryShortcutManager>>,
  #[cfg(all(desktop, feature = "global-shortcut"))]
  pub global_shortcut_manager: &'a GlobalShortcutManager<T>,
  #[cfg(feature = "clipboard")]
  pub clipboard: Arc<Mutex<WryClipboard>>,
  #[cfg(all(desktop, feature = "system-tray"))]
  pub system_tray_manager: SystemTrayManager,
}

struct UserMessageContext {
  windows: Arc<RefCell<HashMap<WebviewId, Window>>>,
  webview_id_map: WebviewIdStore,
  #[cfg(all(desktop, feature = "global-shortcut"))]
  shortcut_manager: Arc<Mutex<WryShortcutManager>>,
  #[cfg(feature = "clipboard")]
  clipboard: Arc<Mutex<WryClipboard>>,
  #[cfg(all(desktop, feature = "system-tray"))]
  system_tray_manager: SystemTrayManager,
}

/// A Tauri [`Runtime`] wrapper around wry.
pub struct Wry<T: UserEvent> {
  context: Context<T>,
  #[cfg(all(desktop, feature = "global-shortcut"))]
  global_shortcut_manager: GlobalShortcutManager<T>,
  #[cfg(feature = "clipboard")]
  clipboard_manager: ClipboardManager<T>,
  event_loop: EventLoop<Message<T>>,
}

impl<T: UserEvent> fmt::Debug for Wry<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut d = f.debug_struct("Wry");
    d.field("main_thread_id", &self.context.main_thread_id)
      .field("event_loop", &self.event_loop)
      .field("windows", &self.context.main_thread_context.windows)
      .field("web_context", &self.context.main_thread_context.web_context);

    #[cfg(all(desktop, feature = "system-tray"))]
    d.field(
      "system_tray_manager",
      &self.context.main_thread_context.system_tray_manager,
    );

    #[cfg(all(desktop, feature = "global-shortcut"))]
    #[cfg(feature = "global-shortcut")]
    d.field(
      "shortcut_manager",
      &self.context.main_thread_context.shortcut_manager,
    )
    .field("global_shortcut_manager", &self.global_shortcut_manager);

    #[cfg(feature = "clipboard")]
    d.field("clipboard", &self.context.main_thread_context.clipboard)
      .field("clipboard_manager", &self.clipboard_manager);

    d.finish()
  }
}

impl<T: UserEvent> Wry<T> {
  fn init(event_loop: EventLoop<Message<T>>) -> Result<Self> {
    let main_thread_id = current_thread().id();
    let web_context = WebContextStore::default();

    #[cfg(all(desktop, feature = "global-shortcut"))]
    let shortcut_manager = Arc::new(Mutex::new(WryShortcutManager::new(&event_loop)));

    #[cfg(feature = "clipboard")]
    let clipboard = Arc::new(Mutex::new(WryClipboard::new()));

    let windows = Arc::new(RefCell::new(HashMap::default()));
    let webview_id_map = WebviewIdStore::default();

    #[cfg(all(desktop, feature = "system-tray"))]
    let system_tray_manager = Default::default();

    let context = Context {
      webview_id_store: webview_id_map,
      main_thread_id,
      next_id: Arc::new(AtomicU64::new(1)),
      proxy: event_loop.create_proxy(),
      main_thread_context: DispatcherMainThreadContext {
        window_target: event_loop.deref().clone(),
        web_context,
        #[cfg(all(desktop, feature = "global-shortcut"))]
        shortcut_manager,
        #[cfg(feature = "clipboard")]
        clipboard,
        windows,
        #[cfg(all(desktop, feature = "system-tray"))]
        system_tray_manager,
      },
      plugins: Default::default(),
    };

    #[cfg(all(desktop, feature = "global-shortcut"))]
    let global_shortcut_manager = GlobalShortcutManager {
      context: context.clone(),
      shortcuts_store: Default::default(),
      listeners_store: Default::default(),
    };

    #[cfg(feature = "clipboard")]
    #[allow(clippy::redundant_clone)]
    let clipboard_manager = ClipboardManager {
      context: context.clone(),
    };

    Ok(Self {
      context,
      #[cfg(all(desktop, feature = "global-shortcut"))]
      global_shortcut_manager,
      #[cfg(feature = "clipboard")]
      clipboard_manager,
      event_loop,
    })
  }
}

impl<T: UserEvent> Runtime<T> for Wry<T> {
  type Dispatcher = WryDispatcher<T>;
  type Handle = WryHandle<T>;

  #[cfg(all(desktop, feature = "global-shortcut"))]
  type GlobalShortcutManager = GlobalShortcutManager<T>;

  #[cfg(feature = "clipboard")]
  type ClipboardManager = ClipboardManager<T>;

  #[cfg(all(desktop, feature = "system-tray"))]
  type SystemTrayHandler = SystemTrayHandle<T>;

  type EventLoopProxy = EventProxy<T>;

  fn new() -> Result<Self> {
    let event_loop = EventLoop::<Message<T>>::with_user_event();
    Self::init(event_loop)
  }

  #[cfg(any(windows, linuxy))]
  fn new_any_thread() -> Result<Self> {
    #[cfg(linuxy)]
    use wry::application::platform::unix::EventLoopExtUnix;
    #[cfg(windows)]
    use wry::application::platform::windows::EventLoopExtWindows;
    let event_loop = EventLoop::<Message<T>>::new_any_thread();
    Self::init(event_loop)
  }

  fn create_proxy(&self) -> EventProxy<T> {
    EventProxy(self.event_loop.create_proxy())
  }

  fn handle(&self) -> Self::Handle {
    WryHandle {
      context: self.context.clone(),
    }
  }

  #[cfg(all(desktop, feature = "global-shortcut"))]
  fn global_shortcut_manager(&self) -> Self::GlobalShortcutManager {
    self.global_shortcut_manager.clone()
  }

  #[cfg(feature = "clipboard")]
  fn clipboard_manager(&self) -> Self::ClipboardManager {
    self.clipboard_manager.clone()
  }

  fn create_window(&self, pending: PendingWindow<T, Self>) -> Result<DetachedWindow<T, Self>> {
    let label = pending.label.clone();
    let menu_ids = pending.menu_ids.clone();
    let js_event_listeners = pending.js_event_listeners.clone();
    let window_id = self.context.next_id();

    let webview = Window::new(
      window_id,
      &self.event_loop,
      &self.context.main_thread_context.web_context,
      self.context.clone(),
      pending,
    )?;

    let dispatcher = WryDispatcher {
      webview_id: window_id,
      context: self.context.clone(),
    };

    self
      .context
      .main_thread_context
      .windows
      .borrow_mut()
      .insert(window_id, webview);

    Ok(DetachedWindow {
      label,
      dispatcher,
      menu_ids,
      js_event_listeners,
    })
  }

  #[cfg(all(desktop, feature = "system-tray"))]
  fn system_tray(&self, system_tray: SystemTray) -> Result<Self::SystemTrayHandler> {
    let id = system_tray.id;
    let (tary, listeners, items) = create_system_tray(id, system_tray, &self.event_loop)?;

    self
      .context
      .main_thread_context
      .system_tray_manager
      .trays
      .lock()
      .unwrap()
      .insert(
        id,
        SystemTrayContext {
          tray: Arc::new(Mutex::new(Some(tary))),
          listeners_store: Arc::new(Mutex::new(listeners)),
          tray_menu_items_store: Arc::new(Mutex::new(items)),
        },
      );

    Ok(SystemTrayHandle {
      context: self.context.clone(),
      id,
      proxy: self.event_loop.create_proxy(),
    })
  }

  #[cfg(all(desktop, feature = "system-tray"))]
  fn on_system_tray_event<F: Fn(SystemTrayId, &SystemTrayEvent) + Send + 'static>(&mut self, f: F) {
    self
      .context
      .main_thread_context
      .system_tray_manager
      .global_listeners
      .lock()
      .unwrap()
      .push(Arc::new(Box::new(f)));
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

  #[cfg(target_os = "macos")]
  fn show(&self) {
    self.event_loop.show_application();
  }

  #[cfg(target_os = "macos")]
  fn hide(&self) {
    self.event_loop.hide_application();
  }

  fn set_device_event_filter(&mut self, filter: DeviceEventFilter) {
    self
      .event_loop
      .set_device_event_filter(DeviceEventFilterWrapper::from(filter).0);
  }

  #[cfg(desktop)]
  fn run_iteration<F: FnMut(RunEvent<T>) + 'static>(&mut self, mut callback: F) -> RunIteration {
    use wry::application::platform::run_return::EventLoopExtRunReturn;
    let windows = self.context.main_thread_context.windows.clone();
    let webview_id_map = self.context.webview_id_store.clone();
    let web_context = &self.context.main_thread_context.web_context;
    let plugins = self.context.plugins.clone();
    #[cfg(all(desktop, feature = "system-tray"))]
    let system_tray_manager = self.context.main_thread_context.system_tray_manager.clone();

    #[cfg(all(desktop, feature = "global-shortcut"))]
    let shortcut_manager = self.context.main_thread_context.shortcut_manager.clone();
    #[cfg(all(desktop, feature = "global-shortcut"))]
    let global_shortcut_manager = self.global_shortcut_manager.clone();

    #[cfg(feature = "clipboard")]
    let clipboard = self.context.main_thread_context.clipboard.clone();
    let mut iteration = RunIteration::default();

    let proxy = self.event_loop.create_proxy();

    self
      .event_loop
      .run_return(|event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::MainEventsCleared = &event {
          *control_flow = ControlFlow::Exit;
        }

        for p in plugins.lock().unwrap().iter_mut() {
          let prevent_default = p.on_event(
            &event,
            event_loop,
            &proxy,
            control_flow,
            EventLoopIterationContext {
              callback: &mut callback,
              webview_id_store: webview_id_map.clone(),
              windows: windows.clone(),
              #[cfg(all(desktop, feature = "global-shortcut"))]
              shortcut_manager: shortcut_manager.clone(),
              #[cfg(all(desktop, feature = "global-shortcut"))]
              global_shortcut_manager: &global_shortcut_manager,
              #[cfg(feature = "clipboard")]
              clipboard: clipboard.clone(),
              #[cfg(all(desktop, feature = "system-tray"))]
              system_tray_manager: system_tray_manager.clone(),
            },
            web_context,
          );
          if prevent_default {
            return;
          }
        }

        iteration = handle_event_loop(
          event,
          event_loop,
          control_flow,
          EventLoopIterationContext {
            callback: &mut callback,
            windows: windows.clone(),
            webview_id_store: webview_id_map.clone(),
            #[cfg(all(desktop, feature = "global-shortcut"))]
            shortcut_manager: shortcut_manager.clone(),
            #[cfg(all(desktop, feature = "global-shortcut"))]
            global_shortcut_manager: &global_shortcut_manager,
            #[cfg(feature = "clipboard")]
            clipboard: clipboard.clone(),
            #[cfg(all(desktop, feature = "system-tray"))]
            system_tray_manager: system_tray_manager.clone(),
          },
          web_context,
        );
      });

    iteration
  }

  fn run<F: FnMut(RunEvent<T>) + 'static>(self, mut callback: F) {
    let windows = self.context.main_thread_context.windows.clone();
    let webview_id_map = self.context.webview_id_store.clone();
    let web_context = self.context.main_thread_context.web_context;
    let plugins = self.context.plugins.clone();

    #[cfg(all(desktop, feature = "system-tray"))]
    let system_tray_manager = self.context.main_thread_context.system_tray_manager;

    #[cfg(all(desktop, feature = "global-shortcut"))]
    let shortcut_manager = self.context.main_thread_context.shortcut_manager.clone();
    #[cfg(all(desktop, feature = "global-shortcut"))]
    let global_shortcut_manager = self.global_shortcut_manager.clone();

    #[cfg(feature = "clipboard")]
    let clipboard = self.context.main_thread_context.clipboard.clone();

    let proxy = self.event_loop.create_proxy();

    self.event_loop.run(move |event, event_loop, control_flow| {
      for p in plugins.lock().unwrap().iter_mut() {
        let prevent_default = p.on_event(
          &event,
          event_loop,
          &proxy,
          control_flow,
          EventLoopIterationContext {
            callback: &mut callback,
            webview_id_store: webview_id_map.clone(),
            windows: windows.clone(),
            #[cfg(all(desktop, feature = "global-shortcut"))]
            shortcut_manager: shortcut_manager.clone(),
            #[cfg(all(desktop, feature = "global-shortcut"))]
            global_shortcut_manager: &global_shortcut_manager,
            #[cfg(feature = "clipboard")]
            clipboard: clipboard.clone(),
            #[cfg(all(desktop, feature = "system-tray"))]
            system_tray_manager: system_tray_manager.clone(),
          },
          &web_context,
        );
        if prevent_default {
          return;
        }
      }
      handle_event_loop(
        event,
        event_loop,
        control_flow,
        EventLoopIterationContext {
          callback: &mut callback,
          webview_id_store: webview_id_map.clone(),
          windows: windows.clone(),
          #[cfg(all(desktop, feature = "global-shortcut"))]
          shortcut_manager: shortcut_manager.clone(),
          #[cfg(all(desktop, feature = "global-shortcut"))]
          global_shortcut_manager: &global_shortcut_manager,
          #[cfg(feature = "clipboard")]
          clipboard: clipboard.clone(),
          #[cfg(all(desktop, feature = "system-tray"))]
          system_tray_manager: system_tray_manager.clone(),
        },
        &web_context,
      );
    })
  }
}

/// A handle to the Wry runtime.
#[derive(Debug, Clone)]
pub struct WryHandle<T: UserEvent> {
  context: Context<T>,
}

// SAFETY: this is safe since the `Context` usage is guarded on `send_user_message`.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T: UserEvent> Sync for WryHandle<T> {}

impl<T: UserEvent> WryHandle<T> {
  /// Creates a new tao window using a callback, and returns its window id.
  pub fn create_tao_window<F: FnOnce() -> (String, WryWindowBuilder) + Send + 'static>(
    &self,
    f: F,
  ) -> Result<Weak<WryWindow>> {
    let (tx, rx) = channel();
    send_user_message(
      &self.context,
      Message::CreateWindow(self.context.next_id(), Box::new(f), tx),
    )?;
    rx.recv().unwrap()
  }

  /// Gets the [`WebviewId'] associated with the given [`WindowId`].
  pub fn webview_id(&self, window_id: WindowId) -> WebviewId {
    *self
      .context
      .webview_id_store
      .0
      .lock()
      .unwrap()
      .get(&window_id)
      .unwrap()
  }

  /// Send a message to the event loop.
  pub fn send_event(&self, message: Message<T>) -> Result<()> {
    self
      .context
      .proxy
      .send_event(message)
      .map_err(|_| Error::FailedToSendMessage)?;
    Ok(())
  }

  pub fn plugin<P: PluginBuilder<T> + 'static>(&mut self, plugin: P)
  where
    <P as PluginBuilder<T>>::Plugin: Send,
  {
    self
      .context
      .plugins
      .lock()
      .unwrap()
      .push(Box::new(plugin.build(self.context.clone())));
  }
}

impl<T: UserEvent> RuntimeHandle<T> for WryHandle<T> {
  type Runtime = Wry<T>;

  fn create_proxy(&self) -> EventProxy<T> {
    EventProxy(self.context.proxy.clone())
  }

  // Creates a window by dispatching a message to the event loop.
  // Note that this must be called from a separate thread, otherwise the channel will introduce a deadlock.
  fn create_window(
    &self,
    pending: PendingWindow<T, Self::Runtime>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    self.context.create_webview(pending)
  }

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    send_user_message(&self.context, Message::Task(Box::new(f)))
  }

  #[cfg(all(desktop, feature = "system-tray"))]
  fn system_tray(
    &self,
    system_tray: SystemTray,
  ) -> Result<<Self::Runtime as Runtime<T>>::SystemTrayHandler> {
    let id = system_tray.id;
    let (tx, rx) = channel();
    send_user_message(
      &self.context,
      Message::Tray(id, TrayMessage::Create(system_tray, tx)),
    )?;
    rx.recv().unwrap()?;
    Ok(SystemTrayHandle {
      context: self.context.clone(),
      id,
      proxy: self.context.proxy.clone(),
    })
  }

  fn raw_display_handle(&self) -> RawDisplayHandle {
    self
      .context
      .main_thread_context
      .window_target
      .raw_display_handle()
  }

  #[cfg(target_os = "macos")]
  fn show(&self) -> tauri_runtime::Result<()> {
    send_user_message(
      &self.context,
      Message::Application(ApplicationMessage::Show),
    )
  }

  #[cfg(target_os = "macos")]
  fn hide(&self) -> tauri_runtime::Result<()> {
    send_user_message(
      &self.context,
      Message::Application(ApplicationMessage::Hide),
    )
  }

  #[cfg(target_os = "android")]
  fn find_class<'a>(
    &'a self,
    env: jni::JNIEnv<'a>,
    activity: jni::objects::JObject<'a>,
    name: impl Into<String>,
  ) -> std::result::Result<jni::objects::JClass<'a>, jni::errors::Error> {
    find_class(env, activity, name.into())
  }

  #[cfg(target_os = "android")]
  fn run_on_android_context<F>(&self, f: F)
  where
    F: FnOnce(jni::JNIEnv<'_>, jni::objects::JObject<'_>, jni::objects::JObject<'_>)
      + Send
      + 'static,
  {
    dispatch(f)
  }
}

fn handle_event_loop<T: UserEvent>(
  event: Event<'_, Message<T>>,
  event_loop: &EventLoopWindowTarget<Message<T>>,
  control_flow: &mut ControlFlow,
  context: EventLoopIterationContext<'_, T>,
  web_context: &WebContextStore,
) -> RunIteration {
  let EventLoopIterationContext {
    callback,
    webview_id_store,
    windows,
    #[cfg(all(desktop, feature = "global-shortcut"))]
    shortcut_manager,
    #[cfg(all(desktop, feature = "global-shortcut"))]
    global_shortcut_manager,
    #[cfg(feature = "clipboard")]
    clipboard,
    #[cfg(all(desktop, feature = "system-tray"))]
    system_tray_manager,
  } = context;
  if *control_flow != ControlFlow::Exit {
    *control_flow = ControlFlow::Wait;
  }

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

    Event::LoopDestroyed => {
      callback(RunEvent::Exit);
    }

    #[cfg(all(desktop, feature = "global-shortcut"))]
    Event::GlobalShortcutEvent(accelerator_id) => {
      handle_global_shortcut_event(accelerator_id, global_shortcut_manager)
    }

    Event::MenuEvent {
      window_id,
      menu_id,
      origin: MenuType::MenuBar,
      ..
    } => handle_window_menu_event(menu_id, window_id, &windows, &webview_id_store),

    #[cfg(all(desktop, feature = "system-tray"))]
    Event::MenuEvent {
      origin: MenuType::ContextMenu,
      ..
    }
    | Event::TrayEvent { .. } => handle_system_tray_event(event, system_tray_manager),

    Event::UserEvent(Message::Webview(_, WebviewMessage::WebviewEvent(_)))
    | Event::WindowEvent { .. } => {
      handle_window_event(event, &windows, &webview_id_store, callback, control_flow)
    }

    Event::UserEvent(message) => match message {
      Message::Window(id, WindowMessage::Close) => {
        on_window_close(id, windows.clone());
      }
      Message::UserEvent(t) => callback(RunEvent::UserEvent(t)),
      message => {
        return handle_user_message(
          event_loop,
          message,
          UserMessageContext {
            webview_id_map: webview_id_store,
            #[cfg(all(desktop, feature = "global-shortcut"))]
            shortcut_manager,
            #[cfg(feature = "clipboard")]
            clipboard,
            windows,
            #[cfg(all(desktop, feature = "system-tray"))]
            system_tray_manager,
          },
          web_context,
        );
      }
    },
    _ => (),
  }

  let it = RunIteration {
    window_count: windows.borrow().len(),
  };
  it
}

fn handle_user_message<T: UserEvent>(
  event_loop: &EventLoopWindowTarget<Message<T>>,
  message: Message<T>,
  context: UserMessageContext,
  web_context: &WebContextStore,
) -> RunIteration {
  let UserMessageContext {
    webview_id_map,
    #[cfg(all(desktop, feature = "global-shortcut"))]
    shortcut_manager,
    #[cfg(feature = "clipboard")]
    clipboard,
    windows,
    #[cfg(all(desktop, feature = "system-tray"))]
    system_tray_manager,
  } = context;
  match message {
    Message::Task(task) => task(),
    #[cfg(target_os = "macos")]
    Message::Application(application_message) => match application_message {
      ApplicationMessage::Show => {
        event_loop.show_application();
      }
      ApplicationMessage::Hide => {
        event_loop.hide_application();
      }
    },
    Message::Window(id, window_message) => {
      if let WindowMessage::UpdateMenuItem(item_id, update) = window_message {
        handle_window_menu_update(update, item_id, id, &windows)
      } else {
        handle_window_message(window_message, id, &windows);
      }
    }
    Message::Webview(id, webview_message) => match webview_message {
      WebviewMessage::EvaluateScript(script) => {
        if let Some(WindowHandle::Webview { inner: webview, .. }) =
          windows.borrow().get(&id).and_then(|w| w.inner.as_ref())
        {
          if let Err(e) = webview.evaluate_script(&script) {
            debug_eprintln!("{}", e);
          }
        }
      }
      WebviewMessage::Print => {
        if let Some(WindowHandle::Webview { inner: webview, .. }) =
          windows.borrow().get(&id).and_then(|w| w.inner.as_ref())
        {
          let _ = webview.print();
        }
      }
      WebviewMessage::WebviewEvent(_event) => { /* already handled */ }
    },
    Message::CreateWebview(webview_id, handler) => match handler(event_loop, web_context) {
      Ok(webview) => {
        windows.borrow_mut().insert(webview_id, webview);
      }
      Err(e) => {
        debug_eprintln!("{}", e);
      }
    },
    Message::CreateWindow(window_id, handler, sender) => {
      let (label, builder) = handler();
      if let Ok(window) = builder.build(event_loop) {
        webview_id_map.insert(window.id(), window_id);

        let w = Arc::new(window);

        windows.borrow_mut().insert(
          window_id,
          Window {
            label,
            inner: Some(WindowHandle::Window(w.clone())),
            menu_items: Default::default(),
            window_event_listeners_store: Default::default(),
            menu_event_listeners_store: Default::default(),
          },
        );
        sender.send(Ok(Arc::downgrade(&w))).unwrap();
      } else {
        sender.send(Err(Error::CreateWindow)).unwrap();
      }
    }

    #[cfg(all(desktop, feature = "system-tray"))]
    Message::Tray(id, message) => {
      handle_system_tray_message(id, message, system_tray_manager, event_loop)
    }
    #[cfg(all(desktop, feature = "global-shortcut"))]
    Message::GlobalShortcut(message) => handle_global_shortcut_message(message, &shortcut_manager),
    #[cfg(feature = "clipboard")]
    Message::Clipboard(message) => handle_clipboard_message(message, &clipboard),
    Message::UserEvent(_) => (),
  }

  let it = RunIteration {
    window_count: windows.borrow().len(),
  };
  it
}

pub fn on_window_close(webview_id: WebviewId, windows: Arc<RefCell<HashMap<WebviewId, Window>>>) {
  if let Some(mut window_wrapper) = windows.borrow_mut().get_mut(&webview_id) {
    window_wrapper.inner = None;
  }
}
