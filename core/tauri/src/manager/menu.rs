// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  sync::{Mutex, MutexGuard},
};

use crate::{
  menu::{Menu, MenuId},
  AppHandle, Runtime, Window,
};

pub struct MenuManager<R: Runtime> {
  /// A set containing a reference to the active menus, including
  /// the app-wide menu and the window-specific menus
  ///
  /// This should be mainly used to acceess [`Menu::haccel`]
  /// to setup the accelerator handling in the event loop
  pub menus: Mutex<HashMap<MenuId, Menu<R>>>,
  /// The menu set to all windows.
  pub menu: Mutex<Option<Menu<R>>>,
  /// Menu event listeners to all windows.
  pub global_event_listeners: Mutex<Vec<crate::app::GlobalMenuEventListener<AppHandle<R>>>>,
  /// Menu event listeners to specific windows.
  pub event_listeners: Mutex<HashMap<String, crate::app::GlobalMenuEventListener<Window<R>>>>,
}

impl<R: Runtime> MenuManager<R> {
  /// App-wide menu.
  pub fn menu_lock(&self) -> MutexGuard<'_, Option<Menu<R>>> {
    self.menu.lock().expect("poisoned menu mutex")
  }

  pub fn menus_stash_lock(&self) -> MutexGuard<'_, HashMap<MenuId, Menu<R>>> {
    self.menus.lock().expect("poisoned menu mutex")
  }

  pub fn is_menu_in_use<I: PartialEq<MenuId>>(&self, id: &I) -> bool {
    self
      .menu_lock()
      .as_ref()
      .map(|m| id.eq(m.id()))
      .unwrap_or(false)
  }

  pub fn insert_menu_into_stash(&self, menu: &Menu<R>) {
    self
      .menus_stash_lock()
      .insert(menu.id().clone(), menu.clone());
  }

  pub(crate) fn prepare_window_menu_creation_handler(
    &self,
    window_menu: Option<&crate::window::WindowMenu<R>>,
  ) -> Option<impl Fn(tauri_runtime::window::RawWindow<'_>)> {
    {
      if let Some(menu) = window_menu {
        self
          .menus_stash_lock()
          .insert(menu.menu.id().clone(), menu.menu.clone());
      }
    }

    #[cfg(target_os = "macos")]
    return None;

    #[cfg_attr(target_os = "macos", allow(unused_variables, unreachable_code))]
    if let Some(menu) = &window_menu {
      let menu = menu.menu.clone();
      Some(move |raw: tauri_runtime::window::RawWindow<'_>| {
        #[cfg(target_os = "windows")]
        let _ = menu.inner().init_for_hwnd(raw.hwnd as _);
        #[cfg(any(
          target_os = "linux",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "netbsd",
          target_os = "openbsd"
        ))]
        let _ = menu
          .inner()
          .init_for_gtk_window(raw.gtk_window, raw.default_vbox);
      })
    } else {
      None
    }
  }
}
