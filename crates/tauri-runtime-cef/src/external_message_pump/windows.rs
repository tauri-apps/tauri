// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Windows backend for the CEF external message pump.
//!
//! Mirrors cefclient's `main_message_loop_external_pump_win.cc`: a message-only
//! window owns a `WM_TIMER`, and `OnScheduleMessagePumpWork` posts a private
//! `WM_HAVE_WORK` message to it. winit runs the thread's own
//! `GetMessage`/`DispatchMessage` loop, which delivers both messages to our
//! window procedure — so CEF is pumped from the same loop, including while
//! Windows runs a modal move/resize loop that winit's `ApplicationHandler`
//! callbacks never observe.
//!
//! Reference:
//! <https://github.com/chromiumembedded/cef/blob/b2d312cd48fe0195f9736fd7c761a89abd5bf2be/tests/shared/browser/main_message_loop_external_pump_win.cc>

use std::sync::Weak;

use windows::{
  Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, GWLP_USERDATA, GetWindowLongPtrW,
      HWND_MESSAGE, KillTimer, PostMessageW, RegisterClassExW, SetTimer, SetWindowLongPtrW,
      WINDOW_EX_STYLE, WM_TIMER, WM_USER, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
    },
  },
  core::{PCWSTR, w},
};

use super::PumpState;

/// Window class for the pump's message-only window.
const WINDOW_CLASS: PCWSTR = w!("TauriCefExternalMessagePump");
/// Private message posted by `OnScheduleMessagePumpWork`; matches cefclient's
/// `kMsgHaveWork` (`WM_USER + 1`). The `LPARAM` carries the delay in ms.
const WM_HAVE_WORK: u32 = WM_USER + 1;
/// Timer id used with `SetTimer`/`KillTimer` on the pump window.
const TIMER_ID: usize = 1;

pub(super) struct PlatformPump {
  hwnd: HWND,
  timer_pending: bool,
}

// SAFETY: `hwnd` is created on, and its timer is only armed/disarmed from, the
// main (winit) thread. The sole cross-thread use is `PostMessageW` in
// `post_schedule_work`, which Win32 explicitly permits from any thread.
unsafe impl Send for PlatformPump {}

impl PlatformPump {
  pub(super) fn new(state: Weak<PumpState>) -> Self {
    let hinstance: HINSTANCE = unsafe { GetModuleHandleW(None) }
      .map(|module| HINSTANCE(module.0))
      .unwrap_or_default();

    let class = WNDCLASSEXW {
      cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
      lpfnWndProc: Some(wndproc),
      hInstance: hinstance,
      lpszClassName: WINDOW_CLASS,
      ..Default::default()
    };
    // Registering an already-registered class fails harmlessly; this also lets a
    // second runtime in the same process reuse the class.
    unsafe { RegisterClassExW(&class) };

    let hwnd = unsafe {
      CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        WINDOW_CLASS,
        PCWSTR::null(),
        WS_OVERLAPPEDWINDOW,
        0,
        0,
        0,
        0,
        Some(HWND_MESSAGE),
        None,
        Some(hinstance),
        None,
      )
    }
    .expect("failed to create CEF external message pump window");

    // Store a Weak back-reference for the window procedure; freed in `Drop`. No
    // timer is armed and no message is posted yet, so the window cannot be
    // dispatched anything that reads this slot before it is set.
    let state = Box::into_raw(Box::new(state));
    unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize) };

    Self {
      hwnd,
      timer_pending: false,
    }
  }

  pub(super) fn post_schedule_work(&mut self, delay_ms: i64) {
    // Thread-safe; lands in `wndproc` (WM_HAVE_WORK) on the owner thread.
    let _ = unsafe {
      PostMessageW(
        Some(self.hwnd),
        WM_HAVE_WORK,
        WPARAM(0),
        LPARAM(delay_ms as isize),
      )
    };
  }

  pub(super) fn set_timer(&mut self, delay_ms: i64) {
    debug_assert!(!self.timer_pending);
    debug_assert!(delay_ms > 0);
    self.timer_pending = true;
    unsafe { SetTimer(Some(self.hwnd), TIMER_ID, delay_ms as u32, None) };
  }

  pub(super) fn kill_timer(&mut self) {
    if self.timer_pending {
      let _ = unsafe { KillTimer(Some(self.hwnd), TIMER_ID) };
      self.timer_pending = false;
    }
  }

  pub(super) fn is_timer_pending(&self) -> bool {
    self.timer_pending
  }
}

impl Drop for PlatformPump {
  fn drop(&mut self) {
    unsafe {
      if self.timer_pending {
        let _ = KillTimer(Some(self.hwnd), TIMER_ID);
      }

      let state = SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0) as *mut Weak<PumpState>;
      if !state.is_null() {
        drop(Box::from_raw(state));
      }

      let _ = DestroyWindow(self.hwnd);
    }
  }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  if msg == WM_TIMER || msg == WM_HAVE_WORK {
    let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const Weak<PumpState>;
    if !state.is_null()
      && let Some(state) = unsafe { &*state }.upgrade()
    {
      if msg == WM_HAVE_WORK {
        state.on_schedule_work(lparam.0 as i64);
      } else {
        state.on_timer_timeout();
      }
    }
  }

  unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
