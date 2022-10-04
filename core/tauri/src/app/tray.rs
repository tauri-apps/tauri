// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use crate::{
  runtime::{
    menu::{
      MenuHash, MenuId, MenuIdRef, MenuUpdate, SystemTrayMenu, SystemTrayMenuEntry, TrayHandle,
    },
    window::dpi::{PhysicalPosition, PhysicalSize},
    RuntimeHandle, SystemTrayEvent as RuntimeSystemTrayEvent,
  },
  Icon, Runtime,
};
use crate::{sealed::RuntimeOrDispatch, Manager};

use rand::distributions::{Alphanumeric, DistString};
use tauri_macros::default_runtime;
use tauri_runtime::TrayId;
use tauri_utils::debug_eprintln;

use std::{
  collections::{hash_map::DefaultHasher, HashMap},
  fmt,
  hash::{Hash, Hasher},
  sync::{Arc, Mutex},
};

type TrayEventHandler = dyn Fn(SystemTrayEvent) + Send + Sync + 'static;

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
#[derive(Clone)]
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
  on_event: Option<Arc<TrayEventHandler>>,
  // TODO: icon_as_template and menu_on_left_click should be an Option instead :(
  #[cfg(target_os = "macos")]
  menu_on_left_click_set: bool,
  #[cfg(target_os = "macos")]
  icon_as_template_set: bool,
  #[cfg(target_os = "macos")]
  title: Option<String>,
}

impl fmt::Debug for SystemTray {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut d = f.debug_struct("SystemTray");
    d.field("id", &self.id)
      .field("icon", &self.icon)
      .field("menu", &self.menu);
    #[cfg(target_os = "macos")]
    {
      d.field("icon_as_template", &self.icon_as_template)
        .field("menu_on_left_click", &self.menu_on_left_click);
    }
    d.finish()
  }
}

impl Default for SystemTray {
  fn default() -> Self {
    Self {
      id: Alphanumeric.sample_string(&mut rand::thread_rng(), 16),
      icon: None,
      menu: None,
      on_event: None,
      #[cfg(target_os = "macos")]
      icon_as_template: false,
      #[cfg(target_os = "macos")]
      menu_on_left_click: false,
      #[cfg(target_os = "macos")]
      icon_as_template_set: false,
      #[cfg(target_os = "macos")]
      menu_on_left_click_set: false,
      #[cfg(target_os = "macos")]
      title: None,
    }
  }
}

impl SystemTray {
  /// Creates a new system tray that only renders an icon.
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::SystemTray;
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let tray_handle = SystemTray::new().build(app)?;
  ///     Ok(())
  ///   });
  /// ```
  pub fn new() -> Self {
    Default::default()
  }

  pub(crate) fn menu(&self) -> Option<&SystemTrayMenu> {
    self.menu.as_ref()
  }

  /// Sets the tray identifier, used to retrieve its handle and to identify a tray event source.
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::SystemTray;
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let tray_handle = SystemTray::new()
  ///       .with_id("tray-id")
  ///       .build(app)?;
  ///     Ok(())
  ///   });
  /// ```
  #[must_use]
  pub fn with_id<I: Into<String>>(mut self, id: I) -> Self {
    self.id = id.into();
    self
  }

  /// Sets the tray [`Icon`].
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::{Icon, SystemTray};
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let tray_handle = SystemTray::new()
  ///       // dummy and invalid Rgba icon; see the Icon documentation for more information
  ///       .with_icon(Icon::Rgba { rgba: Vec::new(), width: 0, height: 0 })
  ///       .build(app)?;
  ///     Ok(())
  ///   });
  /// ```
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
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::SystemTray;
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let mut tray_builder = SystemTray::new();
  ///     #[cfg(target_os = "macos")]
  ///     {
  ///       tray_builder = tray_builder.with_icon_as_template(true);
  ///     }
  ///     let tray_handle = tray_builder.build(app)?;
  ///     Ok(())
  ///   });
  /// ```
  #[cfg(target_os = "macos")]
  #[must_use]
  pub fn with_icon_as_template(mut self, is_template: bool) -> Self {
    self.icon_as_template_set = true;
    self.icon_as_template = is_template;
    self
  }

  /// Sets whether the menu should appear when the tray receives a left click. Defaults to `true`.
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::SystemTray;
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let mut tray_builder = SystemTray::new();
  ///     #[cfg(target_os = "macos")]
  ///     {
  ///       tray_builder = tray_builder.with_menu_on_left_click(false);
  ///     }
  ///     let tray_handle = tray_builder.build(app)?;
  ///     Ok(())
  ///   });
  /// ```
  #[cfg(target_os = "macos")]
  #[must_use]
  pub fn with_menu_on_left_click(mut self, menu_on_left_click: bool) -> Self {
    self.menu_on_left_click_set = true;
    self.menu_on_left_click = menu_on_left_click;
    self
  }

  /// Sets the menu title`
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::SystemTray;
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let mut tray_builder = SystemTray::new();
  ///     #[cfg(target_os = "macos")]
  ///     {
  ///       tray_builder = tray_builder.with_title("My App");
  ///     }
  ///     let tray_handle = tray_builder.build(app)?;
  ///     Ok(())
  ///   });
  /// ```
  #[cfg(target_os = "macos")]
  #[must_use]
  pub fn with_title(mut self, title: &str) -> Self {
    self.title = Some(title.to_owned());
    self
  }

