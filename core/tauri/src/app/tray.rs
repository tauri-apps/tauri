// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use crate::{
  runtime::{
    menu::{
      MenuHash, MenuId, MenuIdRef, MenuUpdate, SystemTrayMenu, SystemTrayMenuEntry, TrayHandle,
    },
    window::dpi::{PhysicalPosition, PhysicalSize},
    SystemTrayEvent as RuntimeSystemTrayEvent,
  },
  Icon, Runtime,
};

use rand::distributions::{Alphanumeric, DistString};
use tauri_macros::default_runtime;
use tauri_runtime::TrayId;
use tauri_utils::debug_eprintln;

use std::{
  collections::{hash_map::DefaultHasher, HashMap},
  hash::{Hash, Hasher},
  sync::{Arc, Mutex},
};

pub(crate) fn get_menu_ids(map: &mut HashMap<MenuHash, MenuId>, menu: &SystemTrayMenu) {
  for item in &menu.items {
    match item {
      SystemTrayMenuEntry::CustomItem(c) => {
        map.insert(c.id, c.id_str.clone());
      }
      SystemTrayMenuEntry::Submenu(s) => get_menu_ids(map, &s.inner),
      _ => {}
    }
  }
}

/// Represents a System Tray instance.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct SystemTray {
  /// The tray identifier. Defaults to a random string.
  pub id: String,
  /// The tray icon.
  pub icon: Option<tauri_runtime::Icon>,
  /// The tray menu.
  pub menu: Option<SystemTrayMenu>,
  /// Whether the icon is a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc) icon or not.
  #[cfg(target_os = "macos")]
  pub icon_as_template: bool,
  /// Whether the menu should appear when the tray receives a left click. Defaults to `true`
  #[cfg(target_os = "macos")]
  pub menu_on_left_click: bool,
}

impl SystemTray {
  /// Creates a new system tray that only renders an icon.
  pub fn new() -> Self {
    let mut tray = Self::default();
    tray.id = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    tray
  }

  pub(crate) fn menu(&self) -> Option<&SystemTrayMenu> {
    self.menu.as_ref()
  }

  /// Sets the tray icon.
  #[must_use]
  pub fn with_icon<I: TryInto<tauri_runtime::Icon>>(mut self, icon: I) -> Self
  where
    I::Error: std::error::Error,
  {
    match icon.try_into() {
      Ok(icon) => {
        self.icon.replace(icon);
      }
      Err(e) => {
        debug_eprintln!("Failed to load tray icon: {}", e);
      }
    }
    self
  }

  /// Sets the icon as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc).
  ///
  /// Images you mark as template images should consist of only black and clear colors.
  /// You can use the alpha channel in the image to adjust the opacity of black content.
  #[cfg(target_os = "macos")]
  #[must_use]
  pub fn with_icon_as_template(mut self, is_template: bool) -> Self {
    self.icon_as_template = is_template;
    self
  }

  /// Sets whether the menu should appear when the tray receives a left click. Defaults to `true`.
  #[cfg(target_os = "macos")]
  #[must_use]
  pub fn with_menu_on_left_click(mut self, menu_on_left_click: bool) -> Self {
    self.menu_on_left_click = menu_on_left_click;
    self
  }

  /// Sets the menu to show when the system tray is right clicked.
  #[must_use]
  pub fn with_menu(mut self, menu: SystemTrayMenu) -> Self {
    self.menu.replace(menu);
    self
  }
}

fn hash(id: &str) -> MenuHash {
  let mut hasher = DefaultHasher::new();
  id.hash(&mut hasher);
  hasher.finish() as MenuHash
}

impl From<SystemTray> for tauri_runtime::SystemTray {
  fn from(tray: SystemTray) -> Self {
    let mut t = tauri_runtime::SystemTray::new();
    t = t.with_id(hash(&tray.id));
    if let Some(i) = tray.icon {
      t = t.with_icon(i);
    }

    if let Some(menu) = tray.menu {
      t = t.with_menu(menu);
    }

    #[cfg(target_os = "macos")]
    {
      t = t.with_icon_as_template(tray.icon_as_template);
      t = t.with_menu_on_left_click(tray.menu_on_left_click);
    }

    t
  }
}

/// System tray event.
#[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
#[non_exhaustive]
pub enum SystemTrayEvent {
  /// Tray context menu item was clicked.
  #[non_exhaustive]
  MenuItemClick {
    /// The tray id.
    tray_id: String,
    /// The id of the menu item.
    id: MenuId,
  },
  /// Tray icon received a left click.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Unsupported
  #[non_exhaustive]
  LeftClick {
    /// The tray id.
    tray_id: String,
    /// The position of the tray icon.
    position: PhysicalPosition<f64>,
    /// The size of the tray icon.
    size: PhysicalSize<f64>,
  },
  /// Tray icon received a right click.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** Unsupported
  /// - **macOS:** `Ctrl` + `Left click` fire this event.
  #[non_exhaustive]
  RightClick {
    /// The tray id.
    tray_id: String,
    /// The position of the tray icon.
    position: PhysicalPosition<f64>,
    /// The size of the tray icon.
    size: PhysicalSize<f64>,
  },
  /// Fired when a menu item receive a `Double click`
  ///
  /// ## Platform-specific
  ///
  /// - **macOS / Linux:** Unsupported
  ///
  #[non_exhaustive]
  DoubleClick {
    /// The tray id.
    tray_id: String,
    /// The position of the tray icon.
    position: PhysicalPosition<f64>,
    /// The size of the tray icon.
    size: PhysicalSize<f64>,
  },
}

