// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Native menus. Build a menu tree from opaque handles — a root [`Menu`], then
//! normal / check / predefined items and submenus — then attach it as the app
//! menu, a window menu, or a tray menu. Menu clicks arrive on the event queue
//! as `{"type":"menu-event","id":<item id>}` (wired up in [`crate::build_app`]).
//!
//! [`Menu`]: tauri::menu::Menu

use std::os::raw::c_char;

use tauri::menu::{CheckMenuItem, Menu, MenuItem, MenuItemKind, PredefinedMenuItem, Submenu};

use crate::error::{catch, fail, ERR_GENERIC, ERR_INVALID_ARG, ERR_INVALID_HANDLE, OK};
use crate::state::{self, Entry};
use crate::{try_cstr, write_owned_str};

fn opt(s: &str) -> Option<&str> {
  (!s.is_empty()).then_some(s)
}

fn store_item(item: MenuItemKind<TauriRuntime>, out: *mut u64) -> i32 {
  unsafe { *out = state::insert(Entry::MenuItem(item)) };
  OK
}

use crate::Rt as TauriRuntime;

// ---------------------------------------------------------------------------
// construction

/// Creates an empty menu. On OK, `*out_menu` is a menu handle.
#[no_mangle]
pub unsafe extern "C" fn tauri_menu_new(app: u64, out_menu: *mut u64) -> i32 {
  catch(|| {
    if out_menu.is_null() {
      return fail(ERR_INVALID_ARG, "out_menu is null");
    }
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    match Menu::new(&app_state.handle) {
      Ok(menu) => {
        unsafe { *out_menu = state::insert(Entry::Menu(menu)) };
        OK
      }
      Err(e) => fail(ERR_GENERIC, format!("failed to create menu: {e}")),
    }
  })
}

/// Creates a normal menu item. `id` (empty = auto), `accelerator` (empty = none).
#[no_mangle]
pub unsafe extern "C" fn tauri_menu_item_new(
  app: u64,
  id: *const c_char,
  text: *const c_char,
  enabled: bool,
  accelerator: *const c_char,
  out_item: *mut u64,
) -> i32 {
  catch(|| {
    let id = try_cstr!(id);
    let text = try_cstr!(text);
    let accel = try_cstr!(accelerator);
    if out_item.is_null() {
      return fail(ERR_INVALID_ARG, "out_item is null");
    }
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    let item = if id.is_empty() {
      MenuItem::new(&app_state.handle, text, enabled, opt(accel))
    } else {
      MenuItem::with_id(&app_state.handle, id, text, enabled, opt(accel))
    };
    match item {
      Ok(item) => store_item(MenuItemKind::MenuItem(item), out_item),
      Err(e) => fail(ERR_GENERIC, format!("failed to create menu item: {e}")),
    }
  })
}

/// Creates a checkable menu item.
#[no_mangle]
pub unsafe extern "C" fn tauri_menu_check_item_new(
  app: u64,
  id: *const c_char,
  text: *const c_char,
  enabled: bool,
  checked: bool,
  accelerator: *const c_char,
  out_item: *mut u64,
) -> i32 {
  catch(|| {
    let id = try_cstr!(id);
    let text = try_cstr!(text);
    let accel = try_cstr!(accelerator);
    if out_item.is_null() {
      return fail(ERR_INVALID_ARG, "out_item is null");
    }
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    let item = if id.is_empty() {
      CheckMenuItem::new(&app_state.handle, text, enabled, checked, opt(accel))
    } else {
      CheckMenuItem::with_id(&app_state.handle, id, text, enabled, checked, opt(accel))
    };
    match item {
      Ok(item) => store_item(MenuItemKind::Check(item), out_item),
      Err(e) => fail(ERR_GENERIC, format!("failed to create check item: {e}")),
    }
  })
}

