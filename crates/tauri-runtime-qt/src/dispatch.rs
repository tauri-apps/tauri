// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [`WindowDispatch`] and [`WebviewDispatch`]: thread-safe handles that run every operation on
//! the Qt main thread.

use std::{
  fmt,
  sync::{Arc, Mutex},
};

use qtshim::{
  app,
  webengine::{WindowFlag, window_state as ws},
};
use raw_window_handle::{RawWindowHandle, WindowHandle};
use tauri_runtime::{
  Cookie, Error, Icon, ProgressBarState, ProgressBarStatus, ResizeDirection, Result,
  UserAttentionType, UserEvent, WebviewDispatch, WindowDispatch,
  dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Rect, Size},
  monitor::Monitor,
  webview::{DetachedWebview, PendingWebview},
  window::{CursorIcon, DetachedWindow, PendingWindow, RawWindow, WindowId, WindowSizeConstraints},
};
use tauri_utils::{Theme, config::Color};
use url::Url;

use crate::{
  Context, MainState, QtRuntime, QtWebview, QtWindowBuilder, SendWrapper, WebviewEntry,
  WebviewKind, WindowEntry, close_requested, create_webview, create_window, current_theme,
  destroy_window, rates, util,
};

// ---------------------------------------------------------------------------------------------
// window dispatcher

/// Thread-safe handle to a window.
pub struct QtWindowDispatcher<T: UserEvent> {
  pub(crate) window_id: WindowId,
  pub(crate) context: Context<T>,
}

impl<T: UserEvent> Clone for QtWindowDispatcher<T> {
  fn clone(&self) -> Self {
    Self {
      window_id: self.window_id,
      context: self.context.clone(),
    }
  }
}

impl<T: UserEvent> fmt::Debug for QtWindowDispatcher<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("QtWindowDispatcher")
      .field("window_id", &self.window_id)
      .finish()
  }
}

impl<T: UserEvent> QtWindowDispatcher<T> {
  /// Run `f` with the window on the main thread and return its result.
  fn with<R, F>(&self, f: F) -> Result<R>
  where
    R: Send + 'static,
    F: FnOnce(&WindowEntry) -> R + Send + 'static,
  {
    let id = self.window_id;
    self
      .context
      .run_on_main(move |st| st.windows.borrow().get(&id).map(f))?
      .ok_or(Error::WindowNotFound)
  }

  /// Fire-and-forget operation on the window.
  fn act<F>(&self, f: F) -> Result<()>
  where
    F: FnOnce(&WindowEntry) + Send + 'static,
  {
    let id = self.window_id;
    self.context.run_or_post(move |st| {
      if let Some(w) = st.windows.borrow().get(&id) {
        f(w)
      }
    })
  }

  fn state_bit(&self, bit: u32) -> Result<bool> {
    self.with(move |w| w.window.state() & bit != 0)
  }

  fn set_flag(&self, flag: WindowFlag, on: bool) -> Result<()> {
    self.act(move |w| w.window.set_flag(flag, on))
  }
}

