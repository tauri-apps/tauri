// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  sync::{
    Arc, Mutex,
    mpsc::{self, Receiver, Sender},
  },
};

use raw_window_handle::HasWindowHandle;
use raw_window_handle::RawWindowHandle;
use tauri_runtime::{
  Error, Icon, ProgressBarState, Result, UserAttentionType, UserEvent, WindowDispatch,
  WindowEventId,
  dpi::{PhysicalPosition, PhysicalSize, Position, Size},
  monitor::Monitor,
  webview::{DetachedWebview, PendingWebview},
  window::{
    CursorIcon, DetachedWindow, DetachedWindowWebview, PendingWindow, RawWindow, WindowEvent,
    WindowId, WindowSizeConstraints,
  },
};
use tauri_utils::{Theme, config::Color};
use winit::{
  event_loop::ActiveEventLoop,
  monitor::{Fullscreen, MonitorHandle},
  window::{Window as WinitWindow, WindowAttributes, WindowLevel},
};

use crate::platform::{EventLoopExt, MonitorExt};
#[cfg(windows)]
use std::marker::PhantomData;
#[cfg(target_os = "macos")]
use winit::platform::macos::WindowExtMacOS;
#[cfg(windows)]
use winit::platform::windows::WindowExtWindows;

use crate::{
  cef_impl::client as browser_client,
  runtime::{CefRuntime, Message, RuntimeContext, WinitCefApp},
  webview::{AppWebview, CefWebviewDispatcher, create_webview_detached},
  window_builder::WindowBuilderWrapper,
};

pub(crate) struct SendRawWindowHandle(pub raw_window_handle::RawWindowHandle);
unsafe impl Send for SendRawWindowHandle {}

pub(crate) fn tauri_theme_to_winit_theme(theme: Option<Theme>) -> Option<winit::window::Theme> {
  theme.map(|theme| match theme {
    Theme::Light => winit::window::Theme::Light,
    Theme::Dark => winit::window::Theme::Dark,
    _ => winit::window::Theme::Light,
  })
}

fn winit_theme_to_tauri_theme(theme: winit::window::Theme) -> Theme {
  match theme {
    winit::window::Theme::Light => Theme::Light,
    winit::window::Theme::Dark => Theme::Dark,
  }
}

fn tauri_resize_direction_to_winit(
  direction: tauri_runtime::ResizeDirection,
) -> winit::window::ResizeDirection {
  match direction {
    tauri_runtime::ResizeDirection::East => winit::window::ResizeDirection::East,
    tauri_runtime::ResizeDirection::North => winit::window::ResizeDirection::North,
    tauri_runtime::ResizeDirection::NorthEast => winit::window::ResizeDirection::NorthEast,
    tauri_runtime::ResizeDirection::NorthWest => winit::window::ResizeDirection::NorthWest,
    tauri_runtime::ResizeDirection::South => winit::window::ResizeDirection::South,
    tauri_runtime::ResizeDirection::SouthEast => winit::window::ResizeDirection::SouthEast,
    tauri_runtime::ResizeDirection::SouthWest => winit::window::ResizeDirection::SouthWest,
    tauri_runtime::ResizeDirection::West => winit::window::ResizeDirection::West,
  }
}

fn calculate_window_center_position(
  window_size: PhysicalSize<u32>,
  monitor: &MonitorHandle,
) -> PhysicalPosition<i32> {
  let work_area = monitor.work_area();
  PhysicalPosition::new(
    work_area.position.x + ((work_area.size.width as i32 - window_size.width as i32).max(0) / 2),
    work_area.position.y + ((work_area.size.height as i32 - window_size.height as i32).max(0) / 2),
  )
}

fn find_monitor_for_position(
  monitors: impl Iterator<Item = MonitorHandle>,
  position: Position,
) -> Option<MonitorHandle> {
  monitors.into_iter().find(|monitor| {
    let Some(monitor_position) = monitor.position() else {
      return false;
    };
    let Some(video_mode) = monitor.current_video_mode() else {
      return false;
    };

    let monitor_size = video_mode.size();
    let position = position.to_physical::<i32>(monitor.scale_factor());

    monitor_position.x <= position.x
      && position.x < monitor_position.x + monitor_size.width as i32
      && monitor_position.y <= position.y
      && position.y < monitor_position.y + monitor_size.height as i32
  })
}

fn center_window_attributes(event_loop: &dyn ActiveEventLoop, attrs: &mut WindowAttributes) {
  let monitor = attrs
    .position
    .and_then(|position| {
      let monitors = event_loop.available_monitors();
      find_monitor_for_position(monitors, position)
    })
    .or_else(|| event_loop.primary_monitor());

  let Some(monitor) = monitor else {
    return;
  };

  let desired_size = attrs
    .surface_size
    .unwrap_or_else(|| PhysicalSize::new(800, 600).into());
  let window_size = desired_size.to_physical::<u32>(monitor.scale_factor());

  let position = calculate_window_center_position(window_size, &monitor);
  attrs.position = Some(position.into());
}