/// Creates a predefined menu item. `kind` is one of: separator, copy, cut,
/// paste, select_all, undo, redo, minimize, maximize, fullscreen, hide,
/// hide_others, show_all, close_window, quit, about, services. `text` (empty =
/// default) overrides the label where supported.
#[no_mangle]
pub unsafe extern "C" fn tauri_menu_predefined_item_new(
  app: u64,
  kind: *const c_char,
  text: *const c_char,
  out_item: *mut u64,
) -> i32 {
  catch(|| {
    let kind = try_cstr!(kind);
    let text = try_cstr!(text);
    if out_item.is_null() {
      return fail(ERR_INVALID_ARG, "out_item is null");
    }
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    let m = &app_state.handle;
    let t = opt(text);
    let item = match kind {
      "separator" => PredefinedMenuItem::separator(m),
      "copy" => PredefinedMenuItem::copy(m, t),
      "cut" => PredefinedMenuItem::cut(m, t),
      "paste" => PredefinedMenuItem::paste(m, t),
      "select_all" => PredefinedMenuItem::select_all(m, t),
      "undo" => PredefinedMenuItem::undo(m, t),
      "redo" => PredefinedMenuItem::redo(m, t),
      "minimize" => PredefinedMenuItem::minimize(m, t),
      "maximize" => PredefinedMenuItem::maximize(m, t),
      "fullscreen" => PredefinedMenuItem::fullscreen(m, t),
      "hide" => PredefinedMenuItem::hide(m, t),
      "hide_others" => PredefinedMenuItem::hide_others(m, t),
      "show_all" => PredefinedMenuItem::show_all(m, t),
      "close_window" => PredefinedMenuItem::close_window(m, t),
      "quit" => PredefinedMenuItem::quit(m, t),
      "about" => PredefinedMenuItem::about(m, t, None),
      "services" => PredefinedMenuItem::services(m, t),
      other => {
        return fail(
          ERR_INVALID_ARG,
          format!("unknown predefined item `{other}`"),
        )
      }
    };
    match item {
      Ok(item) => store_item(MenuItemKind::Predefined(item), out_item),
      Err(e) => fail(
        ERR_GENERIC,
        format!("failed to create predefined item: {e}"),
      ),
    }
  })
}

/// Creates a submenu (a menu that is also an item). Append children with
/// `tauri_submenu_append`.
#[no_mangle]
pub unsafe extern "C" fn tauri_submenu_new(
  app: u64,
  id: *const c_char,
  text: *const c_char,
  enabled: bool,
  out_submenu: *mut u64,
) -> i32 {
  catch(|| {
    let id = try_cstr!(id);
    let text = try_cstr!(text);
    if out_submenu.is_null() {
      return fail(ERR_INVALID_ARG, "out_submenu is null");
    }
    let Some(app_state) = state::app(app) else {
      return fail(ERR_INVALID_HANDLE, "invalid app handle");
    };
    let submenu = if id.is_empty() {
      Submenu::new(&app_state.handle, text, enabled)
    } else {
      Submenu::with_id(&app_state.handle, id, text, enabled)
    };
    match submenu {
      Ok(submenu) => store_item(MenuItemKind::Submenu(submenu), out_submenu),
      Err(e) => fail(ERR_GENERIC, format!("failed to create submenu: {e}")),
    }
  })
}

// ---------------------------------------------------------------------------
// composition

fn append_item<F>(item: u64, append: F) -> i32
where
  F: FnOnce(&MenuItemKind<TauriRuntime>) -> tauri::Result<()>,
{
  let Some(kind) = state::menu_item(item) else {
    return fail(ERR_INVALID_HANDLE, "invalid menu item handle");
  };
  match append(&kind) {
    Ok(()) => OK,
    Err(e) => fail(ERR_GENERIC, e.to_string()),
  }
}

macro_rules! append_kind {
  ($target:expr, $kind:expr) => {
    match $kind {
      MenuItemKind::MenuItem(i) => $target.append(i),
      MenuItemKind::Submenu(i) => $target.append(i),
      MenuItemKind::Predefined(i) => $target.append(i),
      MenuItemKind::Check(i) => $target.append(i),
      MenuItemKind::Icon(i) => $target.append(i),
    }
  };
}

/// Appends an item to a root menu.
#[no_mangle]
pub extern "C" fn tauri_menu_append(menu: u64, item: u64) -> i32 {
  catch(|| {
    let Some(menu) = state::menu(menu) else {
      return fail(ERR_INVALID_HANDLE, "invalid menu handle");
    };
    append_item(item, |kind| append_kind!(menu, kind))
  })
}

/// Appends an item to a submenu (an item handle created by `tauri_submenu_new`).
#[no_mangle]
pub extern "C" fn tauri_submenu_append(submenu: u64, item: u64) -> i32 {
  catch(|| {
    let Some(MenuItemKind::Submenu(submenu)) = state::menu_item(submenu) else {
      return fail(ERR_INVALID_HANDLE, "invalid submenu handle");
    };
    append_item(item, |kind| append_kind!(submenu, kind))
  })
}

// ---------------------------------------------------------------------------
// attach

/// Sets a menu as the app-wide menu (macOS menu bar / Windows fallback).
#[no_mangle]
pub extern "C" fn tauri_menu_set_as_app_menu(menu: u64) -> i32 {
  catch(|| {
    let Some(menu) = state::menu(menu) else {
      return fail(ERR_INVALID_HANDLE, "invalid menu handle");
    };
    match menu.set_as_app_menu() {
      Ok(_) => OK,
      Err(e) => fail(ERR_GENERIC, e.to_string()),
    }
  })
}

