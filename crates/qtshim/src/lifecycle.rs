// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Which thread may use Qt, and whether Qt is running.
//!
//! Qt must only be used on the thread that created the `QApplication`, between
//! [`app::init`](crate::app::init) and [`app::shutdown`](crate::app::shutdown). Every safe entry
//! point checks this with [`ensure`] (and panics on misuse), so no call can reach Qt from the
//! wrong thread or after it's gone. The application is created at most once per process, so
//! handles that outlive `shutdown` stay unusable.
//!
//! The two entry points meant for other threads (`app::dispatch` and
//! `webengine::Responder::respond`) instead hold a read lock on [`ALIVE`] while they hand work to
//! Qt; `shutdown` takes the write lock before destroying the application, so they can't race.

use std::{
  cell::Cell,
  sync::{
    PoisonError, RwLock,
    atomic::{AtomicBool, Ordering},
  },
};

thread_local! {
  /// Set on the main thread between `init` and `shutdown`.
  static RUNNING_HERE: Cell<bool> = const { Cell::new(false) };
  /// How many Qt -> Rust callbacks are on the stack.
  static CALLBACK_DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// Whether the `QApplication` exists (any thread may read it).
static ALIVE: RwLock<bool> = RwLock::new(false);
/// Whether some thread has called `init`. Never reset: after `shutdown` the application can't be
/// created again, so Qt handles that outlived it can never pass [`ensure`] again.
static OWNED: AtomicBool = AtomicBool::new(false);

/// Whether this is the main thread and the application is running.
pub(crate) fn is_running_here() -> bool {
  RUNNING_HERE.with(Cell::get)
}

/// Panic unless this is the main thread and the application is running.
#[track_caller]
pub(crate) fn ensure() {
  if !is_running_here() {
    panic!(
      "qtshim: Qt can only be used on the thread that called `app::init`, before `app::shutdown`"
    );
  }
}

/// Make the current thread the main thread, before the `QApplication` is created. Panics if
/// another thread already is, or if the application was shut down.
#[track_caller]
pub(crate) fn start() {
  if is_running_here() {
    return;
  }
  if OWNED.swap(true, Ordering::AcqRel) {
    panic!("qtshim: `app::init` was already called (on another thread, or before `app::shutdown`)");
  }
  RUNNING_HERE.with(|r| r.set(true));
}

/// Let other threads hand work to the application, once the `QApplication` exists.
pub(crate) fn set_alive() {
  *ALIVE.write().unwrap_or_else(PoisonError::into_inner) = true;
}

/// Marks a Qt -> Rust callback as running until dropped.
pub(crate) struct CallbackGuard(());

impl Drop for CallbackGuard {
  fn drop(&mut self) {
    CALLBACK_DEPTH.with(|d| d.set(d.get() - 1));
  }
}

/// Call at the start of every function Qt calls back into, so [`stop`] can refuse to destroy
/// the application while Qt code is still on the stack.
pub(crate) fn enter_callback() -> CallbackGuard {
  CALLBACK_DEPTH.with(|d| d.set(d.get() + 1));
  CallbackGuard(())
}

/// Mark the application as gone, waiting for other threads to finish handing work to it.
///
/// Panics if called from inside a Qt callback (including anything run by the event loop).
#[track_caller]
pub(crate) fn stop() {
  if CALLBACK_DEPTH.with(Cell::get) > 0 {
    panic!("qtshim: `app::shutdown` can't be called from inside the event loop or a Qt callback");
  }
  *ALIVE.write().unwrap_or_else(PoisonError::into_inner) = false;
  RUNNING_HERE.with(|r| r.set(false));
}

/// Run `f` if the application is alive, keeping it alive meanwhile. For use from any thread.
pub(crate) fn while_alive<R>(f: impl FnOnce() -> R) -> Option<R> {
  let alive = ALIVE.read().unwrap_or_else(PoisonError::into_inner);
  alive.then(f)
}

#[cfg(test)]
mod tests {
  use crate::{app, menu::Widget};

  #[test]
  #[should_panic(expected = "Qt can only be used on the thread that called `app::init`")]
  fn qt_calls_panic_without_init() {
    app::screens();
  }

  #[test]
  #[should_panic(expected = "Qt can only be used on the thread that called `app::init`")]
  fn foreign_pointers_are_not_trusted_without_init() {
    Widget::from_ptr(std::ptr::dangling_mut());
  }

  #[test]
  fn dispatch_without_app_drops_the_closure() {
    assert!(!app::is_running());
    assert!(!app::dispatch(|| unreachable!()));
  }

  #[test]
  #[should_panic(expected = "`app::init` was already called")]
  fn the_application_is_created_once() {
    // Only flips the flags: no `QApplication` is involved.
    super::start();
    super::stop();
    super::start();
  }

  #[test]
  #[should_panic(expected = "event loop or a Qt callback")]
  fn shutdown_refuses_inside_callbacks() {
    let _guard = super::enter_callback();
    super::stop();
  }

  #[test]
  fn not_running_on_a_fresh_thread() {
    assert!(!super::is_running_here());
    assert!(!std::thread::spawn(super::is_running_here).join().unwrap());
  }

  #[test]
  #[should_panic(expected = "Qt can only be used on the thread that called `app::init`")]
  fn ensure_panics_without_init() {
    super::ensure();
  }

  #[test]
  fn while_alive_skips_without_app() {
    // No unit test makes the application alive (that would race with the other tests).
    assert_eq!(super::while_alive(|| unreachable!()), None::<()>);
  }

  #[test]
  fn callback_guards_nest() {
    let outer = super::enter_callback();
    let inner = super::enter_callback();
    drop(inner);
    // Still inside `outer`.
    let refused = std::panic::catch_unwind(super::stop).is_err();
    drop(outer);
    assert!(refused);
    // All guards gone: `stop` only resets this thread's flags (no application here).
    super::stop();
    assert!(!super::is_running_here());
  }

  #[test]
  fn callback_guard_released_on_unwind() {
    let r = std::panic::catch_unwind(|| {
      let _guard = super::enter_callback();
      panic!("inside a callback");
    });
    assert!(r.is_err());
    super::stop();
  }
}
