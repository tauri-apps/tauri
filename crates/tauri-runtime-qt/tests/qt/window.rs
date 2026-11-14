// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `QtWindowDispatcher` and window creation: builder attributes, getters/setters, window
//! events (Resized, Moved, Focused), monitors, handles, and calls on destroyed windows.

use std::{thread, time::Duration};

use qtshim::test_harness::Case;
use tauri_runtime::{
  Error, Icon, ProgressBarState, ProgressBarStatus, ResizeDirection, Runtime, UserAttentionType,
  WindowDispatch,
  dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, PixelUnit, Size},
  webview::WebviewAttributes,
  window::{CursorIcon, PendingWindow, RawWindow, WindowBuilder, WindowSizeConstraints},
};
use tauri_runtime_qt::QtWindowBuilder;
use tauri_utils::config::Color;

use crate::common::*;

pub fn cases() -> Vec<Case> {
  vec![
    Case::pass(
      "window: builder attributes are applied at creation",
      builder_attributes,
    ),
    Case::pass(
      "window: builder size constraints, position and centering",
      builder_geometry,
    ),
    Case::pass(
      "window: prevent_overflow clamps the size to the work area",
      prevent_overflow,
    ),
    Case::pass(
      "window: an invisible window stays hidden until show",
      invisible_until_shown,
    ),
    Case::pass(
      "window: a hidden window built maximized stays hidden",
      hidden_maximized,
    ),
    Case::pass(
      "window: maximize and fullscreen on a hidden window keep it hidden",
      hidden_window_state_setters,
    ),
    Case::pass(
      "window: hiding window buttons keeps a frameless window frameless",
      undecorated_with_buttons,
    ),
    Case::pass(
      "window: boolean setters round-trip through the getters",
      boolean_setters,
    ),
    Case::pass("window: set_size resizes and emits Resized", set_size),
    Case::pass(
      "window: min/max size and size constraints clamp the size",
      size_limits,
    ),
    Case::pass("window: set_position moves and emits Moved", set_position),
    Case::pass(
      "window: maximize, minimize and fullscreen states",
      window_states,
    ),
    Case::pass("window: set_focus focuses and emits Focused", focus),
    Case::pass(
      "window: setters from another thread are seen by later getters",
      setters_from_other_thread,
    ),
    Case::pass(
      "window: cursor, icon, dragging and attention calls succeed",
      misc_setters,
    ),
    Case::pass(
      "window: platform-specific setters are accepted as no-ops",
      noop_setters,
    ),
    Case::pass("window: monitor queries through the dispatcher", monitors),
    Case::pass(
      "window: qt_window is a live widget; gtk_window and default_vbox fail",
      native_handles,
    ),
    Case::pass(
      "window: transient_for_qt accepts a window and ignores a bogus pointer",
      transient_for,
    ),
    Case::pass(
      "window: dispatcher calls on a destroyed window",
      destroyed_window,
    ),
    Case::pass(
      "window: create_window and create_webview through the dispatcher",
      dispatcher_creates,
    ),
    Case::pass(
      "window: an invalid proxy url fails creation and leaves no window",
      invalid_proxy,
    ),
    Case::pass(
      "window: a valid proxy url on the first webview is accepted",
      valid_proxy,
    ),
  ]
}

fn icon() -> Icon<'static> {
  Icon {
    rgba: vec![200; 16 * 16 * 4].into(),
    width: 16,
    height: 16,
  }
}

