// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(windows)]
pub mod windows {
  use cef::*;
  use windows::Win32::Foundation::*;
  use windows::Win32::UI::WindowsAndMessaging::*;
  use windows::core::{PCWSTR, w};

  /// Same as [WNDPROC] but without the Option wrapper.
  type WindowProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

  const ORIGINAL_WND_PROP: PCWSTR = w!("TAURI_CEF_ORIGINAL_WND_PROC");

  /// Subclasses the given window to handle draggable regions
  /// by replacing its window procedure with `root_window_proc`
  /// and storing the original procedure as a property to be called later.
  pub fn subclass_window_for_dragging(window: &mut cef::Window) {
    let hwnd = window.window_handle();
    let hwnd = HWND(hwnd.0 as _);
    subclass_window(hwnd, root_window_proc);
  }

  /// Subclasses a window by replacing its window procedure with the given `proc`
  /// and storing the original procedure as a property for later use.
  fn subclass_window(hwnd: HWND, proc: WindowProc) {
    // If already subclassed, return early
    let orginial_wnd_proc = unsafe { GetPropW(hwnd, ORIGINAL_WND_PROP) };
    if !orginial_wnd_proc.is_invalid() {
      return;
    }

    // Reset last error
    unsafe { SetLastError(ERROR_SUCCESS) };

    // Set the new window procedure and get the orginal one
    let original_wnd_proc = unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, proc as isize) };
    if original_wnd_proc == 0 && unsafe { GetLastError() } != ERROR_SUCCESS {
      return;
    }

    unsafe {
      // Store the original window proc as a property for later use
      let _ = SetPropW(
        hwnd,
        ORIGINAL_WND_PROP,
        Some(HANDLE(original_wnd_proc as _)),
      );
    }
  }

  /// The root window procedure to handle WM_NCLBUTTONDOWN
  /// by calling DefWindowProcW directly to allow dragging
  /// and forwarding other messages to the original CEF window procedure.
  unsafe extern "system" fn root_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {
    if msg == WM_NCLBUTTONDOWN {
      return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    // For other messages, call the original CEF window procedure
    let original_wnd_proc = GetPropW(hwnd, ORIGINAL_WND_PROP);
    let original_wnd_proc = std::mem::transmute::<_, WindowProc>(original_wnd_proc.0);
    CallWindowProcW(Some(original_wnd_proc), hwnd, msg, wparam, lparam)
  }
}
