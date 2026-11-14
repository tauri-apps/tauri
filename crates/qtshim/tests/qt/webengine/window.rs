// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `qtshim::webengine::Window`: geometry, state, flags, close requests, native handles.
//!
//! Runs offscreen: there is no window manager, so positions are honoured as requested, frames
//! have no decorations, and "minimized" is only a state flag.

use std::{cell::RefCell, rc::Rc, time::Duration};

use qtshim::test_harness::Case;
use qtshim::{
  app::{self, EventKind, Reply},
  menu::{Container, MenuBar, StandardAction, Widget},
  webengine::{Profile, View, ViewOptions, Window, WindowFlag, window_state},
};

use crate::common::{clear_events, events, has_event, init, pump, record_events, wait_for};

pub fn cases() -> Vec<Case> {
  vec![
    // creation
    Case::panics(
      "window: new before app::init panics",
      "Qt can only be used",
      new_before_init,
    ),
    Case::pass("window: new is hidden, 800x600, untitled", new_defaults),
    Case::pass("window: transparent window shows", new_transparent),
    Case::pass("window: Debug output names the pointer", debug_output),
    // title
    Case::pass("window: title round-trips unicode", title_unicode),
    Case::pass("window: empty title round-trips", title_empty),
    Case::pass("window: NUL bytes are stripped from the title", title_nul),
    // visibility
    Case::pass(
      "window: set_visible toggles the VISIBLE state",
      visible_toggle,
    ),
    // geometry
    Case::pass(
      "window: resize updates geometry and fires WindowResized",
      resize_event,
    ),
    Case::pass(
      "window: move_to updates position and fires WindowMoved",
      move_event,
    ),
    Case::pass("window: min size clamps resize", min_size_clamps),
    Case::pass("window: max size clamps resize", max_size_clamps),
    Case::pass(
      "window: None removes size constraints",
      size_constraints_removed,
    ),
    Case::pass(
      "window: set_resizable(false) fixes the size",
      resizable_false,
    ),
    Case::pass(
      "window: set_resizable(true) keeps min/max constraints",
      resizable_true_keeps_constraints,
    ),
    Case::pass(
      "window: resize of a non-resizable window changes its fixed size",
      resize_non_resizable,
    ),
    Case::pass(
      "window: center puts the frame in the middle of the screen",
      center,
    ),
    Case::pass("window: screen and scale_factor are sane", screen_and_scale),
    // state
    Case::pass("window: set_minimized sets and clears MINIMIZED", minimized),
    Case::pass("window: set_maximized sets and clears MAXIMIZED", maximized),
    Case::pass(
      "window: set_fullscreen sets and clears FULLSCREEN",
      fullscreen,
    ),
    Case::pass(
      "window: maximize, fullscreen and minimize keep a hidden window hidden",
      maximize_keeps_hidden,
    ),
    Case::pass(
      "window: fullscreen_on a point on the screen goes fullscreen",
      fullscreen_on_screen,
    ),
    Case::pass(
      "window: fullscreen_on a point off every screen does nothing",
      fullscreen_on_nowhere,
    ),
    Case::pass("window: focus restores a minimized window", focus_restores),
    Case::pass("window: focus activates and fires WindowFocus", focus_event),
    // flags
    Case::pass("window: default state bits", default_state_bits),
    Case::pass(
      "window: Decorations flag toggles DECORATED",
      flag_decorations,
    ),
    Case::pass("window: AlwaysOnTop flag toggles ON_TOP", flag_on_top),
    Case::pass("window: Closable flag toggles CLOSABLE only", flag_closable),
    Case::pass(
      "window: Minimizable flag toggles MINIMIZABLE only",
      flag_minimizable,
    ),
    Case::pass(
      "window: Maximizable flag toggles MAXIMIZABLE only",
      flag_maximizable,
    ),
    Case::pass(
      "window: every flag keeps a visible window visible",
      flags_keep_visible,
    ),
    Case::pass(
      "window: AllWorkspaces sets EWMH sticky (X11; run with QT_QPA_PLATFORM=xcb)",
      all_workspaces_x11,
    ),
    Case::pass("window: set_enabled toggles ENABLED", enabled),
    // appearance and input
    Case::pass("window: set_icon accepts a valid buffer", icon_valid),
    Case::pass(
      "window: set_icon ignores short and empty buffers",
      icon_short,
    ),
    Case::pass(
      "window: set_icon with overflowing dimensions is ignored",
      icon_overflow,
    ),
    Case::pass("window: set_background Some and None", background),
    Case::pass("window: cursor functions don't crash", cursor),
    Case::pass(
      "window: cursor visibility applies to views",
      cursor_with_view,
    ),
    Case::pass(
      "window: start_move/start_resize fail on a hidden window",
      start_move_hidden,
    ),
    Case::pass(
      "window: start_move/start_resize don't crash when shown",
      start_move_shown,
    ),
    Case::pass("window: alert doesn't crash", alert),
    // relations and handles
    Case::pass("window: set_transient_for another window", transient_for),
    Case::pass(
      "window: set_transient_for itself doesn't hang",
      transient_for_self,
    ),
    Case::pass(
      "window: set_transient_for_widget with a live widget",
      transient_for_widget,
    ),
    Case::pass(
      "window: set_transient_for_widget with a dead widget is a no-op",
      transient_for_dead_widget,
    ),
    Case::pass(
      "window: widget_ptr round-trips through Widget::from_ptr",
      widget_round_trip,
    ),
    Case::pass(
      "window: widget handle dies with the window",
      widget_dies_with_window,
    ),
    Case::pass(
      "window: content_size equals the client size without a menubar",
      content_size_plain,
    ),
    Case::pass(
      "window: the inner size excludes a menubar",
      content_size_menubar,
    ),
    Case::pass(
      "window: resize sets the area below a menubar, and toggling it emits Resized",
      resize_with_menubar,
    ),
    Case::pass(
      "window: the edges of a frameless window show resize cursors",
      frameless_resize_cursor,
    ),
    Case::pass(
      "window: native_handle is None offscreen",
      native_handle_offscreen,
    ),
    // close requests
    Case::pass(
      "window: close request is blocked and reported",
      close_request_blocked,
    ),
    Case::pass(
      "window: dropping the window inside the close handler",
      drop_in_close_handler,
    ),
    Case::pass(
      "window: dropping a window inside a resize handler",
      drop_in_resize_handler,
    ),
    // edge cases
    Case::pass("window: transient-for cycle doesn't hang", transient_cycle),
    Case::pass(
      "window: negative and zero sizes are clamped",
      resize_negative,
    ),
    Case::pass(
      "window: huge sizes don't crash a visible window",
      resize_huge,
    ),
    Case::pass("window: min size larger than max size", min_above_max),
    Case::pass("window: half-unset size constraints", half_constraints),
    Case::pass("window: extreme positions don't crash", move_extreme),
    // lifecycle
    Case::pass("window: create and drop many windows", many_windows),
    Case::pass(
      "window: create and drop many windows inside the loop",
      many_windows_in_loop,
    ),
    Case::pass(
      "window: dropping a window makes its views inert",
      views_inert_after_window_drop,
    ),
    Case::pass(
      "window: url() of a view whose window was dropped",
      view_url_after_window_drop,
    ),
    Case::pass("window: view dropped before its window", view_dropped_first),
    Case::panics(
      "window: title after shutdown panics",
      "Qt can only be used",
      use_after_shutdown,
    ),
    Case::panics(
      "window: resize after shutdown panics",
      "Qt can only be used",
      resize_after_shutdown,
    ),
    Case::pass(
      "window: drop after shutdown doesn't crash",
      drop_after_shutdown,
    ),
    Case::panics(
      "window: use from another thread panics",
      "Qt can only be used",
      use_from_other_thread,
    ),
  ]
}

