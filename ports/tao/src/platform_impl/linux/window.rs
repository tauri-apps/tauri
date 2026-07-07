// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Copyright 2014-2021 The winit contributors
// Copyright 2021-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

use std::{
  collections::VecDeque,
  rc::Rc,
  sync::{
    atomic::{AtomicBool, AtomicI32, Ordering},
    RwLock,
  },
};

use gtk4::{gdk, glib, prelude::*, CssProvider, Settings};

#[cfg(feature = "libadwaita")]
use libadwaita::prelude::AdwApplicationWindowExt;

use crate::{
  dpi::{LogicalSize, PhysicalPosition, PhysicalSize, Position, Size},
  error::{ExternalError, NotSupportedError, OsError as RootOsError},
  icon::Icon,
  monitor::MonitorHandle as RootMonitorHandle,
  window::{
    CursorIcon, Fullscreen, ProgressBarState, ResizeDirection, Theme, UserAttentionType,
    WindowAttributes, WindowSizeConstraints, RGBA,
  },
};

use super::{
  event_loop::EventLoopWindowTarget,
  gtk_window::ApplicationWindow,
  monitor::{self, MonitorHandle},
  util, PlatformSpecificWindowBuilderAttributes,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub(crate) u32);

impl WindowId {
  pub fn dummy() -> Self {
    WindowId(u32::MAX)
  }
}

pub struct Window {
  /// Window id.
  pub(crate) window_id: WindowId,
  /// Gtk application window.
  pub(crate) window: ApplicationWindow,
  pub(crate) default_vbox: Option<gtk4::Box>,
  /// Window requests sender
  pub(crate) window_requests_tx: async_channel::Sender<(WindowId, WindowRequest)>,
  scale_factor: Rc<AtomicI32>,
  maximized: Rc<AtomicBool>,
  minimized: Rc<AtomicBool>,
  // `Window` is `Send` and `Sync`, need a `RwLock` not a `RefCell`
  // otherwise unsynchronized &RefCell from multiple threads
  fullscreen: RwLock<Option<Fullscreen>>,
  // `Window` is `Send` and `Sync`, need a `RwLock` not a `RefCell`
  // otherwise unsynchronized &RefCell from multiple threads
  inner_size_constraints: RwLock<WindowSizeConstraints>,
  /// Draw event Sender
  draw_tx: async_channel::Sender<WindowId>,
  // `Window` is `Send` and `Sync`, need a `RwLock` not a `RefCell`
  // otherwise unsynchronized &RefCell from multiple threads
  preferred_theme: RwLock<Option<Theme>>,
  css_provider: CssProvider,
}

impl Window {
  pub(crate) fn new<T>(
    event_loop_window_target: &EventLoopWindowTarget<T>,
    attributes: WindowAttributes,
    pl_attribs: PlatformSpecificWindowBuilderAttributes,
  ) -> Result<Self, RootOsError> {
    let app = &event_loop_window_target.app;
    let window_requests_tx = event_loop_window_target.window_requests_tx.clone();
    let draw_tx = event_loop_window_target.draw_tx.clone();

    let window = ApplicationWindow::new(app, &attributes, &pl_attribs);

    let default_vbox = if pl_attribs.default_vbox {
      let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
      // AdwApplicationWindow uses set_content(), gtk4::ApplicationWindow uses set_child()
      #[cfg(feature = "libadwaita")]
      window.set_content(Some(&box_));
      #[cfg(not(feature = "libadwaita"))]
      window.set_child(Some(&box_));
      Some(box_)
    } else {
      None
    };

    let window_id = WindowId(window.id());
    event_loop_window_target
      .windows
      .borrow_mut()
      .insert(window_id);

    // Set Width/Height & Resizable
    let win_scale_factor = window.scale_factor();
    let (width, height) = attributes
      .inner_size
      .map(|size| size.to_logical::<f64>(win_scale_factor as f64).into())
      .unwrap_or((800, 600));

    window.set_default_size(width, height);

    if attributes.maximized {
      let maximize_process = util::WindowMaximizeProcess::new(window.clone(), attributes.resizable);
      glib::idle_add_local_full(glib::Priority::HIGH_IDLE, move || {
        let mut maximize_process = maximize_process.borrow_mut();
        maximize_process.next_step()
      });
    } else {
      window.set_resizable(attributes.resizable);
    }

    // Set Min/Max Size
    util::set_size_constraints(&window, attributes.inner_size_constraints);

    // Rest attributes
    if let Some(Fullscreen::Borderless(m)) = &attributes.fullscreen {
      if let Some(monitor) = m {
        window.fullscreen_on_monitor(&monitor.inner.monitor);
      } else {
        window.fullscreen();
      }
    }

    // Set initial `preferred_theme` value to current portal color-scheme
    #[cfg(feature = "dbus")]
    let preferred_theme = super::portal::theme().ok();
    #[cfg(not(feature = "dbus"))]
    let preferred_theme = None;

    if let Some(theme) = preferred_theme {
      if let Some(settings) = Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(theme == Theme::Dark);
      }
    }

