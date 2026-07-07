// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Copyright 2014-2021 The winit contributors
// Copyright 2021-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

use std::{
  cell::RefCell,
  collections::{HashSet, VecDeque},
  error::Error,
  process,
  rc::Rc,
  sync::atomic::{AtomicBool, Ordering},
  time::Instant,
};

use gtk4::{
  cairo::{RectangleInt, Region},
  gdk::{self, Cursor, ScrollDirection, SurfaceEdge},
  glib::{self, closure_local, MainContext},
  prelude::*,
  EventControllerFocus, EventControllerKey, EventControllerMotion, EventControllerScroll,
  EventControllerScrollFlags, GestureClick, Settings,
};

// Libadwaita support - conditional Application type
#[cfg(feature = "libadwaita")]
use libadwaita as adw;

#[cfg(not(feature = "libadwaita"))]
use gtk4::Application;

#[cfg(feature = "libadwaita")]
type GtkApp = adw::Application;
#[cfg(not(feature = "libadwaita"))]
type GtkApp = Application;

#[cfg(feature = "x11")]
use crate::platform_impl::platform::device;
use crate::{
  dpi::{LogicalPosition, LogicalSize, PhysicalPosition},
  error::ExternalError,
  event::{
    ElementState, Event, MouseButton, MouseScrollDelta, StartCause, TouchPhase, WindowEvent,
  },
  event_loop::{ControlFlow, EventLoopClosed, EventLoopWindowTarget as RootELW},
  keyboard::ModifiersState,
  monitor::MonitorHandle as RootMonitorHandle,
  window::{
    CursorIcon, Fullscreen, ProgressBarState, ResizeDirection, Theme, WindowId as RootWindowId,
  },
};

use super::{
  gtk_window::ApplicationWindow,
  keyboard,
  monitor::{self, MonitorHandle},
  taskbar,
  util::{self},
  window::{WindowId, WindowRequest},
  DEVICE_ID,
};

use taskbar::TaskbarIndicator;

#[derive(Clone)]
pub struct EventLoopWindowTarget<T> {
  /// Gdk display
  pub(crate) display: gdk::Display,
  /// Gtk application
  pub(crate) app: GtkApp,
  /// Window Ids of the application
  pub(crate) windows: Rc<RefCell<HashSet<WindowId>>>,
  /// Window requests sender
  pub(crate) window_requests_tx: async_channel::Sender<(WindowId, WindowRequest)>,
  /// Draw event sender
  pub(crate) draw_tx: async_channel::Sender<WindowId>,
  _marker: std::marker::PhantomData<T>,
}

impl<T> EventLoopWindowTarget<T> {
  #[inline]
  pub fn monitor_from_point(&self, _: f64, _: f64) -> Option<MonitorHandle> {
    None
  }

  #[inline]
  pub fn available_monitors(&self) -> VecDeque<MonitorHandle> {
    monitor::available_monitors(&self.display)
  }

  #[inline]
  pub fn primary_monitor(&self) -> Option<RootMonitorHandle> {
    monitor::primary_monitor(&self.display)
  }

  #[cfg(feature = "rwh_05")]
  pub fn raw_display_handle_rwh_05(&self) -> rwh_05::RawDisplayHandle {
    let display = self.display.clone();
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
  pub fn raw_display_handle_rwh_06(&self) -> Result<rwh_06::RawDisplayHandle, rwh_06::HandleError> {
    let display = self.display.clone();
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
        let xdisplay = std::ptr::NonNull::new(unsafe {
          gdk4_x11::ffi::gdk_x11_display_get_xdisplay(display.as_ptr() as *mut _)
        });
        let xscreen = display.screen().screen_number();
        Ok(rwh_06::XlibDisplayHandle::new(xdisplay, xscreen).into())
      }
      #[cfg(not(feature = "x11"))]
      Err(rwh_06::HandleError::NotSupported)
    }
  }

  pub fn is_wayland(&self) -> bool {
    self.display.backend().is_wayland()
  }

  #[cfg(feature = "x11")]
  pub fn is_x11(&self) -> bool {
    self.display.backend().is_x11()
  }

  #[inline]
  pub fn cursor_position(&self) -> Result<PhysicalPosition<f64>, ExternalError> {
    Ok(PhysicalPosition::new(0., 0.))
  }

  #[inline]
  pub fn set_progress_bar(&self, progress: ProgressBarState) {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((WindowId::dummy(), WindowRequest::ProgressBarState(progress)))
    {
      log::warn!("Fail to send update progress bar request: {e}");
    }
  }

  #[inline]
  pub fn set_badge_count(&self, count: Option<i64>, desktop_filename: Option<String>) {
    if let Err(e) = self.window_requests_tx.send_blocking((
      WindowId::dummy(),
      WindowRequest::BadgeCount(count, desktop_filename),
    )) {
      log::warn!("Fail to send update progress bar request: {e}");
    }
  }

  #[inline]
  pub fn set_theme(&self, theme: Option<Theme>) {
    if let Err(e) = self
      .window_requests_tx
      .send_blocking((WindowId::dummy(), WindowRequest::SetTheme(theme)))
    {
      log::warn!("Fail to send update theme request: {e}");
    }
  }
}