// --- helpers ----------------------------------------------------------------------------------

/// A shown window of the given client size, with the event loop run until it's exposed.
fn shown(id: u64, w: i32, h: i32) -> Rc<Window> {
  let window = Rc::new(Window::new(id, false));
  window.resize(w, h);
  window.set_visible(true);
  let win = window.clone();
  wait_for("the window to be shown", move || {
    win.state() & window_state::VISIBLE != 0
  });
  window
}

fn size(window: &Window) -> (i32, i32) {
  let g = window.geometry().inner;
  (g.2, g.3)
}

fn has(window: &Window, bit: u32) -> bool {
  window.state() & bit != 0
}

fn phys(v: i32, scale: f64) -> i64 {
  (f64::from(v) * scale).round() as i64
}

// --- creation ---------------------------------------------------------------------------------

fn new_before_init() {
  let _ = Window::new(1, false);
}

fn new_defaults() {
  init();
  let window = Window::new(1, false);
  assert!(!has(&window, window_state::VISIBLE));
  assert_eq!(size(&window), (800, 600));
  assert_eq!(window.title(), "");
}

fn new_transparent() {
  init();
  let window = Window::new(7, true);
  window.set_visible(true);
  assert!(has(&window, window_state::VISIBLE));
  pump(Duration::from_millis(50));
}

fn debug_output() {
  init();
  let window = Window::new(1, false);
  assert!(format!("{window:?}").starts_with("Window { raw: "));
}

// --- title ------------------------------------------------------------------------------------

fn title_unicode() {
  init();
  let window = Window::new(1, false);
  let title = "héllo — 窓 🪟 \u{202e}rtl";
  window.set_title(title);
  assert_eq!(window.title(), title);
}

fn title_empty() {
  init();
  let window = Window::new(1, false);
  window.set_title("something");
  window.set_title("");
  assert_eq!(window.title(), "");
}

fn title_nul() {
  init();
  let window = Window::new(1, false);
  window.set_title("a\0b\0");
  assert_eq!(window.title(), "ab");
}

// --- visibility -------------------------------------------------------------------------------

fn visible_toggle() {
  init();
  let window = Window::new(1, false);
  window.set_visible(true);
  assert!(has(&window, window_state::VISIBLE));
  window.set_visible(false);
  assert!(!has(&window, window_state::VISIBLE));
  window.set_visible(true);
  assert!(has(&window, window_state::VISIBLE));
}

// --- geometry ---------------------------------------------------------------------------------

fn resize_event() {
  init();
  record_events(|_| Reply::default());
  let window = shown(3, 300, 200);
  clear_events();
  window.resize(321, 234);
  assert_eq!(size(&window), (321, 234));
  let s = window.scale_factor();
  let (w, h) = (phys(321, s), phys(234, s));
  wait_for("WindowResized", move || {
    has_event(|e| e.kind == EventKind::WindowResized && e.window == 3 && e.a == w && e.b == h)
  });
}

