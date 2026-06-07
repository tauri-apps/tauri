//! Windows implementation of the platform window helpers.

#![allow(clippy::missing_safety_doc)]

use cef::ImplWindow;
use tauri_runtime::dpi::{PhysicalPosition, Position};
use tauri_runtime::window::CursorIcon;
use tauri_runtime::{ProgressBarState, ResizeDirection, UserAttentionType};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::System::Com::{CLSCTX_SERVER, CoCreateInstance};
use windows::Win32::UI::Controls::MARGINS;
use windows::Win32::UI::Input::KeyboardAndMouse::{
  EnableWindow, GetActiveWindow, IsWindowEnabled, ReleaseCapture,
};
use windows::Win32::UI::Shell::{ITaskbarList, ITaskbarList3, TaskbarList};
use windows::Win32::UI::WindowsAndMessaging::*;

fn hwnd(window: &cef::Window) -> HWND {
  HWND(window.window_handle().0 as _)
}

pub fn set_skip_taskbar(window: &cef::Window, skip: bool) {
  let hwnd = hwnd(window);
  unsafe {
    if let Ok(taskbar) = CoCreateInstance::<_, ITaskbarList>(&TaskbarList, None, CLSCTX_SERVER) {
      if skip {
        let _ = taskbar.DeleteTab(hwnd);
      } else {
        let _ = taskbar.AddTab(hwnd);
      }
    }
  }
}

pub fn set_always_on_bottom(window: &cef::Window, on_bottom: bool) {
  let hwnd = hwnd(window);
  unsafe {
    let insert_after = if on_bottom {
      HWND_BOTTOM
    } else {
      HWND_NOTOPMOST
    };
    let _ = SetWindowPos(
      hwnd,
      Some(insert_after),
      0,
      0,
      0,
      0,
      SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
    );
  }
}

/// Showing a window on all virtual desktops is not supported on Windows.
pub fn set_visible_on_all_workspaces(_window: &cef::Window, _visible: bool) {}

pub fn set_shadow(window: &cef::Window, enable: bool) {
  let hwnd = hwnd(window);
  // For borderless windows the shadow is controlled by extending the DWM frame
  // by a 1px margin (enable) or zero margins (disable).
  let margins = if enable {
    MARGINS {
      cxLeftWidth: 1,
      cxRightWidth: 1,
      cyTopHeight: 1,
      cyBottomHeight: 1,
    }
  } else {
    MARGINS::default()
  };
  unsafe {
    let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);
  }
}

pub fn request_user_attention(window: &cef::Window, request_type: Option<UserAttentionType>) {
  let hwnd = hwnd(window);
  // Skip requesting attention if the window is already active and not minimized
  // (matching `tao`).
  unsafe {
    if GetActiveWindow() == hwnd && !IsIconic(hwnd).as_bool() {
      return;
    }
  }
  let (flags, count) = match request_type {
    Some(UserAttentionType::Critical) => (FLASHW_ALL | FLASHW_TIMERNOFG, u32::MAX),
    Some(UserAttentionType::Informational) => (FLASHW_TRAY, 4),
    None => (FLASHW_STOP, 0),
  };
  let info = FLASHWINFO {
    cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
    hwnd,
    dwFlags: flags,
    uCount: count,
    dwTimeout: 0,
  };
  unsafe {
    let _ = FlashWindowEx(&info);
  }
}

pub fn global_cursor_position() -> Option<PhysicalPosition<f64>> {
  let mut point = POINT::default();
  unsafe {
    GetCursorPos(&mut point).ok()?;
  }
  Some(PhysicalPosition::new(point.x as f64, point.y as f64))
}

pub fn set_cursor_position(window: &cef::Window, position: Position, scale_factor: f64) {
  // The position is relative to the window's client area (like `tao`); convert
  // it to screen coordinates before moving the cursor.
  let hwnd = hwnd(window);
  let physical = position.to_physical::<i32>(scale_factor);
  let mut point = POINT {
    x: physical.x,
    y: physical.y,
  };
  unsafe {
    if ClientToScreen(hwnd, &mut point).as_bool() {
      let _ = SetCursorPos(point.x, point.y);
    }
  }
}

pub fn set_cursor_visible(_window: &cef::Window, visible: bool) {
  unsafe {
    ShowCursor(visible);
  }
}

pub fn set_cursor_grab(window: &cef::Window, grab: bool) {
  let hwnd = hwnd(window);
  unsafe {
    if grab {
      let mut rect = RECT::default();
      if GetClientRect(hwnd, &mut rect).is_ok() {
        let mut top_left = POINT {
          x: rect.left,
          y: rect.top,
        };
        let mut bottom_right = POINT {
          x: rect.right,
          y: rect.bottom,
        };
        let _ = ClientToScreen(hwnd, &mut top_left);
        let _ = ClientToScreen(hwnd, &mut bottom_right);
        let clip = RECT {
          left: top_left.x,
          top: top_left.y,
          right: bottom_right.x,
          bottom: bottom_right.y,
        };
        let _ = ClipCursor(Some(&clip as *const RECT));
      }
    } else {
      let _ = ClipCursor(None);
    }
  }
}

