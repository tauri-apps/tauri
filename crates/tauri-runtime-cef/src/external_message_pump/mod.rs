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
//! This is a port of upstream cefclient's external pump — same semantics and
//! logic, adapted to Rust. The platform-independent scheduling/reentrancy logic
//! lives here; each platform supplies a [`PlatformPump`] backend that drives a
//! timer:
//!
//! - Windows: a `WM_TIMER` on a message-only window.
//! - macOS: an `NSTimer` in the common and event-tracking run-loop modes.
//! - Linux/BSD: a GLib timeout serviced by the winit loop.
//!
//! On Windows and macOS the timer lives on the same native loop winit already
//! runs, so CEF keeps painting and processing IPC even while the OS spins a
//! nested modal loop winit cannot observe (window move/resize on Windows, menu
//! and event tracking on macOS). On Linux/BSD the GLib timeout still fires
//! inside nested GLib loops (e.g. GTK menus/dialogs) for the same reason.
//!
//! Reference implementation (cefclient base class):
//! - <https://github.com/chromiumembedded/cef/blob/b2d312cd48fe0195f9736fd7c761a89abd5bf2be/tests/shared/browser/main_message_loop_external_pump.cc>
//! - <https://github.com/chromiumembedded/cef/blob/b2d312cd48fe0195f9736fd7c761a89abd5bf2be/tests/shared/browser/main_message_loop_external_pump.h>

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

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
use winit::event_loop::EventLoopProxy;

/// Sentinel delay used to (re)arm the fallback "max delay" tick. Kept within 32
/// bits for Win32/AppKit timer API compatibility, matching cefclient's
/// `kTimerDelayPlaceholder`.
const TIMER_DELAY_PLACEHOLDER: i64 = i32::MAX as i64;

/// Upper bound on how long we wait between [`cef::do_message_loop_work`] calls
/// (~30fps), matching cefclient's `kMaxTimerDelay`.
const MAX_TIMER_DELAY_MS: i64 = 1000 / 30;

/// Handle to the external message pump. Cloning shares the same underlying
/// state; the backing platform resources are released when the last clone drops.
#[derive(Clone)]
pub(crate) struct CefExternalPump {
  state: Arc<PumpState>,
}

impl CefExternalPump {
  pub(crate) fn new(
    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "netbsd",
      target_os = "openbsd"
    ))]
    proxy: EventLoopProxy,
  ) -> Self {
    let state = Arc::new_cyclic(|weak| PumpState {
      is_active: AtomicBool::new(false),
      reentrancy_detected: AtomicBool::new(false),
      platform: Mutex::new(PlatformPump::new(
        weak.clone(),
        #[cfg(any(
          target_os = "linux",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "netbsd",
          target_os = "openbsd"
        ))]
        proxy,
      )),
    });

    Self { state }
  }

  /// Called from CEF's `OnScheduleMessagePumpWork`. May run on any thread.
  pub(crate) fn schedule_message_pump_work(&self, delay_ms: i64) {
    self.state.schedule_message_pump_work(delay_ms);
  }

  /// Explicit tick, used to drive CEF before winit's loop is running (startup)
  /// and after winit processes a batch of events. Must run on the owner thread.
  pub(crate) fn do_message_loop_work(&self) {
    self.state.do_work();
  }

  /// When the GLib timer is next due, so the winit loop can wake to service
  /// GLib (see [`linux`]).
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
  fn schedule_message_pump_work(&self, delay_ms: i64) {
    if let Ok(mut platform) = self.platform.lock() {
      platform.post_schedule_work(delay_ms);
    }
  }

  /// Runs on the owner thread once a scheduling request is delivered. Mirrors
  /// cefclient's `OnScheduleWork`.
  fn on_schedule_work(&self, delay_ms: i64) {
    {
      let Ok(mut platform) = self.platform.lock() else {
        return;
      };

      // An already-pending timer covers the fallback tick; don't reset it.
      if delay_ms == TIMER_DELAY_PLACEHOLDER && platform.is_timer_pending() {
        return;
      }

      platform.kill_timer();
    }

    if delay_ms <= 0 {
      self.do_work();
      return;
    }

    if let Ok(mut platform) = self.platform.lock() {
      platform.set_timer(delay_ms.min(MAX_TIMER_DELAY_MS));
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
      // The work was discarded because we were already inside
      // do_message_loop_work; repost so it runs on the next clean turn.
      self.schedule_message_pump_work(0);
      return;
    }

    // Arm the fallback tick so CEF work it didn't explicitly announce (e.g. via
    // its own internal timers) still runs within the max delay.
    let timer_pending = self
      .platform
      .lock()
      .map(|platform| platform.is_timer_pending())
      .unwrap_or(true);
    if !timer_pending {
      self.schedule_message_pump_work(TIMER_DELAY_PLACEHOLDER);
    }
  }

  /// Mirrors cefclient's `PerformMessageLoopWork`.
  fn perform_message_loop_work(&self) -> bool {
    if self.is_active.swap(true, Ordering::SeqCst) {
      // do_message_loop_work can trigger CEF callbacks (paint, IPC) that
      // re-enter this method. Record it so the caller reschedules the work.
      self.reentrancy_detected.store(true, Ordering::SeqCst);
      return false;
    }

    self.reentrancy_detected.store(false, Ordering::SeqCst);
    cef::do_message_loop_work();
    self.is_active.store(false, Ordering::SeqCst);

    self.reentrancy_detected.load(Ordering::SeqCst)
  }
}