fn move_event() {
  init();
  record_events(|_| Reply::default());
  let window = shown(4, 200, 150);
  clear_events();
  window.move_to(37, 41);
  let outer = window.geometry().outer;
  assert_eq!((outer.0, outer.1), (37, 41), "{:?}", window.geometry());
  let s = window.scale_factor();
  let (x, y) = (phys(37, s), phys(41, s));
  wait_for("WindowMoved", move || {
    has_event(|e| e.kind == EventKind::WindowMoved && e.window == 4 && e.a == x && e.b == y)
  });
}

fn min_size_clamps() {
  init();
  let window = Window::new(1, false);
  window.set_min_size(Some((400, 300)));
  window.resize(100, 100);
  assert_eq!(size(&window), (400, 300));
  window.resize(500, 100);
  assert_eq!(size(&window), (500, 300));
}

fn max_size_clamps() {
  init();
  let window = Window::new(1, false);
  window.set_max_size(Some((300, 200)));
  // Setting a max below the current size shrinks the window.
  assert_eq!(size(&window), (300, 200));
  window.resize(1000, 1000);
  assert_eq!(size(&window), (300, 200));
  window.resize(100, 150);
  assert_eq!(size(&window), (100, 150));
}

fn size_constraints_removed() {
  init();
  let window = Window::new(1, false);
  window.set_min_size(Some((400, 300)));
  window.set_max_size(Some((500, 400)));
  window.set_min_size(None);
  window.set_max_size(None);
  window.resize(50, 60);
  assert_eq!(size(&window), (50, 60));
  window.resize(1500, 1200);
  assert_eq!(size(&window), (1500, 1200));
}

fn resizable_false() {
  init();
  let window = Window::new(1, false);
  window.resize(300, 200);
  assert!(has(&window, window_state::RESIZABLE));
  window.set_resizable(false);
  assert!(!has(&window, window_state::RESIZABLE));
  // A fixed size ignores min/max changes.
  window.set_min_size(Some((400, 400)));
  assert_eq!(size(&window), (300, 200));
  window.set_resizable(true);
  assert!(has(&window, window_state::RESIZABLE));
}

fn resizable_true_keeps_constraints() {
  init();
  let window = Window::new(1, false);
  window.set_min_size(Some((200, 100)));
  window.set_max_size(Some((400, 300)));
  window.set_resizable(false);
  window.set_resizable(true);
  window.resize(10, 10);
  assert_eq!(size(&window), (200, 100));
  window.resize(1000, 1000);
  assert_eq!(size(&window), (400, 300));
}

fn resize_non_resizable() {
  init();
  let window = Window::new(1, false);
  window.set_resizable(false);
  window.resize(250, 150);
  assert_eq!(size(&window), (250, 150));
  assert!(!has(&window, window_state::RESIZABLE));
}

fn center() {
  init();
  let window = shown(1, 200, 100);
  window.move_to(0, 0);
  window.center();
  let screen = window.screen();
  let f = window.geometry().outer;
  let (cx, cy) = (f.0 + f.2 / 2, f.1 + f.3 / 2);
  let (sx, sy) = (
    screen.available_x + screen.available_width / 2,
    screen.available_y + screen.available_height / 2,
  );
  assert!(
    (cx - sx).abs() <= 1 && (cy - sy).abs() <= 1,
    "frame {f:?} not centered on {screen:?}"
  );
}

fn screen_and_scale() {
  init();
  let window = shown(1, 200, 100);
  let s = window.scale_factor();
  assert!(s > 0.0 && s.is_finite(), "{s}");
  let screen = window.screen();
  assert!(screen.width > 0 && screen.height > 0, "{screen:?}");
  assert!(
    screen.available_width > 0 && screen.available_height > 0,
    "{screen:?}"
  );
  assert_eq!(screen.scale, s);
  assert!(
    app::screens().iter().any(|sc| sc.name == screen.name),
    "{screen:?} not in app::screens()"
  );
}

// --- state ------------------------------------------------------------------------------------

fn minimized() {
  init();
  let window = shown(1, 200, 100);
  window.set_minimized(true);
  assert!(has(&window, window_state::MINIMIZED));
  window.set_minimized(false);
  assert!(!has(&window, window_state::MINIMIZED));
}

fn maximized() {
  init();
  let window = shown(1, 200, 100);
  window.set_maximized(true);
  assert!(has(&window, window_state::MAXIMIZED));
  // Offscreen applies the state: the window covers the available screen area.
  let win = window.clone();
  let screen = window.screen();
  wait_for("the maximized size", move || {
    size(&win).0 >= screen.available_width - 20
  });
  window.set_maximized(false);
  assert!(!has(&window, window_state::MAXIMIZED));
}

fn fullscreen() {
  init();
  let window = shown(1, 200, 100);
  window.set_fullscreen(true);
  assert!(has(&window, window_state::FULLSCREEN));
  let screen = window.screen();
  let win = window.clone();
  wait_for("the fullscreen size", move || {
    size(&win) == (screen.width, screen.height)
  });
  window.set_fullscreen(false);
  assert!(!has(&window, window_state::FULLSCREEN));
}