pub(crate) fn paired_size_constraint(
  width: Option<tauri_runtime::dpi::PixelUnit>,
  height: Option<tauri_runtime::dpi::PixelUnit>,
) -> Option<Size> {
  match (width, height) {
    (
      Some(tauri_runtime::dpi::PixelUnit::Logical(width)),
      Some(tauri_runtime::dpi::PixelUnit::Logical(height)),
    ) => Some(Size::Logical(tauri_runtime::dpi::LogicalSize::new(
      width.into(),
      height.into(),
    ))),
    (
      Some(tauri_runtime::dpi::PixelUnit::Physical(width)),
      Some(tauri_runtime::dpi::PixelUnit::Physical(height)),
    ) => Some(Size::Physical(PhysicalSize::new(
      width.into(),
      height.into(),
    ))),
    _ => None,
  }
}

pub(crate) enum WindowMessage {
  AddEventListener(WindowEventId, Box<dyn Fn(&WindowEvent) + Send>),
  Close,
  Destroy,
  ScaleFactor(Sender<Result<f64>>),
  InnerPosition(Sender<Result<PhysicalPosition<i32>>>),
  OuterPosition(Sender<Result<PhysicalPosition<i32>>>),
  InnerSize(Sender<Result<PhysicalSize<u32>>>),
  OuterSize(Sender<Result<PhysicalSize<u32>>>),
  IsFullscreen(Sender<Result<bool>>),
  IsMinimized(Sender<Result<bool>>),
  IsMaximized(Sender<Result<bool>>),
  IsFocused(Sender<Result<bool>>),
  IsDecorated(Sender<Result<bool>>),
  IsResizable(Sender<Result<bool>>),
  IsMaximizable(Sender<Result<bool>>),
  IsMinimizable(Sender<Result<bool>>),
  IsClosable(Sender<Result<bool>>),
  IsVisible(Sender<Result<bool>>),
  IsEnabled(Sender<Result<bool>>),
  IsAlwaysOnTop(Sender<Result<bool>>),
  Title(Sender<Result<String>>),
  CurrentMonitor(Sender<Result<Option<Monitor>>>),
  PrimaryMonitor(Sender<Result<Option<Monitor>>>),
  MonitorFromPoint(Sender<Result<Option<Monitor>>>, f64, f64),
  AvailableMonitors(Sender<Result<Vec<Monitor>>>),
  RawWindowHandle(Sender<Result<SendRawWindowHandle>>),
  Theme(Sender<Result<Theme>>),
  Center,
  RequestUserAttention(Option<UserAttentionType>),
  SetEnabled(bool),
  SetResizable(bool),
  SetMaximizable(bool),
  SetMinimizable(bool),
  SetClosable(bool),
  SetTitle(String),
  Maximize,
  Unmaximize,
  Minimize,
  Unminimize,
  Show,
  Hide,
  SetDecorations(bool),
  SetShadow(bool),
  SetAlwaysOnBottom(bool),
  SetAlwaysOnTop(bool),
  SetVisibleOnAllWorkspaces(bool),
  SetContentProtected(bool),
  SetSize(Size),
  SetMinSize(Option<Size>),
  SetMaxSize(Option<Size>),
  SetSizeConstraints(WindowSizeConstraints),
  SetPosition(Position),
  SetFullscreen(bool),
  #[cfg(target_os = "macos")]
  SetSimpleFullscreen(bool),
  SetFocus,
  SetFocusable(bool),
  SetIcon(Icon<'static>),
  SetSkipTaskbar(bool),
  SetCursorGrab(bool),
  SetCursorVisible(bool),
  SetCursorIcon(CursorIcon),
  SetCursorPosition(Position),
  SetIgnoreCursorEvents(bool),
  SetProgressBar(ProgressBarState),
  SetBadgeCount(Option<i64>, Option<String>),
  SetBadgeLabel(Option<String>),
  SetOverlayIcon(Option<Icon<'static>>),
  SetTitleBarStyle(tauri_utils::TitleBarStyle),
  SetTrafficLightPosition(Position),
  SetTheme(Option<Theme>),
  SetBackgroundColor(Option<Color>),
  StartDragging,
  StartResizeDragging(tauri_runtime::ResizeDirection),
}

pub(crate) struct AppWindow {
  pub(crate) id: WindowId,
  pub(crate) label: String,
  pub(crate) window: Box<dyn WinitWindow>,
  attrs: AppWindowAttrs,
  pub(crate) children: Vec<AppWebview>,
  pub(crate) listeners: Arc<Mutex<HashMap<WindowEventId, Box<dyn Fn(&WindowEvent) + Send>>>>,
}

struct AppWindowAttrs {
  inner: WindowAttributes,
  #[cfg(target_os = "macos")]
  traffic_light_position: Option<Position>,
}

#[cfg(windows)]
pub(crate) type CefWindowHandle = cef::sys::HWND;
#[cfg(not(windows))]
pub(crate) type CefWindowHandle = *mut std::ffi::c_void;

impl AppWindow {
  pub(crate) fn center(&self) {
    let monitor = self.window.current_monitor();
    let monitor = monitor.or_else(|| self.window.primary_monitor());
    let Some(monitor) = monitor else {
      return;
    };

    let position = calculate_window_center_position(self.window.outer_size(), &monitor);
    self.window.set_outer_position(Position::Physical(position));
  }

  pub(crate) fn raw_handle_as_cef_handle(&self) -> CefWindowHandle {
    let handle = self
      .window
      .window_handle()
      .expect("failed to get window handle");
    match handle.as_raw() {
      #[cfg(windows)]
      RawWindowHandle::Win32(handle) => cef::sys::HWND(handle.hwnd.get() as *mut _),
      #[cfg(target_os = "macos")]
      RawWindowHandle::AppKit(handle) => handle.ns_view.as_ptr().cast(),
      #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
      ))]
      RawWindowHandle::Xlib(handle) => handle.window as usize as *mut c_void,
      #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
      ))]
      RawWindowHandle::Xcb(handle) => handle.window.get() as usize as *mut c_void,
      #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
      ))]
      RawWindowHandle::Wayland(handle) => handle.surface.as_ptr().cast(),
      other => panic!("expected platform window handle, got {other:?}"),
    }
  }
}

