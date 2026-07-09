// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Window creation and `WebviewWindow` methods. Windows are opaque handles;
//! each `tauri_window_*` function mirrors the equivalent method on
//! [`tauri::WebviewWindow`]. All functions are callable from any thread —
//! operations dispatch through the running event loop.

use std::os::raw::c_char;

use tauri::utils::config::WindowConfig;
use tauri::{
  LogicalPosition, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Position, Size,
  WebviewWindow, Wry,
};

use crate::error::{
  catch, fail, ERR_GENERIC, ERR_INVALID_ARG, ERR_INVALID_HANDLE, ERR_NOT_FOUND, OK,
};
use crate::state::{self, Entry};
use crate::{try_cstr, write_owned_str};

// ---------------------------------------------------------------------------
// helpers

fn with_window(window: u64, f: impl FnOnce(&WebviewWindow<Wry>) -> tauri::Result<()>) -> i32 {
  let Some(window) = state::window(window) else {
    return fail(ERR_INVALID_HANDLE, "invalid window handle");
  };
  match f(&window) {
    Ok(()) => OK,
    Err(e) => fail(ERR_GENERIC, e.to_string()),
  }
}

fn window_get<T>(
  window: u64,
  out: *mut T,
  f: impl FnOnce(&WebviewWindow<Wry>) -> tauri::Result<T>,
) -> i32 {
  if out.is_null() {
    return fail(ERR_INVALID_ARG, "output pointer is null");
  }
  let Some(window) = state::window(window) else {
    return fail(ERR_INVALID_HANDLE, "invalid window handle");
  };
  match f(&window) {
    Ok(value) => {
      unsafe { *out = value };
      OK
    }
    Err(e) => fail(ERR_GENERIC, e.to_string()),
  }
}

fn window_get_string(
  window: u64,
  out: *mut *mut c_char,
  f: impl FnOnce(&WebviewWindow<Wry>) -> tauri::Result<String>,
) -> i32 {
  if out.is_null() {
    return fail(ERR_INVALID_ARG, "output pointer is null");
  }
  let Some(window) = state::window(window) else {
    return fail(ERR_INVALID_HANDLE, "invalid window handle");
  };
  match f(&window) {
    Ok(value) => {
      write_owned_str(out, value);
      OK
    }
    Err(e) => fail(ERR_GENERIC, e.to_string()),
  }
}

