// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Linux/BSD backend for the CEF external message pump.
//!
//! Upstream cefclient:
//! <https://github.com/chromiumembedded/cef/blob/b41c5c64fc50871630678e3d0c8c9bfe77cc6353/tests/shared/browser/main_message_loop_external_pump_linux.cc>
//!
//! Mirrors cefclient's `main_message_loop_external_pump_linux.cc`: a custom
//! low-priority `GSource` is attached to the default GLib context, delayed work
//! is tracked as an absolute deadline, and cross-thread scheduling wakes the
//! GLib poll through a pipe.

use std::{
  mem,
  os::raw::{c_int, c_uint},
  sync::Weak,
  time::{Duration, Instant},
};

use gtk::glib::{self, ffi, translate::ToGlibPtr};

use super::PumpState;

#[repr(C)]
struct WorkSource {
  source: ffi::GSource,
  source_state: *mut SourceState,
}

struct SourceState {
  state: Weak<PumpState>,

  // The time when we need to do delayed work.
  delayed_work_time: Option<Instant>,

  // We use a wakeup pipe to make sure we'll get out of the glib polling phase
  // when another thread has scheduled us to do some work. There is a glib
  // mechanism g_main_context_wakeup, but this won't guarantee that our event's
  // Dispatch() will be called.
  wakeup_pipe_read: c_int,
  wakeup_pipe_write: c_int,

  // Boxed to keep the GPollFD address stable while it is registered with the
  // work source.
  wakeup_gpollfd: Box<ffi::GPollFD>,
}

pub(super) struct PlatformPump {
  // The work source. It is destroyed when the message pump is destroyed.
  work_source: *mut ffi::GSource,
  source_state: Box<SourceState>,
}

// SAFETY: `on_schedule_message_pump_work` is the only method called from other
// threads, and it only writes to the wakeup pipe. GLib source/timer state is
// mutated on the owner thread when GLib dispatches the attached source.
unsafe impl Send for PlatformPump {}

impl PlatformPump {
  pub(super) fn new(state: Weak<PumpState>) -> Self {
    // The runtime services callbacks from GLib's default MainContext.
    let context = glib::MainContext::default();

    // Create our wakeup pipe, which is used to flag when work was scheduled.
    let mut fds = [0; 2];
    let ret = unsafe { libc::pipe(fds.as_mut_ptr()) };
    assert_eq!(ret, 0, "failed to create CEF message pump wakeup pipe");

    let mut source_state = Box::new(SourceState {
      state,
      delayed_work_time: None,
      wakeup_pipe_read: fds[0],
      wakeup_pipe_write: fds[1],
      wakeup_gpollfd: Box::new(ffi::GPollFD {
        fd: fds[0],
        events: ffi::G_IO_IN as _,
        revents: 0,
      }),
    });

    let work_source = unsafe {
      let source = ffi::g_source_new(
        &raw mut WORK_SOURCE_FUNCS,
        mem::size_of::<WorkSource>() as c_uint,
      );
      assert!(
        !source.is_null(),
        "failed to create CEF message pump GSource"
      );

      (*(source as *mut WorkSource)).source_state = &mut *source_state;
      ffi::g_source_add_poll(source, &mut *source_state.wakeup_gpollfd);
      // Use a low priority so that we let other events in the queue go first.
      ffi::g_source_set_priority(source, ffi::G_PRIORITY_DEFAULT_IDLE);
      // This is needed to allow Run calls inside Dispatch.
      ffi::g_source_set_can_recurse(source, ffi::GTRUE);
      ffi::g_source_attach(source, context.to_glib_none().0);
      source
    };

    Self {
      work_source,
      source_state,
    }
  }

  pub(super) fn on_schedule_message_pump_work(&mut self, delay_ms: i64) {
    // This can be called on any thread, so we don't want to touch any state
    // variables as we would then need locks all over. This ensures that if we
    // are sleeping in a poll that we will wake up.
    let written = retry_eintr(|| unsafe {
      libc::write(
        self.source_state.wakeup_pipe_write,
        (&delay_ms as *const i64).cast(),
        mem::size_of::<i64>(),
      )
    });
    if written != mem::size_of::<i64>() as isize {
      log::error!("could not write to the CEF message pump wakeup pipe");
    }
  }

  pub(super) fn set_timer(&mut self, delay_ms: i64) {
    debug_assert!(delay_ms > 0);

    let now = Instant::now();
    self.source_state.delayed_work_time = Some(now + Duration::from_millis(delay_ms as u64));
  }

  pub(super) fn kill_timer(&mut self) {
    self.source_state.delayed_work_time = None;
  }

  pub(super) fn is_timer_pending(&self) -> bool {
    get_time_interval_milliseconds(self.source_state.delayed_work_time) > 0
  }

  pub(super) fn deadline(&self) -> Option<Instant> {
    self.source_state.delayed_work_time
  }
}

impl Drop for PlatformPump {
  fn drop(&mut self) {
    unsafe {
      ffi::g_source_destroy(self.work_source);
      ffi::g_source_unref(self.work_source);
      libc::close(self.source_state.wakeup_pipe_read);
      libc::close(self.source_state.wakeup_pipe_write);
    }
  }
}

