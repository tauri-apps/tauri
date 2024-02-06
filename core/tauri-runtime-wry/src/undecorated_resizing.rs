// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(any(
  windows,
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]

const CLIENT: isize = 0b0000;
const LEFT: isize = 0b0001;
const RIGHT: isize = 0b0010;
const TOP: isize = 0b0100;
const BOTTOM: isize = 0b1000;
const TOPLEFT: isize = TOP | LEFT;
const TOPRIGHT: isize = TOP | RIGHT;
const BOTTOMLEFT: isize = BOTTOM | LEFT;
const BOTTOMRIGHT: isize = BOTTOM | RIGHT;

const BORDERLESS_RESIZE_INSET: f64 = 5.0;

#[cfg(windows)]
pub use self::windows::*;
#[cfg(not(windows))]
pub use gtk::*;

#[derive(Debug, PartialEq, Eq)]
enum HitTestResult {
  Client,
  Left,
  Right,
  Top,
  Bottom,
  TopLeft,
  TopRight,
  BottomLeft,
  BottomRight,
  NoWhere,
}

#[cfg(windows)]
mod windows {
  use super::{
    HitTestResult, BORDERLESS_RESIZE_INSET, BOTTOM, BOTTOMLEFT, BOTTOMRIGHT, CLIENT, LEFT, RIGHT,
    TOP, TOPLEFT, TOPRIGHT,
  };

  use tao::{
    dpi::PhysicalSize,
    window::{CursorIcon, ResizeDirection, Window},
  };

  const MESSAGE_MOUSEMOVE: &str = "__internal_on_mousemove__|";
  const MESSAGE_MOUSEDOWN: &str = "__internal_on_mousedown__|";
  pub const SCRIPT: &str = r#"
;(function () {
  document.addEventListener('mousemove', (e) => {
    window.ipc.postMessage(
      `__internal_on_mousemove__|${e.clientX},${e.clientY}`
    )
  })
  document.addEventListener('mousedown', (e) => {
    window.ipc.postMessage(
      `__internal_on_mousedown__|${e.clientX},${e.clientY}`
    )
  })
})()
"#;

  impl HitTestResult {
    fn drag_resize_window(&self, window: &Window) {
      let _ = window.drag_resize_window(match self {
        HitTestResult::Left => ResizeDirection::West,
        HitTestResult::Right => ResizeDirection::East,
        HitTestResult::Top => ResizeDirection::North,
        HitTestResult::Bottom => ResizeDirection::South,
        HitTestResult::TopLeft => ResizeDirection::NorthWest,
        HitTestResult::TopRight => ResizeDirection::NorthEast,
        HitTestResult::BottomLeft => ResizeDirection::SouthWest,
        HitTestResult::BottomRight => ResizeDirection::SouthEast,
        _ => unreachable!(),
      });
    }

    fn change_cursor(&self, window: &Window) {
      let _ = window.set_cursor_icon(match self {
        HitTestResult::Left => CursorIcon::WResize,
        HitTestResult::Right => CursorIcon::EResize,
        HitTestResult::Top => CursorIcon::NResize,
        HitTestResult::Bottom => CursorIcon::SResize,
        HitTestResult::TopLeft => CursorIcon::NwResize,
        HitTestResult::TopRight => CursorIcon::NeResize,
        HitTestResult::BottomLeft => CursorIcon::SwResize,
        HitTestResult::BottomRight => CursorIcon::SeResize,
        _ => CursorIcon::Default,
      });
    }
  }

  fn hit_test(window_size: PhysicalSize<u32>, x: i32, y: i32, scale: f64) -> HitTestResult {
    let (width, height) = (window_size.width, window_size.height);

    let top = 0;
    let left = 0;
    let bottom = top + height as i32;
    let right = left + width as i32;

    let inset = (BORDERLESS_RESIZE_INSET * scale) as i32;

    #[rustfmt::skip]
      let result =
          (LEFT * (if x < (left + inset) { 1 } else { 0 }))
        | (RIGHT * (if x >= (right - inset) { 1 } else { 0 }))
        | (TOP * (if y < (top + inset) { 1 } else { 0 }))
        | (BOTTOM * (if y >= (bottom - inset) { 1 } else { 0 }));

    match result {
      CLIENT => HitTestResult::Client,
      LEFT => HitTestResult::Left,
      RIGHT => HitTestResult::Right,
      TOP => HitTestResult::Top,
      BOTTOM => HitTestResult::Bottom,
      TOPLEFT => HitTestResult::TopLeft,
      TOPRIGHT => HitTestResult::TopRight,
      BOTTOMLEFT => HitTestResult::BottomLeft,
      BOTTOMRIGHT => HitTestResult::BottomRight,
      _ => HitTestResult::NoWhere,
    }
  }

  // Returns whether handled or not
  pub fn handle_request<T: crate::UserEvent>(
    context: crate::Context<T>,
    window_id: crate::WindowId,
    request: &str,
  ) -> bool {
    if let Some(args) = request.strip_prefix(MESSAGE_MOUSEMOVE) {
      if let Some(window) = context
        .main_thread
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|w| w.inner.as_ref())
      {
        if !window.is_decorated() && window.is_resizable() && !window.is_maximized() {
          let (x, y) = args.split_once(',').unwrap();
          let (x, y) = (x.parse().unwrap(), y.parse().unwrap());
          hit_test(window.inner_size(), x, y, window.scale_factor()).change_cursor(&window);
        }
      }

      return true;
    } else if let Some(args) = request.strip_prefix(MESSAGE_MOUSEDOWN) {
      if let Some(window) = context
        .main_thread
        .windows
        .borrow()
        .get(&window_id)
        .and_then(|w| w.inner.as_ref())
      {
        if !window.is_decorated() && window.is_resizable() && !window.is_maximized() {
          let (x, y) = args.split_once(',').unwrap();
          let (x, y) = (x.parse().unwrap(), y.parse().unwrap());
          let res = hit_test(window.inner_size(), x, y, window.scale_factor());
          match res {
            HitTestResult::Client | HitTestResult::NoWhere => {}
            res => res.drag_resize_window(&window),
          }
        }
      }

      return true;
    }

