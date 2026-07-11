// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `Manager` / `AppHandle` methods that operate on a running app: the path
//! resolver, focused-window lookup, runtime capabilities, one-shot event
//! subscriptions and a few app-wide window controls. Each mirrors the
//! equivalent method in the `tauri` crate and is callable from any thread.

use std::os::raw::c_char;

use tauri::{Listener, Manager};

use crate::error::{
  catch, fail, ERR_GENERIC, ERR_INVALID_ARG, ERR_INVALID_HANDLE, ERR_NOT_FOUND, OK,
};
use crate::state::{self, Entry};
use crate::{try_cstr, write_owned_str};

/// Resolves one of the platform base directories. `kind` mirrors the
/// `PathResolver` method names in camelCase (e.g. "appData", "home", "temp").
/// On OK, `*out_path` is an owned UTF-8 path — free with `tauri_string_free`.
#[no_mangle]
pub unsafe extern "C" fn tauri_app_path(app: u64, kind: *const c_char, out_path: *mut *mut c_char) -> i32 {
  catch(|| {
    let kind = try_cstr!(kind);
    if out_path.is_null() {
      return fail(ERR_INVALID_ARG, "out_path is null");
    }
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    let path = app_state.handle.path();
    let resolved = match kind {
      "appConfig" => path.app_config_dir(),
      "appData" => path.app_data_dir(),
      "appLocalData" => path.app_local_data_dir(),
      "appCache" => path.app_cache_dir(),
      "appLog" => path.app_log_dir(),
      "audio" => path.audio_dir(),
      "cache" => path.cache_dir(),
      "config" => path.config_dir(),
      "data" => path.data_dir(),
      "localData" => path.local_data_dir(),
      "desktop" => path.desktop_dir(),
      "document" => path.document_dir(),
      "download" => path.download_dir(),
      "executable" => path.executable_dir(),
      "font" => path.font_dir(),
      "home" => path.home_dir(),
      "picture" => path.picture_dir(),
      "public" => path.public_dir(),
      "resource" => path.resource_dir(),
      "runtime" => path.runtime_dir(),
      "template" => path.template_dir(),
      "video" => path.video_dir(),
      "temp" => path.temp_dir(),
      other => return fail(ERR_INVALID_ARG, format!("unknown path kind `{other}`")),
    };
    match resolved {
      Ok(path) => {
        write_owned_str(out_path, path.to_string_lossy().into_owned());
        OK
      }
      Err(e) => fail(ERR_GENERIC, format!("path `{kind}` unavailable: {e}")),
    }
  })
}

/// The webview window that currently has focus. Returns `ERR_NOT_FOUND` if no
/// window is focused (or the focused window has no webview). On OK,
/// `*out_window` is a window handle; release with `tauri_handle_close`.
#[no_mangle]
pub unsafe extern "C" fn tauri_app_get_focused_window(app: u64, out_window: *mut u64) -> i32 {
  catch(|| {
    if out_window.is_null() {
      return fail(ERR_INVALID_ARG, "out_window is null");
    }
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    let Some(window) = app_state.handle.get_focused_window() else {
      return fail(ERR_NOT_FOUND, "no focused window");
    };
    let label = window.label().to_string();
    match app_state.handle.get_webview_window(&label) {
      Some(webview) => {
        unsafe { *out_window = state::insert(Entry::Window(webview)) };
        OK
      }
      None => fail(ERR_NOT_FOUND, format!("focused window `{label}` has no webview")),
    }
  })
}

/// Adds a capability to the running app (JSON or TOML string, capability file
/// format). Requires the `dynamic-acl` feature (enabled).
#[no_mangle]
pub unsafe extern "C" fn tauri_app_add_capability(app: u64, capability: *const c_char) -> i32 {
  catch(|| {
    let capability = try_cstr!(capability);
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    match app_state.handle.add_capability(capability.to_string()) {
      Ok(()) => OK,
      Err(e) => fail(ERR_GENERIC, format!("invalid capability: {e}")),
    }
  })
}

/// Subscribes to an event and removes the subscription after the first
/// delivery. Deliveries arrive on the event queue as
/// `{"type":"event","event":...,"id":...,"payload":...}`.
#[no_mangle]
pub unsafe extern "C" fn tauri_app_once(app: u64, event: *const c_char, out_listener: *mut u32) -> i32 {
  catch(|| {
    let event = try_cstr!(event);
    if out_listener.is_null() {
      return fail(ERR_INVALID_ARG, "out_listener is null");
    }
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    let tx = app_state.tx.clone();
    let name = event.to_string();
    let listener_id = app_state.handle.once_any(name.clone(), move |event| {
      let payload: serde_json::Value = serde_json::from_str(event.payload())
        .unwrap_or_else(|_| serde_json::Value::String(event.payload().to_string()));
      let message = serde_json::json!({
        "type": "event",
        "event": name,
        "id": event.id(),
        "payload": payload,
      });
      let _ = tx.send(message.to_string());
    });
    unsafe { *out_listener = listener_id };
    OK
  })
}

