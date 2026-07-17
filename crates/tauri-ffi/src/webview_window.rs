// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Window creation and `WebviewWindow` methods. Windows are opaque handles;
//! each `tauri_webview_window_*` function mirrors the equivalent method on
//! [`tauri::WebviewWindow`]. All functions are callable from any thread —
//! operations dispatch through the running event loop.

use std::os::raw::c_char;

use crate::Rt as TauriRuntime;
use tauri::utils::config::{WindowConfig, WindowEffectsConfig};
use tauri::window::{Color, ProgressBarState};
use tauri::{
  CursorIcon, LogicalPosition, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Position,
  Size, Theme, UserAttentionType, WebviewWindow, WindowSizeConstraints,
};

use crate::error::{
  catch, fail, unsupported, ERR_GENERIC, ERR_INVALID_ARG, ERR_INVALID_HANDLE, ERR_NOT_FOUND, OK,
};
use crate::state::{self, Entry};
use crate::{try_cstr, write_owned_str};

// ---------------------------------------------------------------------------
// helpers

fn with_window(
  window: u64,
  f: impl FnOnce(&WebviewWindow<TauriRuntime>) -> tauri::Result<()>,
) -> i32 {
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
  f: impl FnOnce(&WebviewWindow<TauriRuntime>) -> tauri::Result<T>,
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
  f: impl FnOnce(&WebviewWindow<TauriRuntime>) -> tauri::Result<String>,
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
  f: impl FnOnce(&WebviewWindow<TauriRuntime>) -> tauri::Result<(T, T)>,
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

/// Builds a `Size` from width/height, or `None` when either is non-positive
/// (used to clear min/max size constraints).
fn optional_size(width: f64, height: f64, physical: bool) -> Option<Size> {
  if width <= 0.0 || height <= 0.0 {
    return None;
  }
  Some(if physical {
    PhysicalSize::new(width.round() as u32, height.round() as u32).into()
  } else {
    LogicalSize::new(width, height).into()
  })
}

// ---------------------------------------------------------------------------
// creation & lookup

/// Creates a webview window from a `WindowConfig` JSON object. Blocks until
/// the running event loop has created it — call only while the app runs.
#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_create(
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
pub unsafe extern "C" fn tauri_app_get_webview_window(
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
      None => fail(
        ERR_NOT_FOUND,
        format!("no webview window labeled `{label}`"),
      ),
    }
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_app_webview_window_labels(
  app: u64,
  out_labels_json: *mut *mut c_char,
) -> i32 {
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
pub unsafe extern "C" fn tauri_webview_window_label(
  window: u64,
  out_label: *mut *mut c_char,
) -> i32 {
  catch(|| window_get_string(window, out_label, |w| Ok(w.label().to_string())))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_title(
  window: u64,
  out_title: *mut *mut c_char,
) -> i32 {
  catch(|| window_get_string(window, out_title, |w| w.title()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_url(window: u64, out_url: *mut *mut c_char) -> i32 {
  catch(|| window_get_string(window, out_url, |w| w.url().map(|url| url.to_string())))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_scale_factor(
  window: u64,
  out_scale: *mut f64,
) -> i32 {
  catch(|| window_get(window, out_scale, |w| w.scale_factor()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_inner_size(
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
pub unsafe extern "C" fn tauri_webview_window_outer_size(
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
pub unsafe extern "C" fn tauri_webview_window_inner_position(
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
pub unsafe extern "C" fn tauri_webview_window_outer_position(
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
pub unsafe extern "C" fn tauri_webview_window_is_visible(
  window: u64,
  out_visible: *mut bool,
) -> i32 {
  catch(|| window_get(window, out_visible, |w| w.is_visible()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_is_focused(
  window: u64,
  out_focused: *mut bool,
) -> i32 {
  catch(|| window_get(window, out_focused, |w| w.is_focused()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_is_fullscreen(
  window: u64,
  out_fullscreen: *mut bool,
) -> i32 {
  catch(|| window_get(window, out_fullscreen, |w| w.is_fullscreen()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_is_maximized(
  window: u64,
  out_maximized: *mut bool,
) -> i32 {
  catch(|| window_get(window, out_maximized, |w| w.is_maximized()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_is_minimized(
  window: u64,
  out_minimized: *mut bool,
) -> i32 {
  catch(|| window_get(window, out_minimized, |w| w.is_minimized()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_is_resizable(
  window: u64,
  out_resizable: *mut bool,
) -> i32 {
  catch(|| window_get(window, out_resizable, |w| w.is_resizable()))
}

// ---------------------------------------------------------------------------
// setters & actions

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_set_title(window: u64, title: *const c_char) -> i32 {
  catch(|| {
    let title = try_cstr!(title);
    with_window(window, |w| w.set_title(title))
  })
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_size(
  window: u64,
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
    with_window(window, |w| w.set_size(size))
  })
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_position(
  window: u64,
  x: f64,
  y: f64,
  physical: bool,
) -> i32 {
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
pub extern "C" fn tauri_webview_window_set_fullscreen(window: u64, fullscreen: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_fullscreen(fullscreen)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_resizable(window: u64, resizable: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_resizable(resizable)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_always_on_top(window: u64, always_on_top: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_always_on_top(always_on_top)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_decorations(window: u64, decorations: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_decorations(decorations)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_focus(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.set_focus()))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_zoom(window: u64, scale: f64) -> i32 {
  catch(|| with_window(window, |w| w.set_zoom(scale)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_show(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.show()))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_hide(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.hide()))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_center(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.center()))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_maximize(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.maximize()))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_unmaximize(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.unmaximize()))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_minimize(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.minimize()))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_unminimize(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.unminimize()))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_close(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.close()))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_destroy(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.destroy()))
}

// ---------------------------------------------------------------------------
// webview

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_eval(window: u64, js: *const c_char) -> i32 {
  catch(|| {
    let js = try_cstr!(js);
    with_window(window, |w| w.eval(js))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_navigate(window: u64, url: *const c_char) -> i32 {
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
pub extern "C" fn tauri_webview_window_reload(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.reload()))
}

// ---------------------------------------------------------------------------
// additional getters

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_is_decorated(
  window: u64,
  out_decorated: *mut bool,
) -> i32 {
  catch(|| window_get(window, out_decorated, |w| w.is_decorated()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_is_closable(
  window: u64,
  out_closable: *mut bool,
) -> i32 {
  catch(|| window_get(window, out_closable, |w| w.is_closable()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_is_maximizable(
  window: u64,
  out_maximizable: *mut bool,
) -> i32 {
  catch(|| window_get(window, out_maximizable, |w| w.is_maximizable()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_is_minimizable(
  window: u64,
  out_minimizable: *mut bool,
) -> i32 {
  catch(|| window_get(window, out_minimizable, |w| w.is_minimizable()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_is_always_on_top(
  window: u64,
  out_always_on_top: *mut bool,
) -> i32 {
  catch(|| window_get(window, out_always_on_top, |w| w.is_always_on_top()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_is_enabled(
  window: u64,
  out_enabled: *mut bool,
) -> i32 {
  catch(|| window_get(window, out_enabled, |w| w.is_enabled()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_is_menu_visible(
  window: u64,
  out_visible: *mut bool,
) -> i32 {
  catch(|| window_get(window, out_visible, |w| w.is_menu_visible()))
}

// Devtools methods only exist in debug builds or when the `devtools` feature is
// enabled (see [`crate`] Cargo features). The FFI symbols stay present on every
// build; the fallback records a clear error when the capability is absent.
#[cfg(not(any(debug_assertions, feature = "devtools")))]
const DEVTOOLS_UNAVAILABLE: &str =
  "devtools not available; build tauri-ffi with the `devtools` feature";

#[cfg(any(debug_assertions, feature = "devtools"))]
fn window_is_devtools_open(window: u64, out_open: *mut bool) -> i32 {
  window_get(window, out_open, |w| Ok(w.is_devtools_open()))
}
#[cfg(not(any(debug_assertions, feature = "devtools")))]
fn window_is_devtools_open(window: u64, out_open: *mut bool) -> i32 {
  let _ = (window, out_open);
  fail(ERR_GENERIC, DEVTOOLS_UNAVAILABLE)
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_is_devtools_open(
  window: u64,
  out_open: *mut bool,
) -> i32 {
  catch(|| window_is_devtools_open(window, out_open))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_theme(
  window: u64,
  out_theme: *mut *mut c_char,
) -> i32 {
  catch(|| window_get_string(window, out_theme, |w| w.theme().map(|t| t.to_string())))
}

// ---------------------------------------------------------------------------
// additional boolean setters

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_closable(window: u64, closable: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_closable(closable)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_maximizable(window: u64, maximizable: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_maximizable(maximizable)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_minimizable(window: u64, minimizable: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_minimizable(minimizable)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_always_on_bottom(
  window: u64,
  always_on_bottom: bool,
) -> i32 {
  catch(|| with_window(window, |w| w.set_always_on_bottom(always_on_bottom)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_content_protected(window: u64, protected: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_content_protected(protected)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_skip_taskbar(window: u64, skip: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_skip_taskbar(skip)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_shadow(window: u64, enable: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_shadow(enable)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_visible_on_all_workspaces(
  window: u64,
  visible: bool,
) -> i32 {
  catch(|| with_window(window, |w| w.set_visible_on_all_workspaces(visible)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_ignore_cursor_events(window: u64, ignore: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_ignore_cursor_events(ignore)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_cursor_visible(window: u64, visible: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_cursor_visible(visible)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_cursor_grab(window: u64, grab: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_cursor_grab(grab)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_enabled(window: u64, enabled: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_enabled(enabled)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_focusable(window: u64, focusable: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_focusable(focusable)))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_simple_fullscreen(window: u64, enable: bool) -> i32 {
  catch(|| with_window(window, |w| w.set_simple_fullscreen(enable)))
}

// ---------------------------------------------------------------------------
// size / position setters

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_min_size(
  window: u64,
  width: f64,
  height: f64,
  physical: bool,
) -> i32 {
  catch(|| {
    let size = optional_size(width, height, physical);
    with_window(window, |w| w.set_min_size(size))
  })
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_max_size(
  window: u64,
  width: f64,
  height: f64,
  physical: bool,
) -> i32 {
  catch(|| {
    let size = optional_size(width, height, physical);
    with_window(window, |w| w.set_max_size(size))
  })
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_cursor_position(
  window: u64,
  x: f64,
  y: f64,
  physical: bool,
) -> i32 {
  catch(|| {
    let position: Position = if physical {
      PhysicalPosition::new(x.round() as i32, y.round() as i32).into()
    } else {
      LogicalPosition::new(x, y).into()
    };
    with_window(window, |w| w.set_cursor_position(position))
  })
}

// ---------------------------------------------------------------------------
// enum / string / structured setters

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_set_theme(window: u64, theme: *const c_char) -> i32 {
  catch(|| {
    let theme = try_cstr!(theme);
    // `Theme` is `#[non_exhaustive]`; construct it through its Deserialize impl.
    let theme: Option<Theme> = match theme.to_lowercase().as_str() {
      "" => None,
      lower @ ("light" | "dark") => {
        Some(serde_json::from_value(serde_json::Value::String(lower.to_string())).unwrap())
      }
      _ => {
        return fail(
          ERR_INVALID_ARG,
          "invalid theme (expected \"light\", \"dark\" or \"\")",
        )
      }
    };
    with_window(window, |w| w.set_theme(theme))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_set_cursor_icon(
  window: u64,
  icon: *const c_char,
) -> i32 {
  catch(|| {
    let icon = try_cstr!(icon);
    // Unknown names fall back to `CursorIcon::Default` (via its Deserialize impl).
    let icon: CursorIcon =
      serde_json::from_value(serde_json::Value::String(icon.to_string())).unwrap_or_default();
    with_window(window, |w| w.set_cursor_icon(icon))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_request_user_attention(
  window: u64,
  kind: *const c_char,
) -> i32 {
  catch(|| {
    let kind = try_cstr!(kind);
    let request_type = match kind.to_lowercase().as_str() {
      "" => None,
      "critical" => Some(UserAttentionType::Critical),
      "informational" => Some(UserAttentionType::Informational),
      _ => {
        return fail(
          ERR_INVALID_ARG,
          "invalid attention type (expected \"critical\", \"informational\" or \"\")",
        )
      }
    };
    with_window(window, |w| w.request_user_attention(request_type))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_set_progress_bar(
  window: u64,
  state_json: *const c_char,
) -> i32 {
  catch(|| {
    let json = try_cstr!(state_json);
    let state: ProgressBarState = match serde_json::from_str(json) {
      Ok(state) => state,
      Err(e) => return fail(ERR_INVALID_ARG, format!("invalid progress bar state: {e}")),
    };
    with_window(window, |w| w.set_progress_bar(state))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_set_effects(
  window: u64,
  effects_json: *const c_char,
) -> i32 {
  catch(|| {
    let json = try_cstr!(effects_json);
    let trimmed = json.trim();
    let effects: Option<WindowEffectsConfig> = if trimmed.is_empty() || trimmed == "null" {
      None
    } else {
      match serde_json::from_str(json) {
        Ok(effects) => Some(effects),
        Err(e) => return fail(ERR_INVALID_ARG, format!("invalid window effects: {e}")),
      }
    };
    with_window(window, |w| w.set_effects(effects))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_set_size_constraints(
  window: u64,
  constraints_json: *const c_char,
) -> i32 {
  catch(|| {
    let json = try_cstr!(constraints_json);
    let constraints: WindowSizeConstraints = match serde_json::from_str(json) {
      Ok(constraints) => constraints,
      Err(e) => return fail(ERR_INVALID_ARG, format!("invalid size constraints: {e}")),
    };
    with_window(window, |w| w.set_size_constraints(constraints))
  })
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_background_color(
  window: u64,
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
    with_window(window, |w| w.set_background_color(Some(color)))
  })
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_set_badge_count(window: u64, count: i32) -> i32 {
  catch(|| {
    let count = if count < 0 { None } else { Some(count as i64) };
    with_window(window, |w| w.set_badge_count(count))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_set_badge_label(
  window: u64,
  label: *const c_char,
) -> i32 {
  catch(|| {
    let label = try_cstr!(label);
    #[cfg(target_os = "macos")]
    {
      let label = if label.is_empty() {
        None
      } else {
        Some(label.to_string())
      };
      with_window(window, |w| w.set_badge_label(label))
    }
    #[cfg(not(target_os = "macos"))]
    {
      let _ = (window, label);
      unsupported("tauri_webview_window_set_badge_label", "macOS")
    }
  })
}

// ---------------------------------------------------------------------------
// platform-specific
//
// Exported on every platform so the ABI symbol table never varies; on
// unsupported platforms they return ERR_UNSUPPORTED without side effects.

/// Sets the title bar style ("visible", "transparent" or "overlay"). macOS only.
#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_set_title_bar_style(
  window: u64,
  style: *const c_char,
) -> i32 {
  catch(|| {
    let style = try_cstr!(style);
    #[cfg(target_os = "macos")]
    {
      let style = match parse_title_bar_style(style) {
        Ok(style) => style,
        Err(code) => return code,
      };
      with_window(window, |w| w.set_title_bar_style(style))
    }
    #[cfg(not(target_os = "macos"))]
    {
      let _ = (window, style);
      unsupported("tauri_webview_window_set_title_bar_style", "macOS")
    }
  })
}

#[cfg(target_os = "macos")]
pub(crate) fn parse_title_bar_style(style: &str) -> Result<tauri::TitleBarStyle, i32> {
  match style.to_lowercase().as_str() {
    "visible" => Ok(tauri::TitleBarStyle::Visible),
    "transparent" => Ok(tauri::TitleBarStyle::Transparent),
    "overlay" => Ok(tauri::TitleBarStyle::Overlay),
    _ => Err(fail(
      ERR_INVALID_ARG,
      "invalid title bar style (expected \"visible\", \"transparent\" or \"overlay\")",
    )),
  }
}

/// Sets (or clears, when `rgba` is null) the taskbar overlay icon from RGBA
/// pixel data. Windows only.
#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_set_overlay_icon(
  window: u64,
  rgba: *const u8,
  width: u32,
  height: u32,
) -> i32 {
  catch(|| {
    #[cfg(target_os = "windows")]
    {
      let icon = match unsafe { crate::winsupport::rgba_image(rgba, width, height) } {
        Ok(icon) => icon,
        Err(code) => return code,
      };
      with_window(window, |w| w.set_overlay_icon(icon))
    }
    #[cfg(not(target_os = "windows"))]
    {
      let _ = (window, rgba, width, height);
      unsupported("tauri_webview_window_set_overlay_icon", "Windows")
    }
  })
}

/// The NSWindow pointer backing the window, as an integer. macOS only.
#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_ns_window(
  window: u64,
  out_ns_window: *mut u64,
) -> i32 {
  catch(|| {
    #[cfg(target_os = "macos")]
    {
      window_get(window, out_ns_window, |w| w.ns_window().map(|p| p as u64))
    }
    #[cfg(not(target_os = "macos"))]
    {
      let _ = (window, out_ns_window);
      unsupported("tauri_webview_window_ns_window", "macOS")
    }
  })
}

/// The NSView pointer backing the window's content view, as an integer.
/// macOS only.
#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_ns_view(window: u64, out_ns_view: *mut u64) -> i32 {
  catch(|| {
    #[cfg(target_os = "macos")]
    {
      window_get(window, out_ns_view, |w| w.ns_view().map(|p| p as u64))
    }
    #[cfg(not(target_os = "macos"))]
    {
      let _ = (window, out_ns_view);
      unsupported("tauri_webview_window_ns_view", "macOS")
    }
  })
}

/// The HWND of the window, as an integer. Windows only.
#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_hwnd(window: u64, out_hwnd: *mut u64) -> i32 {
  catch(|| {
    #[cfg(target_os = "windows")]
    {
      window_get(window, out_hwnd, |w| w.hwnd().map(|h| h.0 as u64))
    }
    #[cfg(not(target_os = "windows"))]
    {
      let _ = (window, out_hwnd);
      unsupported("tauri_webview_window_hwnd", "Windows")
    }
  })
}

// ---------------------------------------------------------------------------
// additional actions

#[no_mangle]
pub extern "C" fn tauri_webview_window_start_dragging(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.start_dragging()))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_print(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.print()))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_clear_all_browsing_data(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.clear_all_browsing_data()))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_hide_menu(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.hide_menu()))
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_show_menu(window: u64) -> i32 {
  catch(|| with_window(window, |w| w.show_menu()))
}

#[cfg(any(debug_assertions, feature = "devtools"))]
fn window_open_devtools(window: u64) -> i32 {
  with_window(window, |w| {
    w.open_devtools();
    Ok(())
  })
}
#[cfg(not(any(debug_assertions, feature = "devtools")))]
fn window_open_devtools(window: u64) -> i32 {
  let _ = window;
  fail(ERR_GENERIC, DEVTOOLS_UNAVAILABLE)
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_open_devtools(window: u64) -> i32 {
  catch(|| window_open_devtools(window))
}

#[cfg(any(debug_assertions, feature = "devtools"))]
fn window_close_devtools(window: u64) -> i32 {
  with_window(window, |w| {
    w.close_devtools();
    Ok(())
  })
}
#[cfg(not(any(debug_assertions, feature = "devtools")))]
fn window_close_devtools(window: u64) -> i32 {
  let _ = window;
  fail(ERR_GENERIC, DEVTOOLS_UNAVAILABLE)
}

#[no_mangle]
pub extern "C" fn tauri_webview_window_close_devtools(window: u64) -> i32 {
  catch(|| window_close_devtools(window))
}

// ---------------------------------------------------------------------------
// monitors

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_current_monitor(
  window: u64,
  out_monitor_json: *mut *mut c_char,
) -> i32 {
  catch(|| {
    window_get_string(window, out_monitor_json, |w| {
      Ok(serde_json::to_string(&w.current_monitor()?).unwrap_or_else(|_| "null".into()))
    })
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_primary_monitor(
  window: u64,
  out_monitor_json: *mut *mut c_char,
) -> i32 {
  catch(|| {
    window_get_string(window, out_monitor_json, |w| {
      Ok(serde_json::to_string(&w.primary_monitor()?).unwrap_or_else(|_| "null".into()))
    })
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_available_monitors(
  window: u64,
  out_monitors_json: *mut *mut c_char,
) -> i32 {
  catch(|| {
    window_get_string(window, out_monitors_json, |w| {
      Ok(serde_json::to_string(&w.available_monitors()?).unwrap_or_else(|_| "[]".into()))
    })
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_monitor_from_point(
  window: u64,
  x: f64,
  y: f64,
  out_monitor_json: *mut *mut c_char,
) -> i32 {
  catch(|| {
    window_get_string(window, out_monitor_json, |w| {
      Ok(serde_json::to_string(&w.monitor_from_point(x, y)?).unwrap_or_else(|_| "null".into()))
    })
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_webview_window_cursor_position(
  window: u64,
  out_x: *mut f64,
  out_y: *mut f64,
) -> i32 {
  catch(|| {
    window_get_pair(window, out_x, out_y, |w| {
      w.cursor_position().map(|p| (p.x, p.y))
    })
  })
}