fn builder_attributes() {
  let mut rt = runtime();
  let b = QtWindowBuilder::new()
    .title("attrs")
    .inner_size(320.0, 240.0)
    .resizable(false)
    .always_on_top(true)
    .maximizable(false)
    .minimizable(false)
    .closable(false)
    .skip_taskbar(true)
    .focusable(false)
    .always_on_bottom(true)
    .background_color(Color(10, 20, 30, 255))
    .icon(icon())
    .unwrap();
  let w = window(&rt, "attrs", b);
  let d = &w.dispatcher;
  assert_eq!(w.label, "attrs");
  assert!(w.webview.is_none());
  assert_eq!(d.title().unwrap(), "attrs");
  let scale = d.scale_factor().unwrap();
  assert!(scale > 0.0);
  assert_eq!(
    d.inner_size().unwrap(),
    LogicalSize::new(320.0, 240.0).to_physical::<u32>(scale)
  );
  let outer = d.outer_size().unwrap();
  assert!(outer.width >= 320 && outer.height >= 240, "{outer:?}");
  assert!(!d.is_resizable().unwrap());
  assert!(d.is_decorated().unwrap());
  assert!(d.is_always_on_top().unwrap());
  assert!(!d.is_maximizable().unwrap());
  assert!(!d.is_minimizable().unwrap());
  assert!(!d.is_closable().unwrap());
  assert!(d.is_visible().unwrap());
  assert!(d.is_enabled().unwrap());
  pump(&mut rt, Duration::from_millis(10));
}

fn builder_geometry() {
  let mut rt = runtime();
  let at = window(
    &rt,
    "at",
    builder()
      .position(30.0, 40.0)
      .inner_size_constraints(WindowSizeConstraints {
        min_width: Some(PixelUnit::Logical(350.0.into())),
        min_height: None,
        max_width: None,
        max_height: Some(PixelUnit::Logical(250.0.into())),
      }),
  );
  // min width 350 > 400? no: 400 stays; max height 250 < 300 clamps.
  let scale = at.dispatcher.scale_factor().unwrap();
  let size = at.dispatcher.inner_size().unwrap().to_logical::<f64>(scale);
  assert_eq!((size.width, size.height), (400.0, 250.0));
  let pos = at.dispatcher.outer_position().unwrap();
  assert_eq!(pos, LogicalPosition::new(30.0, 40.0).to_physical(scale));

  let min = window(&rt, "min", builder().min_inner_size(500.0, 350.0));
  let size = min
    .dispatcher
    .inner_size()
    .unwrap()
    .to_logical::<f64>(scale);
  assert_eq!((size.width, size.height), (500.0, 350.0));

  let max = window(&rt, "max", builder().max_inner_size(200.0, 100.0));
  let size = max
    .dispatcher
    .inner_size()
    .unwrap()
    .to_logical::<f64>(scale);
  assert_eq!((size.width, size.height), (200.0, 100.0));

  // Centered on its screen.
  let c = window(&rt, "centered", builder().center());
  let monitor = c.dispatcher.current_monitor().unwrap().unwrap();
  let outer_pos = c.dispatcher.outer_position().unwrap();
  let outer = c.dispatcher.outer_size().unwrap();
  let center_x = outer_pos.x + outer.width as i32 / 2;
  let screen_center_x = monitor.work_area.position.x + monitor.work_area.size.width as i32 / 2;
  assert!(
    (center_x - screen_center_x).abs() <= 2,
    "window center {center_x}, screen center {screen_center_x}"
  );
  pump(&mut rt, Duration::from_millis(10));
}

fn prevent_overflow() {
  let rt = runtime();
  let monitor = rt.primary_monitor().unwrap();
  let scale = monitor.scale_factor;
  let work = monitor.work_area.size.to_logical::<f64>(scale);
  let big = (work.width + 500.0, work.height + 500.0);
  let w = window(
    &rt,
    "clamped",
    builder().inner_size(big.0, big.1).prevent_overflow(),
  );
  let size = w.dispatcher.inner_size().unwrap().to_logical::<f64>(scale);
  assert!(
    size.width <= work.width && size.height <= work.height,
    "{size:?} > {work:?}"
  );

  let m = window(
    &rt,
    "margin",
    builder()
      .inner_size(big.0, big.1)
      .prevent_overflow_with_margin(Size::Logical(LogicalSize::new(100.0, 50.0))),
  );
  let size = m.dispatcher.inner_size().unwrap().to_logical::<f64>(scale);
  assert!(
    size.width <= work.width - 100.0 && size.height <= work.height - 50.0,
    "{size:?} vs {work:?}"
  );
}