fn maximize_keeps_hidden() {
  init();
  let window = Window::new(1, false);
  window.set_maximized(true);
  window.set_fullscreen(true);
  window.set_minimized(true);
  // Like tao: the state applies, the window stays hidden until shown.
  assert!(!has(&window, window_state::VISIBLE));
  window.set_minimized(false);
  window.set_fullscreen(false);
  assert!(has(&window, window_state::MAXIMIZED));
  window.set_visible(true);
  assert!(has(
    &window,
    window_state::VISIBLE | window_state::MAXIMIZED
  ));
}

fn fullscreen_on_screen() {
  init();
  let window = shown(1, 200, 100);
  let s = window.screen();
  window.fullscreen_on(s.x + s.width / 2, s.y + s.height / 2);
  assert!(has(&window, window_state::FULLSCREEN));
}

fn fullscreen_on_nowhere() {
  init();
  let window = shown(1, 200, 100);
  window.fullscreen_on(-1_000_000, -1_000_000);
  assert!(!has(&window, window_state::FULLSCREEN));
}

fn focus_restores() {
  init();
  let window = shown(1, 200, 100);
  window.set_minimized(true);
  assert!(has(&window, window_state::MINIMIZED));
  window.focus();
  assert!(!has(&window, window_state::MINIMIZED));
}

fn focus_event() {
  init();
  record_events(|_| Reply::default());
  let a = shown(1, 200, 100);
  let b = shown(2, 200, 100);
  a.focus();
  let wa = a.clone();
  wait_for("window 1 to be active", move || {
    has(&wa, window_state::ACTIVE)
  });
  wait_for("WindowFocus(1, true)", || {
    has_event(|e| e.kind == EventKind::WindowFocus && e.window == 1 && e.a == 1)
  });
  clear_events();
  b.focus();
  let wb = b.clone();
  wait_for("window 2 to be active", move || {
    has(&wb, window_state::ACTIVE)
  });
  wait_for("WindowFocus(1, false) and (2, true)", || {
    has_event(|e| e.kind == EventKind::WindowFocus && e.window == 1 && e.a == 0)
      && has_event(|e| e.kind == EventKind::WindowFocus && e.window == 2 && e.a == 1)
  });
  assert!(!has(&a, window_state::ACTIVE));
}

// --- flags ------------------------------------------------------------------------------------

fn default_state_bits() {
  init();
  let window = Window::new(1, false);
  let s = window.state();
  for bit in [
    window_state::ENABLED,
    window_state::DECORATED,
    window_state::RESIZABLE,
    window_state::CLOSABLE,
    window_state::MINIMIZABLE,
    window_state::MAXIMIZABLE,
  ] {
    assert!(s & bit != 0, "bit {bit} missing from {s:#b}");
  }
  for bit in [
    window_state::MINIMIZED,
    window_state::MAXIMIZED,
    window_state::FULLSCREEN,
    window_state::ACTIVE,
    window_state::VISIBLE,
    window_state::ON_TOP,
  ] {
    assert!(s & bit == 0, "bit {bit} unexpectedly in {s:#b}");
  }
}

/// Toggle `flag` off and on, checking `bit` follows it and the other button bits don't change.
fn check_flag(flag: WindowFlag, bit: u32, on_means_set: bool) {
  init();
  let window = Window::new(1, false);
  let others = |s: u32| s & !bit;
  let before = window.state();
  window.set_flag(flag, !on_means_set);
  assert!(!has(&window, bit), "{flag:?} {:#b}", window.state());
  assert_eq!(others(window.state()), others(before), "{flag:?}");
  window.set_flag(flag, on_means_set);
  assert!(has(&window, bit), "{flag:?} {:#b}", window.state());
  assert_eq!(others(window.state()), others(before), "{flag:?}");
}

fn flag_decorations() {
  check_flag(WindowFlag::Decorations, window_state::DECORATED, true);
}

fn flag_on_top() {
  init();
  let window = Window::new(1, false);
  window.set_flag(WindowFlag::AlwaysOnTop, true);
  assert!(has(&window, window_state::ON_TOP));
  window.set_flag(WindowFlag::AlwaysOnTop, false);
  assert!(!has(&window, window_state::ON_TOP));
}

fn flag_closable() {
  check_flag(WindowFlag::Closable, window_state::CLOSABLE, true);
}

fn flag_minimizable() {
  check_flag(WindowFlag::Minimizable, window_state::MINIMIZABLE, true);
}

fn flag_maximizable() {
  check_flag(WindowFlag::Maximizable, window_state::MAXIMIZABLE, true);
}

fn flags_keep_visible() {
  init();
  let window = shown(1, 200, 100);
  for flag in [
    WindowFlag::Decorations,
    WindowFlag::AlwaysOnTop,
    WindowFlag::AlwaysOnBottom,
    WindowFlag::Focusable,
    WindowFlag::Closable,
    WindowFlag::Minimizable,
    WindowFlag::Maximizable,
    WindowFlag::IgnoreCursorEvents,
    WindowFlag::SkipTaskbar,
    WindowFlag::AllWorkspaces,
  ] {
    for on in [false, true, false, true] {
      window.set_flag(flag, on);
      assert!(has(&window, window_state::VISIBLE), "{flag:?} = {on}");
    }
  }
  pump(Duration::from_millis(20));
  // Setting the flags again doesn't change what the defaults left on.
  window.set_flag(WindowFlag::AlwaysOnTop, false);
  assert!(has(
    &window,
    window_state::DECORATED | window_state::CLOSABLE
  ));
}