/// Sets a menu as a bare window's menu.
#[no_mangle]
pub extern "C" fn tauri_menu_set_as_window_menu(window: u64, menu: u64) -> i32 {
  catch(|| {
    let Some(window) = state::bare_window(window) else {
      return fail(ERR_INVALID_HANDLE, "invalid window handle");
    };
    let Some(menu) = state::menu(menu) else {
      return fail(ERR_INVALID_HANDLE, "invalid menu handle");
    };
    match menu.set_as_window_menu(&window) {
      Ok(_) => OK,
      Err(e) => fail(ERR_GENERIC, e.to_string()),
    }
  })
}

/// Sets (or clears, when menu==0) a tray's context menu.
#[no_mangle]
pub extern "C" fn tauri_tray_set_menu(tray: u64, menu: u64) -> i32 {
  catch(|| {
    let Some(tray) = state::tray(tray) else {
      return fail(ERR_INVALID_HANDLE, "invalid tray handle");
    };
    let result = if menu == 0 {
      tray.set_menu(None::<Menu<TauriRuntime>>)
    } else {
      let Some(menu) = state::menu(menu) else {
        return fail(ERR_INVALID_HANDLE, "invalid menu handle");
      };
      tray.set_menu(Some(menu))
    };
    match result {
      Ok(()) => OK,
      Err(e) => fail(ERR_GENERIC, e.to_string()),
    }
  })
}

// ---------------------------------------------------------------------------
// item mutators

/// The item's id, as an owned string.
#[no_mangle]
pub unsafe extern "C" fn tauri_menu_item_id(item: u64, out_id: *mut *mut c_char) -> i32 {
  catch(|| {
    if out_id.is_null() {
      return fail(ERR_INVALID_ARG, "out_id is null");
    }
    let Some(kind) = state::menu_item(item) else {
      return fail(ERR_INVALID_HANDLE, "invalid menu item handle");
    };
    write_owned_str(out_id, kind.id().0.clone());
    OK
  })
}

fn item_op(item: u64, f: impl FnOnce(&MenuItemKind<TauriRuntime>) -> tauri::Result<()>) -> i32 {
  let Some(kind) = state::menu_item(item) else {
    return fail(ERR_INVALID_HANDLE, "invalid menu item handle");
  };
  match f(&kind) {
    Ok(()) => OK,
    Err(e) => fail(ERR_GENERIC, e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn tauri_menu_item_set_text(item: u64, text: *const c_char) -> i32 {
  catch(|| {
    let text = try_cstr!(text);
    item_op(item, |kind| match kind {
      MenuItemKind::MenuItem(i) => i.set_text(text),
      MenuItemKind::Submenu(i) => i.set_text(text),
      MenuItemKind::Check(i) => i.set_text(text),
      MenuItemKind::Icon(i) => i.set_text(text),
      MenuItemKind::Predefined(i) => i.set_text(text),
    })
  })
}

#[no_mangle]
pub extern "C" fn tauri_menu_item_set_enabled(item: u64, enabled: bool) -> i32 {
  catch(|| {
    item_op(item, |kind| match kind {
      MenuItemKind::MenuItem(i) => i.set_enabled(enabled),
      MenuItemKind::Submenu(i) => i.set_enabled(enabled),
      MenuItemKind::Check(i) => i.set_enabled(enabled),
      MenuItemKind::Icon(i) => i.set_enabled(enabled),
      MenuItemKind::Predefined(_) => Ok(()),
    })
  })
}

/// Sets the checked state (no-op on non-check items).
#[no_mangle]
pub extern "C" fn tauri_menu_item_set_checked(item: u64, checked: bool) -> i32 {
  catch(|| {
    item_op(item, |kind| match kind {
      MenuItemKind::Check(i) => i.set_checked(checked),
      _ => Ok(()),
    })
  })
}

/// Sets the accelerator (empty string clears it; no-op on unsupported items).
#[no_mangle]
pub unsafe extern "C" fn tauri_menu_item_set_accelerator(
  item: u64,
  accelerator: *const c_char,
) -> i32 {
  catch(|| {
    let accel = try_cstr!(accelerator);
    let accel = opt(accel);
    item_op(item, |kind| match kind {
      MenuItemKind::MenuItem(i) => i.set_accelerator(accel),
      MenuItemKind::Check(i) => i.set_accelerator(accel),
      MenuItemKind::Icon(i) => i.set_accelerator(accel),
      _ => Ok(()),
    })
  })
}