impl<T: UserEvent> WinitCefApp<T> {
  pub(crate) fn create_window(
    &mut self,
    event_loop: &dyn ActiveEventLoop,
    window_id: WindowId,
    webview_id: Option<u32>,
    pending: Box<PendingWindow<T, CefRuntime<T>>>,
    _after_window_creation: Option<Box<dyn Fn(RawWindow) + Send>>,
  ) {
    let mut attrs = pending.window_builder.inner.clone();
    if pending.window_builder.center {
      center_window_attributes(event_loop, &mut attrs);
    }

    let window = event_loop
      .create_window(attrs.clone())
      .expect("failed to create winit window");

    let winit_id = window.id();
    let appwindow = AppWindow {
      id: window_id,
      label: pending.label.clone(),
      window,
      attrs: AppWindowAttrs {
        inner: attrs,
        #[cfg(target_os = "macos")]
        traffic_light_position: pending.window_builder.traffic_light_position,
      },
      children: Vec::new(),
      listeners: Default::default(),
    };

    #[cfg(target_os = "macos")]
    if let Some(position) = &appwindow.attrs.traffic_light_position {
      appwindow.apply_traffic_light_position(position);
    }

    #[cfg(target_os = "macos")]
    appwindow.set_visible_on_all_workspaces(pending.window_builder.visible_on_all_workspaces);

    #[cfg(windows)]
    if let Some(after_window_creation) = _after_window_creation {
      let hwnd = appwindow.hwnd();
      after_window_creation(RawWindow {
        hwnd: hwnd.0 as isize,
        _marker: &PhantomData,
      });
    }

    self
      .state
      .winid_id_to_window_id_map
      .insert(winit_id, window_id);
    self.state.windows.insert(window_id, appwindow);

    if let (Some(webview_id), Some(webview)) = (webview_id, pending.webview) {
      self.create_browser_child(
        window_id,
        webview_id,
        browser_client::DragDropEventTarget::Window,
        webview,
      );
    }
  }

