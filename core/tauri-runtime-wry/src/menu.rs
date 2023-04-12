use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tauri_runtime::menu::{Menu, MenuEntry, MenuEvent, MenuHash, MenuUpdate};
use wry::application::menu::{CustomMenuItem as WryCustomMenuItem, MenuBar, MenuId};
use wry::application::window::WindowId;

#[cfg(target_os = "macos")]
use crate::wrappers::NativeImageWrapper;
use crate::wrappers::{MenuItemAttributesWrapper, MenuItemWrapper};
use crate::{WebviewIdStore, Window};

pub type MenuEventListenerId = u64;
pub type MenuEventListener = Box<dyn Fn(&MenuEvent) + Send>;
pub type WindowMenuEventListenerStore = Arc<Mutex<HashMap<MenuEventListenerId, MenuEventListener>>>;

pub fn handle_window_menu_update(
  update: MenuUpdate,
  item_id: u16,
  window_id: u64,
  windows: &Arc<RefCell<HashMap<u64, Window>>>,
) {
  if let Some(menu_items) = windows
    .borrow_mut()
    .get_mut(&window_id)
    .map(|w| &mut w.menu_items)
  {
    if let Some(menu_items) = menu_items.as_mut() {
      let item = menu_items.get_mut(&item_id).expect("menu item not found");
      match update {
        MenuUpdate::SetEnabled(enabled) => item.set_enabled(enabled),
        MenuUpdate::SetTitle(title) => item.set_title(&title),
        MenuUpdate::SetSelected(selected) => item.set_selected(selected),
        #[cfg(target_os = "macos")]
        MenuUpdate::SetNativeImage(image) => {
          item.set_native_image(NativeImageWrapper::from(image).0)
        }
      }
    }
  }
}

pub fn handle_window_menu_event(
  item_id: MenuId,
  window_id: Option<WindowId>,
  windows: &Arc<RefCell<HashMap<u64, Window>>>,
  webview_id_store: &WebviewIdStore,
) {
  #[allow(unused_mut)]
  let mut window_id = window_id.unwrap(); // always Some on menu event

  #[cfg(target_os = "macos")]
  {
    // safety: we're only checking to see if the window_id is 0
    // which is the value sent by macOS when the window is minimized (NSApplication::sharedApplication::mainWindow is null)
    if window_id == unsafe { WindowId::dummy() } {
      window_id = *webview_id_map.0.lock().unwrap().keys().next().unwrap();
    }
  }

  let event = MenuEvent {
    menu_item_id: item_id.0,
  };
  let window_menu_event_listeners = {
    // on macOS the window id might be the inspector window if it is detached
    let webview_id = if let Some(webview_id) = webview_id_store.get(&window_id) {
      webview_id
    } else {
      *webview_id_store.0.lock().unwrap().values().next().unwrap()
    };
    windows
      .borrow()
      .get(&webview_id)
      .unwrap()
      .menu_event_listeners_store
      .clone()
  };
  let listeners = window_menu_event_listeners.lock().unwrap();
  let handlers = listeners.values();
  for handler in handlers {
    handler(&event);
  }
}

pub fn to_wry_menu(
  custom_menu_items: &mut HashMap<MenuHash, WryCustomMenuItem>,
  menu: Menu,
) -> MenuBar {
  let mut wry_menu = MenuBar::new();
  for item in menu.items {
    match item {
      MenuEntry::CustomItem(c) => {
        let mut attributes = MenuItemAttributesWrapper::from(&c).0;
        attributes = attributes.with_id(MenuId(c.id));
        #[allow(unused_mut)]
        let mut item = wry_menu.add_item(attributes);
        #[cfg(target_os = "macos")]
        if let Some(native_image) = c.native_image {
          item.set_native_image(NativeImageWrapper::from(native_image).0);
        }
        custom_menu_items.insert(c.id, item);
      }
      MenuEntry::NativeItem(i) => {
        wry_menu.add_native_item(MenuItemWrapper::from(i).0);
      }
      MenuEntry::Submenu(submenu) => {
        wry_menu.add_submenu(
          &submenu.title,
          submenu.enabled,
          to_wry_menu(custom_menu_items, submenu.inner),
        );
      }
    }
  }
  wry_menu
}