impl<T: UserEvent> WindowDispatch<T> for QtWindowDispatcher<T> {
  type Runtime = QtRuntime<T>;
  type WindowBuilder = QtWindowBuilder;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    if std::thread::current().id() == self.context.shared.main_thread {
      f();
      Ok(())
    } else {
      self.context.post(move |_| f())
    }
  }

  // --- getters ---------------------------------------------------------------------------

  fn scale_factor(&self) -> Result<f64> {
    self.with(|w| w.window.scale_factor())
  }

  fn inner_position(&self) -> Result<PhysicalPosition<i32>> {
    self.with(|w| {
      let (x, y, _, _) = w.window.geometry().inner;
      util::physical_position(x, y, w.window.scale_factor())
    })
  }

  fn outer_position(&self) -> Result<PhysicalPosition<i32>> {
    self.with(|w| {
      let (x, y, _, _) = w.window.geometry().outer;
      util::physical_position(x, y, w.window.scale_factor())
    })
  }

  fn inner_size(&self) -> Result<PhysicalSize<u32>> {
    self.with(|w| {
      let (_, _, width, height) = w.window.geometry().inner;
      util::physical_size(width, height, w.window.scale_factor())
    })
  }

  fn outer_size(&self) -> Result<PhysicalSize<u32>> {
    self.with(|w| {
      let (_, _, width, height) = w.window.geometry().outer;
      util::physical_size(width, height, w.window.scale_factor())
    })
  }

  fn is_fullscreen(&self) -> Result<bool> {
    self.state_bit(ws::FULLSCREEN)
  }
  fn is_minimized(&self) -> Result<bool> {
    self.state_bit(ws::MINIMIZED)
  }
  fn is_maximized(&self) -> Result<bool> {
    self.state_bit(ws::MAXIMIZED)
  }
  fn is_focused(&self) -> Result<bool> {
    self.state_bit(ws::ACTIVE)
  }
  fn is_decorated(&self) -> Result<bool> {
    self.state_bit(ws::DECORATED)
  }
  fn is_resizable(&self) -> Result<bool> {
    self.state_bit(ws::RESIZABLE)
  }
  fn is_maximizable(&self) -> Result<bool> {
    self.state_bit(ws::MAXIMIZABLE)
  }
  fn is_minimizable(&self) -> Result<bool> {
    self.state_bit(ws::MINIMIZABLE)
  }
  fn is_closable(&self) -> Result<bool> {
    self.state_bit(ws::CLOSABLE)
  }
  fn is_visible(&self) -> Result<bool> {
    self.state_bit(ws::VISIBLE)
  }
  fn is_enabled(&self) -> Result<bool> {
    self.state_bit(ws::ENABLED)
  }
  fn is_always_on_top(&self) -> Result<bool> {
    self.state_bit(ws::ON_TOP)
  }

  fn title(&self) -> Result<String> {
    self.with(|w| w.window.title())
  }

  fn current_monitor(&self) -> Result<Option<Monitor>> {
    self.with(|w| Some(util::monitor_from_screen(&w.window.screen())))
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    self.context.run_on_main(|_| crate::primary_monitor())
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Result<Option<Monitor>> {
    self
      .context
      .run_on_main(move |_| util::screen_at_physical(x, y).map(|s| util::monitor_from_screen(&s)))
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    self.context.run_on_main(|_| crate::available_monitors())
  }

  /// Qt windows aren't GTK windows.
  fn gtk_window(&self) -> Result<*mut std::ffi::c_void> {
    Err(util::unsupported("gtk_window"))
  }

  /// Qt windows aren't GTK windows.
  fn default_vbox(&self) -> Result<*mut std::ffi::c_void> {
    Err(util::unsupported("default_vbox"))
  }

  /// The window's `QWidget*`; `tauri` installs muda's Qt menubar on it.
  fn qt_window(&self) -> Result<*mut std::ffi::c_void> {
    self
      .with(|w| w.window.widget_ptr().as_ptr() as usize)
      .map(|p| p as *mut std::ffi::c_void)
  }

  fn window_handle(&self) -> std::result::Result<WindowHandle<'_>, raw_window_handle::HandleError> {
    use raw_window_handle::{HandleError, WaylandWindowHandle, XlibWindowHandle};
    let handle = self
      .with(|w| {
        w.window.native_handle().map(|h| match h {
          app::NativeHandle::Wayland(p) => (1u8, p.as_ptr() as usize),
          app::NativeHandle::X11(id) => (2u8, id),
        })
      })
      .map_err(|_| HandleError::Unavailable)?
      .ok_or(HandleError::Unavailable)?;
    let raw: RawWindowHandle = match handle {
      (1, p) => WaylandWindowHandle::new(
        std::ptr::NonNull::new(p as *mut std::ffi::c_void).ok_or(HandleError::Unavailable)?,
      )
      .into(),
      (_, id) => XlibWindowHandle::new(id as std::ffi::c_ulong).into(),
    };
    Ok(unsafe { WindowHandle::borrow_raw(raw) })
  }

  fn theme(&self) -> Result<Theme> {
    self.context.run_on_main(|_| current_theme())
  }

  // --- setters ---------------------------------------------------------------------------

  fn center(&self) -> Result<()> {
    self.act(|w| w.window.center())
  }

  fn request_user_attention(&self, request_type: Option<UserAttentionType>) -> Result<()> {
    match request_type {
      Some(kind) => self.act(move |w| w.window.alert(kind == UserAttentionType::Critical)),
      None => Ok(()),
    }
  }

  fn create_window<F: Fn(RawWindow) + Send + 'static>(
    &mut self,
    pending: PendingWindow<T, QtRuntime<T>>,
    _after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, QtRuntime<T>>> {
    let pending = SendWrapper(pending);
    self
      .context
      .run_on_main(move |st| create_window(st, pending.into_inner()))?
  }

  fn create_webview(
    &mut self,
    pending: PendingWebview<T, QtRuntime<T>>,
  ) -> Result<DetachedWebview<T, QtRuntime<T>>> {
    let id = self.window_id;
    let pending = SendWrapper(pending);
    self.context.run_on_main(move |st| {
      create_webview(st, id, WebviewKind::WindowChild, pending.into_inner())
    })?
  }

  fn set_resizable(&self, resizable: bool) -> Result<()> {
    self.act(move |w| w.window.set_resizable(resizable))
  }
  fn set_enabled(&self, enabled: bool) -> Result<()> {
    self.act(move |w| w.window.set_enabled(enabled))
  }
  fn set_maximizable(&self, maximizable: bool) -> Result<()> {
    self.set_flag(WindowFlag::Maximizable, maximizable)
  }
  fn set_minimizable(&self, minimizable: bool) -> Result<()> {
    self.set_flag(WindowFlag::Minimizable, minimizable)
  }
  fn set_closable(&self, closable: bool) -> Result<()> {
    self.set_flag(WindowFlag::Closable, closable)
  }
  fn set_title<S: Into<String>>(&self, title: S) -> Result<()> {
    let title = title.into();
    self.act(move |w| w.window.set_title(&title))
  }
  fn maximize(&self) -> Result<()> {
    self.act(|w| w.window.set_maximized(true))
  }
  fn unmaximize(&self) -> Result<()> {
    self.act(|w| w.window.set_maximized(false))
  }
  fn minimize(&self) -> Result<()> {
    self.act(|w| w.window.set_minimized(true))
  }
  fn unminimize(&self) -> Result<()> {
    self.act(|w| w.window.set_minimized(false))
  }
  fn show(&self) -> Result<()> {
    self.act(|w| w.window.set_visible(true))
  }
  fn hide(&self) -> Result<()> {
    self.act(|w| w.window.set_visible(false))
  }
  fn close(&self) -> Result<()> {
    let id = self.window_id;
    self.context.post(move |st| close_requested(st, id))
  }
  fn destroy(&self) -> Result<()> {
    let id = self.window_id;
    self.context.post(move |st| destroy_window(st, id))
  }
  fn set_decorations(&self, decorations: bool) -> Result<()> {
    self.set_flag(WindowFlag::Decorations, decorations)
  }
  /// Shadows are drawn by the compositor / Qt decorations on Linux.
  fn set_shadow(&self, _enable: bool) -> Result<()> {
    Ok(())
  }
  fn set_always_on_bottom(&self, always_on_bottom: bool) -> Result<()> {
    self.set_flag(WindowFlag::AlwaysOnBottom, always_on_bottom)
  }
  fn set_always_on_top(&self, always_on_top: bool) -> Result<()> {
    self.set_flag(WindowFlag::AlwaysOnTop, always_on_top)
  }
  /// X11 only (EWMH); no-op on Wayland, which has no protocol for it.
  fn set_visible_on_all_workspaces(&self, visible_on_all_workspaces: bool) -> Result<()> {
    self.set_flag(WindowFlag::AllWorkspaces, visible_on_all_workspaces)
  }
  fn set_background_color(&self, color: Option<Color>) -> Result<()> {
    self.act(move |w| w.window.set_background(color.map(Into::into)))
  }
  /// Not available on Linux; no-op.
  fn set_content_protected(&self, _protected: bool) -> Result<()> {
    Ok(())
  }
  fn set_size(&self, size: Size) -> Result<()> {
    self.act(move |w| {
      let (width, height) = util::size_to_logical(size, w.window.scale_factor());
      w.window.resize(width, height)
    })
  }
  fn set_min_size(&self, size: Option<Size>) -> Result<()> {
    self.act(move |w| {
      let scale = w.window.scale_factor();
      w.window
        .set_min_size(size.map(|s| util::size_to_logical(s, scale)))
    })
  }
  fn set_max_size(&self, size: Option<Size>) -> Result<()> {
    self.act(move |w| {
      let scale = w.window.scale_factor();
      w.window
        .set_max_size(size.map(|s| util::size_to_logical(s, scale)))
    })
  }
  fn set_size_constraints(&self, c: WindowSizeConstraints) -> Result<()> {
    self.act(move |w| {
      let scale = w.window.scale_factor();
      let l = |u: Option<tauri_runtime::dpi::PixelUnit>| {
        u.map(|u| util::round(util::unit_to_logical(u, scale)))
      };
      let (min_w, min_h, max_w, max_h) = (
        l(c.min_width),
        l(c.min_height),
        l(c.max_width),
        l(c.max_height),
      );
      w.window.set_min_size(
        (min_w.is_some() || min_h.is_some()).then(|| (min_w.unwrap_or(0), min_h.unwrap_or(0))),
      );
      w.window.set_max_size(
        (max_w.is_some() || max_h.is_some())
          .then(|| (max_w.unwrap_or(i32::MAX / 2), max_h.unwrap_or(i32::MAX / 2))),
      );
    })
  }
  /// Wayland doesn't let clients position their windows; this only works on X11.
  fn set_position(&self, position: Position) -> Result<()> {
    self.act(move |w| {
      let (x, y) = util::position_to_logical(position, w.window.scale_factor());
      w.window.move_to(x, y)
    })
  }
  fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
    self.act(move |w| w.window.set_fullscreen(fullscreen))
  }
  fn set_fullscreen_on_monitor(&self, position: PhysicalPosition<f64>) -> Result<()> {
    self.act(move |w| {
      if let Some(s) = util::screen_at_physical(position.x, position.y) {
        w.window.fullscreen_on(s.x + 1, s.y + 1);
      }
    })
  }
  fn set_focus(&self) -> Result<()> {
    self.act(|w| w.window.focus())
  }
  fn set_focusable(&self, focusable: bool) -> Result<()> {
    self.set_flag(WindowFlag::Focusable, focusable)
  }
  fn set_icon(&self, icon: Icon) -> Result<()> {
    let (rgba, width, height) = (icon.rgba.into_owned(), icon.width, icon.height);
    self.act(move |w| w.window.set_icon(&rgba, width, height))
  }
  fn set_skip_taskbar(&self, skip: bool) -> Result<()> {
    self.set_flag(WindowFlag::SkipTaskbar, skip)
  }
  fn set_cursor_grab(&self, grab: bool) -> Result<()> {
    self.act(move |w| w.window.set_cursor_grab(grab))
  }
  fn set_cursor_visible(&self, visible: bool) -> Result<()> {
    self.act(move |w| w.window.set_cursor_visible(visible))
  }
  fn set_cursor_icon(&self, icon: CursorIcon) -> Result<()> {
    let shape = util::cursor_shape(icon);
    self.act(move |w| w.window.set_cursor(shape))
  }
  /// Wayland doesn't let clients warp the pointer; this only works on X11.
  fn set_cursor_position<Pos: Into<Position>>(&self, position: Pos) -> Result<()> {
    let position = position.into();
    self.act(move |w| {
      let (x, y) = util::position_to_logical(position, w.window.scale_factor());
      w.window.set_cursor_position(x, y)
    })
  }
  fn set_ignore_cursor_events(&self, ignore: bool) -> Result<()> {
    self.set_flag(WindowFlag::IgnoreCursorEvents, ignore)
  }
  fn start_dragging(&self) -> Result<()> {
    self.act(|w| {
      w.window.start_move();
    })
  }
  fn start_resize_dragging(&self, direction: ResizeDirection) -> Result<()> {
    let edges = util::resize_edges(direction);
    self.act(move |w| {
      w.window.start_resize(edges);
    })
  }
  fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>) -> Result<()> {
    self
      .context
      .update_launcher(Some(count.filter(|c| *c != 0)), None, desktop_filename);
    Ok(())
  }
  /// macOS only.
  fn set_badge_label(&self, _label: Option<String>) -> Result<()> {
    Ok(())
  }
  /// Windows only.
  fn set_overlay_icon(&self, _icon: Option<Icon>) -> Result<()> {
    Ok(())
  }
  fn set_progress_bar(&self, state: ProgressBarState) -> Result<()> {
    let progress = match state.status {
      Some(ProgressBarStatus::None) => Some(None),
      _ => state
        .progress
        .map(|p| Some(p.min(100) as f64 / 100.0))
        .or_else(|| state.status.map(|_| Some(0.0))),
    };
    self
      .context
      .update_launcher(None, progress, state.desktop_filename);
    Ok(())
  }
  /// macOS only.
  fn set_title_bar_style(&self, _style: tauri_utils::TitleBarStyle) -> Result<()> {
    Ok(())
  }
  /// macOS only.
  fn set_traffic_light_position(&self, _position: Position) -> Result<()> {
    Ok(())
  }
  /// App-wide, like on every Linux runtime.
  fn set_theme(&self, theme: Option<Theme>) -> Result<()> {
    self.context.post(move |_| crate::apply_theme(theme))
  }
}