fn invisible_until_shown() {
  let mut rt = runtime();
  let w = window(&rt, "hidden", builder().visible(false));
  assert!(!w.dispatcher.is_visible().unwrap());
  w.dispatcher.show().unwrap();
  assert!(w.dispatcher.is_visible().unwrap());
  w.dispatcher.hide().unwrap();
  assert!(!w.dispatcher.is_visible().unwrap());
  pump(&mut rt, Duration::from_millis(10));
}

fn hidden_maximized() {
  let mut rt = runtime();
  let w = window(&rt, "hidden", builder().visible(false).maximized(true));
  pump(&mut rt, Duration::from_millis(10));
  assert!(!w.dispatcher.is_visible().unwrap());
  w.dispatcher.show().unwrap();
  wait_for(&mut rt, "maximized once shown", || {
    w.dispatcher.is_maximized().unwrap()
  });
}

fn hidden_window_state_setters() {
  let mut rt = runtime();
  let w = window(&rt, "hidden", builder().visible(false));
  w.dispatcher.maximize().unwrap();
  w.dispatcher.set_fullscreen(true).unwrap();
  w.dispatcher.set_fullscreen(false).unwrap();
  pump(&mut rt, Duration::from_millis(10));
  assert!(!w.dispatcher.is_visible().unwrap());
}

fn undecorated_with_buttons() {
  let mut rt = runtime();
  let w = window(
    &rt,
    "frameless",
    builder().decorations(false).closable(false),
  );
  assert!(!w.dispatcher.is_decorated().unwrap());
  assert!(!w.dispatcher.is_closable().unwrap());

  let w = window(&rt, "later", builder());
  w.dispatcher.set_decorations(false).unwrap();
  w.dispatcher.set_minimizable(false).unwrap();
  pump(&mut rt, Duration::from_millis(10));
  assert!(!w.dispatcher.is_decorated().unwrap());
}

fn boolean_setters() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder());
  let d = &w.dispatcher;
  type Getter = fn(&<Rt as Runtime<Msg>>::WindowDispatcher) -> tauri_runtime::Result<bool>;
  type Setter = fn(&<Rt as Runtime<Msg>>::WindowDispatcher, bool) -> tauri_runtime::Result<()>;
  let pairs: [(&str, Setter, Getter); 7] = [
    ("resizable", |d, v| d.set_resizable(v), |d| d.is_resizable()),
    (
      "decorations",
      |d, v| d.set_decorations(v),
      |d| d.is_decorated(),
    ),
    (
      "always_on_top",
      |d, v| d.set_always_on_top(v),
      |d| d.is_always_on_top(),
    ),
    (
      "maximizable",
      |d, v| d.set_maximizable(v),
      |d| d.is_maximizable(),
    ),
    (
      "minimizable",
      |d, v| d.set_minimizable(v),
      |d| d.is_minimizable(),
    ),
    ("closable", |d, v| d.set_closable(v), |d| d.is_closable()),
    ("enabled", |d, v| d.set_enabled(v), |d| d.is_enabled()),
  ];
  for (name, set, get) in pairs {
    for v in [false, true, false, true] {
      set(d, v).unwrap();
      assert_eq!(get(d).unwrap(), v, "{name} = {v}");
    }
  }
  d.set_title("renamed").unwrap();
  assert_eq!(d.title().unwrap(), "renamed");
  d.set_title("").unwrap();
  assert_eq!(d.title().unwrap(), "");
  d.set_title("ünïcødé ✓").unwrap();
  assert_eq!(d.title().unwrap(), "ünïcødé ✓");
  pump(&mut rt, Duration::from_millis(10));
}