fn all_workspaces_x11() {
  init();
  let window = Window::new(1, false);
  window.resize(200, 100);
  let Some(app::NativeHandle::X11(id)) = window.native_handle() else {
    return; // not X11 (offscreen, Wayland): a no-op
  };
  fn xprop(args: &[&str]) -> String {
    let out = std::process::Command::new("xprop")
      .args(args)
      .output()
      .expect("xprop");
    String::from_utf8_lossy(&out.stdout).into_owned()
  }
  let props = move || {
    let id = id.to_string();
    let state = xprop(&["-id", &id, "_NET_WM_STATE"]).contains("_NET_WM_STATE_STICKY");
    let all = xprop(&["-id", &id, "_NET_WM_DESKTOP"]).contains("4294967295");
    (state, all)
  };
  // Unmapped: the properties the window manager reads when it manages the window.
  window.set_flag(WindowFlag::AllWorkspaces, true);
  assert_eq!(props(), (true, true));
  window.set_flag(WindowFlag::AllWorkspaces, false);
  assert_eq!(props(), (false, false));
  window.set_flag(WindowFlag::AllWorkspaces, true);

  // Mapped, the window manager owns both. KWin doesn't implement _NET_WM_STATE_STICKY and,
  // with a single desktop, reports "all desktops" as desktop 0: nothing to observe then.
  let root = xprop(&["-root", "_NET_NUMBER_OF_DESKTOPS", "_NET_SUPPORTED"]);
  let desktops: u32 = root
    .lines()
    .find_map(|l| l.strip_prefix("_NET_NUMBER_OF_DESKTOPS(CARDINAL) = "))
    .and_then(|n| n.trim().parse().ok())
    .unwrap_or(1);
  let wm_sticky = root.contains("_NET_WM_STATE_STICKY");
  if desktops < 2 && !wm_sticky {
    eprintln!("skipping the window-manager half: one desktop and no _NET_WM_STATE_STICKY");
    return;
  }
  let sticky = move || {
    let (state, all) = props();
    (!wm_sticky || state) && (desktops < 2 || all)
  };
  window.set_visible(true);
  wait_for("sticky after show", sticky);
  // While mapped: EWMH client messages to the window manager.
  window.set_flag(WindowFlag::AllWorkspaces, false);
  wait_for("not sticky", move || !sticky());
  window.set_flag(WindowFlag::AllWorkspaces, true);
  wait_for("sticky again", sticky);
  // Hidden and shown again (the window manager drops the state on unmap).
  window.set_visible(false);
  pump(Duration::from_millis(100));
  window.set_visible(true);
  wait_for("sticky after re-show", sticky);
}

fn enabled() {
  init();
  let window = Window::new(1, false);
  window.set_enabled(false);
  assert!(!has(&window, window_state::ENABLED));
  window.set_enabled(true);
  assert!(has(&window, window_state::ENABLED));
}

// --- appearance and input ---------------------------------------------------------------------

fn icon_valid() {
  init();
  let window = shown(1, 200, 100);
  let rgba: Vec<u8> = (0..16 * 16).flat_map(|i| [i as u8, 0, 255, 255]).collect();
  window.set_icon(&rgba, 16, 16);
  // A larger buffer than needed is fine too.
  window.set_icon(&[0; 1024], 4, 4);
  pump(Duration::from_millis(20));
}

fn icon_short() {
  init();
  let window = Window::new(1, false);
  window.set_icon(&[0; 15], 2, 2);
  window.set_icon(&[], 16, 16);
  window.set_icon(&[], 0, 0);
  window.set_icon(&[0; 4], 1, 0);
}

// Regression: `width * height * 4` in `Window::set_icon` used to be computed in u32: a debug build
// panicked, and a release build wrapped it to 0 and let Qt read past the buffer.
fn icon_overflow() {
  init();
  let window = Window::new(1, false);
  window.set_icon(&[0; 4], 0x4000_0000, 1);
  window.set_icon(&[0; 4], 0x8000_0000, 2);
  window.set_icon(&[0; 4], u32::MAX, u32::MAX);
}

fn background() {
  init();
  let window = shown(1, 200, 100);
  window.set_background(Some((10, 20, 30, 255)));
  pump(Duration::from_millis(20));
  window.set_background(Some((10, 20, 30, 0)));
  window.set_background(None);
  pump(Duration::from_millis(20));
  let transparent = Window::new(2, true);
  transparent.set_background(Some((0, 0, 0, 0)));
  transparent.set_visible(true);
  transparent.set_background(None);
  pump(Duration::from_millis(20));
}

fn cursor() {
  init();
  let window = shown(1, 200, 100);
  window.set_cursor(13); // Qt::PointingHandCursor
  window.set_cursor_visible(false);
  window.set_cursor(2); // hidden: remembered, not applied
  window.set_cursor_visible(true);
  window.set_cursor(-5); // not a Qt::CursorShape
  window.set_cursor(10_000);
  window.set_cursor_grab(true);
  window.set_cursor_grab(false);
  window.set_cursor_grab(false);
  window.set_cursor_position(10, 10);
  window.set_cursor_position(-100_000, 100_000);
  pump(Duration::from_millis(20));
}

fn cursor_with_view() {
  init();
  let window = shown(1, 200, 100);
  let view = View::new(1, &window, Profile::get(None, true), ViewOptions::default());
  view.set_fill(true);
  view.load_html("<p>cursor</p>", None);
  let v = Rc::new(view);
  let v2 = v.clone();
  wait_for("the page to load", move || v2.url().starts_with("data:"));
  window.set_cursor_visible(false);
  window.set_cursor_visible(true);
  pump(Duration::from_millis(20));
}