// ---------------------------------------------------------------------------------------------
// webview dispatcher

/// Thread-safe handle to a webview.
pub struct QtWebviewDispatcher<T: UserEvent> {
  pub(crate) window_id: Arc<Mutex<WindowId>>,
  pub(crate) webview_id: u32,
  pub(crate) context: Context<T>,
}

impl<T: UserEvent> Clone for QtWebviewDispatcher<T> {
  fn clone(&self) -> Self {
    Self {
      window_id: self.window_id.clone(),
      webview_id: self.webview_id,
      context: self.context.clone(),
    }
  }
}

impl<T: UserEvent> fmt::Debug for QtWebviewDispatcher<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("QtWebviewDispatcher")
      .field("webview_id", &self.webview_id)
      .finish()
  }
}

impl<T: UserEvent> QtWebviewDispatcher<T> {
  fn with<R, F>(&self, f: F) -> Result<R>
  where
    R: Send + 'static,
    F: FnOnce(&WindowEntry, &WebviewEntry) -> R + Send + 'static,
  {
    let window_id = self.window_id.clone();
    let id = self.webview_id;
    self
      .context
      .run_on_main(move |st| {
        let wid = *window_id.lock().unwrap();
        let windows = st.windows.borrow();
        let w = windows.get(&wid)?;
        let v = w.webview(id)?;
        Some(f(w, v))
      })?
      .ok_or(Error::WindowNotFound)
  }

