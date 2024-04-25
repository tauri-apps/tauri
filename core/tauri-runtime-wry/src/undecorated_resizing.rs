// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
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

#[cfg(not(windows))]
pub use self::gtk::*;
#[cfg(windows)]
pub use self::windows::*;

#[cfg(windows)]
type WindowDimensions = u32;
#[cfg(not(windows))]
type WindowDimensions = i32;
#[cfg(windows)]
type WindowPositions = i32;
#[cfg(not(windows))]
type WindowPositions = f64;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

fn hit_test(
  width: WindowDimensions,
  height: WindowDimensions,
  x: WindowPositions,
  y: WindowPositions,
  border_x: WindowPositions,
  border_y: WindowPositions,
) -> HitTestResult {
  #[cfg(windows)]
  let (top, left) = (0, 0);
  #[cfg(not(windows))]
  let (top, left) = (0., 0.);

  let bottom = top + height as WindowPositions;
  let right = left + width as WindowPositions;

  #[rustfmt::skip]
  let result = (LEFT * (x < left + border_x) as isize)
             | (RIGHT * (x >= right - border_x) as isize)
             | (TOP * (y < top + border_y) as isize)
             | (BOTTOM * (y >= bottom - border_y) as isize);

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

#[cfg(windows)]
mod windows {
  use super::{hit_test, HitTestResult};

  use tao::{
    dpi::LogicalPosition,
    window::{CursorIcon, ResizeDirection, Window},
  };
  use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXFRAME, SM_CXPADDEDBORDER, SM_CYFRAME,
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
    if (e.button === 0) {
      window.ipc.postMessage(
        `__internal_on_mousedown__|${e.clientX},${e.clientY}`
      )
    }
  })
})()
"#;

  impl HitTestResult {
    fn drag_resize_window(&self, window: &Window) {
      self.change_cursor(window);
      let edge = match self {
        HitTestResult::Left => ResizeDirection::West,
        HitTestResult::Right => ResizeDirection::East,
        HitTestResult::Top => ResizeDirection::North,
        HitTestResult::Bottom => ResizeDirection::South,
        HitTestResult::TopLeft => ResizeDirection::NorthWest,
        HitTestResult::TopRight => ResizeDirection::NorthEast,
        HitTestResult::BottomLeft => ResizeDirection::SouthWest,
        HitTestResult::BottomRight => ResizeDirection::SouthEast,

        // if not on an edge, don't start resizing
        _ => return,
      };
      let _ = window.drag_resize_window(edge);
    }

    fn change_cursor(&self, window: &Window) {
      let cursor = match self {
        HitTestResult::Left => CursorIcon::WResize,
        HitTestResult::Right => CursorIcon::EResize,
        HitTestResult::Top => CursorIcon::NResize,
        HitTestResult::Bottom => CursorIcon::SResize,
        HitTestResult::TopLeft => CursorIcon::NwResize,
        HitTestResult::TopRight => CursorIcon::NeResize,
        HitTestResult::BottomLeft => CursorIcon::SwResize,
        HitTestResult::BottomRight => CursorIcon::SeResize,

        // if not on an edge, don't change the cursor, otherwise we cause flickering
        _ => return,
      };
      window.set_cursor_icon(cursor);
    }
  }

  // Returns whether handled or not
  pub fn handle_request<T: crate::UserEvent>(
    context: crate::Context<T>,
    window_id: crate::WindowId,
    request: &http::Request<String>,
  ) -> bool {
    if let Some(args) = request.body().strip_prefix(MESSAGE_MOUSEMOVE) {
      if let Some(window) = context.main_thread.windows.0.borrow().get(&window_id) {
        if let Some(w) = window.inner.as_ref() {
          if !w.is_decorated()
            && w.is_resizable()
            && !w.is_maximized()
            && !window.is_window_fullscreen
          {
            let (x, y) = args.split_once(',').unwrap();
            let (x, y): (i32, i32) = (x.parse().unwrap(), y.parse().unwrap());
            let postion = LogicalPosition::new(x, y).to_physical::<i32>(w.scale_factor());
            let size = w.inner_size();
            let padded_border = unsafe { GetSystemMetrics(SM_CXPADDEDBORDER) };
            let border_x = unsafe { GetSystemMetrics(SM_CXFRAME) + padded_border };
            let border_y = unsafe { GetSystemMetrics(SM_CYFRAME) + padded_border };
            hit_test(
              size.width,
              size.height,
              postion.x,
              postion.y,
              border_x,
              border_y,
            )
            .change_cursor(w);
          }
        }
      }

      return true;
    }
    if let Some(args) = request.body().strip_prefix(MESSAGE_MOUSEDOWN) {
      if let Some(window) = context.main_thread.windows.0.borrow().get(&window_id) {
        if let Some(w) = window.inner.as_ref() {
          if !w.is_decorated()
            && w.is_resizable()
            && !w.is_maximized()
            && !window.is_window_fullscreen
          {
            let (x, y) = args.split_once(',').unwrap();
            let (x, y): (i32, i32) = (x.parse().unwrap(), y.parse().unwrap());
            let postion = LogicalPosition::new(x, y).to_physical::<i32>(w.scale_factor());
            let size = w.inner_size();
            let padded_border = unsafe { GetSystemMetrics(SM_CXPADDEDBORDER) };
            let border_x = unsafe { GetSystemMetrics(SM_CXFRAME) + padded_border };
            let border_y = unsafe { GetSystemMetrics(SM_CYFRAME) + padded_border };
            hit_test(
              size.width,
              size.height,
              postion.x,
              postion.y,
              border_x,
              border_y,
            )
            .drag_resize_window(w);
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
  use super::{hit_test, HitTestResult};

  const BORDERLESS_RESIZE_INSET: i32 = 5;

  impl HitTestResult {
    fn to_gtk_edge(self) -> gtk::gdk::WindowEdge {
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
      move |webview: &webkit2gtk::WebView, event: &gtk::gdk::EventButton| {
        if event.button() == 1 {
          // This one should be GtkBox
          if let Some(window) = webview.parent().and_then(|w| w.parent()) {
            // Safe to unwrap unless this is not from tao
            let window: gtk::Window = window.downcast().unwrap();
            if !window.is_decorated() && window.is_resizable() && !window.is_maximized() {
              if let Some(window) = window.window() {
                let (root_x, root_y) = event.root();
                let (window_x, window_y) = window.position();
                let (client_x, client_y) = (root_x - window_x as f64, root_y - window_y as f64);
                let border = window.scale_factor() * BORDERLESS_RESIZE_INSET;
                let edge = hit_test(
                  window.width(),
                  window.height(),
                  client_x,
                  client_y,
                  border as _,
                  border as _,
                )
                .to_gtk_edge();

                // we ignore the `__Unknown` variant so the webview receives the click correctly if it is not on the edges.
                match edge {
                  WindowEdge::__Unknown(_) => (),
                  _ => {
                    window.begin_resize_drag(edge, 1, root_x as i32, root_y as i32, event.time())
                  }
                }
              }
            }
          }
        }

        Propagation::Proceed
      },
    );

    webview.connect_touch_event(
      move |webview: &webkit2gtk::WebView, event: &gtk::gdk::Event| {
        // This one should be GtkBox
        if let Some(window) = webview.parent().and_then(|w| w.parent()) {
          // Safe to unwrap unless this is not from tao
          let window: gtk::Window = window.downcast().unwrap();
          if !window.is_decorated() && window.is_resizable() && !window.is_maximized() {
            if let Some(window) = window.window() {
              if let Some((root_x, root_y)) = event.root_coords() {
                if let Some(device) = event.device() {
                  let (window_x, window_y) = window.position();
                  let (client_x, client_y) = (root_x - window_x as f64, root_y - window_y as f64);
                  let border = window.scale_factor() * BORDERLESS_RESIZE_INSET;
                  let edge = hit_test(
                    window.width(),
                    window.height(),
                    client_x,
                    client_y,
                    border as _,
                    border as _,
                  )
                  .to_gtk_edge();

                  // we ignore the `__Unknown` variant so the window receives the click correctly if it is not on the edges.
                  match edge {
                    WindowEdge::__Unknown(_) => (),
                    _ => window.begin_resize_drag_for_device(
                      edge,
                      &device,
                      0,
                      root_x as i32,
                      root_y as i32,
                      event.time(),
                    ),
                  }
                }
              }
            }
          }
        }

        Propagation::Proceed
      },
    );
  }
}