fn start_move_hidden() {
  init();
  let window = Window::new(1, false);
  assert!(!window.start_move());
  assert!(!window.start_resize(1 | 2));
}

fn start_move_shown() {
  init();
  let window = shown(1, 200, 100);
  // Offscreen has no window manager to hand the drag to; just make sure it answers.
  let _ = window.start_move();
  for edges in [0, 1, 2, 4, 8, 1 | 2, 4 | 8, 15, u32::MAX] {
    let _ = window.start_resize(edges);
  }
  pump(Duration::from_millis(20));
}

fn alert() {
  init();
  let window = shown(1, 200, 100);
  window.alert(false);
  window.alert(true);
  let hidden = Window::new(2, false);
  hidden.alert(true);
  pump(Duration::from_millis(20));
}

// --- relations and handles --------------------------------------------------------------------

fn transient_for() {
  init();
  let parent = shown(1, 300, 200);
  let child = Window::new(2, false);
  child.set_transient_for(&parent);
  child.set_visible(true);
  pump(Duration::from_millis(20));
  // The parent can go first.
  drop(parent);
  pump(Duration::from_millis(20));
  assert!(has(&child, window_state::VISIBLE));
}

fn transient_for_self() {
  init();
  let window = Window::new(1, false);
  window.set_transient_for(&window);
  window.set_visible(true);
  pump(Duration::from_millis(20));
}

fn transient_for_widget() {
  init();
  let parent = shown(1, 300, 200);
  let child = Window::new(2, false);
  child.set_transient_for_widget(&parent.widget());
  // A widget inside the parent window works too (its top-level window is used).
  let bar = MenuBar::get_or_create(&parent.widget()).expect("menubar");
  child.set_transient_for_widget(bar.widget());
  child.set_visible(true);
  pump(Duration::from_millis(20));
}

fn transient_for_dead_widget() {
  init();
  let child = Window::new(2, false);
  let widget = {
    let parent = Window::new(1, false);
    parent.widget()
  };
  // Outside the loop the window is deleted right away.
  assert!(!widget.is_alive());
  child.set_transient_for_widget(&widget);
  child.set_visible(true);
  pump(Duration::from_millis(20));
}

fn widget_round_trip() {
  init();
  let window = Window::new(1, false);
  let ptr = window.widget_ptr();
  let found = Widget::from_ptr(ptr.as_ptr()).expect("the window's widget is known to Qt");
  assert_eq!(found, window.widget());
  assert_eq!(found.as_ptr(), Some(ptr));
  assert!(found.is_alive());
  // Two different windows have different widgets.
  let other = Window::new(2, false);
  assert_ne!(other.widget(), window.widget());
}

fn widget_dies_with_window() {
  init();
  let window = Window::new(1, false);
  let widget = window.widget();
  let ptr = window.widget_ptr();
  window.set_visible(true);
  let alive = Rc::new(RefCell::new(Some(window)));
  // Drop it inside the loop, where Qt deletes it later.
  let a = alive.clone();
  app::post(move || drop(a.borrow_mut().take()));
  let w = widget.clone();
  wait_for("the widget to die", move || !w.is_alive());
  assert_eq!(widget.as_ptr(), None);
  assert!(Widget::from_ptr(ptr.as_ptr()).is_none());
}

fn content_size_plain() {
  init();
  let window = shown(1, 320, 240);
  let win = window.clone();
  wait_for("the content to be laid out", move || {
    win.content_size() == (320, 240)
  });
}

fn content_size_menubar() {
  init();
  let window = shown(1, 320, 240);
  let bar = MenuBar::get_or_create(&window.widget()).expect("menubar");
  let _ = bar.add_menu("&File", true, None).expect("menu");
  let win = window.clone();
  wait_for("the menubar to take space", move || {
    let (w, h) = win.content_size();
    w == 320 && h > 0 && h < 240
  });
  // Like a native client area, the inner size is the area below the menubar (where webviews go).
  assert_eq!(size(&window), window.content_size());
  // Removing the menubar gives the space back.
  MenuBar::remove(&window.widget());
  let win = window.clone();
  wait_for("the content to grow back", move || {
    win.content_size() == (320, 240)
  });
}

fn resize_with_menubar() {
  init();
  let window = shown(1, 320, 240);
  let bar = MenuBar::get_or_create(&window.widget()).expect("menubar");
  let _ = bar.add_menu("&File", true, None).expect("menu");
  let win = window.clone();
  wait_for("the menubar to take space", move || {
    win.content_size().1 < 240
  });
  // Showing the bar shrank the content area: that's a resize of the inner area.
  record_events(|_| Reply::default());
  window.resize(400, 300);
  let win = window.clone();
  wait_for("the inner area to be 400x300", move || {
    win.content_size() == (400, 300)
  });
  assert_eq!(size(&window), (400, 300));
  // Hiding the bar keeps the window's size: the inner area grows and Resized reports it.
  clear_events();
  MenuBar::set_visible(&window.widget(), false);
  let win = window.clone();
  wait_for("the bar's space to be given back", move || {
    win.content_size().1 > 300
  });
  let (w, h) = window.content_size();
  let s = window.scale_factor();
  assert!(
    has_event(|e| e.kind == EventKind::WindowResized && e.a == phys(w, s) && e.b == phys(h, s)),
    "no Resized for {w}x{h}: {:?}",
    events()
  );
}

