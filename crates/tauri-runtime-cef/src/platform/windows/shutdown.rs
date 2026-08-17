// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Windows shutdown-event handling.
//!
//! Nothing in winit or CEF reacts to the Windows session ending: winit's
//! window procedure hands `WM_QUERYENDSESSION`/`WM_ENDSESSION` to
//! `DefWindowProcW` without surfacing an event, and console control events
//! keep their default handler, which calls `ExitProcess` from the
//! control-dispatch thread. Both paths end a CEF app badly:
//!
//! - At session end (shutdown/restart/logoff) the message loop never spins
//!   again once `WM_ENDSESSION` returns, so a clean `cef::shutdown` cannot
//!   happen and the OS terminates the process with a non-zero exit code that
//!   reads as a crash.
//! - `ExitProcess` runs `DLL_PROCESS_DETACH`/`atexit` teardown while
//!   Chromium's threads are still live, which can deadlock or crash on top of
//!   the non-zero `STATUS_CONTROL_C_EXIT` it reports.
//!
//! Handle both the way Chrome does. Session end is acknowledged and the
//! process ends itself with `TerminateProcess(self, 0)`: an explicit success
//! exit code, and no in-process teardown left behind for the OS kill to
//! corrupt. Console control events that leave time for it (Ctrl+C /
//! Ctrl+Break) instead request the runtime's regular graceful-exit path on
//! the main loop, so Ctrl+C in `tauri dev` drains browsers and runs
//! `cef::shutdown` exactly like closing the last window; the
//! terminal-close/logoff/shutdown events get a short grace period for that
//! same path and then force the success code before Windows' own kill timer
//! reports a failure one.

use std::sync::OnceLock;

use windows::{
  Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    System::{
      Console::{
        CTRL_BREAK_EVENT, CTRL_C_EVENT, CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT,
        SetConsoleCtrlHandler,
      },
      LibraryLoader::GetModuleHandleW,
      Threading::{GetCurrentProcess, Sleep, TerminateProcess},
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassExW, WINDOW_EX_STYLE,
      WM_ENDSESSION, WNDCLASSEXW, WS_POPUP,
    },
  },
  core::{BOOL, PCWSTR, w},
};

const K_CLASS_NAME: PCWSTR = w!("TauriCEFShutdownHWND");

/// How long the console control handler lets the graceful-exit path run before
/// forcing a zero exit code. Windows kills a console process about five
/// seconds after `CTRL_CLOSE_EVENT`/`CTRL_LOGOFF_EVENT`/`CTRL_SHUTDOWN_EVENT`
/// returns from the handler, so this must stay comfortably under that.
const GRACEFUL_EXIT_GRACE_MS: u32 = 3000;

/// Requests the runtime's graceful exit (`Message::RequestExit(0)`). Global
/// because the console control handler runs on a system-created thread with no
/// way to smuggle state in; set once by [`install`].
static EXIT_REQUESTER: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

/// Owns the hidden window that listens for session-end messages. Created and
/// dropped on the event-loop thread, which is the only thread `DestroyWindow`
/// may be called from.
pub(crate) struct ShutdownEvents {
  hwnd: HWND,
}

/// Install the session-end window and the console control handler.
///
/// Must run in the browser process after `cef::initialize`: console control
/// handlers are invoked most-recently-registered first, so registering after
/// CEF guarantees ours runs ahead of anything Chromium may have installed.
pub(crate) fn install(request_exit: Box<dyn Fn() + Send + Sync>) -> Option<ShutdownEvents> {
  let _ = EXIT_REQUESTER.set(request_exit);

  let _ = unsafe { SetConsoleCtrlHandler(Some(console_ctrl_handler), true) };

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

  // `WM_QUERYENDSESSION`/`WM_ENDSESSION` are broadcast to top-level windows
  // only: the external pump's message-only window never sees them, and winit
  // windows may not exist yet (or ever, for tray-only apps). A hidden
  // top-level window guarantees a recipient on the event-loop thread. Never
  // shown, so it stays out of the taskbar and Alt-Tab.
  let hwnd = unsafe {
    CreateWindowExW(
      WINDOW_EX_STYLE::default(),
      K_CLASS_NAME,
      PCWSTR::null(),
      WS_POPUP,
      0,
      0,
      0,
      0,
      None,
      None,
      Some(hinstance),
      None,
    )
  }
  .ok()?;

  Some(ShutdownEvents { hwnd })
}

impl Drop for ShutdownEvents {
  fn drop(&mut self) {
    let _ = unsafe { SetConsoleCtrlHandler(Some(console_ctrl_handler), false) };
    let _ = unsafe { DestroyWindow(self.hwnd) };
  }
}

unsafe extern "system" fn wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  // `WM_QUERYENDSESSION` falls through to `DefWindowProcW`, which answers TRUE
  // (agree to end the session) — same as Chrome, which never vetoes shutdown.
  if msg == WM_ENDSESSION {
    // A FALSE wparam means some other app vetoed the shutdown; nothing ends.
    if wparam.0 != 0 {
      // The session is ending (or the Restart Manager asked us to close, e.g.
      // for an installer update). Once this handler returns the process can
      // be terminated at any point and the message loop never runs again, so
      // the graceful close-browsers → `cef::shutdown` dance is impossible.
      // Exit Chrome-style: skip in-process teardown entirely — running it
      // under Chromium's live threads is exactly what crashes — and set an
      // explicit success code so the end of the process isn't recorded as a
      // crash.
      unsafe {
        let _ = TerminateProcess(GetCurrentProcess(), 0);
      }
    }
    return LRESULT(0);
  }

  unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Runs on a system-created control-dispatch thread, never the event loop.
unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> BOOL {
  match ctrl_type {
    // Interactive interrupts have no deadline: hand them to the graceful-exit
    // path (`ExitRequested` → close browsers → `cef::shutdown`) so Ctrl+C in
    // `tauri dev` exits with code 0 instead of the default handler's
    // `ExitProcess(STATUS_CONTROL_C_EXIT)` tearing down DLLs under Chromium's
    // live threads. Returning TRUE suppresses that default handler.
    CTRL_C_EVENT | CTRL_BREAK_EVENT => {
      if let Some(request_exit) = EXIT_REQUESTER.get() {
        request_exit();
        BOOL(1)
      } else {
        BOOL(0)
      }
    }
    // The terminal was closed, or the session is ending for a console-mode
    // process: Windows kills the process a few seconds after this returns no
    // matter what. Ask for the graceful exit and give it the grace period — a
    // clean exit ends the process mid-`Sleep` — then force the success code
    // rather than letting the timeout kill report `STATUS_CONTROL_C_EXIT`.
    CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT => {
      if let Some(request_exit) = EXIT_REQUESTER.get() {
        request_exit();
        unsafe { Sleep(GRACEFUL_EXIT_GRACE_MS) };
      }
      unsafe {
        let _ = TerminateProcess(GetCurrentProcess(), 0);
      }
      BOOL(1)
    }
    _ => BOOL(0),
  }
}
