// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Helpers shared by the Qt test cases. Every case runs in its own process (see
//! `qtshim::test_harness`), so each one starts with [`init`] and may leave Qt running: [`teardown`]
//! shuts it down afterwards.

#![allow(dead_code)] // not every helper is used with every feature set

use std::{
  cell::RefCell,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  thread,
  time::{Duration, Instant},
};

use qtshim::app::{self, Event, EventKind, Reply};

/// How long [`wait_for`] waits by default. Generous: the first web page load starts Chromium.
pub const WAIT: Duration = Duration::from_secs(20);

/// Create the application (offscreen in tests).
pub fn init() {
  app::init(Some("qtshim-test"));
}

/// Passed to `qtshim::test_harness::main`: shut Qt down if the case left it running.
pub fn teardown() {
  if app::is_running() {
    app::shutdown();
  }
}

thread_local! {
  static COND: RefCell<Option<Box<dyn FnMut() -> bool>>> = const { RefCell::new(None) };
}

/// Run the event loop ([`app::exec`]) until `cond` returns `true` (checked every few
/// milliseconds) or `timeout` passes. Returns whether `cond` became true.
///
/// Must not be nested (don't call it from a callback).
pub fn run_until(timeout: Duration, cond: impl FnMut() -> bool + 'static) -> bool {
  let deadline = Instant::now() + timeout;
  let met = Arc::new(AtomicBool::new(false));
  let done = Arc::new(AtomicBool::new(false));
  COND.with(|c| *c.borrow_mut() = Some(Box::new(cond)));
  // A ticker thread keeps the loop waking up; the condition itself stays on the main thread.
  let ticker = {
    let (met, done) = (met.clone(), done.clone());
    thread::spawn(move || {
      while !done.load(Ordering::Acquire) {
        thread::sleep(Duration::from_millis(5));
        let (met, done) = (met.clone(), done.clone());
        let queued = app::dispatch(move || {
          if done.load(Ordering::Acquire) {
            return;
          }
          let ok = COND.with(|c| c.borrow_mut().as_mut().is_some_and(|f| f()));
          if ok || Instant::now() >= deadline {
            met.store(ok, Ordering::Release);
            done.store(true, Ordering::Release);
            app::quit(0);
          }
        });
        if !queued {
          break;
        }
      }
    })
  };
  app::exec();
  done.store(true, Ordering::Release);
  ticker.join().unwrap();
  COND.with(|c| c.borrow_mut().take());
  met.load(Ordering::Acquire)
}

/// [`run_until`] with [`WAIT`], panicking with `what` on timeout.
#[track_caller]
pub fn wait_for(what: &str, cond: impl FnMut() -> bool + 'static) {
  assert!(run_until(WAIT, cond), "timed out waiting for {what}");
}

/// Run the event loop for `duration`.
pub fn pump(duration: Duration) {
  run_until(duration, || false);
}

/// An owned copy of an [`Event`].
#[derive(Debug, Clone, PartialEq)]
pub struct Recorded {
  pub kind: EventKind,
  pub window: u64,
  pub view: u64,
  pub a: i64,
  pub b: i64,
  pub c: i64,
  pub d: i64,
  pub f: f64,
  pub s1: String,
  pub s2: String,
}

impl From<&Event<'_>> for Recorded {
  fn from(e: &Event<'_>) -> Self {
    Self {
      kind: e.kind,
      window: e.window,
      view: e.view,
      a: e.a,
      b: e.b,
      c: e.c,
      d: e.d,
      f: e.f,
      s1: e.s1.to_owned(),
      s2: e.s2.to_owned(),
    }
  }
}

thread_local! {
  static EVENTS: RefCell<Vec<Recorded>> = const { RefCell::new(Vec::new()) };
}

/// Install an event handler that records every event (except `AboutToBlock`, which is very
/// frequent) and answers with `reply`.
pub fn record_events(reply: impl Fn(&Event<'_>) -> Reply + 'static) {
  app::set_event_handler(move |e| {
    if e.kind != EventKind::AboutToBlock {
      EVENTS.with(|ev| ev.borrow_mut().push(e.into()));
    }
    reply(e)
  });
}

/// The events recorded so far.
pub fn events() -> Vec<Recorded> {
  EVENTS.with(|e| e.borrow().clone())
}

/// Whether an event matching `pred` was recorded.
pub fn has_event(pred: impl Fn(&Recorded) -> bool) -> bool {
  EVENTS.with(|e| e.borrow().iter().any(pred))
}

/// Forget the recorded events.
pub fn clear_events() {
  EVENTS.with(|e| e.borrow_mut().clear());
}
