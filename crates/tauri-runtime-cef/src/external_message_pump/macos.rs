// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! macOS backend for the CEF external message pump.
//!
//! Upstream cefclient:
//! <https://github.com/chromiumembedded/cef/blob/b41c5c64fc50871630678e3d0c8c9bfe77cc6353/tests/shared/browser/main_message_loop_external_pump_mac.mm>
//!
//! Mirrors cefclient's `main_message_loop_external_pump_mac.mm`: scheduling
//! requests are posted back onto the owning AppKit thread with
//! `performSelector:onThread:`, and delayed work is driven by an `NSTimer`
//! installed in the common and event-tracking run-loop modes so it keeps firing
//! while AppKit spins a nested menu/tracking loop (e.g. a webview context menu)
//! that winit's callbacks never observe.
use std::sync::Weak;

use objc2::{AnyThread, DefinedClass, define_class, msg_send, rc::Retained, sel};
use objc2_app_kit::NSEventTrackingRunLoopMode;
use objc2_foundation::{
  NSNumber, NSObject, NSObjectNSThreadPerformAdditions, NSObjectProtocol, NSRunLoop,
  NSRunLoopCommonModes, NSThread, NSTimer,
};

use super::PumpState;

// Object that handles event callbacks on the owner thread.
define_class! {
  #[unsafe(super(NSObject))]
  #[ivars = Weak<PumpState>]
  struct EventHandler;

  impl EventHandler {
    #[unsafe(method(scheduleWork:))]
    fn handle_schedule_work(&self, delay_ms: &NSNumber) {
      let Some(state) = self.ivars().upgrade() else {
        return;
      };
      state.on_schedule_work(delay_ms.as_i64());
    }

    #[unsafe(method(timerTimeout:))]
    fn handle_timer_timeout(&self, _: &NSTimer) {
      let Some(state) = self.ivars().upgrade() else {
        return;
      };
      state.on_timer_timeout();
    }
  }

  unsafe impl NSObjectProtocol for EventHandler {}
}

impl EventHandler {
  fn new(state: Weak<PumpState>) -> Retained<Self> {
    let this = Self::alloc().set_ivars(state);
    unsafe { msg_send![super(this), init] }
  }
}

pub(super) struct PlatformPump {
  // Owner thread that will run events.
  owner_thread: Retained<NSThread>,

  // Used to handle event callbacks on the owner thread.
  event_handler: Retained<EventHandler>,

  // Pending work timer.
  timer: Option<Retained<NSTimer>>,
}

// SAFETY: the owner thread and timer are only touched on the AppKit thread that
// constructed the pump; `on_schedule_message_pump_work` marshals back to it
// before use.
unsafe impl Send for PlatformPump {}

impl PlatformPump {
  pub(super) fn new(state: Weak<PumpState>) -> Self {
    Self {
      owner_thread: NSThread::currentThread(),
      event_handler: EventHandler::new(state),
      timer: None,
    }
  }

  pub(super) fn on_schedule_message_pump_work(&mut self, delay_ms: i64) {
    // This method may be called on any thread.
    let delay_ms = NSNumber::new_i32(delay_ms as i32);
    unsafe {
      self
        .event_handler
        .performSelector_onThread_withObject_waitUntilDone(
          sel!(scheduleWork:),
          &self.owner_thread,
          Some(&delay_ms),
          false,
        );
    }
  }

  pub(super) fn set_timer(&mut self, delay_ms: i64) {
    debug_assert!(delay_ms > 0);
    debug_assert!(self.timer.is_none());

    let timer = unsafe {
      NSTimer::timerWithTimeInterval_target_selector_userInfo_repeats(
        delay_ms as f64 / 1000.0,
        &self.event_handler,
        sel!(timerTimeout:),
        None,
        false,
      )
    };

    // Add the timer to default and tracking runloop modes.
    let run_loop = NSRunLoop::currentRunLoop();
    unsafe {
      run_loop.addTimer_forMode(&timer, NSRunLoopCommonModes);
      run_loop.addTimer_forMode(&timer, NSEventTrackingRunLoopMode);
    }

    self.timer = Some(timer);
  }

  pub(super) fn kill_timer(&mut self) {
    if let Some(timer) = self.timer.take() {
      timer.invalidate();
    }
  }

  pub(super) fn is_timer_pending(&self) -> bool {
    self.timer.is_some()
  }
}