  pub(crate) fn handle_window_message(
    &mut self,
    event_loop: &dyn ActiveEventLoop,
    window_id: WindowId,
    message: WindowMessage,
  ) {
    // Handle Close and Destroy messages first to avoid borrowing issues with the window.
    match message {
      WindowMessage::Close => {
        self.request_window_close(window_id, event_loop);
        return;
      }
      WindowMessage::Destroy => {
        self.close_window(window_id, event_loop);
        return;
      }
      _ => {}
    }

    let Some(app_window) = self.state.windows.get_mut(&window_id) else {
      return;
    };
    let window = &app_window.window;

    match message {
      WindowMessage::AddEventListener(id, listener) => {
        app_window.listeners.lock().unwrap().insert(id, listener);
      }
      WindowMessage::Close | WindowMessage::Destroy => unreachable!("handled before borrowing"),
      WindowMessage::ScaleFactor(tx) => _ = tx.send(Ok(window.scale_factor())),
      WindowMessage::InnerSize(tx) => _ = tx.send(Ok(window.surface_size())),
      WindowMessage::OuterSize(tx) => _ = tx.send(Ok(window.outer_size())),
      WindowMessage::IsFullscreen(tx) => _ = tx.send(Ok(window.fullscreen().is_some())),
      WindowMessage::IsMinimized(tx) => _ = tx.send(Ok(window.is_minimized().unwrap_or(false))),
      WindowMessage::IsMaximized(tx) => _ = tx.send(Ok(window.is_maximized())),
      WindowMessage::IsFocused(tx) => _ = tx.send(Ok(window.has_focus())),
      WindowMessage::IsDecorated(tx) => _ = tx.send(Ok(window.is_decorated())),
      WindowMessage::IsResizable(tx) => _ = tx.send(Ok(window.is_resizable())),
      WindowMessage::IsMaximizable(tx) => {
        let is_maximizable = app_window
          .window
          .enabled_buttons()
          .contains(winit::window::WindowButtons::MAXIMIZE);
        let _ = tx.send(Ok(is_maximizable));
      }
      WindowMessage::IsMinimizable(tx) => {
        let is_minimizable = app_window
          .window
          .enabled_buttons()
          .contains(winit::window::WindowButtons::MINIMIZE);
        let _ = tx.send(Ok(is_minimizable));
      }
      WindowMessage::IsClosable(tx) => {
        let is_closable = app_window
          .window
          .enabled_buttons()
          .contains(winit::window::WindowButtons::CLOSE);
        let _ = tx.send(Ok(is_closable));
      }
      WindowMessage::IsVisible(tx) => _ = tx.send(Ok(window.is_visible().unwrap_or(true))),
      WindowMessage::IsEnabled(tx) => _ = tx.send(Ok(app_window.is_enabled())),
      WindowMessage::IsAlwaysOnTop(tx) => {
        let is_on_top = app_window.attrs.inner.window_level == WindowLevel::AlwaysOnTop;
        let _ = tx.send(Ok(is_on_top));
      }
      WindowMessage::Title(tx) => _ = tx.send(Ok(window.title())),
      WindowMessage::InnerPosition(tx) | WindowMessage::OuterPosition(tx) => {
        let pos = app_window
          .window
          .outer_position()
          .map_err(|_| Error::FailedToGetMonitor);
        let _ = tx.send(pos);
      }
      WindowMessage::CurrentMonitor(tx) => {
        let current = window.current_monitor();
        let current = current.map(|m| winit_monitor_to_tauri_monitor(&m));
        let _ = tx.send(Ok(current));
      }
      WindowMessage::PrimaryMonitor(tx) => {
        let primary = window.primary_monitor();
        let primary = primary.map(|m| winit_monitor_to_tauri_monitor(&m));
        let _ = tx.send(Ok(primary));
      }
      WindowMessage::MonitorFromPoint(tx, x, y) => {
        let mut available_monitors = window.available_monitors();
        let monitor = available_monitors
          .find(|m| {
            let pos = m.position().unwrap_or_default();
            let vm = m.current_video_mode();
            let size = vm.map(|v| v.size()).unwrap_or_default();
            x >= pos.x as f64
              && x <= pos.x as f64 + size.width as f64
              && y >= pos.y as f64
              && y <= pos.y as f64 + size.height as f64
          })
          .map(|m| winit_monitor_to_tauri_monitor(&m));
        let _ = tx.send(Ok(monitor));
      }
      WindowMessage::AvailableMonitors(tx) => {
        let monitors = app_window
          .window
          .available_monitors()
          .map(|m| winit_monitor_to_tauri_monitor(&m))
          .collect();
        let _ = tx.send(Ok(monitors));
      }
      WindowMessage::RawWindowHandle(tx) => {
        let handle = window.window_handle();
        let send_handle = handle
          .map(|h| SendRawWindowHandle(h.as_raw()))
          .map_err(|e| Error::FailedToSendMessage);
        let _ = tx.send(send_handle);
      }
      WindowMessage::Theme(tx) => {
        let theme = window.theme();
        let theme = theme.map(winit_theme_to_tauri_theme);
        let theme = theme.unwrap_or(Theme::Light);
        let _ = tx.send(Ok(theme));
      }
      WindowMessage::Center => {
        app_window.center();
      }
      WindowMessage::RequestUserAttention(attention) => {
        window.request_user_attention(match attention {
          Some(UserAttentionType::Critical) => Some(winit::window::UserAttentionType::Critical),
          Some(UserAttentionType::Informational) => {
            Some(winit::window::UserAttentionType::Informational)
          }
          None => None,
        })
      }
      WindowMessage::SetEnabled(value) => app_window.set_enabled(value),
      WindowMessage::SetResizable(value) => window.set_resizable(value),
      WindowMessage::SetTitle(title) => window.set_title(&title),
      WindowMessage::Maximize => window.set_maximized(true),
      WindowMessage::Unmaximize => window.set_maximized(false),
      WindowMessage::Minimize => window.set_minimized(true),
      WindowMessage::Unminimize => window.set_minimized(false),
      WindowMessage::Show => window.set_visible(true),
      WindowMessage::Hide => window.set_visible(false),
      WindowMessage::SetDecorations(value) => window.set_decorations(value),
      WindowMessage::SetSize(size) => _ = window.request_surface_size(size),
      WindowMessage::SetPosition(position) => window.set_outer_position(position),
      WindowMessage::SetFullscreen(value) => {
        app_window
          .window
          .set_fullscreen(value.then_some(Fullscreen::Borderless(None)));
      }
      #[cfg(target_os = "macos")]
      WindowMessage::SetSimpleFullscreen(value) => {
        window.set_simple_fullscreen(value);
      }
      WindowMessage::SetFocus => window.focus_window(),
      WindowMessage::SetMinSize(min_size) => window.set_min_surface_size(min_size),
      WindowMessage::SetMaxSize(max_size) => window.set_max_surface_size(max_size),
      WindowMessage::SetMaximizable(value) => {
        let mut buttons = window.enabled_buttons();
        buttons.set(winit::window::WindowButtons::MAXIMIZE, value);
        window.set_enabled_buttons(buttons);
      }
      WindowMessage::SetMinimizable(value) => {
        let mut buttons = window.enabled_buttons();
        buttons.set(winit::window::WindowButtons::MINIMIZE, value);
        window.set_enabled_buttons(buttons);
      }
      WindowMessage::SetClosable(value) => {
        let mut buttons = window.enabled_buttons();
        buttons.set(winit::window::WindowButtons::CLOSE, value);
        window.set_enabled_buttons(buttons);
      }
      WindowMessage::SetAlwaysOnBottom(value) => {
        let level = match value {
          true => WindowLevel::AlwaysOnBottom,
          false => WindowLevel::Normal,
        };
        app_window.attrs.inner.window_level = level;
        window.set_window_level(level);
      }
      WindowMessage::SetAlwaysOnTop(value) => {
        let level = match value {
          true => WindowLevel::AlwaysOnTop,
          false => WindowLevel::Normal,
        };
        app_window.attrs.inner.window_level = level;
        window.set_window_level(level);
      }
      WindowMessage::SetVisibleOnAllWorkspaces(_value) => {
        #[cfg(target_os = "macos")]
        app_window.set_visible_on_all_workspaces(_value);
      }
      WindowMessage::SetContentProtected(value) => window.set_content_protected(value),
      WindowMessage::SetIcon(icon) => {
        if let Ok(icon) = super::window::tauri_icon_to_winit_icon(icon) {
          window.set_window_icon(Some(icon))
        }
      }
      WindowMessage::SetSkipTaskbar(_value) => {
        #[cfg(windows)]
        window.set_skip_taskbar(_value);

        // TODO: linux
      }
      WindowMessage::SetShadow(value) => {
        #[cfg(windows)]
        window.set_undecorated_shadow(value);
        #[cfg(target_os = "macos")]
        window.set_has_shadow(value);
      }
      WindowMessage::SetCursorGrab(value) => {
        let _ = window.set_cursor_grab(match value {
          true => winit::window::CursorGrabMode::Confined,
          false => winit::window::CursorGrabMode::None,
        });
      }
      WindowMessage::SetCursorVisible(value) => window.set_cursor_visible(value),
      WindowMessage::SetCursorIcon(value) => {
        let cursor_icon = tauri_cursor_to_winit_cursor(value);
        window.set_cursor(cursor_icon.into())
      }
      WindowMessage::SetCursorPosition(value) => _ = window.set_cursor_position(value),
      WindowMessage::SetIgnoreCursorEvents(value) => _ = window.set_cursor_hittest(value),

      WindowMessage::SetTrafficLightPosition(_position) => {
        #[cfg(target_os = "macos")]
        {
          app_window.attrs.traffic_light_position = Some(_position.clone());
          app_window.apply_traffic_light_position(&_position);
        }
      }
      WindowMessage::SetTitleBarStyle(_style) => {
        #[cfg(target_os = "macos")]
        app_window.set_title_bar_style(_style);
      }
      WindowMessage::SetBackgroundColor(_color) => app_window.set_background_color(_color),
      WindowMessage::SetTheme(theme) => window.set_theme(tauri_theme_to_winit_theme(theme)),
      WindowMessage::SetBadgeCount(count, desktop_filename) => {
        event_loop.set_badge_count(count, desktop_filename)
      }
      WindowMessage::SetBadgeLabel(label) => event_loop.set_badge_label(label),
      WindowMessage::SetOverlayIcon(_icon) => {
        #[cfg(windows)]
        app_window.set_overlay_icon(_icon);
      }
      WindowMessage::StartDragging => _ = window.drag_window(),
      WindowMessage::StartResizeDragging(direction) => {
        let _ = window.drag_resize_window(tauri_resize_direction_to_winit(direction));
      }
      WindowMessage::SetSizeConstraints(constraints) => {
        // TODO: upstream individual width/height size constraints to winit.
        let min_size = paired_size_constraint(constraints.min_width, constraints.min_height);
        let max_size = paired_size_constraint(constraints.max_width, constraints.max_height);
        window.set_min_surface_size(min_size);
        window.set_max_surface_size(max_size);
      }

      WindowMessage::SetFocusable(_) => {
        // TODO
      }
      WindowMessage::SetProgressBar(state) => {
        #[cfg(target_os = "macos")]
        event_loop.set_progress_bar(state);
        #[cfg(not(target_os = "macos"))]
        app_window.set_progress_bar(state);
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct CefWindowDispatcher<T: UserEvent> {
  pub(crate) window_id: WindowId,
  pub(crate) context: RuntimeContext<T>,
}

fn getter<T: UserEvent, R>(
  context: &RuntimeContext<T>,
  message: Message<T>,
  receiver: Receiver<Result<R>>,
) -> Result<R> {
  context.send_message(message)?;
  receiver.recv().map_err(|_| Error::FailedToReceiveMessage)?
}

macro_rules! window_getter {
  ($self:ident, $variant:ident) => {{
    let (tx, rx) = mpsc::channel();
    getter(
      &$self.context,
      Message::Window {
        window_id: $self.window_id,
        message: WindowMessage::$variant(tx),
      },
      rx,
    )
  }};
}

impl<T: UserEvent> WindowDispatch<T> for CefWindowDispatcher<T> {
  type Runtime = CefRuntime<T>;
  type WindowBuilder = WindowBuilderWrapper;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.context.run_on_main_thread(f)
  }

  fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) -> WindowEventId {
    let id = self.context.next_window_event_id();
    let _ = self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::AddEventListener(id, Box::new(f)),
    });
    id
  }