  fn act<F>(&self, f: F) -> Result<()>
  where
    F: FnOnce(&WindowEntry, &WebviewEntry) + Send + 'static,
  {
    let window_id = self.window_id.clone();
    let id = self.webview_id;
    self.context.run_or_post(move |st| {
      let wid = *window_id.lock().unwrap();
      let windows = st.windows.borrow();
      if let Some(w) = windows.get(&wid)
        && let Some(v) = w.webview(id)
      {
        f(w, v)
      }
    })
  }

  fn cookies_matching(&self, url: Option<Url>) -> Result<Vec<Cookie<'static>>> {
    let raw = self.with(|_, v| v.profile.cookies())?;
    let mut out = Vec::new();
    for s in raw {
      let Ok(c) = Cookie::parse(s) else { continue };
      let c = c.into_owned();
      if let Some(url) = &url {
        let host = url.host_str().unwrap_or_default();
        if let Some(domain) = c.domain()
          && !util::domain_matches(host, domain)
        {
          continue;
        }
        if let Some(path) = c.path()
          && !util::path_matches(url.path(), path)
        {
          continue;
        }
        if c.secure() == Some(true) && url.scheme() == "http" {
          continue;
        }
      }
      out.push(c);
    }
    Ok(out)
  }
}

fn remove_webview<T: UserEvent>(st: &MainState<T>, window_id: WindowId, id: u32) {
  let removed = {
    let Ok(mut windows) = st.windows.try_borrow_mut() else {
      let _ = st.context.post(move |st| remove_webview(st, window_id, id));
      return;
    };
    windows.get_mut(&window_id).and_then(|w| {
      w.webviews
        .iter()
        .position(|v| v.id == id)
        .map(|i| w.webviews.remove(i))
    })
  };
  drop(removed);
  st.webviews.borrow_mut().remove(&id);
  st.protocols.borrow_mut().remove(&id);
}

