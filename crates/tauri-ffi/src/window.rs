// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Bare `Window` handles — the OS window without a webview (`tauri::Window`).
//! A window can host one or more [`crate::webview`]s (multiwebview). Window
//! management methods mirror [`tauri::Window`]; each is callable from any
//! thread. For the common single-webview case use [`crate::webview_window`].

use std::os::raw::c_char;

use crate::Rt as TauriRuntime;
use tauri::utils::config::{WindowConfig, WindowEffectsConfig};
use tauri::webview::WebviewBuilder;
use tauri::window::{Color, ProgressBarState, WindowBuilder};
use tauri::{
  CursorIcon, LogicalPosition, LogicalSize, Manager, Monitor, PhysicalPosition, PhysicalSize,
  Position, Size, Theme, UserAttentionType, Window, WindowSizeConstraints,
};

use crate::error::{
  catch, fail, unsupported, ERR_GENERIC, ERR_INVALID_ARG, ERR_INVALID_HANDLE, ERR_NOT_FOUND, OK,
};
use crate::state::{self, Entry};
use crate::winsupport::{
  act, ffi_action, ffi_getter_bool, ffi_getter_string, ffi_setter_bool, get, get_pair, get_string,
};
use crate::{try_cstr, write_owned_str};

const RESOLVE: fn(u64) -> Option<Window<TauriRuntime>> = state::bare_window;

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
// creation, children, lookup

/// Creates a bare window (no webview) from a `WindowConfig` JSON object. Blocks
/// until the running event loop has created it — call only while the app runs.
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
    let built = WindowBuilder::from_config(&app_state.handle, &config).and_then(|b| b.build());
    match built {
      Ok(window) => {
        unsafe { *out_window = state::insert(Entry::BareWindow(window)) };
        OK
      }
      Err(e) => fail(ERR_GENERIC, format!("failed to create window: {e}")),
    }
  })
}

/// Adds a child webview to the window from a `WindowConfig` JSON object, at the
/// given position and size. physical=true interprets the bounds as physical
/// pixels, otherwise logical. On OK, `*out_webview` is a webview handle.
#[no_mangle]
pub unsafe extern "C" fn tauri_window_add_webview(
  window: u64,
  config_json: *const c_char,
  x: f64,
  y: f64,
  width: f64,
  height: f64,
  physical: bool,
  out_webview: *mut u64,
) -> i32 {
  catch(|| {
    let json = try_cstr!(config_json);
    if out_webview.is_null() {
      return fail(ERR_INVALID_ARG, "out_webview is null");
    }
    let Some(window) = state::bare_window(window) else {
      return fail(ERR_INVALID_HANDLE, "invalid window handle");
    };
    let config: WindowConfig = match serde_json::from_str(json) {
      Ok(config) => config,
      Err(e) => return fail(ERR_INVALID_ARG, format!("invalid webview config: {e}")),
    };
    let (position, size): (Position, Size) = if physical {
      (
        PhysicalPosition::new(x.round() as i32, y.round() as i32).into(),
        PhysicalSize::new(width.round() as u32, height.round() as u32).into(),
      )
    } else {
      (
        LogicalPosition::new(x, y).into(),
        LogicalSize::new(width, height).into(),
      )
    };
    match window.add_child(WebviewBuilder::from_config(&config), position, size) {
      Ok(webview) => {
        unsafe { *out_webview = state::insert(Entry::Webview(webview)) };
        OK
      }
      Err(e) => fail(ERR_GENERIC, format!("failed to add webview: {e}")),
    }
  })
}

/// Labels of the window's child webviews, as an owned JSON string array.
#[no_mangle]
pub unsafe extern "C" fn tauri_window_webviews(
  window: u64,
  out_labels_json: *mut *mut c_char,
) -> i32 {
  catch(|| {
    if out_labels_json.is_null() {
      return fail(ERR_INVALID_ARG, "out_labels_json is null");
    }
    let Some(window) = state::bare_window(window) else {
      return fail(ERR_INVALID_HANDLE, "invalid window handle");
    };
    let labels: Vec<String> = window
      .webviews()
      .iter()
      .map(|w| w.label().to_string())
      .collect();
    write_owned_str(
      out_labels_json,
      serde_json::to_string(&labels).unwrap_or_else(|_| "[]".into()),
    );
    OK
  })
}