    false
  }
}

#[cfg(not(windows))]
mod gtk {
  use super::{
    HitTestResult, BORDERLESS_RESIZE_INSET, BOTTOM, BOTTOMLEFT, BOTTOMRIGHT, CLIENT, LEFT, RIGHT,
    TOP, TOPLEFT, TOPRIGHT,
  };

  impl HitTestResult {
    fn to_gtk_edge(&self) -> gtk::gdk::WindowEdge {
      match self {
        HitTestResult::Client | HitTestResult::NoWhere => gtk::gdk::WindowEdge::__Unknown(0),
        HitTestResult::Left => gtk::gdk::WindowEdge::West,
        HitTestResult::Right => gtk::gdk::WindowEdge::East,
        HitTestResult::Top => gtk::gdk::WindowEdge::North,
        HitTestResult::Bottom => gtk::gdk::WindowEdge::South,
        HitTestResult::TopLeft => gtk::gdk::WindowEdge::NorthWest,
        HitTestResult::TopRight => gtk::gdk::WindowEdge::NorthEast,
        HitTestResult::BottomLeft => gtk::gdk::WindowEdge::SouthWest,
        HitTestResult::BottomRight => gtk::gdk::WindowEdge::SouthEast,
      }
    }
  }

  fn hit_test(window_size: (i32, i32), x: f64, y: f64, scale: i32) -> HitTestResult {
    let (width, height) = (window_size.0, window_size.0);

    let top = 0.0;
    let left = 0.0;
    let bottom = top + height as f64;
    let right = left + width as f64;

    let inset = BORDERLESS_RESIZE_INSET * scale as f64;

    #[rustfmt::skip]
      let result =
          (LEFT * (if x < (left + inset) { 1 } else { 0 }))
        | (RIGHT * (if x >= (right - inset) { 1 } else { 0 }))
        | (TOP * (if y < (top + inset) { 1 } else { 0 }))
        | (BOTTOM * (if y >= (bottom - inset) { 1 } else { 0 }));

    match result {
      CLIENT => HitTestResult::Client,
      LEFT => HitTestResult::Left,
      RIGHT => HitTestResult::Right,
      TOP => HitTestResult::Top,
      BOTTOM => HitTestResult::Bottom,
      TOPLEFT => HitTestResult::TopLeft,
      TOPRIGHT => HitTestResult::TopRight,
      BOTTOMLEFT => HitTestResult::BottomLeft,
      BOTTOMRIGHT => HitTestResult::BottomRight,
      _ => HitTestResult::NoWhere,
    }
  }

  pub fn attach_resize_handler(webview: &wry::WebView) {
    use gtk::{
      gdk::{prelude::*, WindowEdge},
      glib::Propagation,
      prelude::*,
    };
    use wry::WebViewExtUnix;

    let webview = webview.webview();
    webview.add_events(
      gtk::gdk::EventMask::BUTTON1_MOTION_MASK
        | gtk::gdk::EventMask::BUTTON_PRESS_MASK
        | gtk::gdk::EventMask::TOUCH_MASK,
    );
    webview.connect_button_press_event(
      |webview: &webkit2gtk::WebView, event: &gtk::gdk::EventButton| {
        if event.button() == 1 {
          // This one should be GtkBox
          if let Some(widget) = webview.parent() {
            // This one should be GtkWindow
            if let Some(window) = widget.parent() {
              // Safe to unwrap unless this is not from tao
              let window: gtk::Window = window.downcast().unwrap();
              if !window.is_decorated() && window.is_resizable() && !window.is_maximized() {
                if let Some(window) = window.window() {
                  let (cx, cy) = event.root();
                  let edge = hit_test(
                    (window.width(), window.height()),
                    cx,
                    cy,
                    window.scale_factor(),
                  )
                  .to_gtk_edge();

                  // we ignore the `__Unknown` variant so the webview receives the click correctly if it is not on the edges.
                  match edge {
                    WindowEdge::__Unknown(_) => (),
                    _ => window.begin_resize_drag(edge, 1, cx as i32, cy as i32, event.time()),
                  }
                }
              }
            }
          }
        }

        Propagation::Proceed
      },
    );

    webview.connect_touch_event(|webview: &webkit2gtk::WebView, event: &gtk::gdk::Event| {
      // This one should be GtkBox
      if let Some(widget) = webview.parent() {
        // This one should be GtkWindow
        if let Some(window) = widget.parent() {
          // Safe to unwrap unless this is not from tao
          let window: gtk::Window = window.downcast().unwrap();
          if !window.is_decorated() && window.is_resizable() && !window.is_maximized() {
            if let Some(window) = window.window() {
              if let Some((cx, cy)) = event.root_coords() {
                if let Some(device) = event.device() {
                  let edge = hit_test(
                    (window.width(), window.height()),
                    cx,
                    cy,
                    window.scale_factor(),
                  )
                  .to_gtk_edge();

                  // we ignore the `__Unknown` variant so the window receives the click correctly if it is not on the edges.
                  match edge {
                    WindowEdge::__Unknown(_) => (),
                    _ => window.begin_resize_drag_for_device(
                      edge,
                      &device,
                      0,
                      cx as i32,
                      cy as i32,
                      event.time(),
                    ),
                  }
                }
              }
            }
          }
        }
      }

      Propagation::Proceed
    });
  }
}