/// Sets the app-wide theme. Pass "light" or "dark"; an empty string follows the
/// system theme.
#[no_mangle]
pub unsafe extern "C" fn tauri_app_set_theme(app: u64, theme: *const c_char) -> i32 {
  catch(|| {
    let theme = try_cstr!(theme);
    let theme: Option<tauri::Theme> = match theme.to_lowercase().as_str() {
      "" => None,
      lower @ ("light" | "dark") => {
        Some(serde_json::from_value(serde_json::Value::String(lower.to_string())).unwrap())
      }
      _ => return fail(ERR_INVALID_ARG, "invalid theme (expected \"light\", \"dark\" or \"\")"),
    };
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    app_state.handle.set_theme(theme);
    OK
  })
}

/// The current cursor position, in physical pixels (screen coordinates).
#[no_mangle]
pub unsafe extern "C" fn tauri_app_cursor_position(app: u64, out_x: *mut f64, out_y: *mut f64) -> i32 {
  catch(|| {
    if out_x.is_null() || out_y.is_null() {
      return fail(ERR_INVALID_ARG, "output pointer is null");
    }
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    match app_state.handle.cursor_position() {
      Ok(position) => {
        unsafe {
          *out_x = position.x;
          *out_y = position.y;
        }
        OK
      }
      Err(e) => fail(ERR_GENERIC, format!("cursor_position failed: {e}")),
    }
  })
}

/// Requests an app restart: triggers exit-requested with the restart exit code,
/// then exits and relaunches. Delivered through the event queue like a normal
/// exit; the process relaunches after the loop returns.
#[no_mangle]
pub extern "C" fn tauri_app_request_restart(app: u64) -> i32 {
  catch(|| {
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    app_state.handle.request_restart();
    OK
  })
}

// ---------------------------------------------------------------------------
// platform-specific (macOS)
//
// Exported on every platform so the ABI symbol table never varies; on
// unsupported platforms they return ERR_UNSUPPORTED without side effects.

/// Sets the macOS activation policy: "regular" (normal app with Dock icon),
/// "accessory" (no Dock icon, e.g. menu-bar apps) or "prohibited".
#[no_mangle]
pub unsafe extern "C" fn tauri_app_set_activation_policy(app: u64, policy: *const c_char) -> i32 {
  catch(|| {
    let policy = try_cstr!(policy);
    #[cfg(target_os = "macos")]
    {
      let policy = match policy.to_lowercase().as_str() {
        "regular" => tauri::ActivationPolicy::Regular,
        "accessory" => tauri::ActivationPolicy::Accessory,
        "prohibited" => tauri::ActivationPolicy::Prohibited,
        _ => {
          return fail(
            ERR_INVALID_ARG,
            "invalid activation policy (expected \"regular\", \"accessory\" or \"prohibited\")",
          )
        }
      };
      let Some(app_state) = state::app(app) else {
        return fail(ERR_INVALID_HANDLE, "invalid app handle");
      };
      match app_state.handle.set_activation_policy(policy) {
        Ok(()) => OK,
        Err(e) => fail(ERR_GENERIC, format!("set_activation_policy failed: {e}")),
      }
    }
    #[cfg(not(target_os = "macos"))]
    {
      let _ = (app, policy);
      crate::error::unsupported("tauri_app_set_activation_policy", "macOS")
    }
  })
}

/// Shows or hides the app's Dock icon.
#[no_mangle]
pub extern "C" fn tauri_app_set_dock_visibility(app: u64, visible: bool) -> i32 {
  catch(|| {
    #[cfg(target_os = "macos")]
    {
      let Some(app_state) = state::app(app) else {
        return fail(ERR_INVALID_HANDLE, "invalid app handle");
      };
      match app_state.handle.set_dock_visibility(visible) {
        Ok(()) => OK,
        Err(e) => fail(ERR_GENERIC, format!("set_dock_visibility failed: {e}")),
      }
    }
    #[cfg(not(target_os = "macos"))]
    {
      let _ = (app, visible);
      crate::error::unsupported("tauri_app_set_dock_visibility", "macOS")
    }
  })
}

/// Shows the application (all windows), without focusing it.
#[no_mangle]
pub extern "C" fn tauri_app_show(app: u64) -> i32 {
  catch(|| {
    #[cfg(target_os = "macos")]
    {
      let Some(app_state) = state::app(app) else {
        return fail(ERR_INVALID_HANDLE, "invalid app handle");
      };
      match app_state.handle.show() {
        Ok(()) => OK,
        Err(e) => fail(ERR_GENERIC, format!("show failed: {e}")),
      }
    }
    #[cfg(not(target_os = "macos"))]
    {
      let _ = app;
      crate::error::unsupported("tauri_app_show", "macOS")
    }
  })
}

/// Hides the application (all windows), like Cmd+H.
#[no_mangle]
pub extern "C" fn tauri_app_hide(app: u64) -> i32 {
  catch(|| {
    #[cfg(target_os = "macos")]
    {
      let Some(app_state) = state::app(app) else {
        return fail(ERR_INVALID_HANDLE, "invalid app handle");
      };
      match app_state.handle.hide() {
        Ok(()) => OK,
        Err(e) => fail(ERR_GENERIC, format!("hide failed: {e}")),
      }
    }
    #[cfg(not(target_os = "macos"))]
    {
      let _ = app;
      crate::error::unsupported("tauri_app_hide", "macOS")
    }
  })
}