  fn scale_factor(&self) -> Result<f64> {
    window_getter!(self, ScaleFactor)
  }

  fn inner_position(&self) -> Result<PhysicalPosition<i32>> {
    window_getter!(self, InnerPosition)
  }

  fn outer_position(&self) -> Result<PhysicalPosition<i32>> {
    window_getter!(self, OuterPosition)
  }

  fn inner_size(&self) -> Result<PhysicalSize<u32>> {
    window_getter!(self, InnerSize)
  }

  fn outer_size(&self) -> Result<PhysicalSize<u32>> {
    window_getter!(self, OuterSize)
  }

  fn is_fullscreen(&self) -> Result<bool> {
    window_getter!(self, IsFullscreen)
  }

  fn is_minimized(&self) -> Result<bool> {
    window_getter!(self, IsMinimized)
  }

  fn is_maximized(&self) -> Result<bool> {
    window_getter!(self, IsMaximized)
  }

  fn is_focused(&self) -> Result<bool> {
    window_getter!(self, IsFocused)
  }

  fn is_decorated(&self) -> Result<bool> {
    window_getter!(self, IsDecorated)
  }

  fn is_resizable(&self) -> Result<bool> {
    window_getter!(self, IsResizable)
  }

  fn is_maximizable(&self) -> Result<bool> {
    window_getter!(self, IsMaximizable)
  }