fn frameless_resize_cursor() {
  // Qt::CursorShape values
  const SIZE_HOR: i32 = 6;
  const SIZE_FDIAG: i32 = 8;
  init();
  let window = shown(1, 400, 300);
  window.set_flag(WindowFlag::Decorations, false);
  let view = Rc::new(View::new(
    1,
    &window,
    Profile::get(None, true),
    ViewOptions {
      fill: true,
      ..Default::default()
    },
  ));
  view.load_html("<body style='margin:0'><p>page</p></body>", None);
  let v = view.clone();
  wait_for("the page to render", move || {
    v.url().starts_with("data:") || v.url() == "about:blank" || !v.url().is_empty()
  });
  pump(Duration::from_millis(500));
  let Ok(edge) = window.debug_hover(1, 150) else {
    eprintln!("no QPA headers: skipped");
    return;
  };
  assert_eq!(edge, Some(SIZE_HOR), "left edge");
  assert_eq!(
    window.debug_hover(1, 1).unwrap(),
    Some(SIZE_FDIAG),
    "top-left corner"
  );
  assert_eq!(
    window.debug_hover(200, 150).unwrap(),
    None,
    "middle: the page's cursor"
  );
  // A decorated window resizes through its frame: no cursor of ours.
  window.set_flag(WindowFlag::Decorations, true);
  pump(Duration::from_millis(200));
  assert_eq!(
    window.debug_hover(1, 150).unwrap(),
    None,
    "decorated window"
  );
}

fn native_handle_offscreen() {
  init();
  let window = shown(1, 200, 100);
  assert!(window.native_handle().is_none());
  let hidden = Window::new(2, false);
  assert!(hidden.native_handle().is_none());
}

// --- close requests ---------------------------------------------------------------------------

/// Show and activate a window so `StandardAction::CloseWindow` targets it.
fn active(id: u64) -> Rc<Window> {
  let window = shown(id, 200, 100);
  window.focus();
  let win = window.clone();
  wait_for("the window to be active", move || {
    has(&win, window_state::ACTIVE)
  });
  window
}

fn close_request_blocked() {
  init();
  record_events(|_| Reply::default());
  let window = active(5);
  let widget = window.widget();
  StandardAction::CloseWindow.trigger();
  wait_for("WindowCloseRequested", || {
    has_event(|e| e.kind == EventKind::WindowCloseRequested && e.window == 5)
  });
  pump(Duration::from_millis(50));
  assert!(widget.is_alive());
  assert!(has(&window, window_state::VISIBLE));
  // Exactly one request per close.
  let count = events()
    .iter()
    .filter(|e| e.kind == EventKind::WindowCloseRequested)
    .count();
  assert_eq!(count, 1);
  drop(window);
  assert!(!widget.is_alive());
}

thread_local! {
  static SLOT: RefCell<Option<Window>> = const { RefCell::new(None) };
}

fn drop_in_close_handler() {
  init();
  // `active` returns the only `Rc`: move the window into the slot the handler empties.
  let window = Rc::try_unwrap(active(7)).expect("sole owner");
  let widget = window.widget();
  SLOT.with(|s| *s.borrow_mut() = Some(window));
  record_events(|e| {
    if e.kind == EventKind::WindowCloseRequested && e.window == 7 {
      SLOT.with(|s| drop(s.borrow_mut().take()));
    }
    Reply::default()
  });
  StandardAction::CloseWindow.trigger();
  let w = widget.clone();
  wait_for("the window to be deleted", move || !w.is_alive());
  assert!(SLOT.with(|s| s.borrow().is_none()));
}

fn drop_in_resize_handler() {
  init();
  let window = Rc::try_unwrap(active(8)).expect("sole owner");
  let widget = window.widget();
  SLOT.with(|s| *s.borrow_mut() = Some(window));
  record_events(|e| {
    if e.kind == EventKind::WindowResized && e.window == 8 {
      SLOT.with(|s| drop(s.borrow_mut().take()));
    }
    Reply::default()
  });
  // Maximizing the active window resizes it from inside Qt's event processing, so the handler
  // drops the window inside its own `resizeEvent`.
  StandardAction::Maximize.trigger();
  let w = widget.clone();
  wait_for("the window to be deleted", move || !w.is_alive());
}

// --- edge cases -----------------------------------------------------------------------------

fn transient_cycle() {
  init();
  let a = Window::new(1, false);
  let b = Window::new(2, false);
  a.set_transient_for(&b);
  b.set_transient_for(&a);
  a.set_visible(true);
  b.set_visible(true);
  a.focus();
  pump(Duration::from_millis(50));
}

fn resize_negative() {
  init();
  let window = shown(1, 200, 100);
  window.resize(-5, -10);
  let (w, h) = size(&window);
  assert!(w >= 0 && h >= 0, "{w}x{h}");
  window.resize(0, 0);
  pump(Duration::from_millis(20));
  window.set_min_size(Some((-50, -50)));
  window.set_max_size(Some((-50, -50)));
  window.resize(10, 10);
  assert_eq!(size(&window), (10, 10));
}