  /// Sets the event listener for this system tray.
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::{Icon, Manager, SystemTray, SystemTrayEvent};
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let handle = app.handle();
  ///     let id = "tray-id";
  ///     SystemTray::new()
  ///       .with_id(id)
  ///       .on_event(move |event| {
  ///         let tray_handle = handle.tray_handle_by_id(id).unwrap();
  ///         match event {
  ///           // show window with id "main" when the tray is left clicked
  ///           SystemTrayEvent::LeftClick { .. } => {
  ///             let window = handle.get_window("main").unwrap();
  ///             window.show().unwrap();
  ///             window.set_focus().unwrap();
  ///           }
  ///           _ => {}
  ///         }
  ///       })
  ///       .build(app)?;
  ///     Ok(())
  ///   });
  /// ```
  #[must_use]
  pub fn on_event<F: Fn(SystemTrayEvent) + Send + Sync + 'static>(mut self, f: F) -> Self {
    self.on_event.replace(Arc::new(f));
    self
  }

  /// Sets the menu to show when the system tray is right clicked.
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu};
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let tray_handle = SystemTray::new()
  ///       .with_menu(
  ///         SystemTrayMenu::new()
  ///           .add_item(CustomMenuItem::new("quit", "Quit"))
  ///           .add_item(CustomMenuItem::new("open", "Open"))
  ///       )
  ///       .build(app)?;
  ///     Ok(())
  ///   });
  /// ```
  #[must_use]
  pub fn with_menu(mut self, menu: SystemTrayMenu) -> Self {
    self.menu.replace(menu);
    self
  }

  /// Builds and shows the system tray.
  ///
  /// # Examples
  ///
  /// ```
  /// use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu};
  ///
  /// tauri::Builder::default()
  ///   .setup(|app| {
  ///     let tray_handle = SystemTray::new()
  ///       .with_menu(
  ///         SystemTrayMenu::new()
  ///           .add_item(CustomMenuItem::new("quit", "Quit"))
  ///           .add_item(CustomMenuItem::new("open", "Open"))
  ///       )
  ///       .build(app)?;
  ///
  ///       tray_handle.get_item("quit").set_enabled(false);
  ///     Ok(())
  ///   });
  /// ```
  pub fn build<R: Runtime, M: Manager<R>>(
    mut self,
    manager: &M,
  ) -> crate::Result<SystemTrayHandle<R>> {
    let mut ids = HashMap::new();
    if let Some(menu) = self.menu() {
      get_menu_ids(&mut ids, menu);
    }
    let ids = Arc::new(Mutex::new(ids));

    if self.icon.is_none() {
      if let Some(tray_icon) = &manager.manager().inner.tray_icon {
        self = self.with_icon(tray_icon.clone());
      }
    }
    #[cfg(target_os = "macos")]
    {
      if !self.icon_as_template_set {
        self.icon_as_template = manager
          .config()
          .tauri
          .system_tray
          .as_ref()
          .map_or(false, |t| t.icon_as_template);
      }
      if !self.menu_on_left_click_set {
        self.menu_on_left_click = manager
          .config()
          .tauri
          .system_tray
          .as_ref()
          .map_or(false, |t| t.menu_on_left_click);
      }
      if self.title.is_none() {
        self.title = manager
          .config()
          .tauri
          .system_tray
          .as_ref()
          .and_then(|t| t.title.clone())
      }
    }

    let tray_id = self.id.clone();

    let mut runtime_tray = tauri_runtime::SystemTray::new();
    runtime_tray = runtime_tray.with_id(hash(&self.id));
    if let Some(i) = self.icon {
      runtime_tray = runtime_tray.with_icon(i);
    }

    if let Some(menu) = self.menu {
      runtime_tray = runtime_tray.with_menu(menu);
    }

    if let Some(on_event) = self.on_event {
      let ids_ = ids.clone();
      let tray_id_ = tray_id.clone();
      runtime_tray = runtime_tray.on_event(move |event| {
        on_event(SystemTrayEvent::from_runtime_event(
          event,
          tray_id_.clone(),
          &ids_,
        ))
      });
    }

    #[cfg(target_os = "macos")]
    {
      runtime_tray = runtime_tray.with_icon_as_template(self.icon_as_template);
      runtime_tray = runtime_tray.with_menu_on_left_click(self.menu_on_left_click);
      if let Some(title) = self.title {
        runtime_tray = runtime_tray.with_title(&title);
      }
    }

    let id = runtime_tray.id;
    let tray_handler = match manager.runtime() {
      RuntimeOrDispatch::Runtime(r) => r.system_tray(runtime_tray),
      RuntimeOrDispatch::RuntimeHandle(h) => h.system_tray(runtime_tray),
      RuntimeOrDispatch::Dispatch(_) => manager
        .app_handle()
        .runtime_handle
        .system_tray(runtime_tray),
    }?;

    let tray_handle = SystemTrayHandle {
      id,
      ids,
      inner: tray_handler,
    };
    manager.manager().attach_tray(tray_id, tray_handle.clone());

    Ok(tray_handle)
  }
}

fn hash(id: &str) -> MenuHash {
  let mut hasher = DefaultHasher::new();
  id.hash(&mut hasher);
  hasher.finish() as MenuHash
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

  /// Adds the title to the tray menu
  #[cfg(target_os = "macos")]
  pub fn set_title(&self, title: &str) -> crate::Result<()> {
    self.inner.set_title(title).map_err(Into::into)
  }

  /// Destroys this system tray.
  pub fn destroy(&self) -> crate::Result<()> {
    self.inner.destroy().map_err(Into::into)
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