  fn is_minimizable(&self) -> Result<bool> {
    window_getter!(self, IsMinimizable)
  }

  fn is_closable(&self) -> Result<bool> {
    window_getter!(self, IsClosable)
  }

  fn is_visible(&self) -> Result<bool> {
    window_getter!(self, IsVisible)
  }

  fn is_enabled(&self) -> Result<bool> {
    window_getter!(self, IsEnabled)
  }

  fn is_always_on_top(&self) -> Result<bool> {
    window_getter!(self, IsAlwaysOnTop)
  }

  fn title(&self) -> Result<String> {
    window_getter!(self, Title)
  }

  fn current_monitor(&self) -> Result<Option<Monitor>> {
    window_getter!(self, CurrentMonitor)
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    window_getter!(self, PrimaryMonitor)
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Result<Option<Monitor>> {
    let (tx, rx) = mpsc::channel();
    getter(
      &self.context,
      Message::Window {
        window_id: self.window_id,
        message: WindowMessage::MonitorFromPoint(tx, x, y),
      },
      rx,
    )
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    window_getter!(self, AvailableMonitors)
  }

  fn window_handle(
    &self,
  ) -> std::result::Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
    let handle: Result<SendRawWindowHandle> = window_getter!(self, RawWindowHandle);
    let handle = handle.map_err(|_| raw_window_handle::HandleError::Unavailable)?;
    Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(handle.0) })
  }

  fn theme(&self) -> Result<Theme> {
    window_getter!(self, Theme)
  }

  fn center(&self) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Center,
    })
  }

  fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::RequestUserAttention(request_type),
    })
  }

  fn create_window<F: Fn(RawWindow<'_>) + Send + 'static>(
    &mut self,
    pending: PendingWindow<T, Self::Runtime>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    create_window_detached(&self.context, pending, after_window_creation)
  }

  fn create_webview(
    &mut self,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>> {
    create_webview_detached(&self.context, self.window_id, pending)
  }

  fn set_resizable(&self, resizable: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetResizable(resizable),
    })
  }

  fn set_enabled(&self, enabled: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetEnabled(enabled),
    })
  }

  fn set_maximizable(&self, maximizable: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetMaximizable(maximizable),
    })
  }

  fn set_minimizable(&self, minimizable: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetMinimizable(minimizable),
    })
  }

  fn set_closable(&self, closable: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetClosable(closable),
    })
  }

  fn set_title<S: Into<String>>(&self, title: S) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetTitle(title.into()),
    })
  }

  fn maximize(&self) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Maximize,
    })
  }

  fn unmaximize(&self) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Unmaximize,
    })
  }

  fn minimize(&self) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Minimize,
    })
  }

  fn unminimize(&self) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Unminimize,
    })
  }

  fn show(&self) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Show,
    })
  }

  fn hide(&self) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Hide,
    })
  }

  fn close(&self) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Close,
    })
  }

  fn destroy(&self) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::Destroy,
    })
  }

  fn set_decorations(&self, decorations: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetDecorations(decorations),
    })
  }

  fn set_shadow(&self, enable: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetShadow(enable),
    })
  }

  fn set_always_on_bottom(&self, value: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetAlwaysOnBottom(value),
    })
  }

  fn set_always_on_top(&self, value: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetAlwaysOnTop(value),
    })
  }

  fn set_visible_on_all_workspaces(&self, value: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetVisibleOnAllWorkspaces(value),
    })
  }

  fn set_content_protected(&self, protected: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetContentProtected(protected),
    })
  }

  fn set_size(&self, size: Size) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetSize(size),
    })
  }

  fn set_min_size(&self, size: Option<Size>) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetMinSize(size),
    })
  }

  fn set_max_size(&self, size: Option<Size>) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetMaxSize(size),
    })
  }

  fn set_size_constraints(&self, constraints: WindowSizeConstraints) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetSizeConstraints(constraints),
    })
  }

  fn set_position(&self, position: Position) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetPosition(position),
    })
  }

  fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetFullscreen(fullscreen),
    })
  }

  #[cfg(target_os = "macos")]
  fn set_simple_fullscreen(&self, enable: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetSimpleFullscreen(enable),
    })
  }

  fn set_focus(&self) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetFocus,
    })
  }

  fn set_focusable(&self, focusable: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetFocusable(focusable),
    })
  }

  fn set_icon(&self, icon: Icon) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetIcon(icon.into_owned()),
    })
  }

  fn set_skip_taskbar(&self, skip: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetSkipTaskbar(skip),
    })
  }

  fn set_cursor_grab(&self, grab: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetCursorGrab(grab),
    })
  }

  fn set_cursor_visible(&self, visible: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetCursorVisible(visible),
    })
  }

  fn set_cursor_icon(&self, icon: CursorIcon) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetCursorIcon(icon),
    })
  }

  fn set_cursor_position<Pos: Into<Position>>(&self, position: Pos) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetCursorPosition(position.into()),
    })
  }

  fn set_ignore_cursor_events(&self, ignore: bool) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetIgnoreCursorEvents(ignore),
    })
  }

  fn start_dragging(&self) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::StartDragging,
    })
  }

  fn start_resize_dragging(&self, direction: tauri_runtime::ResizeDirection) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::StartResizeDragging(direction),
    })
  }

  fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetBadgeCount(count, desktop_filename),
    })
  }

  fn set_badge_label(&self, label: Option<String>) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetBadgeLabel(label),
    })
  }

  fn set_overlay_icon(&self, icon: Option<Icon>) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetOverlayIcon(icon.map(Icon::into_owned)),
    })
  }

  fn set_progress_bar(&self, progress_state: ProgressBarState) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetProgressBar(progress_state),
    })
  }

  fn set_title_bar_style(&self, style: tauri_utils::TitleBarStyle) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetTitleBarStyle(style),
    })
  }

  fn set_traffic_light_position(&self, position: Position) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetTrafficLightPosition(position),
    })
  }

  fn set_theme(&self, theme: Option<Theme>) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetTheme(theme),
    })
  }

  fn set_background_color(&self, color: Option<Color>) -> Result<()> {
    self.context.send_message(Message::Window {
      window_id: self.window_id,
      message: WindowMessage::SetBackgroundColor(color),
    })
  }
}