pub struct EventLoop<T: 'static> {
  /// Window target.
  window_target: RootELW<T>,
  /// User event sender for EventLoopProxy
  pub(crate) user_event_tx: async_channel::Sender<Event<'static, T>>,
  /// Event queue of EventLoop
  events: async_channel::Receiver<Event<'static, T>>,
  /// Draw queue of EventLoop
  draws: async_channel::Receiver<WindowId>,
  /// Boolean to control device event thread
  run_device_thread: Option<Rc<AtomicBool>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PlatformSpecificEventLoopAttributes {
  pub(crate) any_thread: bool,
  pub(crate) app_id: Option<String>,
}

impl<T: 'static> EventLoop<T> {
  pub(crate) fn new(attrs: &PlatformSpecificEventLoopAttributes) -> EventLoop<T> {
    if !attrs.any_thread {
      assert_is_main_thread("new_any_thread");
    }

    let context = MainContext::default();
    context
      .with_thread_default(|| {
        EventLoop::new_gtk(attrs.app_id.as_deref()).expect("Failed to initialize gtk backend!")
      })
      .expect("Failed to initialize gtk backend!")
  }

  fn new_gtk(app_id: Option<&str>) -> Result<EventLoop<T>, Box<dyn Error>> {
    // Initialize GTK or libadwaita
    #[cfg(feature = "libadwaita")]
    adw::init().expect("Failed to initialize libadwaita");
    #[cfg(not(feature = "libadwaita"))]
    gtk4::init()?;

    let context = MainContext::default();

    // Create application with appropriate type
    #[cfg(feature = "libadwaita")]
    let app = adw::Application::new(app_id, gtk4::gio::ApplicationFlags::empty());
    #[cfg(not(feature = "libadwaita"))]
    let app = Application::new(app_id, gtk4::gio::ApplicationFlags::empty());
    let app_ = app.clone();
    app.register(gtk4::gio::Cancellable::NONE)?;

    // Send StartCause::Init event
    let (event_tx, event_rx) = async_channel::unbounded();
    let (draw_tx, draw_rx) = async_channel::unbounded();
    let event_tx_ = event_tx.clone();
    app.connect_activate(move |_| {
      if let Err(e) = event_tx_.send_blocking(Event::NewEvents(StartCause::Init)) {
        log::warn!("Failed to send init event to event channel: {}", e);
      }
    });
    let draw_tx_ = draw_tx.clone();
    let user_event_tx = event_tx.clone();

    // Create event loop window target.
    let (window_requests_tx, window_requests_rx) = async_channel::unbounded();
    let display = gdk::Display::default()
      .expect("GdkDisplay not found. This usually means `gkt_init` hasn't called yet.");
    let window_target = EventLoopWindowTarget {
      display,
      app,
      windows: Rc::new(RefCell::new(HashSet::new())),
      window_requests_tx,
      draw_tx: draw_tx_,
      _marker: std::marker::PhantomData,
    };

    // Spawn x11 thread to receive Device events.
    #[cfg(feature = "x11")]
    let run_device_thread = if window_target.is_x11() {
      let (device_tx, device_rx) = async_channel::unbounded();
      let user_event_tx = user_event_tx.clone();
      let run_device_thread = Rc::new(AtomicBool::new(true));
      let run = run_device_thread.clone();
      device::spawn(device_tx);

      context.spawn_local(async move {
        while let Ok(event) = device_rx.recv().await {
          if let Err(e) = user_event_tx.send_blocking(Event::DeviceEvent {
            device_id: DEVICE_ID,
            event,
          }) {
            log::warn!("Fail to send device event to event channel: {}", e);
          }
          if !run.load(Ordering::Relaxed) {
            break;
          }
        }
        glib::ControlFlow::Break
      });
      Some(run_device_thread)
    } else {
      None
    };
    #[cfg(not(feature = "x11"))]
    let run_device_thread = None;

    let mut taskbar = TaskbarIndicator::new();
    let is_wayland = window_target.is_wayland();

    // Receive portal events
    #[cfg(feature = "dbus")]
    {
      let tx_requests_clone = window_target.window_requests_tx.clone();
      if let Err(e) = super::portal::receive_theme_changed(tx_requests_clone) {
        log::debug!("Unable to receive theme changed events: {e}");
      }
    }

    context.spawn_local(async move {
      // Window Request
      while let Ok((id, request)) = window_requests_rx.recv().await {
        if let Some(window) = app_.window_by_id(id.0) {
          match request {
            WindowRequest::Title(title) => window.set_title(Some(&title)),
            WindowRequest::Size((w, h)) => window.set_default_size(w, h),
            WindowRequest::SizeConstraints(constraints) => {
              util::set_size_constraints(&window, constraints);
            }
            WindowRequest::Visible(visible) => {
              window.set_visible(visible);
            }
            WindowRequest::Focus => {
              window.present();
            }
            WindowRequest::Resizable(resizable) => window.set_resizable(resizable),
            WindowRequest::Closable(closable) => window.set_deletable(closable),
            WindowRequest::Minimized(minimized) => {
              if minimized {
                window.minimize();
              } else {
                window.unminimize();
              }
            }
            WindowRequest::Maximized(maximized, resizable) => {
              if maximized {
                let maximize_process = util::WindowMaximizeProcess::new(window.clone(), resizable);
                glib::idle_add_local_full(glib::Priority::DEFAULT_IDLE, move || {
                  let mut maximize_process = maximize_process.borrow_mut();
                  maximize_process.next_step()
                });
              } else {
                window.unmaximize();
              }
            }
            WindowRequest::DragWindow => {
              let cursor = util::default_pointer(&RootExt::display(&window));
              let surface = window.surface();
              if let (Some(cursor), Some(surface)) = (cursor, surface) {
                let pos = surface.device_position(&cursor);
                let toplevel = util::surface_as_toplevel(surface);

                if let (Ok(toplevel), Some((x, y, _))) = (toplevel, pos) {
                  toplevel.begin_move(
                    &cursor,
                    gdk::BUTTON_PRIMARY as _,
                    x,
                    y,
                    gdk::CURRENT_TIME as _,
                  );
                }
              }
            }
            WindowRequest::DragResizeWindow(direction) => {
              let cursor = util::default_pointer(&RootExt::display(&window));
              let surface = window.surface();
              if let (Some(cursor), Some(surface)) = (cursor, surface) {
                let pos = surface.device_position(&cursor);
                let toplevel = util::surface_as_toplevel(surface);

                if let (Ok(toplevel), Some((x, y, _))) = (toplevel, pos) {
                  toplevel.begin_resize(
                    direction.to_gtk_edge(),
                    Some(&cursor),
                    gdk::BUTTON_PRIMARY as _,
                    x,
                    y,
                    gdk::CURRENT_TIME as _,
                  );
                }
              }
            }
            WindowRequest::Fullscreen(fullscreen) => match fullscreen {
              Some(f) => {
                if let Fullscreen::Borderless(m) = f {
                  if let Some(monitor) = m {
                    window.fullscreen_on_monitor(&monitor.inner.monitor);
                  } else {
                    window.fullscreen();
                  }
                }
              }
              None => window.unfullscreen(),
            },
            WindowRequest::Decorations(decorations) => window.set_decorated(decorations),
            WindowRequest::UserAttention(request_type) => {
              if is_wayland && request_type.is_some() {
                window.present();
              } else {
                #[cfg(feature = "x11")]
                if let Some(surface) = window.surface() {
                  if let Ok(x11_surface) = surface.downcast::<gdk4_x11::X11Surface>() {
                    x11_surface.set_urgency_hint(request_type.is_some());
                  }
                }
              }
            }
            WindowRequest::BackgroundColor(css_provider, color) => {
              let display = RootExt::display(&window);
              gtk4::style_context_remove_provider_for_display(&display, &css_provider);

              if let Some(color) = color {
                let theme = format!(
                  r#"
                    window.tao-window-{} {{
                      background-color: rgba({},{},{},{});
                    }}
                  "#,
                  id.0,
                  color.0,
                  color.1,
                  color.2,
                  color.3 as f64 / 255.0
                );
                css_provider.load_from_data(&theme);

                gtk4::style_context_add_provider_for_display(
                  &display,
                  &css_provider,
                  gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
              }
            }
            WindowRequest::Destroy => window.destroy(),
            WindowRequest::CursorIcon(cursor) => match cursor {
              Some(cr) => window.set_cursor(Cursor::from_name(cr.to_str(), None).as_ref()),
              None => window.set_cursor(Cursor::from_name("none", None).as_ref()),
            },
            WindowRequest::CursorIgnoreEvents(ignore) => {
              let region = if ignore {
                Region::create_rectangle(&RectangleInt::new(0, 0, 1, 1))
              } else {
                Region::create()
              };
              window.surface().unwrap().set_input_region(Some(&region));
            }
            WindowRequest::ProgressBarState(_) => unreachable!(),
            WindowRequest::BadgeCount(_, _) => unreachable!(),
            WindowRequest::SetTheme(_) => unreachable!(),
            WindowRequest::WireUpEvents {
              transparent,
              fullscreen,
              cursor_moved,
            } => {
              let motion_event_controller = EventControllerMotion::new();
              let key_event_controller = EventControllerKey::new();
              let focus_event_controller = EventControllerFocus::new();
              let scroll_event_controller =
                EventControllerScroll::new(EventControllerScrollFlags::BOTH_AXES);
              let primary_click_controller = GestureClick::new();
              let all_click_controller = GestureClick::new();

              // Respond to primary mouse click or touch
              primary_click_controller.set_button(gdk::BUTTON_PRIMARY);

              // Respond to any mouse click or touch
              all_click_controller.set_button(0);

              let fullscreen = Rc::new(AtomicBool::new(fullscreen));
              let fullscreen_ = fullscreen.clone();

              window.connect_fullscreened_notify(move |window| {
                fullscreen_.store(window.is_fullscreen(), Ordering::Relaxed);
              });

              // Add window id as css class so we can style individual windows
              window.add_css_class(&format!("tao-window-{}", id.0));

              // Allow resizing unmaximized non-fullscreen undecorated window
              let fullscreen_ = fullscreen.clone();
              let window_clone = window.clone();
              motion_event_controller.connect_motion(move |_, cx, cy| {
                if !window_clone.is_decorated()
                  && window_clone.is_resizable()
                  && !window_clone.is_maximized()
                {
                  let border = window_clone.scale_factor() * 5;
                  let edge: Option<ResizeDirection> = crate::window::hit_test(
                    (0, 0, window_clone.width(), window_clone.height()),
                    cx.round() as _,
                    cy.round() as _,
                    border,
                    border,
                  );

                  let edge = match &edge {
                    Some(e) if !fullscreen_.load(Ordering::Relaxed) => e.to_cursor_str(),
                    _ => "default",
                  };
                  window_clone.set_cursor(Cursor::from_name(edge, None).as_ref());
                }
              });

              // FIXME: This does nothing if the window has a visible child since the child consumes the click event.
              // Is this even needed?
              let window_clone = window.clone();
              primary_click_controller.connect_pressed(move |event, _, cx, cy| {
                if !window_clone.is_decorated()
                  && window_clone.is_resizable()
                  && !window_clone.is_maximized()
                {
                  let border = window_clone.scale_factor() * 5;
                  let edge = crate::window::hit_test(
                    (0, 0, window_clone.width(), window_clone.height()),
                    cx.round() as _,
                    cy.round() as _,
                    border,
                    border,
                  )
                  .map(|d| d.to_gtk_edge())
                  // we return `SurfaceEdge::__Unknown` to be ignored later.
                  // we must return 8 or bigger, otherwise it will be the same as one of the other 7 variants of `SurfaceEdge` enum.
                  .unwrap_or(SurfaceEdge::__Unknown(8));
                  // Ignore the `__Unknown` variant so the window receives the click correctly if it is not on the edges.
                  match edge {
                    SurfaceEdge::__Unknown(_) => (),
                    _ => {
                      if let Some(surface) = window_clone.surface() {
                        if let Ok(toplevel) = util::surface_as_toplevel(surface) {
                          // FIXME: calling `window.begin_resize_drag` uses the default cursor, it should show a resizing cursor instead
                          toplevel.begin_resize(
                            edge,
                            event.device().as_ref(),
                            event.button() as i32,
                            cx,
                            cy,
                            event.current_event_time(),
                          )
                        }
                      }
                    }
                  }
                }
              });

              let tx_clone = event_tx.clone();
              window.connect_close_request(move |_| {
                if let Err(e) = tx_clone.send_blocking(Event::WindowEvent {
                  window_id: RootWindowId(id),
                  event: WindowEvent::CloseRequested,
                }) {
                  log::warn!("Failed to send window close event to event channel: {}", e);
                }
                glib::Propagation::Stop
              });

              let tx_clone = event_tx.clone();

              let _ = window
                .clone()
                .downcast::<ApplicationWindow>()
                .map(|window| {
                  window.connect_resized(closure_local!(
                    move |window: ApplicationWindow, w: i32, h: i32| {
                      let scale_factor = window.scale_factor();
                      if let Err(e) = tx_clone.send_blocking(Event::WindowEvent {
                        window_id: RootWindowId(id),
                        event: WindowEvent::Resized(
                          LogicalSize::new(w, h).to_physical(scale_factor as f64),
                        ),
                      }) {
                        log::warn!(
                          "Failed to send window resized event to event channel: {}",
                          e
                        );
                      }
                    }
                  ));
                });

              let tx_clone = event_tx.clone();
              focus_event_controller.connect_enter(move |_| {
                if let Err(e) = tx_clone.send_blocking(Event::WindowEvent {
                  window_id: RootWindowId(id),
                  event: WindowEvent::Focused(true),
                }) {
                  log::warn!(
                    "Failed to send window focus-in event to event channel: {}",
                    e
                  );
                }
              });

              let tx_clone = event_tx.clone();
              focus_event_controller.connect_leave(move |_| {
                if let Err(e) = tx_clone.send_blocking(Event::WindowEvent {
                  window_id: RootWindowId(id),
                  event: WindowEvent::Focused(false),
                }) {
                  log::warn!(
                    "Failed to send window focus-out event to event channel: {}",
                    e
                  );
                }
              });

              let tx_clone = event_tx.clone();
              window.connect_destroy(move |_| {
                if let Err(e) = tx_clone.send_blocking(Event::WindowEvent {
                  window_id: RootWindowId(id),
                  event: WindowEvent::Destroyed,
                }) {
                  log::warn!(
                    "Failed to send window destroyed event to event channel: {}",
                    e
                  );
                }
              });

              let tx_clone = event_tx.clone();
              motion_event_controller.connect_enter(move |_, _, _| {
                if let Err(e) = tx_clone.send_blocking(Event::WindowEvent {
                  window_id: RootWindowId(id),
                  event: WindowEvent::CursorEntered {
                    device_id: DEVICE_ID,
                  },
                }) {
                  log::warn!(
                    "Failed to send cursor entered event to event channel: {}",
                    e
                  );
                }
              });

              let tx_clone = event_tx.clone();
              let window_clone = window.clone();
              motion_event_controller.connect_motion(move |_, x, y| {
                if cursor_moved {
                  let scale_factor = window_clone.scale_factor();
                  if let Err(e) = tx_clone.send_blocking(Event::WindowEvent {
                    window_id: RootWindowId(id),
                    event: WindowEvent::CursorMoved {
                      position: LogicalPosition::new(x, y).to_physical(scale_factor as f64),
                      device_id: DEVICE_ID,
                      // this field is depracted so it is fine to pass empty state
                      modifiers: ModifiersState::empty(),
                    },
                  }) {
                    log::warn!("Failed to send cursor moved event to event channel: {}", e);
                  }
                }
              });

              let tx_clone = event_tx.clone();
              motion_event_controller.connect_leave(move |_| {
                if let Err(e) = tx_clone.send_blocking(Event::WindowEvent {
                  window_id: RootWindowId(id),
                  event: WindowEvent::CursorLeft {
                    device_id: DEVICE_ID,
                  },
                }) {
                  log::warn!("Failed to send cursor left event to event channel: {}", e);
                }
              });

              let tx_clone = event_tx.clone();
              all_click_controller.connect_pressed(move |event, _, _, _| {
                let button = event.button();
                if let Err(e) = tx_clone.send_blocking(Event::WindowEvent {
                  window_id: RootWindowId(id),
                  event: WindowEvent::MouseInput {
                    button: match button {
                      gdk::BUTTON_PRIMARY => MouseButton::Left,
                      gdk::BUTTON_MIDDLE => MouseButton::Middle,
                      gdk::BUTTON_SECONDARY => MouseButton::Right,
                      _ => MouseButton::Other(button as u16),
                    },
                    state: ElementState::Pressed,
                    device_id: DEVICE_ID,
                    // this field is depracted so it is fine to pass empty state
                    modifiers: ModifiersState::empty(),
                  },
                }) {
                  log::warn!(
                    "Failed to send mouse input pressed event to event channel: {}",
                    e
                  );
                }
              });

              let tx_clone = event_tx.clone();
              all_click_controller.connect_released(move |event, _, _, _| {
                let button = event.button();
                if let Err(e) = tx_clone.send_blocking(Event::WindowEvent {
                  window_id: RootWindowId(id),
                  event: WindowEvent::MouseInput {
                    button: match button {
                      gdk::BUTTON_PRIMARY => MouseButton::Left,
                      gdk::BUTTON_MIDDLE => MouseButton::Middle,
                      gdk::BUTTON_SECONDARY => MouseButton::Right,
                      _ => MouseButton::Other(button as u16),
                    },
                    state: ElementState::Released,
                    device_id: DEVICE_ID,
                    // this field is depracted so it is fine to pass empty state
                    modifiers: ModifiersState::empty(),
                  },
                }) {
                  log::warn!(
                    "Failed to send mouse input released event to event channel: {}",
                    e
                  );
                }
              });

              let tx_clone = event_tx.clone();
              scroll_event_controller.connect_scroll(move |event, dx, dy| {
                if let Some(gdk_event) = event
                  .current_event()
                  .and_then(|e| e.downcast::<gdk::ScrollEvent>().ok())
                {
                  if let Err(e) = tx_clone.send_blocking(Event::WindowEvent {
                    window_id: RootWindowId(id),
                    event: WindowEvent::MouseWheel {
                      device_id: DEVICE_ID,
                      delta: MouseScrollDelta::LineDelta(-dx as f32, -dy as f32),
                      phase: match gdk_event.direction() {
                        ScrollDirection::Smooth => TouchPhase::Moved,
                        _ => TouchPhase::Ended,
                      },
                      modifiers: ModifiersState::empty(),
                    },
                  }) {
                    log::warn!("Failed to send scroll event to event channel: {}", e);
                  }
                }
                glib::Propagation::Proceed
              });

              let tx_clone = event_tx.clone();
              let keyboard_handler = Rc::new(move |key: gdk::Key, keycode, element_state| {
                // if we have a modifier lets send it
                let mut mods = keyboard::get_modifiers(key, keycode);
                if !mods.is_empty() {
                  // if we release the modifier tell the world
                  if ElementState::Released == element_state {
                    mods = ModifiersState::empty();
                  }

                  if let Err(e) = tx_clone.send_blocking(Event::WindowEvent {
                    window_id: RootWindowId(id),
                    event: WindowEvent::ModifiersChanged(mods),
                  }) {
                    log::warn!(
                      "Failed to send modifiers changed event to event channel: {}",
                      e
                    );
                  } else {
                    // stop here we don't want to send the key event
                    // as we emit the `ModifiersChanged`
                    return glib::ControlFlow::Continue;
                  }
                }

                // todo: implement repeat?
                let event = keyboard::make_key_event(&key, keycode, false, None, element_state);

                if let Some(event) = event {
                  if let Err(e) = tx_clone.send_blocking(Event::WindowEvent {
                    window_id: RootWindowId(id),
                    event: WindowEvent::KeyboardInput {
                      device_id: DEVICE_ID,
                      event,
                      is_synthetic: false,
                    },
                  }) {
                    log::warn!("Failed to send keyboard event to event channel: {}", e);
                  }
                }
                glib::ControlFlow::Continue
              });

              let tx_clone = event_tx.clone();
              // TODO Add actual IME from system
              let ime = gtk4::IMContextSimple::default();
              ime.set_client_widget(Some(&window));
              ime.focus_in();
              ime.connect_commit(move |_, s| {
                if let Err(e) = tx_clone.send_blocking(Event::WindowEvent {
                  window_id: RootWindowId(id),
                  event: WindowEvent::ReceivedImeText(s.to_string()),
                }) {
                  log::warn!(
                    "Failed to send received IME text event to event channel: {}",
                    e
                  );
                }
              });

              let handler = keyboard_handler.clone();
              key_event_controller.connect_key_pressed(move |event, key, keycode, _| {
                handler(
                  key.to_owned(),
                  keycode.try_into().unwrap(),
                  ElementState::Pressed,
                );
                ime.filter_keypress(event.current_event().unwrap());

                glib::Propagation::Proceed
              });

              let handler = keyboard_handler.clone();
              key_event_controller.connect_key_released(move |_, key, keycode, _| {
                handler(
                  key.to_owned(),
                  keycode.try_into().unwrap(),
                  ElementState::Released,
                );
              });

              let draw_clone = draw_tx.clone();
              let redraw_handler = Rc::new(move |window: &gtk4::Window| {
                let draw_clone = draw_clone.clone();
                window.frame_clock().unwrap().connect_paint(move |_| {
                  if let Err(e) = draw_clone.send_blocking(id) {
                    log::warn!("Failed to send redraw event to event channel: {}", e);
                  }
                });
              });

              if window.is_realized() {
                let window_ = window.clone();
                redraw_handler(&window_);
              } else {
                // If the window isn't realized, it won't have a frame clock
                // In this case we need to wait on the realize signal before we can add the redraw event handler.
                let signal_id = Rc::new(RefCell::new(None));
                let signal_id_ = signal_id.clone();
                let handler = redraw_handler.clone();
                let id = window.connect_realize(move |window| {
                  if let Some(id) = signal_id_.take() {
                    handler(window);
                    window.disconnect(id);
                  }
                });
                signal_id.borrow_mut().replace(id);
              }

              // Make window transparent if requested.
              if transparent {
                let display = RootExt::display(&window);

                let provider = gtk4::CssProvider::new();
                let theme = format!(
                  r#"
                    window.tao-window-{} {{
                      background-color: rgba(0,0,0,0.0);
                    }}
                  "#,
                  id.0
                );

                provider.load_from_data(&theme);

                gtk4::style_context_add_provider_for_display(
                  &display,
                  &provider,
                  gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
              }

              window.add_controller(motion_event_controller);
              window.add_controller(key_event_controller);
              window.add_controller(focus_event_controller);
              window.add_controller(scroll_event_controller);
              window.add_controller(primary_click_controller);
              window.add_controller(all_click_controller);
            }
          }
        } else if id == WindowId::dummy() {
          match request {
            WindowRequest::ProgressBarState(state) => {
              taskbar.update(state);
            }
            WindowRequest::BadgeCount(count, desktop_filename) => {
              taskbar.update_count(count, desktop_filename);
            }
            WindowRequest::SetTheme(theme) => {
              if let Some(settings) = Settings::default() {
                settings.set_gtk_application_prefer_dark_theme(theme == Some(Theme::Dark));
                if let Err(e) = event_tx.send_blocking(Event::WindowEvent {
                  window_id: RootWindowId(id),
                  event: WindowEvent::ThemeChanged(theme.unwrap_or_default()),
                }) {
                  log::warn!(
                    "Failed to send window theme changed event to event channel: {}",
                    e
                  );
                }
              }
            }
            _ => unreachable!(),
          }
        }
      }
      glib::ControlFlow::Break
    });

    // Create event loop itself.
    let event_loop = Self {
      window_target: RootELW {
        p: window_target,
        _marker: std::marker::PhantomData,
      },
      user_event_tx,
      events: event_rx,
      draws: draw_rx,
      run_device_thread,
    };

    Ok(event_loop)
  }

  #[inline]
  pub fn run<F>(mut self, callback: F) -> !
  where
    F: FnMut(Event<'_, T>, &RootELW<T>, &mut ControlFlow) + 'static,
  {
    let exit_code = self.run_return(callback);
    process::exit(exit_code)
  }

  /// This is the core event loop logic. It basically loops on `gtk_main_iteration` and processes one
  /// event along with that iteration. Depends on current control flow and what it should do, an
  /// event state is defined. The whole state flow chart runs like following:
  ///
  /// ```ignore
  ///                                   Poll/Wait/WaitUntil
  ///       +-------------------------------------------------------------------------+
  ///       |                                                                         |
  ///       |                   Receiving event from event channel                    |   Receiving event from draw channel
  ///       |                               +-------+                                 |   +---+
  ///       v                               v       |                                 |   v   |
  /// +----------+  Poll/Wait/WaitUntil   +------------+  Poll/Wait/WaitUntil   +-----------+ |
  /// | NewStart | ---------------------> | EventQueue | ---------------------> | DrawQueue | |
  /// +----------+                        +------------+                        +-----------+ |
  ///       |ExitWithCode                        |ExitWithCode            ExitWithCode|   |   |
  ///       +------------------------------------+------------------------------------+   +---+
  ///                                            |
  ///                                            v
  ///                                    +---------------+
  ///                                    | LoopDestroyed |
  ///                                    +---------------+
  /// ```
  ///
  /// There are a dew notibale event will sent to callback when state is transisted:
  /// - On any state moves to `LoopDestroyed`, a `LoopDestroyed` event is sent.
  /// - On `NewStart` to `EventQueue`, a `NewEvents` with corresponding `StartCause` depends on
  ///   current control flow is sent.
  /// - On `EventQueue` to `DrawQueue`, a `MainEventsCleared` event is sent.
  /// - On `DrawQueue` back to `NewStart`, a `RedrawEventsCleared` event is sent.
  pub(crate) fn run_return<F>(&mut self, mut callback: F) -> i32
  where
    F: FnMut(Event<'_, T>, &RootELW<T>, &mut ControlFlow),
  {
    enum EventState {
      NewStart,
      EventQueue,
      DrawQueue,
    }

    let context = MainContext::default();
    let run_device_thread = self.run_device_thread.clone();

    context
      .with_thread_default(|| {
        let mut control_flow = ControlFlow::default();
        let window_target = &self.window_target;
        let events = &self.events;
        let draws = &self.draws;

        window_target.p.app.activate();

        // If this is a secondary (remote) GIO instance, the activate signal
        // was forwarded to the primary instance via D-Bus. Exit immediately so
        // the primary can handle focus (e.g. bring its window to front).
        // Without this, the secondary hangs forever waiting for a StartCause::Init
        // event that never arrives (connect_activate only fires on the primary).
        if window_target.p.app.is_remote() {
          return 0;
        }

        let mut state = EventState::NewStart;
        let exit_code = loop {
          let mut blocking = false;
          match state {
            EventState::NewStart => match control_flow {
              ControlFlow::ExitWithCode(code) => {
                callback(Event::LoopDestroyed, window_target, &mut control_flow);
                break code;
              }
              ControlFlow::Wait => {
                if !events.is_empty() {
                  callback(
                    Event::NewEvents(StartCause::WaitCancelled {
                      start: Instant::now(),
                      requested_resume: None,
                    }),
                    window_target,
                    &mut control_flow,
                  );
                  state = EventState::EventQueue;
                } else {
                  blocking = true;
                }
              }
              ControlFlow::WaitUntil(requested_resume) => {
                let start = Instant::now();
                if start >= requested_resume {
                  callback(
                    Event::NewEvents(StartCause::ResumeTimeReached {
                      start,
                      requested_resume,
                    }),
                    window_target,
                    &mut control_flow,
                  );
                  state = EventState::EventQueue;
                } else if !events.is_empty() {
                  callback(
                    Event::NewEvents(StartCause::WaitCancelled {
                      start,
                      requested_resume: Some(requested_resume),
                    }),
                    window_target,
                    &mut control_flow,
                  );
                  state = EventState::EventQueue;
                } else {
                  blocking = true;
                }
              }
              _ => {
                callback(
                  Event::NewEvents(StartCause::Poll),
                  window_target,
                  &mut control_flow,
                );
                state = EventState::EventQueue;
              }
            },
            EventState::EventQueue => match control_flow {
              ControlFlow::ExitWithCode(code) => {
                callback(Event::LoopDestroyed, window_target, &mut control_flow);
                break code;
              }
              _ => match events.try_recv() {
                Ok(event) => match event {
                  Event::LoopDestroyed => control_flow = ControlFlow::ExitWithCode(1),
                  _ => callback(event, window_target, &mut control_flow),
                },
                Err(_) => {
                  callback(Event::MainEventsCleared, window_target, &mut control_flow);
                  state = EventState::DrawQueue;
                }
              },
            },
            EventState::DrawQueue => match control_flow {
              ControlFlow::ExitWithCode(code) => {
                callback(Event::LoopDestroyed, window_target, &mut control_flow);
                break code;
              }
              _ => {
                if let Ok(id) = draws.try_recv() {
                  callback(
                    Event::RedrawRequested(RootWindowId(id)),
                    window_target,
                    &mut control_flow,
                  );
                }
                callback(Event::RedrawEventsCleared, window_target, &mut control_flow);
                state = EventState::NewStart;
              }
            },
          }
          let context = MainContext::default();
          context.iteration(blocking);
        };
        if let Some(run_device_thread) = run_device_thread {
          run_device_thread.store(false, Ordering::Relaxed);
        }
        exit_code
      })
      .unwrap_or(1)
  }

  #[inline]
  pub fn window_target(&self) -> &RootELW<T> {
    &self.window_target
  }

  /// Creates an `EventLoopProxy` that can be used to dispatch user events to the main event loop.
  pub fn create_proxy(&self) -> EventLoopProxy<T> {
    EventLoopProxy {
      user_event_tx: self.user_event_tx.clone(),
    }
  }
}

/// Used to send custom events to `EventLoop`.
#[derive(Debug)]
pub struct EventLoopProxy<T: 'static> {
  user_event_tx: async_channel::Sender<Event<'static, T>>,
}

impl<T: 'static> Clone for EventLoopProxy<T> {
  fn clone(&self) -> Self {
    Self {
      user_event_tx: self.user_event_tx.clone(),
    }
  }
}

impl<T: 'static> EventLoopProxy<T> {
  /// Send an event to the `EventLoop` from which this proxy was created. This emits a
  /// `UserEvent(event)` event in the event loop, where `event` is the value passed to this
  /// function.
  ///
  /// Returns an `Err` if the associated `EventLoop` no longer exists.
  pub fn send_event(&self, event: T) -> Result<(), EventLoopClosed<T>> {
    self
      .user_event_tx
      .send_blocking(Event::UserEvent(event))
      .map_err(|async_channel::SendError(event)| {
        if let Event::UserEvent(error) = event {
          EventLoopClosed(error)
        } else {
          unreachable!();
        }
      })?;

    let context = MainContext::default();
    context.wakeup();

    Ok(())
  }
}