fn resize_huge() {
  init();
  let window = shown(1, 200, 100);
  window.resize(i32::MAX, i32::MAX);
  // Qt caps widget sizes at QWIDGETSIZE_MAX.
  let (w, h) = size(&window);
  assert!(w <= 16_777_215 && h <= 16_777_215, "{w}x{h}");
  pump(Duration::from_millis(50));
  window.resize(300, 200);
  assert_eq!(size(&window), (300, 200));
}

fn min_above_max() {
  init();
  let window = Window::new(1, false);
  window.set_max_size(Some((100, 100)));
  window.set_min_size(Some((500, 500)));
  window.resize(300, 300);
  let (w, h) = size(&window);
  assert!(
    (100..=500).contains(&w) && (100..=500).contains(&h),
    "{w}x{h}"
  );
}

fn half_constraints() {
  init();
  let window = Window::new(1, false);
  window.set_min_size(Some((-1, 300)));
  window.set_max_size(Some((400, -1)));
  window.resize(10, 10);
  assert_eq!(size(&window), (10, 300));
  window.resize(1000, 1000);
  assert_eq!(size(&window), (400, 1000));
}

fn move_extreme() {
  init();
  let window = shown(1, 200, 100);
  for (x, y) in [
    (i32::MIN, i32::MIN),
    (i32::MAX, i32::MAX),
    (-100, -100),
    (0, 0),
  ] {
    window.move_to(x, y);
  }
  pump(Duration::from_millis(20));
  let outer = window.geometry().outer;
  assert_eq!((outer.0, outer.1), (0, 0));
}

// --- lifecycle --------------------------------------------------------------------------------

fn many_windows() {
  init();
  for i in 0..200 {
    let window = Window::new(i, i % 2 == 0);
    window.set_title(&format!("window {i}"));
    if i % 3 == 0 {
      window.set_visible(true);
    }
  }
  let kept: Vec<_> = (0..50).map(|i| Window::new(1000 + i, false)).collect();
  for w in &kept {
    w.set_visible(true);
  }
  pump(Duration::from_millis(50));
  drop(kept);
  pump(Duration::from_millis(20));
}

fn many_windows_in_loop() {
  init();
  let done = Rc::new(RefCell::new(0));
  for i in 0..100u64 {
    let done = done.clone();
    app::post(move || {
      let window = Window::new(i, false);
      window.set_visible(true);
      window.resize(100 + i as i32, 100);
      drop(window);
      *done.borrow_mut() += 1;
    });
  }
  let d = done.clone();
  wait_for("every window", move || *d.borrow() == 100);
  pump(Duration::from_millis(50));
}

fn views_inert_after_window_drop() {
  init();
  let window = Window::new(1, false);
  let view = View::new(1, &window, Profile::get(None, true), ViewOptions::default());
  view.set_bounds(1, 2, 30, 40);
  assert_eq!(view.bounds(), (1, 2, 30, 40));
  drop(window);
  // Every method is a no-op returning a default.
  assert!(!view.can_go_back());
  assert!(!view.can_go_forward());
  assert_eq!(view.bounds(), (0, 0, 0, 0));
  assert!(!view.is_devtools_open());
  view.set_bounds(5, 5, 5, 5);
  view.set_fill(true);
  view.set_visible(true);
  view.focus();
  view.load_html("<p>x</p>", None);
  view.load_url("about:blank", None);
  view.reload();
  view.go_back();
  view.go_forward();
  view.set_zoom(2.0);
  view.set_background_color((1, 2, 3, 4));
  view.add_init_script("1", true);
  view.eval("1");
  view.eval_with_callback("1", |_| panic!("a dead view ran a script"));
  view.open_devtools();
  view.close_devtools();
  // Reparenting a dead view into a live window does nothing either.
  let other = Window::new(2, false);
  view.reparent(&other);
  pump(Duration::from_millis(50));
  drop(view);
}

// Regression: `qs_view_url` had no `dead` check and read the freed page after the window was
// destroyed.
fn view_url_after_window_drop() {
  init();
  let window = Window::new(1, false);
  let view = View::new(1, &window, Profile::get(None, true), ViewOptions::default());
  drop(window);
  // Scribble over the freed memory so the use-after-free shows.
  let _noise: Vec<_> = (0..64).map(|i| Window::new(100 + i, false)).collect();
  assert_eq!(view.url(), "");
}

fn view_dropped_first() {
  init();
  let window = shown(1, 200, 100);
  let view = View::new(1, &window, Profile::get(None, true), ViewOptions::default());
  view.set_fill(true);
  drop(view);
  pump(Duration::from_millis(20));
  assert_eq!(window.content_size(), (200, 100));
}

fn use_after_shutdown() {
  init();
  let window = Window::new(1, false);
  app::shutdown();
  let _ = window.title();
}

fn resize_after_shutdown() {
  init();
  let window = Window::new(1, false);
  app::shutdown();
  window.resize(1, 1);
}

fn drop_after_shutdown() {
  init();
  let window = Window::new(1, false);
  window.set_visible(true);
  let view = View::new(1, &window, Profile::get(None, true), ViewOptions::default());
  let widget = window.widget();
  app::shutdown();
  drop(view);
  drop(window);
  drop(widget);
}

fn use_from_other_thread() {
  init();
  // `Window` isn't `Send`; the closest a safe caller can get is creating one off the main thread.
  let result = std::thread::spawn(|| {
    let _ = Window::new(1, false);
  })
  .join();
  if let Err(e) = result {
    std::panic::resume_unwind(e);
  }
}
