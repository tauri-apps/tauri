// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Wakes winit's event loop from any thread without re-entering its event handler.
//!
//! [`MainThreadWake`] does not signal winit's proxy itself. It pokes the main run loop and lets a
//! run-loop observer, registered in the common modes for the start of every pass, run a callback
//! on the main thread; the runtime signals the proxy from there once no `ApplicationHandler`
//! callback is running (see `RuntimeContext::wake_event_loop`).
//!
//! Signalling from that observer is what keeps winit's handler from being re-entered:
//! CoreFoundation collects the sources to perform right after the "before sources" observers
//! ran, so a proxy signalled there is performed in the same pass, before the loop polls for
//! AppKit events and dispatches one to a winit callback that might spin a nested loop.

use std::{ffi::c_void, ptr, sync::Arc};

use objc2::MainThreadMarker;
use objc2_core_foundation::{
  CFRetained, CFRunLoop, CFRunLoopActivity, CFRunLoopObserver, CFRunLoopObserverContext,
  kCFRunLoopCommonModes,
};

/// Runs on the main thread at the start of every run-loop pass.
type OnPass = dyn Fn() + Send + Sync;

/// Pokes the main run loop from any thread and runs `on_pass` on the main thread at the start
/// of each of its passes, including those of nested AppKit loops.
#[derive(Clone)]
pub(crate) struct MainThreadWake {
  inner: Arc<Inner>,
}

struct Inner {
  main_loop: CFRetained<CFRunLoop>,
  observer: CFRetained<CFRunLoopObserver>,
}

// SAFETY: run-loop functions are thread-safe per Apple's Threading Programming Guide, and the
// only ones called off the main thread are `CFRunLoopWakeUp` (its documented purpose) and
// `CFRunLoopObserverInvalidate` in `Drop`. The observer's callback runs on the main thread only
// and its state is a `Send + Sync` closure.
unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}

impl Drop for Inner {
  fn drop(&mut self) {
    self.observer.invalidate();
  }
}

/// The observer callout. `info` is the boxed closure handed to CoreFoundation in
/// [`MainThreadWake::new`].
unsafe extern "C-unwind" fn on_before_sources(
  _: *mut CFRunLoopObserver,
  _: CFRunLoopActivity,
  info: *mut c_void,
) {
  // SAFETY: `info` is released only when the observer is deallocated, which cannot happen
  // during its callout as CoreFoundation retains observers while calling them.
  let on_pass = unsafe { &*(info as *const Box<OnPass>) };
  on_pass();
}

unsafe extern "C-unwind" fn release(info: *const c_void) {
  // SAFETY: called once by CoreFoundation when the observer is deallocated, with the pointer
  // `MainThreadWake::new` leaked into the context.
  drop(unsafe { Box::from_raw(info as *mut Box<OnPass>) });
}

impl MainThreadWake {
  /// Registers the observer on the main run loop. Main thread only.
  pub(crate) fn new(on_pass: impl Fn() + Send + Sync + 'static) -> Self {
    debug_assert!(
      MainThreadMarker::new().is_some(),
      "MainThreadWake must be created on the main thread"
    );

    let on_pass: Box<OnPass> = Box::new(on_pass);
    let mut context = CFRunLoopObserverContext {
      version: 0,
      // Owned by the observer from here on; `release` frees it.
      info: Box::into_raw(Box::new(on_pass)) as *mut c_void,
      retain: None,
      release: Some(release),
      copyDescription: None,
    };

    // SAFETY: `context` outlives the call (it is copied), `info` is a valid pointer for the
    // observer's lifetime and `on_before_sources` only reads it as the type it was created as.
    let observer = unsafe {
      CFRunLoopObserver::new(
        None,
        CFRunLoopActivity::BeforeSources.0,
        true,
        0,
        Some(on_before_sources),
        ptr::addr_of_mut!(context),
      )
    }
    .expect("failed to create the wake-up run-loop observer");

    let main_loop = CFRunLoop::main().expect("no main run loop");
    // SAFETY: reading an extern static that CoreFoundation initializes.
    main_loop.add_observer(Some(&observer), unsafe { kCFRunLoopCommonModes });

    Self {
      inner: Arc::new(Inner {
        main_loop,
        observer,
      }),
    }
  }

  /// Makes the main run loop start a new pass soon, so that `on_pass` runs. Any thread.
  ///
  /// Calls coalesce: the loop wakes once however many times this ran before it did.
  pub(crate) fn wake_up(&self) {
    self.inner.main_loop.wake_up();
  }
}