// Return a timeout suitable for the glib loop, -1 to block forever,
// 0 to return right away, or a timeout in milliseconds from now.
fn get_time_interval_milliseconds(from: Option<Instant>) -> c_int {
  let Some(from) = from else {
    return -1;
  };

  // Be careful here. Instant has finer precision than milliseconds, but GLib
  // wants a value in milliseconds. If there are 5.5ms left, should the delay be
  // 5 or 6? It should be 6 to avoid executing delayed work too early.
  let now = Instant::now();
  let delay = from
    .checked_duration_since(now)
    .map(|duration| (duration.as_secs_f64() * 1000.0).ceil() as c_int)
    .unwrap_or(-1);

  // If this value is negative, then we need to run delayed work soon.
  if delay < 0 { 0 } else { delay }
}

// From base/posix/eintr_wrapper.h.
// This provides a wrapper around system calls which may be interrupted by a
// signal and return EINTR. See man 7 signal.
fn retry_eintr<F>(mut f: F) -> isize
where
  F: FnMut() -> isize,
{
  loop {
    let result = f();
    if result != -1 || std::io::Error::last_os_error().raw_os_error() != Some(libc::EINTR) {
      return result;
    }
  }
}

unsafe fn source_state(source: *mut ffi::GSource) -> *mut SourceState {
  unsafe { (*(source as *mut WorkSource)).source_state }
}

// Return the timeout we want passed to poll.
unsafe fn handle_prepare(source_state: *mut SourceState) -> c_int {
  // We don't think we have work to do, but make sure not to block longer than
  // the next time we need to run delayed work.
  let delayed_work_time = unsafe { (*source_state).delayed_work_time };
  get_time_interval_milliseconds(delayed_work_time)
}

unsafe fn handle_check(source_state: *mut SourceState) -> bool {
  // We usually have a single message on the wakeup pipe, since we are only
  // signaled when the queue went from empty to non-empty, but there can be
  // two messages if a task posted a task, hence we read at most two bytes.
  // The glib poll will tell us whether there was data, so this read shouldn't
  // block.
  let have_wakeup = {
    let wakeup_gpollfd = unsafe { &*(*source_state).wakeup_gpollfd };
    (wakeup_gpollfd.revents & ffi::G_IO_IN as u16) != 0
  };
  if have_wakeup {
    let mut delay_ms = [0_i64; 2];
    let num_bytes = retry_eintr(|| unsafe {
      libc::read(
        (*source_state).wakeup_pipe_read,
        delay_ms.as_mut_ptr().cast(),
        mem::size_of::<i64>() * 2,
      )
    });

    if num_bytes < mem::size_of::<i64>() as isize {
      log::error!("error reading from the CEF message pump wakeup pipe");
    }
    if num_bytes == mem::size_of::<i64>() as isize {
      if let Some(state) = unsafe { (*source_state).state.upgrade() } {
        state.on_schedule_work(delay_ms[0]);
      }
    }
    if num_bytes == (mem::size_of::<i64>() * 2) as isize {
      if let Some(state) = unsafe { (*source_state).state.upgrade() } {
        state.on_schedule_work(delay_ms[1]);
      }
    }
  }

  let delayed_work_time = unsafe { (*source_state).delayed_work_time };
  if get_time_interval_milliseconds(delayed_work_time) == 0 {
    // The timer has expired. That condition will stay true until we process
    // that delayed work, so we don't need to record this differently.
    return true;
  }

  false
}

unsafe fn handle_dispatch(source_state: *mut SourceState) {
  if let Some(state) = unsafe { (*source_state).state.upgrade() } {
    state.on_timer_timeout();
  }
}

unsafe extern "C" fn work_source_prepare(
  source: *mut ffi::GSource,
  timeout_ms: *mut c_int,
) -> ffi::gboolean {
  if !timeout_ms.is_null() {
    let source_state = unsafe { source_state(source) };
    unsafe { *timeout_ms = handle_prepare(source_state) };
  }

  // We always return FALSE, so that our timeout is honored.  If we were
  // to return TRUE, the timeout would be considered to be 0 and the poll
  // would never block.  Once the poll is finished, Check will be called.
  ffi::GFALSE
}

unsafe extern "C" fn work_source_check(source: *mut ffi::GSource) -> ffi::gboolean {
  let source_state = unsafe { source_state(source) };
  // Only return TRUE if Dispatch should be called.
  if unsafe { handle_check(source_state) } {
    ffi::GTRUE
  } else {
    ffi::GFALSE
  }
}

unsafe extern "C" fn work_source_dispatch(
  source: *mut ffi::GSource,
  _callback: ffi::GSourceFunc,
  _user_data: ffi::gpointer,
) -> ffi::gboolean {
  let source_state = unsafe { source_state(source) };
  unsafe { handle_dispatch(source_state) };
  // Always return TRUE so our source stays registered.
  ffi::GTRUE
}

// I wish these could be const, but g_source_new wants non-const.
static mut WORK_SOURCE_FUNCS: ffi::GSourceFuncs = ffi::GSourceFuncs {
  prepare: Some(work_source_prepare),
  check: Some(work_source_check),
  dispatch: Some(work_source_dispatch),
  finalize: None,
  closure_callback: None,
  closure_marshal: None,
};