fn window_get_pair<T: Copy>(
  window: u64,
  out_a: *mut T,
  out_b: *mut T,
  f: impl FnOnce(&WebviewWindow<Wry>) -> tauri::Result<(T, T)>,
) -> i32 {
  if out_a.is_null() || out_b.is_null() {
    return fail(ERR_INVALID_ARG, "output pointer is null");
  }
  let Some(window) = state::window(window) else {
    return fail(ERR_INVALID_HANDLE, "invalid window handle");
  };
  match f(&window) {
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

// ---------------------------------------------------------------------------
// creation & lookup

/// Creates a webview window from a `WindowConfig` JSON object. Blocks until
/// the running event loop has created it — call only while the app runs.
#[no_mangle]
pub unsafe extern "C" fn tauri_window_create(
  app: u64,
  config_json: *const c_char,
  out_window: *mut u64,
) -> i32 {
  catch(|| {
    let json = try_cstr!(config_json);
    if out_window.is_null() {
      return fail(ERR_INVALID_ARG, "out_window is null");
    }
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    let config: WindowConfig = match serde_json::from_str(json) {
      Ok(config) => config,
      Err(e) => return fail(ERR_INVALID_ARG, format!("invalid window config: {e}")),
    };
    let built = tauri::webview::WebviewWindowBuilder::from_config(&app_state.handle, &config)
      .and_then(|builder| builder.build());
    match built {
      Ok(window) => {
        unsafe { *out_window = state::insert(Entry::Window(window)) };
        OK
      }
      Err(e) => fail(ERR_GENERIC, format!("failed to create window: {e}")),
    }
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_app_get_window(
  app: u64,
  label: *const c_char,
  out_window: *mut u64,
) -> i32 {
  catch(|| {
    let label = try_cstr!(label);
    if out_window.is_null() {
      return fail(ERR_INVALID_ARG, "out_window is null");
    }
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    match app_state.handle.get_webview_window(label) {
      Some(window) => {
        unsafe { *out_window = state::insert(Entry::Window(window)) };
        OK
      }
      None => fail(ERR_NOT_FOUND, format!("no webview window labeled `{label}`")),
    }
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_app_window_labels(app: u64, out_labels_json: *mut *mut c_char) -> i32 {
  catch(|| {
    if out_labels_json.is_null() {
      return fail(ERR_INVALID_ARG, "out_labels_json is null");
    }
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    let labels: Vec<String> = app_state.handle.webview_windows().keys().cloned().collect();
    write_owned_str(
      out_labels_json,
      serde_json::to_string(&labels).unwrap_or_else(|_| "[]".into()),
    );
    OK
  })
}

// ---------------------------------------------------------------------------
// getters

#[no_mangle]
pub unsafe extern "C" fn tauri_window_label(window: u64, out_label: *mut *mut c_char) -> i32 {
  catch(|| window_get_string(window, out_label, |w| Ok(w.label().to_string())))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_title(window: u64, out_title: *mut *mut c_char) -> i32 {
  catch(|| window_get_string(window, out_title, |w| w.title()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_url(window: u64, out_url: *mut *mut c_char) -> i32 {
  catch(|| window_get_string(window, out_url, |w| w.url().map(|url| url.to_string())))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_scale_factor(window: u64, out_scale: *mut f64) -> i32 {
  catch(|| window_get(window, out_scale, |w| w.scale_factor()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_inner_size(
  window: u64,
  out_width: *mut u32,
  out_height: *mut u32,
) -> i32 {
  catch(|| {
    window_get_pair(window, out_width, out_height, |w| {
      w.inner_size().map(|size| (size.width, size.height))
    })
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_outer_size(
  window: u64,
  out_width: *mut u32,
  out_height: *mut u32,
) -> i32 {
  catch(|| {
    window_get_pair(window, out_width, out_height, |w| {
      w.outer_size().map(|size| (size.width, size.height))
    })
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_inner_position(
  window: u64,
  out_x: *mut i32,
  out_y: *mut i32,
) -> i32 {
  catch(|| {
    window_get_pair(window, out_x, out_y, |w| {
      w.inner_position().map(|position| (position.x, position.y))
    })
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_outer_position(
  window: u64,
  out_x: *mut i32,
  out_y: *mut i32,
) -> i32 {
  catch(|| {
    window_get_pair(window, out_x, out_y, |w| {
      w.outer_position().map(|position| (position.x, position.y))
    })
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_is_visible(window: u64, out_visible: *mut bool) -> i32 {
  catch(|| window_get(window, out_visible, |w| w.is_visible()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_is_focused(window: u64, out_focused: *mut bool) -> i32 {
  catch(|| window_get(window, out_focused, |w| w.is_focused()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_is_fullscreen(window: u64, out_fullscreen: *mut bool) -> i32 {
  catch(|| window_get(window, out_fullscreen, |w| w.is_fullscreen()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_is_maximized(window: u64, out_maximized: *mut bool) -> i32 {
  catch(|| window_get(window, out_maximized, |w| w.is_maximized()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_is_minimized(window: u64, out_minimized: *mut bool) -> i32 {
  catch(|| window_get(window, out_minimized, |w| w.is_minimized()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_is_resizable(window: u64, out_resizable: *mut bool) -> i32 {
  catch(|| window_get(window, out_resizable, |w| w.is_resizable()))
}

// ---------------------------------------------------------------------------
// setters & actions

#[no_mangle]
pub unsafe extern "C" fn tauri_window_set_title(window: u64, title: *const c_char) -> i32 {
  catch(|| {
    let title = try_cstr!(title);
    with_window(window, |w| w.set_title(title))
  })
}

#[no_mangle]
pub extern "C" fn tauri_window_set_size(window: u64, width: f64, height: f64, physical: bool) -> i32 {
  catch(|| {
    let size: Size = if physical {
      PhysicalSize::new(width.round() as u32, height.round() as u32).into()
    } else {
      LogicalSize::new(width, height).into()
    };
    with_window(window, |w| w.set_size(size))
  })
}

#[no_mangle]
pub extern "C" fn tauri_window_set_position(window: u64, x: f64, y: f64, physical: bool) -> i32 {
  catch(|| {
    let position: Position = if physical {
      PhysicalPosition::new(x.round() as i32, y.round() as i32).into()
    } else {
      LogicalPosition::new(x, y).into()
    };
    with_window(window, |w| w.set_position(position))
  })
}

#[no_mangle]
pub extern "C" fn tauri_window_set_fullscreen(window: u64, fullscreen: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_fullscreen(fullscreen)))
}

#[no_mangle]
pub extern "C" fn tauri_window_set_resizable(window: u64, resizable: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_resizable(resizable)))
}

#[no_mangle]
pub extern "C" fn tauri_window_set_always_on_top(window: u64, always_on_top: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_always_on_top(always_on_top)))
}

#[no_mangle]
pub extern "C" fn tauri_window_set_decorations(window: u64, decorations: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_decorations(decorations)))
}

#[no_mangle]
pub extern "C" fn tauri_window_set_focus(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.set_focus()))
}

#[no_mangle]
pub extern "C" fn tauri_window_set_zoom(window: u64, scale: f64) -> i32 {
  catch(|| with_window(window, |w| w.set_zoom(scale)))
}

#[no_mangle]
pub extern "C" fn tauri_window_show(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.show()))
}

#[no_mangle]
pub extern "C" fn tauri_window_hide(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.hide()))
}

#[no_mangle]
pub extern "C" fn tauri_window_center(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.center()))
}

#[no_mangle]
pub extern "C" fn tauri_window_maximize(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.maximize()))
}

#[no_mangle]
pub extern "C" fn tauri_window_unmaximize(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.unmaximize()))
}

#[no_mangle]
pub extern "C" fn tauri_window_minimize(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.minimize()))
}

#[no_mangle]
pub extern "C" fn tauri_window_unminimize(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.unminimize()))
}

#[no_mangle]
pub extern "C" fn tauri_window_close(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.close()))
}

#[no_mangle]
pub extern "C" fn tauri_window_destroy(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.destroy()))
}

// ---------------------------------------------------------------------------
// webview

#[no_mangle]
pub unsafe extern "C" fn tauri_window_eval(window: u64, js: *const c_char) -> i32 {
  catch(|| {
    let js = try_cstr!(js);
    with_window(window, |w| w.eval(js))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_navigate(window: u64, url: *const c_char) -> i32 {
  catch(|| {
    let url = try_cstr!(url);
    let url: tauri::Url = match url.parse() {
      Ok(url) => url,
      Err(e) => return fail(ERR_INVALID_ARG, format!("invalid url: {e}")),
    };
    with_window(window, |w| w.navigate(url))
  })
}

#[no_mangle]
pub extern "C" fn tauri_window_reload(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.reload()))
}