fn set_size() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder());
  let scale = w.dispatcher.scale_factor().unwrap();
  w.dispatcher
    .set_size(LogicalSize::new(500.0, 400.0).into())
    .unwrap();
  let want = LogicalSize::new(500.0, 400.0).to_physical::<u32>(scale);
  assert_eq!(w.dispatcher.inner_size().unwrap(), want);
  wait_window_event(&mut rt, "main", |e| {
    *e == Win::Resized(want.width, want.height)
  });
  // Physical sizes are converted with the scale factor.
  w.dispatcher
    .set_size(PhysicalSize::new(300, 200).into())
    .unwrap();
  assert_eq!(
    w.dispatcher.inner_size().unwrap(),
    PhysicalSize::new(300, 200)
  );
  wait_window_event(&mut rt, "main", |e| *e == Win::Resized(300, 200));
}

fn size_limits() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder());
  let d = &w.dispatcher;
  let logical = |d: &<Rt as Runtime<Msg>>::WindowDispatcher| {
    let s = d
      .inner_size()
      .unwrap()
      .to_logical::<f64>(d.scale_factor().unwrap());
    (s.width, s.height)
  };
  d.set_min_size(Some(LogicalSize::new(300.0, 250.0).into()))
    .unwrap();
  d.set_size(LogicalSize::new(100.0, 100.0).into()).unwrap();
  assert_eq!(logical(d), (300.0, 250.0));
  d.set_min_size(None).unwrap();
  d.set_size(LogicalSize::new(100.0, 100.0).into()).unwrap();
  assert_eq!(logical(d), (100.0, 100.0));

  d.set_max_size(Some(LogicalSize::new(200.0, 150.0).into()))
    .unwrap();
  d.set_size(LogicalSize::new(900.0, 900.0).into()).unwrap();
  assert_eq!(logical(d), (200.0, 150.0));
  d.set_max_size(None).unwrap();
  d.set_size(LogicalSize::new(900.0, 700.0).into()).unwrap();
  assert_eq!(logical(d), (900.0, 700.0));

  d.set_size_constraints(WindowSizeConstraints {
    min_width: Some(PixelUnit::Logical(250.0.into())),
    min_height: None,
    max_width: None,
    max_height: Some(PixelUnit::Logical(300.0.into())),
  })
  .unwrap();
  d.set_size(LogicalSize::new(100.0, 900.0).into()).unwrap();
  assert_eq!(logical(d), (250.0, 300.0));
  d.set_size_constraints(WindowSizeConstraints::default())
    .unwrap();
  d.set_size(LogicalSize::new(100.0, 900.0).into()).unwrap();
  assert_eq!(logical(d), (100.0, 900.0));
  pump(&mut rt, Duration::from_millis(10));
}

fn set_position() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder());
  let scale = w.dispatcher.scale_factor().unwrap();
  w.dispatcher
    .set_position(LogicalPosition::new(120.0, 80.0).into())
    .unwrap();
  let want = LogicalPosition::new(120.0, 80.0).to_physical::<i32>(scale);
  assert_eq!(w.dispatcher.outer_position().unwrap(), want);
  let inner = w.dispatcher.inner_position().unwrap();
  assert!(inner.x >= want.x && inner.y >= want.y, "{inner:?}");
  wait_window_event(&mut rt, "main", |e| matches!(e, Win::Moved(..)));
  w.dispatcher
    .set_position(PhysicalPosition::new(10, 20).into())
    .unwrap();
  assert_eq!(
    w.dispatcher.outer_position().unwrap(),
    PhysicalPosition::new(10, 20)
  );
  // centering moves it again.
  w.dispatcher.center().unwrap();
  assert_ne!(
    w.dispatcher.outer_position().unwrap(),
    PhysicalPosition::new(10, 20)
  );
}

