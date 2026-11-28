// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  sync::{Arc, Mutex, MutexGuard, Weak},
};

use crate::{
  AppHandle, Runtime, Window,
  menu::{Menu, MenuEvent, MenuId},
};

pub struct MenuItemEventListener {
  item: Weak<dyn Send + Sync>,
  handler: Arc<dyn Fn(MenuEvent) + Send + Sync>,
}

pub struct MenuManager<R: Runtime> {
  /// A set containing a reference to the active menus, including
  /// the app-wide menu and the window-specific menus
  ///
  /// This should be mainly used to access [`Menu::haccel`]
  /// to setup the accelerator handling in the event loop
  pub menus: Arc<Mutex<HashMap<MenuId, Menu<R>>>>,
  /// The menu set to all windows.
  pub menu: Mutex<Option<Menu<R>>>,
  /// Menu event listeners to all windows.
  pub global_event_listeners: Mutex<Vec<crate::app::GlobalMenuEventListener<AppHandle<R>>>>,
  /// Menu event listeners to specific windows.
  pub event_listeners: Mutex<HashMap<String, crate::app::GlobalMenuEventListener<Window<R>>>>,
  /// Menu event listeners to specific items, without keeping the items alive.
  pub item_event_listeners: Mutex<HashMap<MenuId, MenuItemEventListener>>,
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
    #[allow(unused)] theme: Option<tauri_utils::Theme>,
  ) -> Option<impl Fn(tauri_runtime::window::RawWindow<'_>) + use<R>> {
    let window_menu = window_menu?;

    self
      .menus_stash_lock()
      .insert(window_menu.menu.id().clone(), window_menu.menu.clone());

    #[cfg(target_os = "macos")]
    return None;

    #[cfg_attr(target_os = "macos", allow(unused_variables, unreachable_code))]
    // Linux without a `gtk3`/`gtk4` feature has no menu integration to run.
    #[cfg_attr(all(not(windows), not(gtk)), allow(unused_variables))]
    {
      let menu = window_menu.menu.clone();
      Some(move |raw: tauri_runtime::window::RawWindow<'_>| {
        #[cfg(target_os = "windows")]
        {
          let theme = theme
            .map(crate::menu::map_to_menu_theme)
            .unwrap_or(muda::MenuTheme::Auto);
          let _ = unsafe { menu.inner().init_for_hwnd_with_theme(raw.hwnd as _, theme) };
        }
        #[cfg(gtk)]
        if crate::gtk_version::check().is_ok() {
          use gtk::glib::translate::FromGlibPtrNone;

          let gtk_window = unsafe {
            gtk::ApplicationWindow::from_glib_none(
              raw.gtk_window as *mut gtk::ffi::GtkApplicationWindow,
            )
          };
          let default_vbox = raw
            .default_vbox
            .map(|vbox| unsafe { gtk::Box::from_glib_none(vbox as *mut gtk::ffi::GtkBox) });
          let _ = menu
            .inner()
            .init_for_gtk_window(&gtk_window, default_vbox.as_ref());
        }
      })
    }
  }

  pub fn on_menu_event<F: Fn(&AppHandle<R>, MenuEvent) + Send + Sync + 'static>(&self, handler: F) {
    self
      .global_event_listeners
      .lock()
      .unwrap()
      .push(Box::new(handler));
  }

  pub fn on_menu_item_event<T, F>(&self, id: MenuId, item: &Arc<T>, handler: F)
  where
    T: Send + Sync + 'static,
    F: Fn(Arc<T>, MenuEvent) + Send + Sync + 'static,
  {
    let weak_item = Arc::downgrade(item);
    let listener = MenuItemEventListener {
      item: weak_item.clone(),
      handler: Arc::new(move |event| {
        if let Some(item) = weak_item.upgrade() {
          handler(item, event);
        }
      }),
    };

    let mut listeners = self.item_event_listeners.lock().unwrap();
    let previous = listeners.insert(id, listener);

    // Drop any replaced handler after releasing the lock: its captures may own menu items.
    drop(listeners);
    drop(previous);
  }

  pub fn dispatch_menu_item_event(&self, event: MenuEvent) {
    let listeners = self.item_event_listeners.lock().unwrap();
    let handler = listeners
      .get(&event.id)
      .map(|listener| listener.handler.clone());

    // The handler may create or drop items, so never call it while holding the lock.
    drop(listeners);

    if let Some(handler) = handler {
      handler(event);
    }
  }

  pub fn remove_menu_item_event(&self, id: &MenuId) {
    let mut listeners = self.item_event_listeners.lock().unwrap();

    // A different, still-live item may have registered a handler with the same ID.
    let listener = listeners
      .get(id)
      .is_some_and(|listener| listener.item.strong_count() == 0)
      .then(|| listeners.remove(id))
      .flatten();

    // Order matters: release the lock before dropping captures that may re-enter menu cleanup.
    drop(listeners);
    drop(listener);
  }
}
