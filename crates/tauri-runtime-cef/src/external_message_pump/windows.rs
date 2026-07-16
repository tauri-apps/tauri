// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Windows backend for the CEF external message pump.
//!
//! Upstream cefclient:
//! <https://github.com/chromiumembedded/cef/blob/b41c5c64fc50871630678e3d0c8c9bfe77cc6353/tests/shared/browser/main_message_loop_external_pump_win.cc>
//!
//! Mirrors cefclient's `main_message_loop_external_pump_win.cc`: a message-only
//! window owns a `WM_TIMER`, and `OnScheduleMessagePumpWork` posts a private
//! `WM_HAVE_WORK` message to it. winit runs the thread's own
//! `GetMessage`/`DispatchMessage` loop, which delivers both messages to our
//! window procedure — so CEF is pumped from the same loop, including while
//! Windows runs a modal move/resize loop that winit's `ApplicationHandler`
//! callbacks never observe.
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

const K_CLASS_NAME: PCWSTR = w!("TauriCEFMainTargetHWND");

// Message sent to get an additional time slice for pumping (processing) another
// task (a series of such messages creates a continuous task pump).
const K_MSG_HAVE_WORK: u32 = WM_USER + 1;

const TIMER_ID: usize = 1;

pub(super) struct PlatformPump {
  // HWND owned by the thread that CefDoMessageLoopWork should be invoked on.
  main_thread_target: HWND,

  // True if a timer event is currently pending.
  timer_pending: bool,
}

// SAFETY: `main_thread_target` is created on, and its timer is only
// armed/disarmed from, the main (winit) thread. The sole cross-thread use is
// `PostMessageW` in `on_schedule_message_pump_work`, which Win32 explicitly
// permits from any thread.
unsafe impl Send for PlatformPump {}

impl PlatformPump {
  pub(super) fn new(state: Weak<PumpState>) -> Self {
    let hinstance: HINSTANCE = unsafe { GetModuleHandleW(None) }
      .map(|module| HINSTANCE(module.0))
      .unwrap_or_default();

    let class = WNDCLASSEXW {
      cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
      lpfnWndProc: Some(wnd_proc),
      hInstance: hinstance,
      lpszClassName: K_CLASS_NAME,
      ..Default::default()
    };
    unsafe { RegisterClassExW(&class) };

    // Create the message handling window.
    let hwnd = unsafe {
      CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        K_CLASS_NAME,
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

    let state = Box::into_raw(Box::new(state));
    unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize) };

    Self {
      main_thread_target: hwnd,
      timer_pending: false,
    }
  }

  pub(super) fn on_schedule_message_pump_work(&mut self, delay_ms: i64) {
    // This method may be called on any thread.
    let _ = unsafe {
      PostMessageW(
        Some(self.main_thread_target),
        K_MSG_HAVE_WORK,
        WPARAM(0),
        LPARAM(delay_ms as isize),
      )
    };
  }

  pub(super) fn set_timer(&mut self, delay_ms: i64) {
    debug_assert!(!self.timer_pending);
    debug_assert!(delay_ms > 0);
    self.timer_pending = true;
    unsafe {
      SetTimer(
        Some(self.main_thread_target),
        TIMER_ID,
        delay_ms as u32,
        None,
      )
    };
  }

  pub(super) fn kill_timer(&mut self) {
    if self.timer_pending {
      let _ = unsafe { KillTimer(Some(self.main_thread_target), TIMER_ID) };
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
        let _ = KillTimer(Some(self.main_thread_target), TIMER_ID);
      }

      let state =
        SetWindowLongPtrW(self.main_thread_target, GWLP_USERDATA, 0) as *mut Weak<PumpState>;
      if !state.is_null() {
        drop(Box::from_raw(state));
      }

      let _ = DestroyWindow(self.main_thread_target);
    }
  }
}

unsafe extern "system" fn wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  if msg == WM_TIMER || msg == K_MSG_HAVE_WORK {
    let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const Weak<PumpState>;
    if !state.is_null()
      && let Some(state) = unsafe { &*state }.upgrade()
    {
      if msg == K_MSG_HAVE_WORK {
        // OnScheduleMessagePumpWork() request.
        state.on_schedule_work(lparam.0 as i64);
      } else {
        // Timer timed out.
        state.on_timer_timeout();
      }
    }
  }

  unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