fn window_states() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder());
  let d = &w.dispatcher;
  assert!(!d.is_maximized().unwrap() && !d.is_minimized().unwrap());
  d.maximize().unwrap();
  wait_for(&mut rt, "maximized", || d.is_maximized().unwrap());
  d.unmaximize().unwrap();
  wait_for(&mut rt, "unmaximized", || !d.is_maximized().unwrap());
  d.minimize().unwrap();
  wait_for(&mut rt, "minimized", || d.is_minimized().unwrap());
  d.unminimize().unwrap();
  wait_for(&mut rt, "unminimized", || !d.is_minimized().unwrap());
  d.set_fullscreen(true).unwrap();
  wait_for(&mut rt, "fullscreen", || d.is_fullscreen().unwrap());
  d.set_fullscreen(false).unwrap();
  wait_for(&mut rt, "not fullscreen", || !d.is_fullscreen().unwrap());
  let monitor = d.current_monitor().unwrap().unwrap();
  d.set_fullscreen_on_monitor(PhysicalPosition::new(
    monitor.position.x as f64 + 1.0,
    monitor.position.y as f64 + 1.0,
  ))
  .unwrap();
  wait_for(&mut rt, "fullscreen on the monitor", || {
    d.is_fullscreen().unwrap()
  });
  // A point outside every monitor is ignored.
  d.set_fullscreen(false).unwrap();
  d.set_fullscreen_on_monitor(PhysicalPosition::new(-1.0e7, -1.0e7))
    .unwrap();
  pump(&mut rt, Duration::from_millis(20));
  assert!(!d.is_fullscreen().unwrap());

  // Builder states.
  let m = window(&rt, "maxed", builder().maximized(true));
  wait_for(&mut rt, "maximized from the builder", || {
    m.dispatcher.is_maximized().unwrap()
  });
  let f = window(&rt, "full", builder().fullscreen(true));
  wait_for(&mut rt, "fullscreen from the builder", || {
    f.dispatcher.is_fullscreen().unwrap()
  });
}

fn focus() {
  let mut rt = runtime();
  let a = window(&rt, "a", builder());
  let b = window(&rt, "b", builder());
  b.dispatcher.set_focus().unwrap();
  wait_for(&mut rt, "b focused", || b.dispatcher.is_focused().unwrap());
  wait_window_event(&mut rt, "b", |e| *e == Win::Focused(true));
  a.dispatcher.set_focus().unwrap();
  wait_for(&mut rt, "a focused", || a.dispatcher.is_focused().unwrap());
  wait_window_event(&mut rt, "b", |e| *e == Win::Focused(false));
  assert!(!b.dispatcher.is_focused().unwrap());
  // Not focusable: accepted (the window manager decides what that means).
  b.dispatcher.set_focusable(false).unwrap();
  b.dispatcher.set_focusable(true).unwrap();
}

fn setters_from_other_thread() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder());
  let d = w.dispatcher.clone();
  let t = thread::spawn(move || {
    d.set_title("from a thread").unwrap();
    d.set_resizable(false).unwrap();
    d.set_size(LogicalSize::new(333.0, 222.0).into()).unwrap();
    // Queued in order: the getters (which wait for the main loop) see the setters.
    assert_eq!(d.title().unwrap(), "from a thread");
    assert!(!d.is_resizable().unwrap());
    let s = d
      .inner_size()
      .unwrap()
      .to_logical::<f64>(d.scale_factor().unwrap());
    assert_eq!((s.width, s.height), (333.0, 222.0));
  });
  wait_for(&mut rt, "the thread", || t.is_finished());
  t.join().unwrap();
}

