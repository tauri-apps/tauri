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

#[allow(clippy::too_many_arguments)]
fn hit_test(
  left: WindowPositions,
  top: WindowPositions,
  right: WindowPositions,
  bottom: WindowPositions,
  cx: WindowPositions,
  cy: WindowPositions,
  border_x: WindowPositions,
  border_y: WindowPositions,
) -> HitTestResult {
  #[rustfmt::skip]
  let result = (LEFT * (cx < left + border_x) as isize)
             | (RIGHT * (cx >= right - border_x) as isize)
             | (TOP * (cy < top + border_y) as isize)
             | (BOTTOM * (cy >= bottom - border_y) as isize);

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

  use windows::core::*;
  use windows::Win32::System::LibraryLoader::*;
  use windows::Win32::UI::WindowsAndMessaging::*;
  use windows::Win32::{Foundation::*, UI::Shell::SetWindowSubclass};
  use windows::Win32::{Graphics::Gdi::*, UI::Shell::DefSubclassProc};

  impl HitTestResult {
    fn to_win32(self) -> i32 {
      match self {
        HitTestResult::Left => HTLEFT as _,
        HitTestResult::Right => HTRIGHT as _,
        HitTestResult::Top => HTTOP as _,
        HitTestResult::Bottom => HTBOTTOM as _,
        HitTestResult::TopLeft => HTTOPLEFT as _,
        HitTestResult::TopRight => HTTOPRIGHT as _,
        HitTestResult::BottomLeft => HTBOTTOMLEFT as _,
        HitTestResult::BottomRight => HTBOTTOMRIGHT as _,
        _ => HTTRANSPARENT,
      }
    }
  }

  const CLASS_NAME: PCWSTR = w!("TAURI_DRAG_RESIZE_BORDERS");
  const WINDOW_NAME: PCWSTR = w!("TAURI_DRAG_RESIZE_WINDOW");

  pub fn attach_resize_handler(hwnd: isize) {
    let parent = HWND(hwnd as _);

    // return early if we already attached
    if unsafe { FindWindowExW(parent, HWND::default(), CLASS_NAME, WINDOW_NAME) }.is_ok() {
      return;
    }

    let class = WNDCLASSEXW {
      cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
      style: WNDCLASS_STYLES::default(),
      lpfnWndProc: Some(drag_resize_window_proc),
      cbClsExtra: 0,
      cbWndExtra: 0,
      hInstance: unsafe { HINSTANCE(GetModuleHandleW(PCWSTR::null()).unwrap_or_default().0) },
      hIcon: HICON::default(),
      hCursor: HCURSOR::default(),
      hbrBackground: HBRUSH::default(),
      lpszMenuName: PCWSTR::null(),
      lpszClassName: CLASS_NAME,
      hIconSm: HICON::default(),
    };

    unsafe { RegisterClassExW(&class) };

    let mut rect = RECT::default();
    unsafe { GetClientRect(parent, &mut rect).unwrap() };
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;

    let Ok(drag_window) = (unsafe {
      CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        CLASS_NAME,
        WINDOW_NAME,
        WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS,
        0,
        0,
        width,
        height,
        parent,
        HMENU::default(),
        GetModuleHandleW(PCWSTR::null()).unwrap_or_default(),
        None,
      )
    }) else {
      return;
    };

    unsafe {
      set_drag_hwnd_rgn(drag_window, width, height);

      let _ = SetWindowPos(
        drag_window,
        HWND_TOP,
        0,
        0,
        0,
        0,
        SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOOWNERZORDER | SWP_NOSIZE,
      );

      let _ = SetWindowSubclass(
        parent,
        Some(subclass_parent),
        (WM_USER + 1) as _,
        drag_window.0 as _,
      );
    }
  }

  unsafe extern "system" fn subclass_parent(
    parent: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _: usize,
    child: usize,
  ) -> LRESULT {
    if msg == WM_SIZE {
      let child = HWND(child as _);

      if is_maximized(parent).unwrap_or(false) {
        let _ = SetWindowPos(
          child,
          HWND_TOP,
          0,
          0,
          0,
          0,
          SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOMOVE,
        );
      } else {
        let mut rect = RECT::default();
        if GetClientRect(parent, &mut rect).is_ok() {
          let width = rect.right - rect.left;
          let height = rect.bottom - rect.top;

          let _ = SetWindowPos(
            child,
            HWND_TOP,
            0,
            0,
            width,
            height,
            SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOMOVE,
          );

          set_drag_hwnd_rgn(child, width, height);
        }
      }
    }

    DefSubclassProc(parent, msg, wparam, lparam)
  }

  unsafe extern "system" fn drag_resize_window_proc(
    child: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {
    match msg {
      WM_NCHITTEST => {
        let Ok(parent) = GetParent(child) else {
          return DefWindowProcW(child, msg, wparam, lparam);
        };
        let style = GetWindowLongPtrW(parent, GWL_STYLE);
        let style = WINDOW_STYLE(style as u32);

        let is_resizable = (style & WS_SIZEBOX).0 != 0;
        if !is_resizable {
          return DefWindowProcW(child, msg, wparam, lparam);
        }

        let mut rect = RECT::default();
        if GetWindowRect(child, &mut rect).is_err() {
          return DefWindowProcW(child, msg, wparam, lparam);
        }

        let (cx, cy) = (GET_X_LPARAM(lparam) as i32, GET_Y_LPARAM(lparam) as i32);

        let padded_border = GetSystemMetrics(SM_CXPADDEDBORDER);
        let border_x = GetSystemMetrics(SM_CXFRAME) + padded_border;
        let border_y = GetSystemMetrics(SM_CYFRAME) + padded_border;

        let res = hit_test(
          rect.left,
          rect.top,
          rect.right,
          rect.bottom,
          cx,
          cy,
          border_x,
          border_y,
        );

        return LRESULT(res.to_win32() as _);
      }

      WM_NCLBUTTONDOWN => {
        let Ok(parent) = GetParent(child) else {
          return DefWindowProcW(child, msg, wparam, lparam);
        };
        let style = GetWindowLongPtrW(parent, GWL_STYLE);
        let style = WINDOW_STYLE(style as u32);

        let is_resizable = (style & WS_SIZEBOX).0 != 0;
        if !is_resizable {
          return DefWindowProcW(child, msg, wparam, lparam);
        }

        let mut rect = RECT::default();
        if GetWindowRect(child, &mut rect).is_err() {
          return DefWindowProcW(child, msg, wparam, lparam);
        }

        let (cx, cy) = (GET_X_LPARAM(lparam) as i32, GET_Y_LPARAM(lparam) as i32);

        let padded_border = GetSystemMetrics(SM_CXPADDEDBORDER);
        let border_x = GetSystemMetrics(SM_CXFRAME) + padded_border;
        let border_y = GetSystemMetrics(SM_CYFRAME) + padded_border;

        let res = hit_test(
          rect.left,
          rect.top,
          rect.right,
          rect.bottom,
          cx,
          cy,
          border_x,
          border_y,
        );

        if res != HitTestResult::NoWhere {
          let points = POINTS {
            x: cx as i16,
            y: cy as i16,
          };

          let _ = PostMessageW(
            parent,
            WM_NCLBUTTONDOWN,
            WPARAM(res.to_win32() as _),
            LPARAM(&points as *const _ as _),
          );
        }

        return LRESULT(0);
      }

      _ => {}
    }

    DefWindowProcW(child, msg, wparam, lparam)
  }

  pub fn detach_resize_handler(hwnd: isize) {
    let hwnd = HWND(hwnd as _);

    let Ok(child) = (unsafe { FindWindowExW(hwnd, HWND::default(), CLASS_NAME, WINDOW_NAME) })
    else {
      return;
    };

    let _ = unsafe { DestroyWindow(child) };
  }

  unsafe fn set_drag_hwnd_rgn(hwnd: HWND, width: i32, height: i32) {
    let padded_border = GetSystemMetrics(SM_CXPADDEDBORDER);
    let border_x = GetSystemMetrics(SM_CXFRAME) + padded_border;
    let border_y = GetSystemMetrics(SM_CYFRAME) + padded_border;

    let hrgn1 = CreateRectRgn(0, 0, width, height);
    let hrgn2 = CreateRectRgn(border_x, border_y, width - border_x, height - border_y);
    CombineRgn(hrgn1, hrgn1, hrgn2, RGN_DIFF);
    SetWindowRgn(hwnd, hrgn1, true);
  }

  fn is_maximized(window: HWND) -> windows::core::Result<bool> {
    let mut placement = WINDOWPLACEMENT {
      length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
      ..WINDOWPLACEMENT::default()
    };
    unsafe { GetWindowPlacement(window, &mut placement)? };
    Ok(placement.showCmd == SW_MAXIMIZE.0 as u32)
  }

  /// Implementation of the `GET_X_LPARAM` macro.
  #[allow(non_snake_case)]
  #[inline]
  fn GET_X_LPARAM(lparam: LPARAM) -> i16 {
    ((lparam.0 as usize) & 0xFFFF) as u16 as i16
  }

  /// Implementation of the `GET_Y_LPARAM` macro.
  #[allow(non_snake_case)]
  #[inline]
  fn GET_Y_LPARAM(lparam: LPARAM) -> i16 {
    (((lparam.0 as usize) & 0xFFFF_0000) >> 16) as u16 as i16
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
                  0.0,
                  0.0,
                  window.width() as f64,
                  window.height() as f64,
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
                    0.0,
                    0.0,
                    window.width() as f64,
                    window.height() as f64,
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
