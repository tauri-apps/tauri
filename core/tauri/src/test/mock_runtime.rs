// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(dead_code)]

use tauri_runtime::{
  menu::{Menu, MenuUpdate, SystemTrayMenu, TrayHandle},
  monitor::Monitor,
  webview::{WindowBuilder, WindowBuilderBase},
  window::{
    dpi::{PhysicalPosition, PhysicalSize, Position, Size},
    DetachedWindow, MenuEvent, PendingWindow, WindowEvent,
  },
  ClipboardManager, Dispatch, GlobalShortcutManager, Icon, Result, RunEvent, Runtime,
  RuntimeHandle, UserAttentionType,
};
#[cfg(feature = "system-tray")]
use tauri_runtime::{SystemTray, SystemTrayEvent};
use tauri_utils::config::WindowConfig;
use uuid::Uuid;

#[cfg(windows)]
use windows::Win32::Foundation::HWND;

use std::{
  collections::HashMap,
  fmt,
  sync::{Arc, Mutex},
};

type ShortcutMap = HashMap<String, Box<dyn Fn() + Send + 'static>>;

#[derive(Clone)]
pub struct RuntimeContext {
  shortcuts: Arc<Mutex<ShortcutMap>>,
  clipboard: Arc<Mutex<Option<String>>>,
}

impl fmt::Debug for RuntimeContext {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("RuntimeContext")
      .field("clipboard", &self.clipboard)
      .finish()
  }
}

#[derive(Debug, Clone)]
pub struct MockRuntimeHandle {
  context: RuntimeContext,
}

impl RuntimeHandle for MockRuntimeHandle {
  type Runtime = MockRuntime;
  /// Create a new webview window.
  fn create_window(
    &self,
    pending: PendingWindow<Self::Runtime>,
  ) -> Result<DetachedWindow<Self::Runtime>> {
    Ok(DetachedWindow {
      label: pending.label,
      dispatcher: MockDispatcher {
        context: self.context.clone(),
      },
      menu_ids: Default::default(),
      js_event_listeners: Default::default(),
    })
  }

  /// Run a task on the main thread.
  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    unimplemented!()
  }

  #[cfg(all(windows, feature = "system-tray"))]
  #[cfg_attr(doc_cfg, doc(cfg(all(windows, feature = "system-tray"))))]
  fn remove_system_tray(&self) -> Result<()> {
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct MockDispatcher {
  context: RuntimeContext,
}

#[derive(Debug, Clone)]
pub struct MockGlobalShortcutManager {
  context: RuntimeContext,
}

impl GlobalShortcutManager for MockGlobalShortcutManager {
  fn is_registered(&self, accelerator: &str) -> Result<bool> {
    Ok(
      self
        .context
        .shortcuts
        .lock()
        .unwrap()
        .contains_key(accelerator),
    )
  }

  fn register<F: Fn() + Send + 'static>(&mut self, accelerator: &str, handler: F) -> Result<()> {
    self
      .context
      .shortcuts
      .lock()
      .unwrap()
      .insert(accelerator.into(), Box::new(handler));
    Ok(())
  }

  fn unregister_all(&mut self) -> Result<()> {
    *self.context.shortcuts.lock().unwrap() = Default::default();
    Ok(())
  }

  fn unregister(&mut self, accelerator: &str) -> Result<()> {
    self.context.shortcuts.lock().unwrap().remove(accelerator);
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct MockClipboardManager {
  context: RuntimeContext,
}

impl ClipboardManager for MockClipboardManager {
  fn write_text<T: Into<String>>(&mut self, text: T) -> Result<()> {
    self.context.clipboard.lock().unwrap().replace(text.into());
    Ok(())
  }

  fn read_text(&self) -> Result<Option<String>> {
    Ok(self.context.clipboard.lock().unwrap().clone())
  }
}

#[derive(Debug, Clone)]
pub struct MockWindowBuilder {}

impl WindowBuilderBase for MockWindowBuilder {}

impl WindowBuilder for MockWindowBuilder {
  fn new() -> Self {
    Self {}
  }

  fn with_config(config: WindowConfig) -> Self {
    Self {}
  }

  fn menu(self, menu: Menu) -> Self {
    self
  }

  fn center(self) -> Self {
    self
  }

  fn position(self, x: f64, y: f64) -> Self {
    self
  }

  fn inner_size(self, min_width: f64, min_height: f64) -> Self {
    self
  }

  fn min_inner_size(self, min_width: f64, min_height: f64) -> Self {
    self
  }

  fn max_inner_size(self, max_width: f64, max_height: f64) -> Self {
    self
  }

  fn resizable(self, resizable: bool) -> Self {
    self
  }

  fn title<S: Into<String>>(self, title: S) -> Self {
    self
  }

  fn fullscreen(self, fullscreen: bool) -> Self {
    self
  }

  fn focus(self) -> Self {
    self
  }

  fn maximized(self, maximized: bool) -> Self {
    self
  }

  fn visible(self, visible: bool) -> Self {
    self
  }

  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  #[cfg_attr(
    doc_cfg,
    doc(cfg(any(not(target_os = "macos"), feature = "macos-private-api")))
  )]
  fn transparent(self, transparent: bool) -> Self {
    self
  }

  fn decorations(self, decorations: bool) -> Self {
    self
  }

  fn always_on_top(self, always_on_top: bool) -> Self {
    self
  }

  fn icon(self, icon: Icon) -> Result<Self> {
    Ok(self)
  }

  fn skip_taskbar(self, skip: bool) -> Self {
    self
  }

  #[cfg(windows)]
  fn parent_window(self, parent: HWND) -> Self {
    self
  }

  #[cfg(windows)]
  fn owner_window(self, owner: HWND) -> Self {
    self
  }

  fn has_icon(&self) -> bool {
    false
  }

  fn get_menu(&self) -> Option<&Menu> {
    None
  }
}