fn misc_setters() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder());
  let d = &w.dispatcher;
  d.set_icon(icon()).unwrap();
  // A malformed icon (too few bytes) must not crash.
  d.set_icon(Icon {
    rgba: vec![0; 3].into(),
    width: 16,
    height: 16,
  })
  .unwrap();
  for icon in [
    CursorIcon::Default,
    CursorIcon::Hand,
    CursorIcon::Text,
    CursorIcon::Grabbing,
    CursorIcon::ZoomIn,
  ] {
    d.set_cursor_icon(icon).unwrap();
  }
  d.set_cursor_visible(false).unwrap();
  d.set_cursor_visible(true).unwrap();
  d.set_cursor_grab(true).unwrap();
  d.set_cursor_grab(false).unwrap();
  d.set_cursor_position(LogicalPosition::new(10.0, 10.0))
    .unwrap();
  d.set_ignore_cursor_events(true).unwrap();
  d.set_ignore_cursor_events(false).unwrap();
  d.start_dragging().unwrap();
  for dir in [
    ResizeDirection::East,
    ResizeDirection::North,
    ResizeDirection::NorthEast,
    ResizeDirection::NorthWest,
    ResizeDirection::South,
    ResizeDirection::SouthEast,
    ResizeDirection::SouthWest,
    ResizeDirection::West,
  ] {
    d.start_resize_dragging(dir).unwrap();
  }
  d.request_user_attention(Some(UserAttentionType::Critical))
    .unwrap();
  d.request_user_attention(Some(UserAttentionType::Informational))
    .unwrap();
  d.request_user_attention(None).unwrap();
  d.set_background_color(Some(Color(1, 2, 3, 255))).unwrap();
  d.set_background_color(None).unwrap();
  d.set_skip_taskbar(true).unwrap();
  d.set_always_on_bottom(true).unwrap();
  d.set_always_on_bottom(false).unwrap();
  d.set_badge_count(Some(3), Some("test.desktop".into()))
    .unwrap();
  d.set_badge_count(Some(0), None).unwrap();
  d.set_badge_count(None, None).unwrap();
  for (status, progress) in [
    (Some(ProgressBarStatus::Normal), Some(50)),
    (Some(ProgressBarStatus::Indeterminate), None),
    (None, Some(250)),
    (Some(ProgressBarStatus::None), None),
  ] {
    d.set_progress_bar(ProgressBarState {
      status,
      progress,
      desktop_filename: None,
    })
    .unwrap();
  }
  pump(&mut rt, Duration::from_millis(20));
  // Still alive and responsive.
  assert!(d.is_visible().unwrap());
}

fn noop_setters() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder());
  let d = &w.dispatcher;
  d.set_shadow(false).unwrap();
  d.set_visible_on_all_workspaces(true).unwrap();
  d.set_content_protected(true).unwrap();
  d.set_badge_label(Some("x".into())).unwrap();
  d.set_overlay_icon(Some(icon())).unwrap();
  d.set_title_bar_style(tauri_utils::TitleBarStyle::Transparent)
    .unwrap();
  d.set_traffic_light_position(LogicalPosition::new(1.0, 1.0).into())
    .unwrap();
  pump(&mut rt, Duration::from_millis(10));
  assert!(d.is_visible().unwrap());
}

fn monitors() {
  let rt = runtime();
  let w = window(&rt, "main", builder());
  let d = &w.dispatcher;
  let current = d.current_monitor().unwrap().unwrap();
  let primary = d.primary_monitor().unwrap().unwrap();
  assert_eq!(current.name, primary.name);
  assert!(!d.available_monitors().unwrap().is_empty());
  let p = primary.position;
  assert_eq!(
    d.monitor_from_point(p.x as f64 + 1.0, p.y as f64 + 1.0)
      .unwrap()
      .unwrap()
      .name,
    primary.name
  );
  assert!(d.monitor_from_point(-1.0e7, 0.0).unwrap().is_none());
}

fn native_handles() {
  let rt = runtime();
  let w = window(&rt, "main", builder());
  let d = &w.dispatcher;
  let ptr = d.qt_window().unwrap();
  assert!(!ptr.is_null());
  assert!(tauri_runtime_qt::debug::Widget::from_ptr(ptr).is_some());
  assert!(d.gtk_window().is_err());
  assert!(d.default_vbox().is_err());
  // Offscreen has no Wayland surface or X11 window: an error, not a crash.
  let _ = d.window_handle();
}

fn transient_for() {
  let mut rt = runtime();
  let parent = window(&rt, "parent", builder());
  let ptr = parent.dispatcher.qt_window().unwrap();
  let child = window(
    &rt,
    "child",
    builder()
      .transient_for_qt(ptr)
      .transient_for(std::ptr::null_mut()),
  );
  assert!(child.dispatcher.is_visible().unwrap());
  // A pointer that isn't a live widget is ignored.
  let stray = window(&rt, "stray", builder().transient_for_qt(0x10 as *mut _));
  assert!(stray.dispatcher.is_visible().unwrap());
  // The parent going away first doesn't take the child down with it.
  parent.dispatcher.destroy().unwrap();
  wait_window_event(&mut rt, "parent", |e| *e == Win::Destroyed);
  assert!(child.dispatcher.is_visible().unwrap());
}