    window.present();

    if !attributes.visible {
      window.set_visible(false);
    }

    // Check if we should paint the transparent background ourselves.
    let mut transparent = false;
    if attributes.transparent && pl_attribs.auto_transparent {
      transparent = true;
    }
    let cursor_moved = pl_attribs.cursor_moved;
    if let Err(e) = window_requests_tx.send_blocking((
      window_id,
      WindowRequest::WireUpEvents {
        transparent,
        fullscreen: attributes.fullscreen.is_some(),
        cursor_moved,
      },
    )) {
      log::warn!("Fail to send wire up events request: {}", e);
    }

    let (scale_factor, maximized, minimized) = Self::setup_signals(&window);

    let win = Self {
      window_id,
      window,
      default_vbox,
      window_requests_tx,
      draw_tx,
      scale_factor,
      maximized,
      minimized,
      fullscreen: RwLock::new(attributes.fullscreen),
      inner_size_constraints: RwLock::new(attributes.inner_size_constraints),
      preferred_theme: RwLock::new(preferred_theme),
      css_provider: CssProvider::new(),
    };

    win.set_background_color(attributes.background_color);

    Ok(win)
  }

  fn setup_signals(window: &ApplicationWindow) -> (Rc<AtomicI32>, Rc<AtomicBool>, Rc<AtomicBool>) {
    let win_scale_factor = window.scale_factor();

    let w_max = window.is_maximized();
    let maximized: Rc<AtomicBool> = Rc::new(w_max.into());
    let minimized = Rc::new(AtomicBool::new(false));
    let max_clone = maximized.clone();
    let minimized_clone = minimized.clone();

    // When a window is realized a new surface is created.
    // All signal handlers on the surface must be re-added when this happens.
    util::on_window_realized(window, move |window| {
      let surface = window.surface().unwrap();

      let toplevel = util::surface_as_toplevel(surface).unwrap();
      let max_clone = max_clone.clone();
      let minimized_clone = minimized_clone.clone();
      toplevel.connect_state_notify(move |t| {
        let state = t.state();
        // Not available on wayland
        minimized_clone.store(
          state.contains(gdk::ToplevelState::MINIMIZED),
          Ordering::Release,
        );
        max_clone.store(
          state.contains(gdk::ToplevelState::MAXIMIZED),
          Ordering::Release,
        );
      });
    });

    let scale_factor: Rc<AtomicI32> = Rc::new(win_scale_factor.into());
    let scale_factor_clone = scale_factor.clone();
    window.connect_scale_factor_notify(move |window| {
      scale_factor_clone.store(window.scale_factor(), Ordering::Release);
    });

    (scale_factor, maximized, minimized)
  }

  pub(crate) fn new_from_gtk_window<T>(
    event_loop_window_target: &EventLoopWindowTarget<T>,
    window: ApplicationWindow,
  ) -> Result<Self, RootOsError> {
    let window_requests_tx = event_loop_window_target.window_requests_tx.clone();
    let draw_tx = event_loop_window_target.draw_tx.clone();

    let window_id = WindowId(window.id());
    event_loop_window_target
      .windows
      .borrow_mut()
      .insert(window_id);

    let (scale_factor, maximized, minimized) = Self::setup_signals(&window);

    let win = Self {
      window_id,
      window,
      default_vbox: None,
      window_requests_tx,
      draw_tx,
      scale_factor,
      maximized,
      minimized,
      fullscreen: RwLock::new(None),
      inner_size_constraints: RwLock::new(WindowSizeConstraints::default()),
      preferred_theme: RwLock::new(None),
      css_provider: CssProvider::new(),
    };

    Ok(win)
  }

  pub fn id(&self) -> WindowId {
    self.window_id
  }

  pub fn scale_factor(&self) -> f64 {
    self.scale_factor.load(Ordering::Acquire) as f64
  }

  pub fn request_redraw(&self) {
    if let Err(e) = self.draw_tx.send_blocking(self.window_id) {
      log::warn!("Failed to send redraw event to event channel: {}", e);
    }
  }

  pub fn inner_position(&self) -> Result<PhysicalPosition<i32>, NotSupportedError> {
    Ok(PhysicalPosition::new(0, 0))
  }

  pub fn outer_position(&self) -> Result<PhysicalPosition<i32>, NotSupportedError> {
    Ok(PhysicalPosition::new(0, 0))
  }

  pub fn set_outer_position<P: Into<Position>>(&self, _: P) {}

  pub fn set_background_color(&self, color: Option<RGBA>) {
    if let Err(e) = self.window_requests_tx.send_blocking((
      self.window_id,
      WindowRequest::BackgroundColor(self.css_provider.clone(), color),
    )) {
      log::warn!("Fail to send size request: {}", e);
    }
  }

  pub fn inner_size(&self) -> PhysicalSize<u32> {
    let (width, height) = &**self.window.inner_size();

    LogicalSize::new(
      width.load(Ordering::Acquire) as u32,
      height.load(Ordering::Acquire) as u32,
    )
    .to_physical(self.scale_factor.load(Ordering::Acquire) as f64)
  }

  pub fn set_inner_size<S: Into<Size>>(&self, size: S) {
    let (width, height) = size.into().to_logical::<i32>(self.scale_factor()).into();

    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::Size((width, height))))
    {
      log::warn!("Fail to send size request: {}", e);
    }
  }

  pub fn outer_size(&self) -> PhysicalSize<u32> {
    let (width, height) = &**self.window.outer_size();

    LogicalSize::new(
      width.load(Ordering::Acquire) as u32,
      height.load(Ordering::Acquire) as u32,
    )
    .to_physical(self.scale_factor.load(Ordering::Acquire) as f64)
  }

  fn set_size_constraints(&self, constraints: WindowSizeConstraints) {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::SizeConstraints(constraints)))
    {
      log::warn!("Fail to send size constraint request: {}", e);
    }
  }

  pub fn set_min_inner_size(&self, size: Option<Size>) {
    let (width, height) = size.map(crate::extract_width_height).unzip();
    let mut size_constraints = self.inner_size_constraints.write().unwrap();
    size_constraints.min_width = width;
    size_constraints.min_height = height;
    self.set_size_constraints(*size_constraints)
  }

  pub fn set_max_inner_size(&self, size: Option<Size>) {
    let (width, height) = size.map(crate::extract_width_height).unzip();
    let mut size_constraints = self.inner_size_constraints.write().unwrap();
    size_constraints.max_width = width;
    size_constraints.max_height = height;
    self.set_size_constraints(*size_constraints)
  }

  pub fn set_inner_size_constraints(&self, constraints: WindowSizeConstraints) {
    *self.inner_size_constraints.write().unwrap() = constraints;
    self.set_size_constraints(constraints)
  }

  pub fn set_title(&self, title: &str) {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::Title(title.to_string())))
    {
      log::warn!("Fail to send title request: {}", e);
    }
  }

  pub fn title(&self) -> String {
    self
      .window
      .title()
      .map(|t| t.as_str().to_string())
      .unwrap_or_default()
  }

  pub fn set_visible(&self, visible: bool) {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::Visible(visible)))
    {
      log::warn!("Fail to send visible request: {}", e);
    }
  }

  pub fn set_focus(&self) {
    if !self.minimized.load(Ordering::Acquire) && self.window.get_visible() {
      if let Err(e) = self
        .window_requests_tx
        .send_blocking((self.window_id, WindowRequest::Focus))
      {
        log::warn!("Fail to send visible request: {}", e);
      }
    }
  }

  pub fn set_focusable(&self, focusable: bool) {
    self.window.set_can_focus(focusable);
  }

  pub fn is_focused(&self) -> bool {
    self.window.is_active()
  }

  pub fn set_resizable(&self, resizable: bool) {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::Resizable(resizable)))
    {
      log::warn!("Fail to send resizable request: {}", e);
    }
  }

  pub fn set_minimizable(&self, _minimizable: bool) {}

  pub fn set_maximizable(&self, _maximizable: bool) {}

  pub fn set_closable(&self, closable: bool) {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::Closable(closable)))
    {
      log::warn!("Fail to send closable request: {}", e);
    }
  }

  pub fn set_minimized(&self, minimized: bool) {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::Minimized(minimized)))
    {
      log::warn!("Fail to send minimized request: {}", e);
    }
  }

  pub fn set_maximized(&self, maximized: bool) {
    let resizable = self.is_resizable();

    if let Err(e) = self.window_requests_tx.send_blocking((
      self.window_id,
      WindowRequest::Maximized(maximized, resizable),
    )) {
      log::warn!("Fail to send maximized request: {}", e);
    }
  }

  pub fn is_always_on_top(&self) -> bool {
    false
  }

  pub fn is_maximized(&self) -> bool {
    self.maximized.load(Ordering::Acquire)
  }

  pub fn is_minimized(&self) -> bool {
    self.minimized.load(Ordering::Acquire)
  }

  pub fn is_resizable(&self) -> bool {
    self.window.is_resizable()
  }

  pub fn is_minimizable(&self) -> bool {
    true
  }

  pub fn is_maximizable(&self) -> bool {
    true
  }
  pub fn is_closable(&self) -> bool {
    self.window.is_deletable()
  }

  pub fn is_decorated(&self) -> bool {
    self.window.is_decorated()
  }

  #[inline]
  pub fn is_visible(&self) -> bool {
    self.window.is_visible()
  }

  pub fn drag_window(&self) -> Result<(), ExternalError> {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::DragWindow))
    {
      log::warn!("Fail to send drag window request: {}", e);
    }
    Ok(())
  }

  pub fn drag_resize_window(&self, direction: ResizeDirection) -> Result<(), ExternalError> {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::DragResizeWindow(direction)))
    {
      log::warn!("Fail to send drag window request: {}", e);
    }
    Ok(())
  }

  pub fn set_fullscreen(&self, fullscreen: Option<Fullscreen>) {
    *self.fullscreen.write().unwrap() = fullscreen.clone();
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::Fullscreen(fullscreen)))
    {
      log::warn!("Fail to send fullscreen request: {}", e);
    }
  }

  pub fn fullscreen(&self) -> Option<Fullscreen> {
    self.fullscreen.read().unwrap().clone()
  }

  pub fn set_decorations(&self, decorations: bool) {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::Decorations(decorations)))
    {
      log::warn!("Fail to send decorations request: {}", e);
    }
  }

  pub fn set_always_on_bottom(&self, _always_on_bottom: bool) {}

  pub fn set_always_on_top(&self, _always_on_top: bool) {}

  pub fn set_window_icon(&self, _window_icon: Option<Icon>) {}

  pub fn set_ime_position<P: Into<Position>>(&self, _position: P) {
    //TODO
  }

  pub fn request_user_attention(&self, request_type: Option<UserAttentionType>) {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::UserAttention(request_type)))
    {
      log::warn!("Fail to send user attention request: {}", e);
    }
  }

  pub fn set_cursor_icon(&self, cursor: CursorIcon) {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::CursorIcon(Some(cursor))))
    {
      log::warn!("Fail to send cursor icon request: {}", e);
    }
  }

  pub fn set_cursor_position<P: Into<Position>>(&self, _position: P) -> Result<(), ExternalError> {
    Err(ExternalError::NotSupported(NotSupportedError::new()))
  }

  pub fn set_cursor_grab(&self, _grab: bool) -> Result<(), ExternalError> {
    Ok(())
  }

  pub fn set_ignore_cursor_events(&self, ignore: bool) -> Result<(), ExternalError> {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::CursorIgnoreEvents(ignore)))
    {
      log::warn!("Fail to send cursor position request: {}", e);
    }

    Ok(())
  }

  pub fn set_cursor_visible(&self, visible: bool) {
    let cursor = if visible {
      Some(CursorIcon::Default)
    } else {
      None
    };
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((self.window_id, WindowRequest::CursorIcon(cursor)))
    {
      log::warn!("Fail to send cursor visibility request: {}", e);
    }
  }

  #[inline]
  pub fn cursor_position(&self) -> Result<PhysicalPosition<f64>, ExternalError> {
    util::cursor_position(&self.window)
  }

  #[inline]
  pub fn current_monitor(&self) -> Option<RootMonitorHandle> {
    monitor::current_monitor(&self.window)
  }

  #[inline]
  pub fn available_monitors(&self) -> VecDeque<MonitorHandle> {
    monitor::available_monitors(&self.display())
  }

  #[inline]
  pub fn primary_monitor(&self) -> Option<RootMonitorHandle> {
    monitor::primary_monitor(&self.display())
  }

  #[inline]
  pub fn monitor_from_point(&self, _: f64, _: f64) -> Option<RootMonitorHandle> {
    None
  }

  #[inline]
  fn display(&self) -> gdk::Display {
    RootExt::display(&self.window)
  }

  fn is_wayland(&self) -> bool {
    self.display().backend().is_wayland()
  }

  #[cfg(feature = "rwh_04")]
  #[inline]
  pub fn raw_window_handle_rwh_04(&self) -> rwh_04::RawWindowHandle {
    if self.is_wayland() {
      use gdk4_wayland::{prelude::WaylandSurfaceExtManual, wayland_client::Proxy};

      let mut window_handle = rwh_04::WaylandHandle::empty();
      if let Some(surface) = self.window.surface() {
        let ptr = surface
          .downcast::<gdk4_wayland::WaylandSurface>()
          .unwrap()
          .wl_surface()
          .unwrap()
          .id()
          .as_ptr();
        window_handle.surface = ptr as *mut _;
      }

      rwh_04::RawWindowHandle::Wayland(window_handle)
    } else {
      let mut window_handle = rwh_04::XlibHandle::empty();
      if let Some(surface) = self.window.surface() {
        window_handle.window = surface.downcast::<gdk4_x11::X11Surface>().unwrap().xid();
      }

      rwh_04::RawWindowHandle::Xlib(window_handle)
    }
  }

  #[cfg(feature = "rwh_05")]
  #[inline]
  pub fn raw_window_handle_rwh_05(&self) -> rwh_05::RawWindowHandle {
    if self.is_wayland() {
      use gdk4_wayland::{prelude::WaylandSurfaceExtManual, wayland_client::Proxy};

      let mut window_handle = rwh_05::WaylandWindowHandle::empty();
      if let Some(surface) = self.window.surface() {
        let ptr = surface
          .downcast::<gdk4_wayland::WaylandSurface>()
          .unwrap()
          .wl_surface()
          .unwrap()
          .id()
          .as_ptr();
        window_handle.surface = ptr as *mut _;
      }

      window_handle.into()
    } else {
      let mut window_handle = rwh_05::XlibWindowHandle::empty();
      if let Some(surface) = self.window.surface() {
        window_handle.window = surface.downcast::<gdk4_x11::X11Surface>().unwrap().xid();
      }
      window_handle.into()
    }
  }

  #[cfg(feature = "rwh_05")]
  #[inline]
  pub fn raw_display_handle_rwh_05(&self) -> rwh_05::RawDisplayHandle {
    let display = self.display();
    if self.is_wayland() {
      use gdk4_wayland::wayland_client::Proxy;
      let display = display
        .downcast::<gdk4_wayland::WaylandDisplay>()
        .unwrap()
        .wl_display()
        .unwrap()
        .id()
        .as_ptr();

      let mut display_handle = rwh_05::WaylandDisplayHandle::empty();
      display_handle.display = display as *mut _;
      display_handle.into()
    } else {
      let display = display.downcast::<gdk4_x11::X11Display>().unwrap();

      let mut display_handle = rwh_05::XlibDisplayHandle::empty();
      display_handle.display =
        unsafe { gdk4_x11::ffi::gdk_x11_display_get_xdisplay(display.as_ptr() as *mut _) };
      display_handle.screen = display.screen().screen_number();
      display_handle.into()
    }
  }

  #[cfg(feature = "rwh_06")]
  #[inline]
  pub fn raw_window_handle_rwh_06(&self) -> Result<rwh_06::RawWindowHandle, rwh_06::HandleError> {
    if let Some(surface) = self.window.surface() {
      if self.is_wayland() {
        use gdk4_wayland::{prelude::WaylandSurfaceExtManual, wayland_client::Proxy};

        Ok(
          rwh_06::WaylandWindowHandle::new({
            let ptr = surface
              .downcast::<gdk4_wayland::WaylandSurface>()
              .unwrap()
              .wl_surface()
              .unwrap()
              .id()
              .as_ptr();
            std::ptr::NonNull::new(ptr as *mut _).expect("wl_surface will never be null")
          })
          .into(),
        )
      } else {
        #[cfg(feature = "x11")]
        {
          Ok(
            rwh_06::XlibWindowHandle::new(
              surface.downcast::<gdk4_x11::X11Surface>().unwrap().xid(),
            )
            .into(),
          )
        }
        #[cfg(not(feature = "x11"))]
        Err(rwh_06::HandleError::NotSupported)
      }
    } else {
      Err(rwh_06::HandleError::Unavailable)
    }
  }

  #[cfg(feature = "rwh_06")]
  #[inline]
  pub fn raw_display_handle_rwh_06(&self) -> Result<rwh_06::RawDisplayHandle, rwh_06::HandleError> {
    let display = self.display();
    if self.is_wayland() {
      use gdk4_wayland::wayland_client::Proxy;

      Ok(
        rwh_06::WaylandDisplayHandle::new({
          let ptr = display
            .downcast::<gdk4_wayland::WaylandDisplay>()
            .unwrap()
            .wl_display()
            .unwrap()
            .id()
            .as_ptr();
          std::ptr::NonNull::new(ptr as *mut _).expect("wl_display will never be null")
        })
        .into(),
      )
    } else {
      #[cfg(feature = "x11")]
      {
        let display = display.downcast::<gdk4_x11::X11Display>().unwrap();

        Ok(
          rwh_06::XlibDisplayHandle::new(
            Some(
              std::ptr::NonNull::new(unsafe {
                gdk4_x11::ffi::gdk_x11_display_get_xdisplay(display.as_ptr() as *mut _)
              })
              .expect("X11 display should never be null"),
            ),
            display.screen().screen_number(),
          )
          .into(),
        )
      }
      #[cfg(not(feature = "x11"))]
      Err(rwh_06::HandleError::NotSupported)
    }
  }

  pub fn set_skip_taskbar(&self, _: bool) -> Result<(), ExternalError> {
    Err(ExternalError::NotSupported(NotSupportedError::new()))
  }

  pub fn set_progress_bar(&self, progress: ProgressBarState) {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((WindowId::dummy(), WindowRequest::ProgressBarState(progress)))
    {
      log::warn!("Fail to send update progress bar request: {}", e);
    }
  }

  pub fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>) {
    if let Err(e) = self.window_requests_tx.send_blocking((
      WindowId::dummy(),
      WindowRequest::BadgeCount(count, desktop_filename),
    )) {
      log::warn!("Fail to send update badge count request: {}", e);
    }
  }

  pub fn theme(&self) -> Theme {
    if let Some(theme) = *self.preferred_theme.read().unwrap() {
      return theme;
    }

    #[cfg(feature = "dbus")]
    if let Ok(portal_theme) = super::portal::theme() {
      return portal_theme;
    }

    if let Some(prefers_dark) =
      Settings::default().map(|s| s.is_gtk_application_prefer_dark_theme())
    {
      if prefers_dark {
        return Theme::Dark;
      }
    }

    Theme::Light
  }

  pub fn set_theme(&self, theme: Option<Theme>) {
    *self.preferred_theme.write().unwrap() = theme;
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((WindowId::dummy(), WindowRequest::SetTheme(theme)))
    {
      log::warn!("Fail to send set theme request: {e}");
    }
  }
}

// We need GtkWindow to initialize WebView, so we have to keep it in the field.
// It is called on any method.
unsafe impl Send for Window {}
unsafe impl Sync for Window {}

#[non_exhaustive]
pub enum WindowRequest {
  Title(String),
  Size((i32, i32)),
  SizeConstraints(WindowSizeConstraints),
  Visible(bool),
  Focus,
  Resizable(bool),
  Closable(bool),
  Minimized(bool),
  Maximized(bool, bool),
  DragWindow,
  DragResizeWindow(ResizeDirection),
  Fullscreen(Option<Fullscreen>),
  Decorations(bool),
  UserAttention(Option<UserAttentionType>),
  CursorIcon(Option<CursorIcon>),
  CursorIgnoreEvents(bool),
  WireUpEvents {
    transparent: bool,
    fullscreen: bool,
    cursor_moved: bool,
  },
  ProgressBarState(ProgressBarState),
  BadgeCount(Option<i64>, Option<String>),
  SetTheme(Option<Theme>),
  BackgroundColor(CssProvider, Option<RGBA>),
}

impl Drop for Window {
  fn drop(&mut self) {
    self.window.destroy();
  }
}