impl SystemTrayEvent {
  pub(crate) fn from_runtime_event(
    event: &RuntimeSystemTrayEvent,
    tray_id: String,
    menu_ids: &Arc<Mutex<HashMap<u16, String>>>,
  ) -> Self {
    match event {
      RuntimeSystemTrayEvent::MenuItemClick(id) => Self::MenuItemClick {
        tray_id,
        id: menu_ids.lock().unwrap().get(id).unwrap().clone(),
      },
      RuntimeSystemTrayEvent::LeftClick { position, size } => Self::LeftClick {
        tray_id,
        position: *position,
        size: *size,
      },
      RuntimeSystemTrayEvent::RightClick { position, size } => Self::RightClick {
        tray_id,
        position: *position,
        size: *size,
      },
      RuntimeSystemTrayEvent::DoubleClick { position, size } => Self::DoubleClick {
        tray_id,
        position: *position,
        size: *size,
      },
    }
  }
}

/// A handle to a system tray. Allows updating the context menu items.
#[default_runtime(crate::Wry, wry)]
#[derive(Debug)]
pub struct SystemTrayHandle<R: Runtime> {
  pub(crate) id: TrayId,
  pub(crate) ids: Arc<Mutex<HashMap<MenuHash, MenuId>>>,
  pub(crate) inner: R::TrayHandler,
}

impl<R: Runtime> Clone for SystemTrayHandle<R> {
  fn clone(&self) -> Self {
    Self {
      id: self.id,
      ids: self.ids.clone(),
      inner: self.inner.clone(),
    }
  }
}

/// A handle to a system tray menu item.
#[default_runtime(crate::Wry, wry)]
#[derive(Debug)]
pub struct SystemTrayMenuItemHandle<R: Runtime> {
  id: MenuHash,
  tray_handler: R::TrayHandler,
}

impl<R: Runtime> Clone for SystemTrayMenuItemHandle<R> {
  fn clone(&self) -> Self {
    Self {
      id: self.id,
      tray_handler: self.tray_handler.clone(),
    }
  }
}

impl<R: Runtime> SystemTrayHandle<R> {
  /// Gets a handle to the menu item that has the specified `id`.
  pub fn get_item(&self, id: MenuIdRef<'_>) -> SystemTrayMenuItemHandle<R> {
    let ids = self.ids.lock().unwrap();
    let iter = ids.iter();
    for (raw, item_id) in iter {
      if item_id == id {
        return SystemTrayMenuItemHandle {
          id: *raw,
          tray_handler: self.inner.clone(),
        };
      }
    }
    panic!("item id not found")
  }

  /// Updates the tray icon.
  pub fn set_icon(&self, icon: Icon) -> crate::Result<()> {
    self.inner.set_icon(icon.try_into()?).map_err(Into::into)
  }

  /// Updates the tray menu.
  pub fn set_menu(&self, menu: SystemTrayMenu) -> crate::Result<()> {
    let mut ids = HashMap::new();
    get_menu_ids(&mut ids, &menu);
    self.inner.set_menu(menu)?;
    *self.ids.lock().unwrap() = ids;
    Ok(())
  }

  /// Support [macOS tray icon template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc) to adjust automatically based on taskbar color.
  #[cfg(target_os = "macos")]
  pub fn set_icon_as_template(&self, is_template: bool) -> crate::Result<()> {
    self
      .inner
      .set_icon_as_template(is_template)
      .map_err(Into::into)
  }
}

impl<R: Runtime> SystemTrayMenuItemHandle<R> {
  /// Modifies the enabled state of the menu item.
  pub fn set_enabled(&self, enabled: bool) -> crate::Result<()> {
    self
      .tray_handler
      .update_item(self.id, MenuUpdate::SetEnabled(enabled))
      .map_err(Into::into)
  }

  /// Modifies the title (label) of the menu item.
  pub fn set_title<S: Into<String>>(&self, title: S) -> crate::Result<()> {
    self
      .tray_handler
      .update_item(self.id, MenuUpdate::SetTitle(title.into()))
      .map_err(Into::into)
  }

  /// Modifies the selected state of the menu item.
  pub fn set_selected(&self, selected: bool) -> crate::Result<()> {
    self
      .tray_handler
      .update_item(self.id, MenuUpdate::SetSelected(selected))
      .map_err(Into::into)
  }

  #[cfg(target_os = "macos")]
  #[cfg_attr(doc_cfg, doc(cfg(target_os = "macos")))]
  pub fn set_native_image(&self, image: crate::NativeImage) -> crate::Result<()> {
    self
      .tray_handler
      .update_item(self.id, MenuUpdate::SetNativeImage(image))
      .map_err(Into::into)
  }
}
