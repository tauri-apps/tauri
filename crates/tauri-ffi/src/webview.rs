// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `Webview` handles — a webview hosted inside a [`crate::window`] (multiwebview).
//! Create webviews with `tauri_window_add_webview`; each `tauri_webview_*`
//! function mirrors a method on [`tauri::Webview`]. For the common
//! single-webview case use [`crate::webview_window`].

use std::os::raw::c_char;

use crate::Rt as TauriRuntime;
use tauri::window::Color;
use tauri::{
  LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size, Webview,
};

use crate::error::{catch, fail, ERR_INVALID_ARG, ERR_INVALID_HANDLE, OK};
use crate::state::{self, Entry};
use crate::try_cstr;
use crate::winsupport::{act, ffi_action, get_pair, get_string};

const RESOLVE: fn(u64) -> Option<Webview<TauriRuntime>> = state::webview;

#[cfg(not(any(debug_assertions, feature = "devtools")))]
const DEVTOOLS_UNAVAILABLE: &str =
  "devtools not available; build tauri-ffi with the `devtools` feature";

// ---------------------------------------------------------------------------
// getters

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_label(webview: u64, out_label: *mut *mut c_char) -> i32 {
  catch(|| get_string(webview, out_label, RESOLVE, |w| Ok(w.label().to_string())))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_url(webview: u64, out_url: *mut *mut c_char) -> i32 {
  catch(|| {
    get_string(webview, out_url, RESOLVE, |w| {
      w.url().map(|u| u.to_string())
    })
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_position(
  webview: u64,
  out_x: *mut i32,
  out_y: *mut i32,
) -> i32 {
  catch(|| {
    get_pair(webview, out_x, out_y, RESOLVE, |w| {
      w.position().map(|p| (p.x, p.y))
    })
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_size(webview: u64, out_w: *mut u32, out_h: *mut u32) -> i32 {
  catch(|| {
    get_pair(webview, out_w, out_h, RESOLVE, |w| {
      w.size().map(|s| (s.width, s.height))
    })
  })
}

/// The parent window of this webview. On OK, `*out_window` is a bare window
/// handle; release with `tauri_handle_close`.
#[no_mangle]
pub unsafe extern "C" fn tauri_webview_get_window(webview: u64, out_window: *mut u64) -> i32 {
  catch(|| {
    if out_window.is_null() {
      return fail(ERR_INVALID_ARG, "out_window is null");
    }
    let Some(webview) = state::webview(webview) else {
      return fail(ERR_INVALID_HANDLE, "invalid webview handle");
    };
    unsafe { *out_window = state::insert(Entry::BareWindow(webview.window())) };
    OK
  })
}

// ---------------------------------------------------------------------------
// actions

ffi_action!(tauri_webview_reload, RESOLVE, reload);
ffi_action!(tauri_webview_print, RESOLVE, print);
ffi_action!(tauri_webview_set_focus, RESOLVE, set_focus);
ffi_action!(tauri_webview_show, RESOLVE, show);
ffi_action!(tauri_webview_hide, RESOLVE, hide);
ffi_action!(tauri_webview_close, RESOLVE, close);
ffi_action!(
  tauri_webview_clear_all_browsing_data,
  RESOLVE,
  clear_all_browsing_data
);

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_eval(webview: u64, js: *const c_char) -> i32 {
  catch(|| {
    let js = try_cstr!(js);
    act(webview, RESOLVE, |w| w.eval(js))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_navigate(webview: u64, url: *const c_char) -> i32 {
  catch(|| {
    let url = try_cstr!(url);
    let url: tauri::Url = match url.parse() {
      Ok(url) => url,
      Err(e) => return fail(ERR_INVALID_ARG, format!("invalid url: {e}")),
    };
    act(webview, RESOLVE, |w| w.navigate(url))
  })
}

#[no_mangle]
pub extern "C" fn tauri_webview_set_zoom(webview: u64, scale: f64) -> i32 {
  catch(|| act(webview, RESOLVE, |w| w.set_zoom(scale)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_set_auto_resize(webview: u64, auto_resize: bool) -> i32 {
  catch(|| act(webview, RESOLVE, |w| w.set_auto_resize(auto_resize)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_set_size(
  webview: u64,
  width: f64,
  height: f64,
  physical: bool,
) -> i32 {
  catch(|| {
    let size: Size = if physical {
      PhysicalSize::new(width.round() as u32, height.round() as u32).into()
    } else {
      LogicalSize::new(width, height).into()
    };
    act(webview, RESOLVE, |w| w.set_size(size))
  })
}

#[no_mangle]
pub extern "C" fn tauri_webview_set_position(webview: u64, x: f64, y: f64, physical: bool) -> i32 {
  catch(|| {
    let position: Position = if physical {
      PhysicalPosition::new(x.round() as i32, y.round() as i32).into()
    } else {
      LogicalPosition::new(x, y).into()
    };
    act(webview, RESOLVE, |w| w.set_position(position))
  })
}

#[no_mangle]
pub extern "C" fn tauri_webview_set_background_color(
  webview: u64,
  r: u32,
  g: u32,
  b: u32,
  a: u32,
) -> i32 {
  catch(|| {
    let color = Color(
      r.min(255) as u8,
      g.min(255) as u8,
      b.min(255) as u8,
      a.min(255) as u8,
    );
    act(webview, RESOLVE, |w| w.set_background_color(Some(color)))
  })
}

/// Reparents this webview into another window.
#[no_mangle]
pub extern "C" fn tauri_webview_reparent(webview: u64, window: u64) -> i32 {
  catch(|| {
    let Some(window) = state::bare_window(window) else {
      return fail(ERR_INVALID_HANDLE, "invalid window handle");
    };
    act(webview, RESOLVE, |w| w.reparent(&window))
  })
}

// ---------------------------------------------------------------------------
// devtools (gated; see crate features)

#[cfg(any(debug_assertions, feature = "devtools"))]
fn webview_open_devtools(webview: u64) -> i32 {
  act(webview, RESOLVE, |w| {
    w.open_devtools();
    Ok(())
  })
}
#[cfg(not(any(debug_assertions, feature = "devtools")))]
fn webview_open_devtools(webview: u64) -> i32 {
  let _ = webview;
  fail(crate::error::ERR_GENERIC, DEVTOOLS_UNAVAILABLE)
}

#[no_mangle]
pub extern "C" fn tauri_webview_open_devtools(webview: u64) -> i32 {
  catch(|| webview_open_devtools(webview))
}

#[cfg(any(debug_assertions, feature = "devtools"))]
fn webview_close_devtools(webview: u64) -> i32 {
  act(webview, RESOLVE, |w| {
    w.close_devtools();
    Ok(())
  })
}
#[cfg(not(any(debug_assertions, feature = "devtools")))]
fn webview_close_devtools(webview: u64) -> i32 {
  let _ = webview;
  fail(crate::error::ERR_GENERIC, DEVTOOLS_UNAVAILABLE)
}

#[no_mangle]
pub extern "C" fn tauri_webview_close_devtools(webview: u64) -> i32 {
  catch(|| webview_close_devtools(webview))
}

#[cfg(any(debug_assertions, feature = "devtools"))]
fn webview_is_devtools_open(webview: u64, out_open: *mut bool) -> i32 {
  crate::winsupport::get(webview, out_open, RESOLVE, |w| Ok(w.is_devtools_open()))
}
#[cfg(not(any(debug_assertions, feature = "devtools")))]
fn webview_is_devtools_open(webview: u64, out_open: *mut bool) -> i32 {
  let _ = (webview, out_open);
  fail(crate::error::ERR_GENERIC, DEVTOOLS_UNAVAILABLE)
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_is_devtools_open(webview: u64, out_open: *mut bool) -> i32 {
  catch(|| webview_is_devtools_open(webview, out_open))
}
