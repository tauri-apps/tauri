// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! macOS external message pump integration for CEF.
//!
//! CEF can request immediate work while AppKit is already dispatching a nested
//! menu/tracking event, for example from a webview context menu. Calling
//! `cef_do_message_loop_work` inline from that callback can re-enter winit's
//! event handler and panic. This pump posts CEF work back onto the owning
//! AppKit thread and drives delayed work with `NSTimer` modes that still run
//! during event tracking, while deferring any reentrant CEF tick to the next
//! scheduled turn.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};

use cef::do_message_loop_work as cef_do_message_loop_work;
use objc2::{AnyThread, DefinedClass, define_class, msg_send, rc::Retained, sel};
use objc2_app_kit::NSEventTrackingRunLoopMode;
use objc2_foundation::{
  NSNumber, NSObject, NSObjectNSThreadPerformAdditions, NSObjectProtocol, NSRunLoop,
  NSRunLoopCommonModes, NSThread, NSTimer,
};

const TIMER_DELAY_PLACEHOLDER: i64 = i32::MAX as i64;
const MAX_TIMER_DELAY_MS: i64 = 1000 / 30;

pub(crate) struct MacosCefPump {
  state: Arc<CefExternalPumpState>,
}

impl MacosCefPump {
  pub(crate) fn new() -> Self {
    let state = Arc::new_cyclic(|weak| CefExternalPumpState {
      is_active: AtomicBool::new(false),
      reentrancy_detected: AtomicBool::new(false),
      platform: Mutex::new(PlatformPump::new(weak.clone())),
    });

    Self { state }
  }

  pub(crate) fn schedule_message_pump_work(&self, delay_ms: i64) {
    self.state.schedule_message_pump_work(delay_ms);
  }

  pub(crate) fn do_message_loop_work(&self) {
    self.state.do_work();
  }
}

impl Clone for MacosCefPump {
  fn clone(&self) -> Self {
    Self {
      state: self.state.clone(),
    }
  }
}

struct CefExternalPumpState {
  is_active: AtomicBool,
  reentrancy_detected: AtomicBool,
  platform: Mutex<PlatformPump>,
}

impl CefExternalPumpState {
  fn schedule_message_pump_work(&self, delay_ms: i64) {
    if let Ok(mut platform) = self.platform.lock() {
      platform.schedule_message_pump_work(delay_ms);
    }
  }

  fn schedule_work_on_owner_thread(&self, delay_ms: i64) {
    {
      let Ok(mut platform) = self.platform.lock() else {
        return;
      };

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

  fn timer_timeout(&self) {
    if let Ok(mut platform) = self.platform.lock() {
      platform.kill_timer();
    }
    self.do_work();
  }

  fn do_work(&self) {
    let was_reentrant = self.perform_message_loop_work();
    if was_reentrant {
      self.schedule_message_pump_work(0);
      return;
    }

    let timer_pending = self
      .platform
      .lock()
      .map(|platform| platform.is_timer_pending())
      .unwrap_or(true);
    if !timer_pending {
      self.schedule_message_pump_work(TIMER_DELAY_PLACEHOLDER);
    }
  }

  fn perform_message_loop_work(&self) -> bool {
    if self.is_active.swap(true, Ordering::SeqCst) {
      self.reentrancy_detected.store(true, Ordering::SeqCst);
      return false;
    }

    self.reentrancy_detected.store(false, Ordering::SeqCst);
    cef_do_message_loop_work();
    self.is_active.store(false, Ordering::SeqCst);

    self.reentrancy_detected.load(Ordering::SeqCst)
  }
}

define_class! {
  #[unsafe(super(NSObject))]
  #[ivars = Weak<CefExternalPumpState>]
  struct EventHandler;

  impl EventHandler {
    #[unsafe(method(scheduleWork:))]
    fn schedule_work(&self, delay_ms: &NSNumber) {
      let Some(pump) = self.ivars().upgrade() else {
        return;
      };
      pump.schedule_work_on_owner_thread(delay_ms.as_i64());
    }

    #[unsafe(method(timerTimeout:))]
    fn timer_timeout(&self, _: &NSTimer) {
      let Some(pump) = self.ivars().upgrade() else {
        return;
      };
      pump.timer_timeout();
    }
  }

  unsafe impl NSObjectProtocol for EventHandler {}
}

impl EventHandler {
  fn new(pump: Weak<CefExternalPumpState>) -> Retained<Self> {
    let this = Self::alloc().set_ivars(pump);
    unsafe { msg_send![super(this), init] }
  }
}

struct PlatformPump {
  owner_thread: Retained<NSThread>,
  event_handler: Retained<EventHandler>,
  timer: Option<Retained<NSTimer>>,
}

unsafe impl Send for PlatformPump {}

impl PlatformPump {
  fn new(pump: Weak<CefExternalPumpState>) -> Self {
    Self {
      owner_thread: NSThread::currentThread(),
      event_handler: EventHandler::new(pump),
      timer: None,
    }
  }

  fn schedule_message_pump_work(&mut self, delay_ms: i64) {
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

  fn set_timer(&mut self, delay_ms: i64) {
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

  fn kill_timer(&mut self) {
    if let Some(timer) = self.timer.take() {
      timer.invalidate();
    }
  }

  fn is_timer_pending(&self) -> bool {
    self.timer.is_some()
  }
}
