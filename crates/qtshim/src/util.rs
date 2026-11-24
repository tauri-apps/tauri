// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Small helpers shared by the modules (not public API).

#![allow(dead_code)] // some helpers are only used with the `webengine` feature

use std::ffi::{CStr, CString, c_char, c_int, c_void};

/// Run `f`, turning a panic into `None` (with a message naming `what`). Every function Qt calls
/// back into uses this: a panic can't unwind through Qt's C++ frames and would abort.
pub(crate) fn catch_panic<R>(what: &str, f: impl FnOnce() -> R) -> Option<R> {
  let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
  if r.is_err() {
    eprintln!("qtshim: panic in {what}");
  }
  r.ok()
}

/// `(width, height)` as C ints if `len` bytes hold a `width` x `height` RGBA image Qt can take:
/// a row (`width * 4` bytes) and the height must fit in an `int`, and nothing may overflow.
pub(crate) fn rgba_dims(len: usize, width: u32, height: u32) -> Option<(c_int, c_int)> {
  let row = u64::from(width) * 4; // can't overflow
  let needed = row.checked_mul(u64::from(height))?;
  (width > 0
    && height > 0
    && row <= i32::MAX as u64
    && height <= i32::MAX as u32
    && needed <= len as u64)
    .then_some((width as c_int, height as c_int))
}

pub(crate) unsafe extern "C" fn drop_box<T>(ud: *mut c_void) {
  drop(unsafe { Box::from_raw(ud as *mut T) });
}

pub(crate) fn boxed<T>(t: T) -> *mut c_void {
  Box::into_raw(Box::new(t)) as *mut c_void
}

pub(crate) unsafe fn str_arg<'a>(s: *const c_char, len: usize) -> &'a str {
  if s.is_null() || len == 0 {
    return "";
  }
  let bytes = unsafe { std::slice::from_raw_parts(s as *const u8, len) };
  std::str::from_utf8(bytes).unwrap_or("")
}

pub(crate) fn cstring(s: &str) -> CString {
  CString::new(s.replace('\0', "")).unwrap()
}

pub(crate) unsafe fn take_cstring(p: *mut c_char) -> String {
  if p.is_null() {
    return String::new();
  }
  let s = unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned();
  unsafe { qtshim_sys::qs_free(p as *mut c_void) };
  s
}

#[cfg(test)]
mod tests {
  use std::{cell::Cell, rc::Rc};

  use super::*;

  #[test]
  fn catch_panic_passes_values_through() {
    assert_eq!(catch_panic("a test", || 42), Some(42));
  }

  #[test]
  fn catch_panic_swallows_panics() {
    assert_eq!(catch_panic("a test", || -> i32 { panic!("boom") }), None);
    // String payloads too.
    assert_eq!(
      catch_panic("a test", || -> () { panic!("{}", String::from("boom")) }),
      None
    );
  }

  #[test]
  fn boxed_and_drop_box_round_trip() {
    struct Flag(Rc<Cell<bool>>);
    impl Drop for Flag {
      fn drop(&mut self) {
        self.0.set(true);
      }
    }
    let dropped = Rc::new(Cell::new(false));
    let p = boxed(Flag(dropped.clone()));
    assert!(!p.is_null());
    assert!(!dropped.get());
    unsafe { drop_box::<Flag>(p) };
    assert!(dropped.get());
  }

  #[test]
  fn str_arg_handles_null_empty_and_invalid() {
    unsafe {
      assert_eq!(str_arg(std::ptr::null(), 5), "");
      let s = b"hello";
      assert_eq!(str_arg(s.as_ptr() as *const c_char, 0), "");
      assert_eq!(str_arg(s.as_ptr() as *const c_char, 5), "hello");
      // Only `len` bytes, no NUL needed.
      assert_eq!(str_arg(s.as_ptr() as *const c_char, 3), "hel");
      let bad = [0xffu8, 0xfe];
      assert_eq!(str_arg(bad.as_ptr() as *const c_char, 2), "");
      let utf8 = "héllo";
      assert_eq!(str_arg(utf8.as_ptr() as *const c_char, utf8.len()), "héllo");
    }
  }

  #[test]
  fn cstring_strips_nuls() {
    assert_eq!(cstring("abc").as_bytes(), b"abc");
    assert_eq!(cstring("a\0b\0").as_bytes(), b"ab");
    assert_eq!(cstring("").as_bytes(), b"");
  }

  #[test]
  fn take_cstring_of_null_is_empty() {
    assert_eq!(unsafe { take_cstring(std::ptr::null_mut()) }, "");
  }
}
