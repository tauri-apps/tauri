// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Holds fullscreen requests back while the window is animating a fullscreen transition.
//!
//! `toggleFullScreen:` is a no-op while AppKit is still animating the previous transition of
//! the window, and winit's macOS backend does not cope with that: a `set_fullscreen` call
//! that arrives during a transition is parked and replayed synchronously from inside
//! `windowDidEnterFullScreen:` / `windowDidExitFullScreen:`, where AppKit still drops the
//! toggle. Its state then disagrees with the window's — an enter is reverted by
//! `windowDidFailToEnterFullScreen:`, while a dropped exit leaves winit reporting a windowed
//! state for a window that is still fullscreen, and the next request toggles the wrong way.
//!
//! So the runtime does not hand winit a request while a transition is in flight. The
//! transitions are tracked through the window's own notifications, the latest request made
//! during one is held here, and once the transition ended it is re-issued through the message
//! channel, which lands in `handle_window_message` on a later pass of the run loop — the same
//! deferral tao gets from dispatching the toggle asynchronously.

use std::cell::Cell;

use objc2::{
  DefinedClass, MainThreadOnly, define_class,
  rc::Retained,
  runtime::{AnyObject, NSObject, NSObjectProtocol},
  sel,
};
use objc2_app_kit::{
  NSWindow, NSWindowDidEnterFullScreenNotification, NSWindowDidExitFullScreenNotification,
  NSWindowWillEnterFullScreenNotification, NSWindowWillExitFullScreenNotification,
};
use objc2_foundation::{
  MainThreadMarker, NSNotification, NSNotificationCenter, NSObjectNSDelayedPerforming,
};

use crate::window::FullscreenTarget;

/// A transition that has not finished after this many seconds is considered over. AppKit's
/// animation takes well under a second; this only guards against a `Did…` notification that
/// never comes (AppKit posts none when it fails to enter fullscreen), so that a request is not
/// held forever.
const TRANSITION_TIMEOUT: f64 = 3.0;

pub(crate) struct FullscreenTransitionIvars {
  /// Whether a transition is in flight.
  in_progress: Cell<bool>,
  /// The latest request made during the transition; `Some(None)` is a request to leave.
  pending: Cell<Option<Option<FullscreenTarget>>>,
  /// Called when a transition ends while a request is pending. Runs inside AppKit's
  /// notification dispatch, so it must only post the request for later.
  on_transition_end: Box<dyn Fn()>,
}

define_class!(
  #[unsafe(super(NSObject))]
  #[name = "TauriCefFullscreenTransition"]
  #[ivars = FullscreenTransitionIvars]
  #[thread_kind = MainThreadOnly]
  pub(crate) struct FullscreenTransition;

  unsafe impl NSObjectProtocol for FullscreenTransition {}

  impl FullscreenTransition {
    #[unsafe(method(windowWillChangeFullScreen:))]
    fn window_will_change_fullscreen(&self, _: &NSNotification) {
      self.ivars().in_progress.set(true);
      // winit retries an initial fullscreen AppKit failed to enter, so `Will…` can repeat
      // before any `Did…`; only the latest attempt's timeout counts.
      self.cancel_timeout();
      // SAFETY: `transitionTimedOut:` is implemented below and takes no argument it reads.
      unsafe {
        self.performSelector_withObject_afterDelay(
          sel!(transitionTimedOut:),
          None,
          TRANSITION_TIMEOUT,
        );
      }
    }

    #[unsafe(method(windowDidChangeFullScreen:))]
    fn window_did_change_fullscreen(&self, _: &NSNotification) {
      self.cancel_timeout();
      self.transition_ended();
    }

    #[unsafe(method(transitionTimedOut:))]
    fn transition_timed_out(&self, _: Option<&AnyObject>) {
      self.transition_ended();
    }
  }
);

impl FullscreenTransition {
  /// Starts tracking `nswindow`'s fullscreen transitions. The observer is removed when it is
  /// deallocated, which happens when the returned reference is dropped.
  pub(crate) fn observe(
    nswindow: &NSWindow,
    on_transition_end: impl Fn() + 'static,
  ) -> Retained<Self> {
    // The window is created by winit on the main thread.
    let mtm = MainThreadMarker::from(nswindow);
    let observer = Self::alloc(mtm).set_ivars(FullscreenTransitionIvars {
      in_progress: Cell::new(false),
      pending: Cell::new(None),
      on_transition_end: Box::new(on_transition_end),
    });
    let observer: Retained<Self> = unsafe { objc2::msg_send![super(observer), init] };

    let center = NSNotificationCenter::defaultCenter();
    let names = unsafe {
      [
        (
          NSWindowWillEnterFullScreenNotification,
          sel!(windowWillChangeFullScreen:),
        ),
        (
          NSWindowWillExitFullScreenNotification,
          sel!(windowWillChangeFullScreen:),
        ),
        (
          NSWindowDidEnterFullScreenNotification,
          sel!(windowDidChangeFullScreen:),
        ),
        (
          NSWindowDidExitFullScreenNotification,
          sel!(windowDidChangeFullScreen:),
        ),
      ]
    };
    for (name, selector) in names {
      // SAFETY: `observer` implements both selectors, and they take the notification as
      // their only argument.
      unsafe {
        center.addObserver_selector_name_object(&observer, selector, Some(name), Some(nswindow));
      }
    }

    observer
  }

  fn cancel_timeout(&self) {
    // SAFETY: cancels the delayed perform scheduled in `windowWillChangeFullScreen:`, which
    // targets this object.
    unsafe {
      NSObject::cancelPreviousPerformRequestsWithTarget_selector_object(
        self,
        sel!(transitionTimedOut:),
        None,
      );
    }
  }

  fn transition_ended(&self) {
    self.ivars().in_progress.set(false);
    if self.ivars().pending.get().is_some() {
      (self.ivars().on_transition_end)();
    }
  }

  /// Whether the window is in the middle of entering or leaving fullscreen.
  pub(crate) fn in_progress(&self) -> bool {
    self.ivars().in_progress.get()
  }

  /// Keeps `target` until the transition in progress ends, replacing any earlier request.
  pub(crate) fn hold(&self, target: Option<FullscreenTarget>) {
    self.ivars().pending.set(Some(target));
  }

  /// The request held during the last transition, if any.
  pub(crate) fn take_pending(&self) -> Option<Option<FullscreenTarget>> {
    self.ivars().pending.take()
  }
}