fn reparent_webview<T: UserEvent>(
  st: &MainState<T>,
  from: Arc<Mutex<WindowId>>,
  id: u32,
  to: WindowId,
) -> Result<()> {
  let old = *from.lock().unwrap();
  if old == to {
    return Ok(());
  }
  if !st.windows.borrow().contains_key(&to) {
    return Err(Error::WindowNotFound);
  }
  let entry = {
    let mut windows = st.windows.borrow_mut();
    let w = windows.get_mut(&old).ok_or(Error::WindowNotFound)?;
    let i = w
      .webviews
      .iter()
      .position(|v| v.id == id)
      .ok_or(Error::WindowNotFound)?;
    w.webviews.remove(i)
  };
  {
    let windows = st.windows.borrow();
    let target = windows.get(&to).ok_or(Error::WindowNotFound)?;
    entry.view.reparent(&target.window);
  }
  st.windows
    .borrow_mut()
    .get_mut(&to)
    .ok_or(Error::WindowNotFound)?
    .webviews
    .push(entry);
  *from.lock().unwrap() = to;
  Ok(())
}

impl<T: UserEvent> WebviewDispatch<T> for QtWebviewDispatcher<T> {
  type Runtime = QtRuntime<T>;

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    if std::thread::current().id() == self.context.shared.main_thread {
      f();
      Ok(())
    } else {
      self.context.post(move |_| f())
    }
  }

  /// Passes a [`QtWebview`] (raw `QWebEngineView*` / `QWebEnginePage*`).
  fn with_webview<F: FnOnce(QtWebview) + Send + 'static>(&self, f: F) -> Result<()> {
    self.act(move |_, v| {
      f(QtWebview {
        view: v.view.qt_view(),
        page: v.view.qt_page(),
      })
    })
  }

  fn open_devtools(&self) {
    let _ = self.act(|_, v| v.view.open_devtools());
  }

  fn close_devtools(&self) {
    let _ = self.act(|_, v| v.view.close_devtools());
  }

  fn is_devtools_open(&self) -> Result<bool> {
    self.with(|_, v| v.view.is_devtools_open())
  }

  fn url(&self) -> Result<String> {
    self.with(|_, v| v.view.url())
  }

  fn bounds(&self) -> Result<Rect> {
    self.with(|_, v| {
      let (x, y, w, h) = v.view.bounds();
      Rect {
        position: LogicalPosition::new(x as f64, y as f64).into(),
        size: LogicalSize::new(w as f64, h as f64).into(),
      }
    })
  }

  fn position(&self) -> Result<PhysicalPosition<i32>> {
    self.with(|w, v| {
      let (x, y, _, _) = v.view.bounds();
      util::physical_position(x, y, w.window.scale_factor())
    })
  }

  fn size(&self) -> Result<PhysicalSize<u32>> {
    self.with(|w, v| {
      let (_, _, width, height) = v.view.bounds();
      util::physical_size(width, height, w.window.scale_factor())
    })
  }

  fn navigate(&self, url: Url) -> Result<()> {
    self.act(move |_, v| v.view.load_url(url.as_str(), None))
  }

  fn reload(&self) -> Result<()> {
    self.act(|_, v| v.view.reload())
  }

  fn go_back(&self) -> Result<()> {
    self.act(|_, v| v.view.go_back())
  }

  fn can_go_back(&self) -> Result<bool> {
    self.with(|_, v| v.view.can_go_back())
  }

  fn go_forward(&self) -> Result<()> {
    self.act(|_, v| v.view.go_forward())
  }

  fn can_go_forward(&self) -> Result<bool> {
    self.with(|_, v| v.view.can_go_forward())
  }

  fn print(&self) -> Result<()> {
    self.act(|_, v| v.view.print())
  }

  fn close(&self) -> Result<()> {
    let window_id = self.window_id.clone();
    let id = self.webview_id;
    self
      .context
      .post(move |st| remove_webview(st, *window_id.lock().unwrap(), id))
  }

  fn set_bounds(&self, bounds: Rect) -> Result<()> {
    self.act(move |w, v| {
      let scale = w.window.scale_factor();
      let (x, y) = util::position_to_logical(bounds.position, scale);
      let (width, height) = util::size_to_logical(bounds.size, scale);
      v.view.set_bounds(x, y, width, height);
      if v.auto_resize.get().is_some() {
        let (ww, wh) = w.window.content_size();
        v.auto_resize.set(rates(x, y, width, height, ww, wh));
      }
    })
  }

  fn set_size(&self, size: Size) -> Result<()> {
    self.act(move |w, v| {
      let (x, y, _, _) = v.view.bounds();
      let (width, height) = util::size_to_logical(size, w.window.scale_factor());
      v.view.set_bounds(x, y, width, height);
    })
  }

  fn set_position(&self, position: Position) -> Result<()> {
    self.act(move |w, v| {
      let (_, _, width, height) = v.view.bounds();
      let (x, y) = util::position_to_logical(position, w.window.scale_factor());
      v.view.set_bounds(x, y, width, height);
    })
  }

  fn set_focus(&self) -> Result<()> {
    self.act(|_, v| v.view.focus())
  }

  fn hide(&self) -> Result<()> {
    self.act(|_, v| v.view.set_visible(false))
  }

  fn show(&self) -> Result<()> {
    self.act(|_, v| v.view.set_visible(true))
  }

  fn eval_script<S: Into<String>>(&self, script: S) -> Result<()> {
    let script = script.into();
    self.act(move |_, v| v.view.eval(&script))
  }

  fn eval_script_with_callback<S: Into<String>>(
    &self,
    script: S,
    callback: impl Fn(String) + Send + 'static,
  ) -> Result<()> {
    let script = script.into();
    self.act(move |_, v| {
      v.view
        .eval_with_callback(&script, move |result| callback(result.to_string()))
    })
  }

  fn reparent(&self, window_id: WindowId) -> Result<()> {
    let from = self.window_id.clone();
    let id = self.webview_id;
    self
      .context
      .run_on_main(move |st| reparent_webview(st, from, id, window_id))?
  }

  fn cookies_for_url(&self, url: Url) -> Result<Vec<Cookie<'static>>> {
    self.cookies_matching(Some(url))
  }

  fn cookies(&self) -> Result<Vec<Cookie<'static>>> {
    self.cookies_matching(None)
  }

  fn set_cookie(&self, cookie: Cookie<'_>) -> Result<()> {
    let raw = cookie.to_string();
    let has_domain = cookie.domain().is_some();
    self.act(move |_, v| {
      let origin = (!has_domain).then(|| v.view.url());
      v.profile.set_cookie(&raw, origin.as_deref())
    })
  }

  fn delete_cookie(&self, cookie: Cookie<'_>) -> Result<()> {
    let raw = cookie.to_string();
    let has_domain = cookie.domain().is_some();
    self.act(move |_, v| {
      let origin = (!has_domain).then(|| v.view.url());
      v.profile.delete_cookie(&raw, origin.as_deref())
    })
  }

  fn set_auto_resize(&self, auto_resize: bool) -> Result<()> {
    self.act(move |w, v| {
      if !auto_resize {
        v.auto_resize.set(None);
        return;
      }
      let (x, y, width, height) = v.view.bounds();
      let (ww, wh) = w.window.content_size();
      v.auto_resize.set(rates(x, y, width, height, ww, wh));
    })
  }

  fn set_zoom(&self, scale_factor: f64) -> Result<()> {
    self.act(move |_, v| v.view.set_zoom(scale_factor))
  }

  fn set_background_color(&self, color: Option<Color>) -> Result<()> {
    let color = color.map(Into::into).unwrap_or((255, 255, 255, 255));
    self.act(move |_, v| v.view.set_background_color(color))
  }

  fn clear_all_browsing_data(&self) -> Result<()> {
    self.act(|_, v| v.profile.clear_data())
  }
}
