// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Android backend for the CEF external message pump.
//!
//! The timer is winit's: [`crate::platform::android`] sleeps `android_main` until the
//! deadline, then posts the timeout to the Java main thread, which is CEF's UI thread.

use std::{
  sync::{OnceLock, Weak},
  time::{Duration, Instant},
};

use super::PumpState;
use crate::platform::android;

static PUMP: OnceLock<Weak<PumpState>> = OnceLock::new();

/// Runs a request from `on_schedule_message_pump_work`. Java main thread only.
pub(crate) fn deliver_schedule(delay_ms: i64) {
  if let Some(state) = PUMP.get().and_then(Weak::upgrade) {
    state.on_schedule_work(delay_ms);
  }
}

/// Runs the expired timer. Java main thread only.
pub(crate) fn deliver_timeout() {
  if let Some(state) = PUMP.get().and_then(Weak::upgrade) {
    state.on_timer_timeout();
  }
}

pub(super) struct PlatformPump {
  timer_pending: bool,
}

impl PlatformPump {
  pub(super) fn new(state: Weak<PumpState>) -> Self {
    let _ = PUMP.set(state);
    Self {
      timer_pending: false,
    }
  }

  pub(super) fn on_schedule_message_pump_work(&mut self, delay_ms: i64) {
    android::schedule_pump_work(delay_ms);
  }

  pub(super) fn set_timer(&mut self, delay_ms: i64) {
    self.timer_pending = true;
    let delay = Duration::from_millis(delay_ms.max(0) as u64);
    android::set_pump_timer(Some(Instant::now() + delay));
  }

  pub(super) fn kill_timer(&mut self) {
    self.timer_pending = false;
    android::set_pump_timer(None);
  }

  pub(super) fn is_timer_pending(&self) -> bool {
    self.timer_pending
  }
}
