// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! macOS backend for the CEF external message pump.
//!
//! Mirrors cefclient's `main_message_loop_external_pump_mac.mm`: scheduling
//! requests are posted back onto the owning AppKit thread with
//! `performSelector:onThread:`, and delayed work is driven by an `NSTimer`
//! installed in the common and event-tracking run-loop modes so it keeps firing
//! while AppKit spins a nested menu/tracking loop (e.g. a webview context menu)
//! that winit's callbacks never observe.
//!
//! Reference:
//! <https://github.com/chromiumembedded/cef/blob/b2d312cd48fe0195f9736fd7c761a89abd5bf2be/tests/shared/browser/main_message_loop_external_pump_mac.mm>

use std::sync::Weak;

use objc2::{AnyThread, DefinedClass, define_class, msg_send, rc::Retained, sel};
use objc2_app_kit::NSEventTrackingRunLoopMode;
use objc2_foundation::{
  NSNumber, NSObject, NSObjectNSThreadPerformAdditions, NSObjectProtocol, NSRunLoop,
  NSRunLoopCommonModes, NSThread, NSTimer,
};

use super::PumpState;

define_class! {
  #[unsafe(super(NSObject))]
  #[ivars = Weak<PumpState>]
  struct EventHandler;

  impl EventHandler {
    #[unsafe(method(scheduleWork:))]
    fn schedule_work(&self, delay_ms: &NSNumber) {
      let Some(state) = self.ivars().upgrade() else {
        return;
      };
      state.on_schedule_work(delay_ms.as_i64());
    }

    #[unsafe(method(timerTimeout:))]
    fn timer_timeout(&self, _: &NSTimer) {
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
  owner_thread: Retained<NSThread>,
  event_handler: Retained<EventHandler>,
  timer: Option<Retained<NSTimer>>,
}

// SAFETY: the owner thread and timer are only touched on the AppKit thread that
// constructed the pump; `post_schedule_work` marshals back to it before use.
unsafe impl Send for PlatformPump {}

impl PlatformPump {
  pub(super) fn new(state: Weak<PumpState>) -> Self {
    Self {
      owner_thread: NSThread::currentThread(),
      event_handler: EventHandler::new(state),
      timer: None,
    }
  }

  pub(super) fn post_schedule_work(&mut self, delay_ms: i64) {
    let delay_ms = isize::try_from(delay_ms).unwrap_or(isize::MAX);
    let delay_ms = NSNumber::new_isize(delay_ms);
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