pub fn set_ignore_cursor_events(window: &cef::Window, ignore: bool) {
  let hwnd = hwnd(window);
  unsafe {
    let mut ex_style = WINDOW_EX_STYLE(GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32);
    if ignore {
      ex_style |= WS_EX_TRANSPARENT | WS_EX_LAYERED;
    } else {
      ex_style &= !(WS_EX_TRANSPARENT | WS_EX_LAYERED);
    }
    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style.0 as isize);
  }
}

pub fn set_cursor_icon(window: &cef::Window, icon: CursorIcon) {
  let hwnd = hwnd(window);
  unsafe {
    if let Ok(cursor) = LoadCursorW(None, cursor_icon_resource(icon)) {
      SetClassLongPtrW(hwnd, GCLP_HCURSOR, cursor.0 as isize);
      let _ = SetCursor(Some(cursor));
    }
  }
}

fn cursor_icon_resource(icon: CursorIcon) -> windows::core::PCWSTR {
  use CursorIcon::*;
  match icon {
    Crosshair => IDC_CROSS,
    Hand | Grab | Grabbing => IDC_HAND,
    Text | VerticalText => IDC_IBEAM,
    Wait => IDC_WAIT,
    Progress => IDC_APPSTARTING,
    Help => IDC_HELP,
    Move | AllScroll => IDC_SIZEALL,
    NotAllowed | NoDrop => IDC_NO,
    EResize | WResize | EwResize | ColResize => IDC_SIZEWE,
    NResize | SResize | NsResize | RowResize => IDC_SIZENS,
    NeResize | SwResize | NeswResize => IDC_SIZENESW,
    NwResize | SeResize | NwseResize => IDC_SIZENWSE,
    _ => IDC_ARROW,
  }
}

pub fn set_progress_bar(window: &cef::Window, state: ProgressBarState) {
  use tauri_runtime::ProgressBarStatus;
  use windows::Win32::UI::Shell::{
    TBPF_ERROR, TBPF_INDETERMINATE, TBPF_NOPROGRESS, TBPF_NORMAL, TBPF_PAUSED,
  };

  let hwnd = hwnd(window);
  unsafe {
    let Ok(taskbar) = CoCreateInstance::<_, ITaskbarList3>(&TaskbarList, None, CLSCTX_SERVER)
    else {
      return;
    };
    if let Some(status) = state.status {
      let flag = match status {
        ProgressBarStatus::None => TBPF_NOPROGRESS,
        ProgressBarStatus::Normal => TBPF_NORMAL,
        ProgressBarStatus::Indeterminate => TBPF_INDETERMINATE,
        ProgressBarStatus::Paused => TBPF_PAUSED,
        ProgressBarStatus::Error => TBPF_ERROR,
      };
      let _ = taskbar.SetProgressState(hwnd, flag);
    }
    if let Some(progress) = state.progress {
      let _ = taskbar.SetProgressValue(hwnd, progress.min(100), 100);
    }
  }
}

/// Application badges are not supported on Windows (matching `tao`); use an
/// overlay icon instead.
pub fn set_badge_count(
  _window: &cef::Window,
  _count: Option<i64>,
  _desktop_filename: Option<String>,
) {
}

/// Enables or disables all user interaction with the window.
///
/// `EnableWindow` disables the window and, crucially, all of its child windows
/// — including the child HWND that hosts the CEF browser — so input is blocked
/// for the whole window. CEF's `View::set_enabled` only covers the root view
/// and leaves the web contents focusable.
pub fn set_enabled(window: &cef::Window, enabled: bool) {
  let _ = unsafe { EnableWindow(hwnd(window), enabled) };
}

/// Reports whether the window is enabled.
pub fn is_enabled(window: &cef::Window) -> bool {
  unsafe { IsWindowEnabled(hwnd(window)) }.as_bool()
}

pub fn start_resize_dragging(window: &cef::Window, direction: ResizeDirection) {
  let hwnd = hwnd(window);
  // Hit-test codes for WM_NCLBUTTONDOWN.
  const HTLEFT: isize = 10;
  const HTRIGHT: isize = 11;
  const HTTOP: isize = 12;
  const HTTOPLEFT: isize = 13;
  const HTTOPRIGHT: isize = 14;
  const HTBOTTOM: isize = 15;
  const HTBOTTOMLEFT: isize = 16;
  const HTBOTTOMRIGHT: isize = 17;
  let hittest = match direction {
    ResizeDirection::East => HTRIGHT,
    ResizeDirection::West => HTLEFT,
    ResizeDirection::North => HTTOP,
    ResizeDirection::South => HTBOTTOM,
    ResizeDirection::NorthEast => HTTOPRIGHT,
    ResizeDirection::NorthWest => HTTOPLEFT,
    ResizeDirection::SouthEast => HTBOTTOMRIGHT,
    ResizeDirection::SouthWest => HTBOTTOMLEFT,
  };
  unsafe {
    let _ = ReleaseCapture();
    let _ = PostMessageW(
      Some(hwnd),
      WM_NCLBUTTONDOWN,
      WPARAM(hittest as usize),
      LPARAM(0),
    );
  }
}
