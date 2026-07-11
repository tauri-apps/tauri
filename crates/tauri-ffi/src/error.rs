// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Error model: every fallible extern fn returns an `i32` status code and
//! stores a detailed message in thread-local storage, readable via
//! `tauri_last_error_message` (borrowed, valid until the next failing FFI
//! call on the same thread).

use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::c_char;

pub const OK: i32 = 0;
pub const ERR_GENERIC: i32 = -1;
pub const ERR_INVALID_HANDLE: i32 = -2;
pub const ERR_TIMEOUT: i32 = -3;
pub const ERR_CLOSED: i32 = -4;
pub const ERR_INVALID_ARG: i32 = -5;
pub const ERR_UNSUPPORTED: i32 = -6;
pub const ERR_PANIC: i32 = -7;
pub const ERR_NOT_FOUND: i32 = -8;

/// Standard failure for a platform-gated function called on a platform it
/// does not support. Every gated function still exists on every platform, so
/// the exported symbol table — and therefore the ABI — is identical across
/// targets; only the behavior differs.
pub fn unsupported(function: &str, platforms: &str) -> i32 {
  fail(
    ERR_UNSUPPORTED,
    format!("{function} is only supported on {platforms}"),
  )
}

thread_local! {
  static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

pub fn set_last_error(message: impl Into<String>) {
  let message = message.into().replace('\0', " ");
  LAST_ERROR.with(|slot| *slot.borrow_mut() = CString::new(message).ok());
}

pub fn last_error_ptr() -> *const c_char {
  LAST_ERROR.with(|slot| {
    slot
      .borrow()
      .as_ref()
      .map_or(std::ptr::null(), |message| message.as_ptr())
  })
}

/// Record `message` and return `code` — the standard failure path.
pub fn fail(code: i32, message: impl Into<String>) -> i32 {
  set_last_error(message);
  code
}

/// Run an FFI body, converting panics into `ERR_PANIC` instead of unwinding
/// across the `extern "C"` boundary (undefined behavior).
pub fn catch(body: impl FnOnce() -> i32) -> i32 {
  match std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)) {
    Ok(code) => code,
    Err(panic) => {
      let message = panic
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| panic.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".into());
      fail(ERR_PANIC, format!("panic: {message}"))
    }
  }
}