/// Looks up a bare window by label. Returns ERR_NOT_FOUND if none.
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
    match app_state.handle.get_window(label) {
      Some(window) => {
        unsafe { *out_window = state::insert(Entry::BareWindow(window)) };
        OK
      }
      None => fail(ERR_NOT_FOUND, format!("no window labeled `{label}`")),
    }
  })
}

/// Labels of all bare windows, as an owned JSON string array.
#[no_mangle]
pub unsafe extern "C" fn tauri_app_window_labels(
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
    let labels: Vec<String> = app_state.handle.windows().keys().cloned().collect();
    write_owned_str(
      out_labels_json,
      serde_json::to_string(&labels).unwrap_or_else(|_| "[]".into()),
    );
    OK
  })
}

// ---------------------------------------------------------------------------
// string / scalar getters

#[no_mangle]
pub unsafe extern "C" fn tauri_window_label(window: u64, out_label: *mut *mut c_char) -> i32 {
  catch(|| get_string(window, out_label, RESOLVE, |w| Ok(w.label().to_string())))
}
ffi_getter_string!(tauri_window_title, RESOLVE, title);
#[no_mangle]
pub unsafe extern "C" fn tauri_window_theme(window: u64, out_theme: *mut *mut c_char) -> i32 {
  catch(|| {
    get_string(window, out_theme, RESOLVE, |w| {
      w.theme().map(|t| t.to_string())
    })
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_scale_factor(window: u64, out_scale: *mut f64) -> i32 {
  catch(|| get(window, out_scale, RESOLVE, |w| w.scale_factor()))
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_inner_size(
  window: u64,
  out_w: *mut u32,
  out_h: *mut u32,
) -> i32 {
  catch(|| {
    get_pair(window, out_w, out_h, RESOLVE, |w| {
      w.inner_size().map(|s| (s.width, s.height))
    })
  })
}
#[no_mangle]
pub unsafe extern "C" fn tauri_window_outer_size(
  window: u64,
  out_w: *mut u32,
  out_h: *mut u32,
) -> i32 {
  catch(|| {
    get_pair(window, out_w, out_h, RESOLVE, |w| {
      w.outer_size().map(|s| (s.width, s.height))
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
    get_pair(window, out_x, out_y, RESOLVE, |w| {
      w.inner_position().map(|p| (p.x, p.y))
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
    get_pair(window, out_x, out_y, RESOLVE, |w| {
      w.outer_position().map(|p| (p.x, p.y))
    })
  })
}
#[no_mangle]
pub unsafe extern "C" fn tauri_window_cursor_position(
  window: u64,
  out_x: *mut f64,
  out_y: *mut f64,
) -> i32 {
  catch(|| {
    get_pair(window, out_x, out_y, RESOLVE, |w| {
      w.cursor_position().map(|p| (p.x, p.y))
    })
  })
}

// ---------------------------------------------------------------------------
// boolean getters

ffi_getter_bool!(tauri_window_is_visible, RESOLVE, is_visible);
ffi_getter_bool!(tauri_window_is_focused, RESOLVE, is_focused);
ffi_getter_bool!(tauri_window_is_fullscreen, RESOLVE, is_fullscreen);
ffi_getter_bool!(tauri_window_is_maximized, RESOLVE, is_maximized);
ffi_getter_bool!(tauri_window_is_minimized, RESOLVE, is_minimized);
ffi_getter_bool!(tauri_window_is_resizable, RESOLVE, is_resizable);
ffi_getter_bool!(tauri_window_is_decorated, RESOLVE, is_decorated);
ffi_getter_bool!(tauri_window_is_closable, RESOLVE, is_closable);
ffi_getter_bool!(tauri_window_is_maximizable, RESOLVE, is_maximizable);
ffi_getter_bool!(tauri_window_is_minimizable, RESOLVE, is_minimizable);
ffi_getter_bool!(tauri_window_is_always_on_top, RESOLVE, is_always_on_top);
ffi_getter_bool!(tauri_window_is_enabled, RESOLVE, is_enabled);
ffi_getter_bool!(tauri_window_is_menu_visible, RESOLVE, is_menu_visible);

// ---------------------------------------------------------------------------
// boolean setters

ffi_setter_bool!(tauri_window_set_fullscreen, RESOLVE, set_fullscreen);
ffi_setter_bool!(tauri_window_set_resizable, RESOLVE, set_resizable);
ffi_setter_bool!(tauri_window_set_always_on_top, RESOLVE, set_always_on_top);
ffi_setter_bool!(tauri_window_set_decorations, RESOLVE, set_decorations);
ffi_setter_bool!(tauri_window_set_closable, RESOLVE, set_closable);
ffi_setter_bool!(tauri_window_set_maximizable, RESOLVE, set_maximizable);
ffi_setter_bool!(tauri_window_set_minimizable, RESOLVE, set_minimizable);
ffi_setter_bool!(
  tauri_window_set_always_on_bottom,
  RESOLVE,
  set_always_on_bottom
);
ffi_setter_bool!(
  tauri_window_set_content_protected,
  RESOLVE,
  set_content_protected
);
ffi_setter_bool!(tauri_window_set_skip_taskbar, RESOLVE, set_skip_taskbar);
ffi_setter_bool!(tauri_window_set_shadow, RESOLVE, set_shadow);
ffi_setter_bool!(
  tauri_window_set_visible_on_all_workspaces,
  RESOLVE,
  set_visible_on_all_workspaces
);
ffi_setter_bool!(
  tauri_window_set_ignore_cursor_events,
  RESOLVE,
  set_ignore_cursor_events
);
ffi_setter_bool!(tauri_window_set_cursor_visible, RESOLVE, set_cursor_visible);
ffi_setter_bool!(tauri_window_set_cursor_grab, RESOLVE, set_cursor_grab);
ffi_setter_bool!(tauri_window_set_enabled, RESOLVE, set_enabled);
ffi_setter_bool!(tauri_window_set_focusable, RESOLVE, set_focusable);
ffi_setter_bool!(
  tauri_window_set_simple_fullscreen,
  RESOLVE,
  set_simple_fullscreen
);

// ---------------------------------------------------------------------------
// unit actions

ffi_action!(tauri_window_set_focus, RESOLVE, set_focus);
ffi_action!(tauri_window_show, RESOLVE, show);
ffi_action!(tauri_window_hide, RESOLVE, hide);
ffi_action!(tauri_window_center, RESOLVE, center);
ffi_action!(tauri_window_maximize, RESOLVE, maximize);
ffi_action!(tauri_window_unmaximize, RESOLVE, unmaximize);
ffi_action!(tauri_window_minimize, RESOLVE, minimize);
ffi_action!(tauri_window_unminimize, RESOLVE, unminimize);
ffi_action!(tauri_window_close, RESOLVE, close);
ffi_action!(tauri_window_destroy, RESOLVE, destroy);
ffi_action!(tauri_window_start_dragging, RESOLVE, start_dragging);
ffi_action!(tauri_window_hide_menu, RESOLVE, hide_menu);
ffi_action!(tauri_window_show_menu, RESOLVE, show_menu);

// ---------------------------------------------------------------------------
// custom setters

#[no_mangle]
pub unsafe extern "C" fn tauri_window_set_title(window: u64, title: *const c_char) -> i32 {
  catch(|| {
    let title = try_cstr!(title);
    act(window, RESOLVE, |w| w.set_title(title))
  })
}

#[no_mangle]
pub extern "C" fn tauri_window_set_size(
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
    act(window, RESOLVE, |w| w.set_size(size))
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
    act(window, RESOLVE, |w| w.set_position(position))
  })
}

#[no_mangle]
pub extern "C" fn tauri_window_set_min_size(
  window: u64,
  width: f64,
  height: f64,
  physical: bool,
) -> i32 {
  catch(|| {
    act(window, RESOLVE, |w| {
      w.set_min_size(optional_size(width, height, physical))
    })
  })
}
#[no_mangle]
pub extern "C" fn tauri_window_set_max_size(
  window: u64,
  width: f64,
  height: f64,
  physical: bool,
) -> i32 {
  catch(|| {
    act(window, RESOLVE, |w| {
      w.set_max_size(optional_size(width, height, physical))
    })
  })
}

#[no_mangle]
pub extern "C" fn tauri_window_set_cursor_position(
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
    act(window, RESOLVE, |w| w.set_cursor_position(position))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_set_theme(window: u64, theme: *const c_char) -> i32 {
  catch(|| {
    let theme = try_cstr!(theme);
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
    act(window, RESOLVE, |w| w.set_theme(theme))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_set_cursor_icon(window: u64, icon: *const c_char) -> i32 {
  catch(|| {
    let icon = try_cstr!(icon);
    let icon: CursorIcon =
      serde_json::from_value(serde_json::Value::String(icon.to_string())).unwrap_or_default();
    act(window, RESOLVE, |w| w.set_cursor_icon(icon))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_request_user_attention(
  window: u64,
  kind: *const c_char,
) -> i32 {
  catch(|| {
    let kind = try_cstr!(kind);
    let request_type = match kind.to_lowercase().as_str() {
      "" => None,
      "critical" => Some(UserAttentionType::Critical),
      "informational" => Some(UserAttentionType::Informational),
      _ => return fail(ERR_INVALID_ARG, "invalid attention type"),
    };
    act(window, RESOLVE, |w| w.request_user_attention(request_type))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_set_progress_bar(
  window: u64,
  state_json: *const c_char,
) -> i32 {
  catch(|| {
    let json = try_cstr!(state_json);
    let state: ProgressBarState = match serde_json::from_str(json) {
      Ok(state) => state,
      Err(e) => return fail(ERR_INVALID_ARG, format!("invalid progress bar state: {e}")),
    };
    act(window, RESOLVE, |w| w.set_progress_bar(state))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_set_effects(window: u64, effects_json: *const c_char) -> i32 {
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
    act(window, RESOLVE, |w| w.set_effects(effects))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_set_size_constraints(
  window: u64,
  constraints_json: *const c_char,
) -> i32 {
  catch(|| {
    let json = try_cstr!(constraints_json);
    let constraints: WindowSizeConstraints = match serde_json::from_str(json) {
      Ok(constraints) => constraints,
      Err(e) => return fail(ERR_INVALID_ARG, format!("invalid size constraints: {e}")),
    };
    act(window, RESOLVE, |w| w.set_size_constraints(constraints))
  })
}

#[no_mangle]
pub extern "C" fn tauri_window_set_background_color(
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
    act(window, RESOLVE, |w| w.set_background_color(Some(color)))
  })
}

#[no_mangle]
pub extern "C" fn tauri_window_set_badge_count(window: u64, count: i32) -> i32 {
  catch(|| {
    let count = if count < 0 { None } else { Some(count as i64) };
    act(window, RESOLVE, |w| w.set_badge_count(count))
  })
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_set_badge_label(window: u64, label: *const c_char) -> i32 {
  catch(|| {
    let label = try_cstr!(label);
    #[cfg(target_os = "macos")]
    {
      let label = (!label.is_empty()).then(|| label.to_string());
      act(window, RESOLVE, |w| w.set_badge_label(label))
    }
    #[cfg(not(target_os = "macos"))]
    {
      let _ = (window, label);
      unsupported("tauri_window_set_badge_label", "macOS")
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
pub unsafe extern "C" fn tauri_window_set_title_bar_style(
  window: u64,
  style: *const c_char,
) -> i32 {
  catch(|| {
    let style = try_cstr!(style);
    #[cfg(target_os = "macos")]
    {
      let style = match crate::webview_window::parse_title_bar_style(style) {
        Ok(style) => style,
        Err(code) => return code,
      };
      act(window, RESOLVE, |w| w.set_title_bar_style(style))
    }
    #[cfg(not(target_os = "macos"))]
    {
      let _ = (window, style);
      unsupported("tauri_window_set_title_bar_style", "macOS")
    }
  })
}

/// Sets (or clears, when `rgba` is null) the taskbar overlay icon from RGBA
/// pixel data. Windows only.
#[no_mangle]
pub unsafe extern "C" fn tauri_window_set_overlay_icon(
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
      act(window, RESOLVE, |w| w.set_overlay_icon(icon))
    }
    #[cfg(not(target_os = "windows"))]
    {
      let _ = (window, rgba, width, height);
      unsupported("tauri_window_set_overlay_icon", "Windows")
    }
  })
}

/// The NSWindow pointer backing the window, as an integer. macOS only.
#[no_mangle]
pub unsafe extern "C" fn tauri_window_ns_window(window: u64, out_ns_window: *mut u64) -> i32 {
  catch(|| {
    #[cfg(target_os = "macos")]
    {
      get(window, out_ns_window, RESOLVE, |w| {
        w.ns_window().map(|p| p as u64)
      })
    }
    #[cfg(not(target_os = "macos"))]
    {
      let _ = (window, out_ns_window);
      unsupported("tauri_window_ns_window", "macOS")
    }
  })
}

/// The NSView pointer backing the window's content view, as an integer.
/// macOS only.
#[no_mangle]
pub unsafe extern "C" fn tauri_window_ns_view(window: u64, out_ns_view: *mut u64) -> i32 {
  catch(|| {
    #[cfg(target_os = "macos")]
    {
      get(window, out_ns_view, RESOLVE, |w| {
        w.ns_view().map(|p| p as u64)
      })
    }
    #[cfg(not(target_os = "macos"))]
    {
      let _ = (window, out_ns_view);
      unsupported("tauri_window_ns_view", "macOS")
    }
  })
}

/// The HWND of the window, as an integer. Windows only.
#[no_mangle]
pub unsafe extern "C" fn tauri_window_hwnd(window: u64, out_hwnd: *mut u64) -> i32 {
  catch(|| {
    #[cfg(target_os = "windows")]
    {
      get(window, out_hwnd, RESOLVE, |w| w.hwnd().map(|h| h.0 as u64))
    }
    #[cfg(not(target_os = "windows"))]
    {
      let _ = (window, out_hwnd);
      unsupported("tauri_window_hwnd", "Windows")
    }
  })
}

// ---------------------------------------------------------------------------
// monitors

fn monitor_json(m: Option<Monitor>) -> String {
  serde_json::to_string(&m).unwrap_or_else(|_| "null".into())
}

#[no_mangle]
pub unsafe extern "C" fn tauri_window_current_monitor(
  window: u64,
  out_json: *mut *mut c_char,
) -> i32 {
  catch(|| {
    get_string(window, out_json, RESOLVE, |w| {
      Ok(monitor_json(w.current_monitor()?))
    })
  })
}
#[no_mangle]
pub unsafe extern "C" fn tauri_window_primary_monitor(
  window: u64,
  out_json: *mut *mut c_char,
) -> i32 {
  catch(|| {
    get_string(window, out_json, RESOLVE, |w| {
      Ok(monitor_json(w.primary_monitor()?))
    })
  })
}
#[no_mangle]
pub unsafe extern "C" fn tauri_window_available_monitors(
  window: u64,
  out_json: *mut *mut c_char,
) -> i32 {
  catch(|| {
    get_string(window, out_json, RESOLVE, |w| {
      Ok(serde_json::to_string(&w.available_monitors()?).unwrap_or_else(|_| "[]".into()))
    })
  })
}
#[no_mangle]
pub unsafe extern "C" fn tauri_window_monitor_from_point(
  window: u64,
  x: f64,
  y: f64,
  out_json: *mut *mut c_char,
) -> i32 {
  catch(|| {
    get_string(window, out_json, RESOLVE, |w| {
      Ok(monitor_json(w.monitor_from_point(x, y)?))
    })
  })
}
