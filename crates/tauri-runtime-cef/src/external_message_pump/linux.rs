// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Linux/BSD backend for the CEF external message pump.
//!
//! cefclient owns the GLib loop and drives a GLib timeout from it
//! (`main_message_loop_external_pump_linux.cc`). Here winit owns an X11 loop
//! instead, so the timeout is attached to the default GLib main context and the
//! winit loop services that context every iteration (see the runtime's
//! `service_glib`), waking at [`PlatformPump::deadline`] via
//! `ControlFlow::WaitUntil`. The timeout still fires inside nested GLib loops
//! (e.g. GTK menus/dialogs) that winit cannot observe, keeping CEF pumping
//! there — the same property the Windows/macOS backends get from `WM_TIMER` /
//! `NSTimer`.
//!
//! Reference:
//! <https://github.com/chromiumembedded/cef/blob/b2d312cd48fe0195f9736fd7c761a89abd5bf2be/tests/shared/browser/main_message_loop_external_pump_linux.cc>

use std::sync::Weak;
use std::time::{Duration, Instant};

use gtk::glib;
use winit::event_loop::EventLoopProxy;

use super::PumpState;

pub(super) struct PlatformPump {
  state: Weak<PumpState>,
  proxy: EventLoopProxy,
  timer: Option<glib::SourceId>,
  deadline: Option<Instant>,
}

impl PlatformPump {
  pub(super) fn new(state: Weak<PumpState>, proxy: EventLoopProxy) -> Self {
    Self {
      state,
      proxy,
      timer: None,
      deadline: None,
    }
  }

  pub(super) fn post_schedule_work(&mut self, delay_ms: i64) {
    // May be called on any thread. Marshal the request onto the default GLib
    // main context, which the winit loop services on the main thread; wake winit
    // so it does so promptly.
    let state = self.state.clone();
    glib::idle_add_once(move || {
      if let Some(state) = state.upgrade() {
        state.on_schedule_work(delay_ms);
      }
    });
    self.proxy.wake_up();
  }

  pub(super) fn set_timer(&mut self, delay_ms: i64) {
    debug_assert!(self.timer.is_none());
    debug_assert!(delay_ms > 0);

    let delay = Duration::from_millis(delay_ms as u64);
    let state = self.state.clone();
    let source = glib::timeout_add_once(delay, move || {
      let Some(state) = state.upgrade() else {
        return;
      };
      // This one-shot source removes itself after firing, so forget it before
      // `on_timer_timeout` runs `kill_timer` — `SourceId::remove` would panic on
      // an already-removed source.
      if let Ok(mut platform) = state.platform.lock() {
        platform.timer = None;
        platform.deadline = None;
      }
      state.on_timer_timeout();
    });

    self.timer = Some(source);
    self.deadline = Some(Instant::now() + delay);
  }

  pub(super) fn kill_timer(&mut self) {
    if let Some(source) = self.timer.take() {
      source.remove();
    }
    self.deadline = None;
  }

  pub(super) fn is_timer_pending(&self) -> bool {
    self.timer.is_some()
  }

  pub(super) fn deadline(&self) -> Option<Instant> {
    self.deadline
  }
}