fn assert_is_main_thread(suggested_method: &str) {
  assert!(
    is_main_thread(),
    "Initializing the event loop outside of the main thread is a significant \
             cross-platform compatibility hazard. If you really, absolutely need to create an \
             EventLoop on a different thread, please use the `EventLoopExtUnix::{suggested_method}` function."
  );
}

#[cfg(target_os = "linux")]
fn is_main_thread() -> bool {
  use libc::{c_long, getpid, syscall, SYS_gettid};

  unsafe { syscall(SYS_gettid) == getpid() as c_long }
}

#[cfg(any(target_os = "dragonfly", target_os = "freebsd", target_os = "openbsd"))]
fn is_main_thread() -> bool {
  use libc::pthread_main_np;

  unsafe { pthread_main_np() == 1 }
}

#[cfg(target_os = "netbsd")]
fn is_main_thread() -> bool {
  std::thread::current().name() == Some("main")
}

impl CursorIcon {
  fn to_str(&self) -> &str {
    match self {
      CursorIcon::Crosshair => "crosshair",
      CursorIcon::Hand => "pointer",
      CursorIcon::Arrow => "arrow",
      CursorIcon::Move => "move",
      CursorIcon::Text => "text",
      CursorIcon::Wait => "wait",
      CursorIcon::Help => "help",
      CursorIcon::Progress => "progress",
      CursorIcon::NotAllowed => "not-allowed",
      CursorIcon::ContextMenu => "context-menu",
      CursorIcon::Cell => "cell",
      CursorIcon::VerticalText => "vertical-text",
      CursorIcon::Alias => "alias",
      CursorIcon::Copy => "copy",
      CursorIcon::NoDrop => "no-drop",
      CursorIcon::Grab => "grab",
      CursorIcon::Grabbing => "grabbing",
      CursorIcon::AllScroll => "all-scroll",
      CursorIcon::ZoomIn => "zoom-in",
      CursorIcon::ZoomOut => "zoom-out",
      CursorIcon::EResize => "e-resize",
      CursorIcon::NResize => "n-resize",
      CursorIcon::NeResize => "ne-resize",
      CursorIcon::NwResize => "nw-resize",
      CursorIcon::SResize => "s-resize",
      CursorIcon::SeResize => "se-resize",
      CursorIcon::SwResize => "sw-resize",
      CursorIcon::WResize => "w-resize",
      CursorIcon::EwResize => "ew-resize",
      CursorIcon::NsResize => "ns-resize",
      CursorIcon::NeswResize => "nesw-resize",
      CursorIcon::NwseResize => "nwse-resize",
      CursorIcon::ColResize => "col-resize",
      CursorIcon::RowResize => "row-resize",
      CursorIcon::Default => "default",
    }
  }
}

impl ResizeDirection {
  fn to_cursor_str(&self) -> &str {
    match self {
      ResizeDirection::East => "e-resize",
      ResizeDirection::North => "n-resize",
      ResizeDirection::NorthEast => "ne-resize",
      ResizeDirection::NorthWest => "nw-resize",
      ResizeDirection::South => "s-resize",
      ResizeDirection::SouthEast => "se-resize",
      ResizeDirection::SouthWest => "sw-resize",
      ResizeDirection::West => "w-resize",
    }
  }

  fn to_gtk_edge(&self) -> SurfaceEdge {
    match self {
      ResizeDirection::East => SurfaceEdge::East,
      ResizeDirection::North => SurfaceEdge::North,
      ResizeDirection::NorthEast => SurfaceEdge::NorthEast,
      ResizeDirection::NorthWest => SurfaceEdge::NorthWest,
      ResizeDirection::South => SurfaceEdge::South,
      ResizeDirection::SouthEast => SurfaceEdge::SouthEast,
      ResizeDirection::SouthWest => SurfaceEdge::SouthWest,
      ResizeDirection::West => SurfaceEdge::West,
    }
  }
}