fn destroyed_window() {
  let mut rt = runtime();
  let keep = window(&rt, "keep", builder());
  let w = window(&rt, "gone", builder());
  w.dispatcher.destroy().unwrap();
  wait_window_event(&mut rt, "gone", |e| *e == Win::Destroyed);
  let d = &w.dispatcher;
  assert!(matches!(d.title(), Err(Error::WindowNotFound)));
  assert!(matches!(d.inner_size(), Err(Error::WindowNotFound)));
  assert!(matches!(d.is_visible(), Err(Error::WindowNotFound)));
  assert!(matches!(d.qt_window(), Err(Error::WindowNotFound)));
  assert!(matches!(d.current_monitor(), Err(Error::WindowNotFound)));
  assert!(d.window_handle().is_err());
  // Setters are fire-and-forget no-ops.
  d.set_title("x").unwrap();
  d.set_size(LogicalSize::new(10.0, 10.0).into()).unwrap();
  d.show().unwrap();
  d.set_resizable(false).unwrap();
  // Creating a webview in it fails.
  let mut d2 = d.clone();
  assert!(matches!(
    d2.create_webview(pending_webview("orphan", "about:blank")),
    Err(Error::WindowNotFound)
  ));
  pump(&mut rt, Duration::from_millis(20));
  assert_eq!(keep.dispatcher.title().unwrap(), "Tauri App");
}

fn dispatcher_creates() {
  let mut rt = runtime();
  let w = window(&rt, "main", builder());
  let mut d = w.dispatcher.clone();
  let other = d
    .create_window(
      PendingWindow::new(builder().title("second"), "second").unwrap(),
      None::<fn(RawWindow)>,
    )
    .unwrap();
  assert_eq!(other.dispatcher.title().unwrap(), "second");
  assert_ne!(other.id, w.id);
  let wv = d
    .create_webview(pending_webview("child", &page("child")))
    .unwrap();
  assert_eq!(wv.label, "child");
  wait_title(&mut rt, &wv.dispatcher, "child");
}

fn invalid_proxy() {
  let mut rt = runtime();
  let mut attrs = WebviewAttributes::new(tauri_utils::config::WebviewUrl::App("x".into()));
  attrs.proxy_url = Some("ftp://proxy.invalid:21".parse().unwrap());
  let mut webview = pending_webview("bad", "about:blank");
  webview.webview_attributes = attrs;
  let mut pending = PendingWindow::new(builder(), "bad").unwrap();
  pending.set_webview(webview);
  let r = rt.create_window(pending, None::<fn(RawWindow)>);
  assert!(matches!(r, Err(Error::InvalidProxyUrl)), "{r:?}");
  // The half-built window is gone: closing the only real window exits.
  let w = window(&rt, "main", builder());
  w.dispatcher.close().unwrap();
  wait_for(&mut rt, "the exit request", || {
    has_event(&Ev::ExitRequested(None))
  });
  assert!(window_events("bad").is_empty());
}

fn valid_proxy() {
  let mut rt = runtime();
  for (label, proxy) in [
    ("http", "http://127.0.0.1:9"),
    ("socks", "socks5://127.0.0.1:9"),
  ] {
    let mut webview = pending_webview(label, "about:blank");
    webview.webview_attributes.proxy_url = Some(proxy.parse().unwrap());
    webview.webview_attributes.additional_browser_args = Some("--disable-gpu".into());
    let (w, _) = webview_window_with(&rt, label, builder(), webview);
    assert!(w.dispatcher.is_visible().unwrap());
  }
  // Custom schemes don't go through the proxy.
  let (_w, wv) = webview_window(&rt, "main", &page("proxied"));
  wait_title(&mut rt, &wv, "proxied");
}
