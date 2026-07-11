// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Generic handle-op helpers + declarative macros shared by the bare `Window`
//! ([`crate::window`]) and `Webview` ([`crate::webview`]) modules, so their
//! many thin getter/setter/action FFI functions aren't hand-duplicated. Each
//! macro takes the extern fn name, the `state` resolver for the handle type,
//! and the method to call.

use std::os::raw::c_char;

use crate::error::{fail, ERR_GENERIC, ERR_INVALID_ARG, ERR_INVALID_HANDLE, OK};
use crate::write_owned_str;

/// Runs `f` on the object resolved from `handle`, mapping `Ok(())`/`Err` to a
/// status code. Used for actions and setters.
pub fn act<O>(
  handle: u64,
  resolve: impl Fn(u64) -> Option<O>,
  f: impl FnOnce(&O) -> tauri::Result<()>,
) -> i32 {
  let Some(obj) = resolve(handle) else {
    return fail(ERR_INVALID_HANDLE, "invalid handle");
  };
  match f(&obj) {
    Ok(()) => OK,
    Err(e) => fail(ERR_GENERIC, e.to_string()),
  }
}

/// Writes a scalar result of `f` to `out`.
pub fn get<T: Copy, O>(
  handle: u64,
  out: *mut T,
  resolve: impl Fn(u64) -> Option<O>,
  f: impl FnOnce(&O) -> tauri::Result<T>,
) -> i32 {
  if out.is_null() {
    return fail(ERR_INVALID_ARG, "output pointer is null");
  }
  let Some(obj) = resolve(handle) else {
    return fail(ERR_INVALID_HANDLE, "invalid handle");
  };
  match f(&obj) {
    Ok(value) => {
      unsafe { *out = value };
      OK
    }
    Err(e) => fail(ERR_GENERIC, e.to_string()),
  }
}

/// Writes an owned string result of `f` to `out`.
pub fn get_string<O>(
  handle: u64,
  out: *mut *mut c_char,
  resolve: impl Fn(u64) -> Option<O>,
  f: impl FnOnce(&O) -> tauri::Result<String>,
) -> i32 {
  if out.is_null() {
    return fail(ERR_INVALID_ARG, "output pointer is null");
  }
  let Some(obj) = resolve(handle) else {
    return fail(ERR_INVALID_HANDLE, "invalid handle");
  };
  match f(&obj) {
    Ok(value) => {
      write_owned_str(out, value);
      OK
    }
    Err(e) => fail(ERR_GENERIC, e.to_string()),
  }
}

/// Writes a pair result of `f` to `out_a`/`out_b`.
pub fn get_pair<T: Copy, O>(
  handle: u64,
  out_a: *mut T,
  out_b: *mut T,
  resolve: impl Fn(u64) -> Option<O>,
  f: impl FnOnce(&O) -> tauri::Result<(T, T)>,
) -> i32 {
  if out_a.is_null() || out_b.is_null() {
    return fail(ERR_INVALID_ARG, "output pointer is null");
  }
  let Some(obj) = resolve(handle) else {
    return fail(ERR_INVALID_HANDLE, "invalid handle");
  };
  match f(&obj) {
    Ok((a, b)) => {
      unsafe {
        *out_a = a;
        *out_b = b;
      }
      OK
    }
    Err(e) => fail(ERR_GENERIC, e.to_string()),
  }
}

/// Borrowed RGBA pixels → `tauri::image::Image`. A null `rgba` means "clear
/// the icon" (`Ok(None)`); mismatched or overflowing dimensions are rejected.
#[cfg(target_os = "windows")]
pub unsafe fn rgba_image<'a>(
  rgba: *const u8,
  width: u32,
  height: u32,
) -> Result<Option<tauri::image::Image<'a>>, i32> {
  if rgba.is_null() {
    return Ok(None);
  }
  let len = (width as usize)
    .checked_mul(height as usize)
    .and_then(|pixels| pixels.checked_mul(4))
    .filter(|len| *len > 0)
    .ok_or_else(|| fail(ERR_INVALID_ARG, "invalid icon dimensions"))?;
  let pixels = unsafe { std::slice::from_raw_parts(rgba, len) };
  Ok(Some(tauri::image::Image::new(pixels, width, height)))
}

/// `fn(handle, out_bool) -> status` calling `obj.method() -> Result<bool>`.
macro_rules! ffi_getter_bool {
  ($fn:ident, $resolve:path, $method:ident) => {
    #[no_mangle]
    pub unsafe extern "C" fn $fn(handle: u64, out: *mut bool) -> i32 {
      $crate::error::catch(|| {
        $crate::winsupport::get(handle, out, $resolve, |o| o.$method())
      })
    }
  };
}

/// `fn(handle, out_owned_str) -> status` calling `obj.method() -> Result<String>`.
macro_rules! ffi_getter_string {
  ($fn:ident, $resolve:path, $method:ident) => {
    #[no_mangle]
    pub unsafe extern "C" fn $fn(handle: u64, out: *mut *mut std::os::raw::c_char) -> i32 {
      $crate::error::catch(|| $crate::winsupport::get_string(handle, out, $resolve, |o| o.$method()))
    }
  };
}

/// `fn(handle) -> status` calling `obj.method() -> Result<()>`.
macro_rules! ffi_action {
  ($fn:ident, $resolve:path, $method:ident) => {
    #[no_mangle]
    pub extern "C" fn $fn(handle: u64) -> i32 {
      $crate::error::catch(|| $crate::winsupport::act(handle, $resolve, |o| o.$method()))
    }
  };
}

/// `fn(handle, value: bool) -> status` calling `obj.method(value) -> Result<()>`.
macro_rules! ffi_setter_bool {
  ($fn:ident, $resolve:path, $method:ident) => {
    #[no_mangle]
    pub extern "C" fn $fn(handle: u64, value: bool) -> i32 {
      $crate::error::catch(|| $crate::winsupport::act(handle, $resolve, |o| o.$method(value)))
    }
  };
}

pub(crate) use {ffi_action, ffi_getter_bool, ffi_getter_string, ffi_setter_bool};