pub(crate) fn create_window_detached<T, F>(
  context: &RuntimeContext<T>,
  pending: PendingWindow<T, CefRuntime<T>>,
  after_window_creation: Option<F>,
) -> Result<DetachedWindow<T, CefRuntime<T>>>
where
  T: UserEvent,
  F: Fn(RawWindow<'_>) + Send + 'static,
{
  let label = pending.label.clone();
  let window_id = context.next_window_id();
  let (webview_id, use_https_scheme, devtools) = pending
    .webview
    .as_ref()
    .map(|w| {
      (
        Some(context.next_webview_id()),
        w.webview_attributes.use_https_scheme,
        w.webview_attributes.devtools,
      )
    })
    .unwrap_or((None, false, None));

  context.send_message(Message::CreateWindow {
    window_id,
    webview_id,
    pending: Box::new(pending),
    after_window_creation: after_window_creation.map(|f| Box::new(f) as _),
  })?;

  let webview = webview_id.map(|webview_id| DetachedWindowWebview {
    webview: DetachedWebview {
      label: label.clone(),
      dispatcher: CefWebviewDispatcher {
        window_id: Arc::new(Mutex::new(window_id)),
        webview_id,
        context: context.clone(),
      },
    },
    use_https_scheme,
    devtools,
  });

  Ok(DetachedWindow {
    id: window_id,
    label,
    dispatcher: CefWindowDispatcher {
      window_id,
      context: context.clone(),
    },
    webview,
  })
}

pub(crate) fn winit_monitor_to_tauri_monitor(monitor: &winit::monitor::MonitorHandle) -> Monitor {
  Monitor {
    name: monitor.name().map(|s| s.to_string()),
    scale_factor: monitor.scale_factor(),
    position: monitor.position().unwrap_or_default(),
    size: monitor
      .current_video_mode()
      .map(|v| v.size())
      .unwrap_or_default(),
    work_area: monitor.work_area(),
  }
}

pub(crate) fn tauri_icon_to_winit_icon(icon: Icon) -> Result<winit::icon::Icon> {
  winit::icon::RgbaIcon::new(icon.rgba.into_owned(), icon.width, icon.height)
    .map(Into::into)
    .map_err(|e| tauri_runtime::Error::InvalidIcon(e.into()))
}

fn tauri_cursor_to_winit_cursor(cursor: CursorIcon) -> winit::cursor::CursorIcon {
  match cursor {
    CursorIcon::Default => winit::cursor::CursorIcon::Default,
    CursorIcon::Crosshair => winit::cursor::CursorIcon::Crosshair,
    CursorIcon::Hand => winit::cursor::CursorIcon::Grab,
    CursorIcon::Arrow => winit::cursor::CursorIcon::Default,
    CursorIcon::Move => winit::cursor::CursorIcon::Move,
    CursorIcon::Text => winit::cursor::CursorIcon::Text,
    CursorIcon::Wait => winit::cursor::CursorIcon::Wait,
    CursorIcon::Help => winit::cursor::CursorIcon::Help,
    CursorIcon::Progress => winit::cursor::CursorIcon::Progress,
    CursorIcon::NotAllowed => winit::cursor::CursorIcon::NotAllowed,
    CursorIcon::ContextMenu => winit::cursor::CursorIcon::ContextMenu,
    CursorIcon::Cell => winit::cursor::CursorIcon::Cell,
    CursorIcon::VerticalText => winit::cursor::CursorIcon::VerticalText,
    CursorIcon::Alias => winit::cursor::CursorIcon::Alias,
    CursorIcon::Copy => winit::cursor::CursorIcon::Copy,
    CursorIcon::NoDrop => winit::cursor::CursorIcon::NoDrop,
    CursorIcon::Grab => winit::cursor::CursorIcon::Grab,
    CursorIcon::Grabbing => winit::cursor::CursorIcon::Grabbing,
    CursorIcon::AllScroll => winit::cursor::CursorIcon::AllScroll,
    CursorIcon::ZoomIn => winit::cursor::CursorIcon::ZoomIn,
    CursorIcon::ZoomOut => winit::cursor::CursorIcon::ZoomOut,
    CursorIcon::EResize => winit::cursor::CursorIcon::EResize,
    CursorIcon::NResize => winit::cursor::CursorIcon::NResize,
    CursorIcon::NeResize => winit::cursor::CursorIcon::NeResize,
    CursorIcon::NwResize => winit::cursor::CursorIcon::NwResize,
    CursorIcon::SResize => winit::cursor::CursorIcon::SResize,
    CursorIcon::SeResize => winit::cursor::CursorIcon::SeResize,
    CursorIcon::SwResize => winit::cursor::CursorIcon::SwResize,
    CursorIcon::WResize => winit::cursor::CursorIcon::WResize,
    CursorIcon::EwResize => winit::cursor::CursorIcon::EwResize,
    CursorIcon::NsResize => winit::cursor::CursorIcon::NsResize,
    CursorIcon::NeswResize => winit::cursor::CursorIcon::NeswResize,
    CursorIcon::NwseResize => winit::cursor::CursorIcon::NwseResize,
    CursorIcon::ColResize => winit::cursor::CursorIcon::ColResize,
    CursorIcon::RowResize => winit::cursor::CursorIcon::RowResize,
    _ => todo!(),
  }
}
