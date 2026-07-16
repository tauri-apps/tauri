// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Self-contained CEF external message pump.
//!
//! With [`cef::Settings::external_message_pump`] enabled, CEF does not run its
//! own message loop. Instead it asks the host to call
//! [`cef::do_message_loop_work`] by invoking `OnScheduleMessagePumpWork(delay)`
//! whenever it has work pending.
//!
//! Upstream cefclient:
//! - <https://github.com/chromiumembedded/cef/blob/b41c5c64fc50871630678e3d0c8c9bfe77cc6353/tests/shared/browser/main_message_loop_external_pump.cc>
//! - <https://github.com/chromiumembedded/cef/blob/b41c5c64fc50871630678e3d0c8c9bfe77cc6353/tests/shared/browser/main_message_loop_external_pump.h>
//!
//! This is a port of upstream cefclient's external pump — same semantics and
//! logic, adapted to Rust. The platform-independent scheduling/reentrancy logic
//! lives here; each platform supplies a [`PlatformPump`] backend that drives a
//! timer:
//!
//! - Windows: a `WM_TIMER` on a message-only window.
//! - macOS: an `NSTimer` in the common and event-tracking run-loop modes.
//! - Linux/BSD: a custom low-priority GLib source with a wakeup pipe.
//!
//! On Windows and macOS the timer lives on the same native loop winit already
//! runs, so CEF keeps painting and processing IPC even while the OS spins a
//! nested modal loop winit cannot observe (window move/resize on Windows, menu
//! and event tracking on macOS). On Linux/BSD the GLib source still dispatches
//! inside nested GLib loops (e.g. GTK menus/dialogs) for the same reason.

use std::sync::{
  Arc, Mutex,
  atomic::{AtomicBool, Ordering},
};

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
use linux::PlatformPump;
#[cfg(target_os = "macos")]
use macos::PlatformPump;
#[cfg(windows)]
use windows::PlatformPump;

// Special timer delay placeholder value. Intentionally 32-bit for Windows and
// OS X platform API compatibility.
const K_TIMER_DELAY_PLACEHOLDER: i64 = i32::MAX as i64;

// The maximum number of milliseconds we're willing to wait between calls to
// DoWork().
const K_MAX_TIMER_DELAY: i64 = 1000 / 30; // 30fps

/// Handle to the external message pump. Cloning shares the same underlying
/// state; the backing platform resources are released when the last clone drops.
#[derive(Clone)]
pub(crate) struct CefExternalPump {
  state: Arc<PumpState>,
}

impl CefExternalPump {
  pub(crate) fn new() -> Self {
    let state = Arc::new_cyclic(|weak| PumpState {
      is_active: AtomicBool::new(false),
      reentrancy_detected: AtomicBool::new(false),
      platform: Mutex::new(PlatformPump::new(weak.clone())),
    });

    Self { state }
  }

  /// Called from CEF's `OnScheduleMessagePumpWork`. May run on any thread.
  pub(crate) fn on_schedule_message_pump_work(&self, delay_ms: i64) {
    self.state.on_schedule_message_pump_work(delay_ms);
  }

  /// Explicit tick, used to drive CEF before winit's loop is running (startup)
  /// and after winit processes a batch of events. Must run on the owner thread.
  pub(crate) fn do_work(&self) {
    self.state.do_work();
  }

  /// When the platform timer is next due. This is only needed by event loops
  /// that do not block in the GLib main context themselves.
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  pub(crate) fn next_deadline(&self) -> Option<std::time::Instant> {
    self.state.platform.lock().ok().and_then(|p| p.deadline())
  }
}

/// Platform-independent pump state, shared with the [`PlatformPump`] backend.
struct PumpState {
  is_active: AtomicBool,
  reentrancy_detected: AtomicBool,
  platform: Mutex<PlatformPump>,
}

impl PumpState {
  /// Post a scheduling request onto the owner thread. The platform backend is
  /// responsible for delivering it there, where it lands back in
  /// [`Self::on_schedule_work`]. Mirrors the platform `OnScheduleMessagePumpWork`.
  fn on_schedule_message_pump_work(&self, delay_ms: i64) {
    if let Ok(mut platform) = self.platform.lock() {
      platform.on_schedule_message_pump_work(delay_ms);
    }
  }

  /// Runs on the owner thread once a scheduling request is delivered. Mirrors
  /// cefclient's `OnScheduleWork`.
  fn on_schedule_work(&self, mut delay_ms: i64) {
    {
      let Ok(mut platform) = self.platform.lock() else {
        return;
      };

      if delay_ms == K_TIMER_DELAY_PLACEHOLDER && platform.is_timer_pending() {
        // Don't set the maximum timer requested from DoWork() if a timer event is
        // currently pending.
        return;
      }

      platform.kill_timer();
    }

    if delay_ms <= 0 {
      // Execute the work immediately.
      self.do_work();
    } else if let Ok(mut platform) = self.platform.lock() {
      // Never wait longer than the maximum allowed time.
      if delay_ms > K_MAX_TIMER_DELAY {
        delay_ms = K_MAX_TIMER_DELAY;
      }

      // Results in call to OnTimerTimeout() after the specified delay.
      platform.set_timer(delay_ms);
    }
  }

  /// Runs on the owner thread when the platform timer fires. Mirrors cefclient's
  /// `OnTimerTimeout`.
  fn on_timer_timeout(&self) {
    if let Ok(mut platform) = self.platform.lock() {
      platform.kill_timer();
    }
    self.do_work();
  }

  /// Mirrors cefclient's `DoWork`.
  fn do_work(&self) {
    let was_reentrant = self.perform_message_loop_work();
    if was_reentrant {
      // Execute the remaining work as soon as possible.
      self.on_schedule_message_pump_work(0);
    } else if !self.is_timer_pending() {
      // Schedule a timer event at the maximum allowed time. This may be dropped
      // in OnScheduleWork() if another timer event is already in-flight.
      self.on_schedule_message_pump_work(K_TIMER_DELAY_PLACEHOLDER);
    }
  }

  fn is_timer_pending(&self) -> bool {
    self
      .platform
      .lock()
      .map(|platform| platform.is_timer_pending())
      .unwrap_or(true)
  }

  /// Mirrors cefclient's `PerformMessageLoopWork`.
  fn perform_message_loop_work(&self) -> bool {
    if self.is_active.load(Ordering::SeqCst) {
      // When CefDoMessageLoopWork() is called there may be various callbacks
      // (such as paint and IPC messages) that result in additional calls to this
      // method. If re-entrancy is detected we must repost a request again to the
      // owner thread to ensure that the discarded call is executed in the future.
      self.reentrancy_detected.store(true, Ordering::SeqCst);
      return false;
    }

    self.reentrancy_detected.store(false, Ordering::SeqCst);

    self.is_active.store(true, Ordering::SeqCst);
    cef::do_message_loop_work();
    self.is_active.store(false, Ordering::SeqCst);

    // |reentrancy_detected_| may have changed due to re-entrant calls to this
    // method.
    self.reentrancy_detected.load(Ordering::SeqCst)
  }
}
