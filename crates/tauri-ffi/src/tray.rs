// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! System tray icons. A tray is an opaque handle built while the app runs;
//! each `tauri_tray_*` function mirrors a method on [`tauri::tray::TrayIcon`].
//! Tray events (click, enter, leave, move) are delivered through the app's
//! event queue as `{"type":"tray-event","id":...,"event":{...}}`.
//!
//! Menus are not yet exposed over the ABI, so trays here are icon + tooltip +
//! title + event handlers. Attaching a native menu is a follow-up once the
//! menu object model lands.

use std::os::raw::c_char;

use tauri::image::Image;
use tauri::tray::{TrayIcon, TrayIconBuilder};
use crate::Rt as TauriRuntime;

use crate::error::{catch, fail, ERR_GENERIC, ERR_INVALID_ARG, ERR_INVALID_HANDLE, OK};
use crate::state::{self, Entry};
use crate::{try_cstr, write_owned_str};

fn with_tray(tray: u64, f: impl FnOnce(&TrayIcon<TauriRuntime>) -> tauri::Result<()>) -> i32 {
  let Some(tray) = state::tray(tray) else {
    return fail(ERR_INVALID_HANDLE, "invalid tray handle");
  };
  match f(&tray) {
    Ok(()) => OK,
    Err(e) => fail(ERR_GENERIC, e.to_string()),
  }
}

/// Creates a system tray icon. `id` names the tray (empty = auto-assigned);
/// use it with `tauri_app_remove_tray_by_id`. On OK, `*out_tray` is a tray
/// handle; release with `tauri_handle_close` (dropping the last handle removes
/// the icon).
#[no_mangle]
pub unsafe extern "C" fn tauri_tray_new(app: u64, id: *const c_char, out_tray: *mut u64) -> i32 {
  catch(|| {
    let id = try_cstr!(id);
    if out_tray.is_null() {
      return fail(ERR_INVALID_ARG, "out_tray is null");
    }
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    let tx = app_state.tx.clone();
    let builder = if id.is_empty() {
      TrayIconBuilder::<TauriRuntime>::new()
    } else {
      TrayIconBuilder::<TauriRuntime>::with_id(id.to_string())
    };
    let builder = builder.on_tray_icon_event(move |tray, event| {
      let message = serde_json::json!({
        "type": "tray-event",
        "id": tray.id().0.clone(),
        "event": serde_json::to_value(&event).unwrap_or(serde_json::Value::Null),
      });
      let _ = tx.send(message.to_string());
    });
    match builder.build(&app_state.handle) {
      Ok(tray) => {
        unsafe { *out_tray = state::insert(Entry::Tray(tray)) };
        OK
      }
      Err(e) => fail(ERR_GENERIC, format!("failed to create tray: {e}")),
    }
  })
}

/// The tray's id, as an owned string.
#[no_mangle]
pub unsafe extern "C" fn tauri_tray_id(tray: u64, out_id: *mut *mut c_char) -> i32 {
  catch(|| {
    if out_id.is_null() {
      return fail(ERR_INVALID_ARG, "out_id is null");
    }
    let Some(tray) = state::tray(tray) else {
      return fail(ERR_INVALID_HANDLE, "invalid tray handle");
    };
    write_owned_str(out_id, tray.id().0.clone());
    OK
  })
}

/// Sets the tray icon from an image file (PNG or ICO). An empty path clears it.
#[no_mangle]
pub unsafe extern "C" fn tauri_tray_set_icon(tray: u64, path: *const c_char) -> i32 {
  catch(|| {
    let path = try_cstr!(path);
    if path.is_empty() {
      return with_tray(tray, |t| t.set_icon(None));
    }
    let image = match Image::from_path(path) {
      Ok(image) => image,
      Err(e) => return fail(ERR_INVALID_ARG, format!("failed to load tray icon `{path}`: {e}")),
    };
    with_tray(tray, |t| t.set_icon(Some(image)))
  })
}

/// Whether the icon is treated as a macOS template image (auto-recolored).
#[no_mangle]
pub extern "C" fn tauri_tray_set_icon_as_template(tray: u64, is_template: bool) -> i32 {
  catch(|| with_tray(tray, |t| t.set_icon_as_template(is_template)))
}

/// Sets the tray tooltip. An empty string clears it.
#[no_mangle]
pub unsafe extern "C" fn tauri_tray_set_tooltip(tray: u64, tooltip: *const c_char) -> i32 {
  catch(|| {
    let tooltip = try_cstr!(tooltip);
    let tooltip = (!tooltip.is_empty()).then(|| tooltip.to_string());
    with_tray(tray, |t| t.set_tooltip(tooltip))
  })
}

/// Sets the tray title (shown next to the icon on macOS/Linux). Empty clears it.
#[no_mangle]
pub unsafe extern "C" fn tauri_tray_set_title(tray: u64, title: *const c_char) -> i32 {
  catch(|| {
    let title = try_cstr!(title);
    let title = (!title.is_empty()).then(|| title.to_string());
    with_tray(tray, |t| t.set_title(title))
  })
}

/// Shows or hides the tray icon.
#[no_mangle]
pub extern "C" fn tauri_tray_set_visible(tray: u64, visible: bool) -> i32 {
  catch(|| with_tray(tray, |t| t.set_visible(visible)))
}

/// Whether a left click opens the tray menu (when one is attached).
#[no_mangle]
pub extern "C" fn tauri_tray_set_show_menu_on_left_click(tray: u64, enable: bool) -> i32 {
  catch(|| with_tray(tray, |t| t.set_show_menu_on_left_click(enable)))
}

/// Removes the tray with the given id from the app, if present. Idempotent.
#[no_mangle]
pub unsafe extern "C" fn tauri_app_remove_tray_by_id(app: u64, id: *const c_char) -> i32 {
  catch(|| {
    let id = try_cstr!(id);
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    app_state.handle.remove_tray_by_id(id);
    OK
  })
}