impl Dispatch for MockDispatcher {
  type Runtime = MockRuntime;

  type WindowBuilder = MockWindowBuilder;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    Ok(())
  }

  fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) -> Uuid {
    Uuid::new_v4()
  }

  fn on_menu_event<F: Fn(&MenuEvent) + Send + 'static>(&self, f: F) -> Uuid {
    Uuid::new_v4()
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  fn open_devtools(&self) {}

  fn scale_factor(&self) -> Result<f64> {
    Ok(1.0)
  }

  fn inner_position(&self) -> Result<PhysicalPosition<i32>> {
    Ok(PhysicalPosition { x: 0, y: 0 })
  }

  fn outer_position(&self) -> Result<PhysicalPosition<i32>> {
    Ok(PhysicalPosition { x: 0, y: 0 })
  }

  fn inner_size(&self) -> Result<PhysicalSize<u32>> {
    Ok(PhysicalSize {
      width: 0,
      height: 0,
    })
  }

  fn outer_size(&self) -> Result<PhysicalSize<u32>> {
    Ok(PhysicalSize {
      width: 0,
      height: 0,
    })
  }

  fn is_fullscreen(&self) -> Result<bool> {
    Ok(false)
  }

  fn is_maximized(&self) -> Result<bool> {
    Ok(false)
  }

  fn is_decorated(&self) -> Result<bool> {
    Ok(false)
  }

  fn is_resizable(&self) -> Result<bool> {
    Ok(false)
  }

  fn is_visible(&self) -> Result<bool> {
    Ok(true)
  }

  fn is_menu_visible(&self) -> Result<bool> {
    Ok(true)
  }

  fn current_monitor(&self) -> Result<Option<Monitor>> {
    Ok(None)
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    Ok(None)
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    Ok(Vec::new())
  }

  #[cfg(windows)]
  fn hwnd(&self) -> Result<HWND> {
    unimplemented!()
  }

  #[cfg(target_os = "macos")]
  fn ns_window(&self) -> Result<*mut std::ffi::c_void> {
    unimplemented!()
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn gtk_window(&self) -> Result<gtk::ApplicationWindow> {
    unimplemented!()
  }

  fn center(&self) -> Result<()> {
    Ok(())
  }

  fn print(&self) -> Result<()> {
    Ok(())
  }

  fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> Result<()> {
    Ok(())
  }

  fn create_window(
    &mut self,
    pending: PendingWindow<Self::Runtime>,
  ) -> Result<DetachedWindow<Self::Runtime>> {
    unimplemented!()
  }

  fn set_resizable(&self, resizable: bool) -> Result<()> {
    Ok(())
  }

  fn set_title<S: Into<String>>(&self, title: S) -> Result<()> {
    Ok(())
  }

  fn maximize(&self) -> Result<()> {
    Ok(())
  }

  fn unmaximize(&self) -> Result<()> {
    Ok(())
  }

  fn minimize(&self) -> Result<()> {
    Ok(())
  }

  fn unminimize(&self) -> Result<()> {
    Ok(())
  }

  fn show_menu(&self) -> Result<()> {
    Ok(())
  }

  fn hide_menu(&self) -> Result<()> {
    Ok(())
  }

  fn show(&self) -> Result<()> {
    Ok(())
  }

  fn hide(&self) -> Result<()> {
    Ok(())
  }

  fn close(&self) -> Result<()> {
    Ok(())
  }

  fn set_decorations(&self, decorations: bool) -> Result<()> {
    Ok(())
  }

  fn set_always_on_top(&self, always_on_top: bool) -> Result<()> {
    Ok(())
  }

  fn set_size(&self, size: Size) -> Result<()> {
    Ok(())
  }

  fn set_min_size(&self, size: Option<Size>) -> Result<()> {
    Ok(())
  }

  fn set_max_size(&self, size: Option<Size>) -> Result<()> {
    Ok(())
  }

  fn set_position(&self, position: Position) -> Result<()> {
    Ok(())
  }

  fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
    Ok(())
  }

  fn set_focus(&self) -> Result<()> {
    Ok(())
  }

  fn set_icon(&self, icon: Icon) -> Result<()> {
    Ok(())
  }

  fn set_skip_taskbar(&self, skip: bool) -> Result<()> {
    Ok(())
  }

  fn start_dragging(&self) -> Result<()> {
    Ok(())
  }

  fn eval_script<S: Into<String>>(&self, script: S) -> Result<()> {
    Ok(())
  }

  fn update_menu_item(&self, id: u16, update: MenuUpdate) -> Result<()> {
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct MockTrayHandler {
  context: RuntimeContext,
}

impl TrayHandle for MockTrayHandler {
  fn set_icon(&self, icon: Icon) -> Result<()> {
    Ok(())
  }
  fn set_menu(&self, menu: SystemTrayMenu) -> Result<()> {
    Ok(())
  }
  fn update_item(&self, id: u16, update: MenuUpdate) -> Result<()> {
    Ok(())
  }
  #[cfg(target_os = "macos")]
  fn set_icon_as_template(&self, is_template: bool) -> Result<()> {
    Ok(())
  }
}

pub struct MockRuntime {
  pub context: RuntimeContext,
  global_shortcut_manager: MockGlobalShortcutManager,
  clipboard_manager: MockClipboardManager,
  #[cfg(feature = "system-tray")]
  tray_handler: MockTrayHandler,
}

impl MockRuntime {
  fn init() -> Self {
    let context = RuntimeContext {
      shortcuts: Default::default(),
      clipboard: Default::default(),
    };
    Self {
      global_shortcut_manager: MockGlobalShortcutManager {
        context: context.clone(),
      },
      clipboard_manager: MockClipboardManager {
        context: context.clone(),
      },
      #[cfg(feature = "system-tray")]
      tray_handler: MockTrayHandler {
        context: context.clone(),
      },
      context,
    }
  }
}

impl Runtime for MockRuntime {
  type Dispatcher = MockDispatcher;
  type Handle = MockRuntimeHandle;
  type GlobalShortcutManager = MockGlobalShortcutManager;
  type ClipboardManager = MockClipboardManager;
  #[cfg(feature = "system-tray")]
  type TrayHandler = MockTrayHandler;

  fn new() -> Result<Self> {
    Ok(Self::init())
  }

  #[cfg(any(windows, target_os = "linux"))]
  fn new_any_thread() -> Result<Self> {
    Ok(Self::init())
  }

  fn handle(&self) -> Self::Handle {
    MockRuntimeHandle {
      context: self.context.clone(),
    }
  }

  fn global_shortcut_manager(&self) -> Self::GlobalShortcutManager {
    self.global_shortcut_manager.clone()
  }

  fn clipboard_manager(&self) -> Self::ClipboardManager {
    self.clipboard_manager.clone()
  }

  fn create_window(&self, pending: PendingWindow<Self>) -> Result<DetachedWindow<Self>> {
    Ok(DetachedWindow {
      label: pending.label,
      dispatcher: MockDispatcher {
        context: self.context.clone(),
      },
      menu_ids: Default::default(),
      js_event_listeners: Default::default(),
    })
  }

  #[cfg(feature = "system-tray")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
  fn system_tray(&self, system_tray: SystemTray) -> Result<Self::TrayHandler> {
    Ok(self.tray_handler.clone())
  }

  #[cfg(feature = "system-tray")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
  fn on_system_tray_event<F: Fn(&SystemTrayEvent) + Send + 'static>(&mut self, f: F) -> Uuid {
    Uuid::new_v4()
  }

  #[cfg(target_os = "macos")]
  #[cfg_attr(doc_cfg, doc(cfg(target_os = "macos")))]
  fn set_activation_policy(&mut self, activation_policy: tauri_runtime::ActivationPolicy) {}

  fn run_iteration<F: Fn(RunEvent) + 'static>(
    &mut self,
    callback: F,
  ) -> tauri_runtime::RunIteration {
    Default::default()
  }

  fn run<F: FnMut(RunEvent) + 'static>(self, callback: F) {
    loop {
      std::thread::sleep(std::time::Duration::from_secs(1));
    }
  }
}
