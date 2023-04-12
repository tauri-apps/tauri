#![cfg(all(desktop, feature = "system-tray"))]

use std::fmt;

use tauri_utils::AtomicCounter;

use crate::dpi::{PhysicalPosition, Rect};
use crate::menu::{CustomMenuItem, MenuUpdate};
use crate::Icon;

static SYSTEM_TRAY_ID: AtomicCounter = AtomicCounter::new();

/// A system tray event.
#[derive(Debug)]
pub enum SystemTrayEvent {
  MenuItemClick(u16),
  LeftClick {
    position: PhysicalPosition<f64>,
    bounds: Rect,
  },
  RightClick {
    position: PhysicalPosition<f64>,
    bounds: Rect,
  },
  DoubleClick {
    position: PhysicalPosition<f64>,
    bounds: Rect,
  },
}

pub type SystemTrayId = u16;
pub type SystemTrayEventListener = dyn Fn(&SystemTrayEvent) + Send + 'static;

#[cfg(all(desktop, feature = "system-tray"))]
#[non_exhaustive]
pub struct SystemTray {
  pub id: SystemTrayId,
  pub icon: Option<Icon>,
  pub menu: Option<SystemTrayMenu>,
  #[cfg(target_os = "macos")]
  pub icon_as_template: bool,
  #[cfg(target_os = "macos")]
  pub menu_on_left_click: bool,
  #[cfg(target_os = "macos")]
  pub title: Option<String>,
  pub on_event: Option<Box<SystemTrayEventListener>>,
  pub tooltip: Option<String>,
}

#[cfg(all(desktop, feature = "system-tray"))]
impl fmt::Debug for SystemTray {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut d = f.debug_struct("SystemTray");
    d.field("id", &self.id)
      .field("icon", &self.icon)
      .field("menu", &self.menu);
    #[cfg(target_os = "macos")]
    {
      d.field("icon_as_template", &self.icon_as_template)
        .field("menu_on_left_click", &self.menu_on_left_click)
        .field("title", &self.title);
    }
    d.finish()
  }
}

#[cfg(all(desktop, feature = "system-tray"))]
impl Clone for SystemTray {
  fn clone(&self) -> Self {
    Self {
      id: self.id,
      icon: self.icon.clone(),
      menu: self.menu.clone(),
      on_event: None,
      #[cfg(target_os = "macos")]
      icon_as_template: self.icon_as_template,
      #[cfg(target_os = "macos")]
      menu_on_left_click: self.menu_on_left_click,
      #[cfg(target_os = "macos")]
      title: self.title.clone(),
      tooltip: self.tooltip.clone(),
    }
  }
}

#[cfg(all(desktop, feature = "system-tray"))]
impl Default for SystemTray {
  fn default() -> Self {
    Self {
      id: SYSTEM_TRAY_ID.next() as _,
      icon: None,
      menu: None,
      #[cfg(target_os = "macos")]
      icon_as_template: false,
      #[cfg(target_os = "macos")]
      menu_on_left_click: false,
      #[cfg(target_os = "macos")]
      title: None,
      on_event: None,
      tooltip: None,
    }
  }
}

#[cfg(all(desktop, feature = "system-tray"))]
impl SystemTray {
  /// Creates a new system tray that only renders an icon.
  pub fn new() -> Self {
    Default::default()
  }

  pub fn menu(&self) -> Option<&SystemTrayMenu> {
    self.menu.as_ref()
  }

  /// Sets the tray id.
  #[must_use]
  pub fn with_id(mut self, id: SystemTrayId) -> Self {
    self.id = id;
    self
  }

  /// Sets the tray icon.
  #[must_use]
  pub fn with_icon(mut self, icon: Icon) -> Self {
    self.icon.replace(icon);
    self
  }

  /// Sets the tray icon as template.
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

  #[cfg(target_os = "macos")]
  #[must_use]
  pub fn with_title(mut self, title: &str) -> Self {
    self.title = Some(title.to_owned());
    self
  }

  /// Sets the tray icon tooltip.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported
  #[must_use]
  pub fn with_tooltip(mut self, tooltip: &str) -> Self {
    self.tooltip = Some(tooltip.to_owned());
    self
  }

  /// Sets the menu to show when the system tray is right clicked.
  #[must_use]
  pub fn with_menu(mut self, menu: SystemTrayMenu) -> Self {
    self.menu.replace(menu);
    self
  }

  #[must_use]
  pub fn on_event<F: Fn(&SystemTrayEvent) + Send + 'static>(mut self, f: F) -> Self {
    self.on_event.replace(Box::new(f));
    self
  }
}

pub trait SystemTrayHandle: fmt::Debug + Clone + Send + Sync {
  fn set_icon(&self, icon: crate::Icon) -> crate::Result<()>;
  fn set_menu(&self, menu: SystemTrayMenu) -> crate::Result<()>;
  fn update_item(&self, id: u16, update: MenuUpdate) -> crate::Result<()>;
  #[cfg(target_os = "macos")]
  fn set_icon_as_template(&self, is_template: bool) -> crate::Result<()>;
  #[cfg(target_os = "macos")]
  fn set_title(&self, title: &str) -> crate::Result<()>;
  fn set_tooltip(&self, tooltip: &str) -> crate::Result<()>;
  fn destroy(&self) -> crate::Result<()>;
}

/// A system tray menu.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct SystemTrayMenu {
  pub items: Vec<SystemTrayMenuEntry>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SystemTraySubmenu {
  pub title: String,
  pub enabled: bool,
  pub inner: SystemTrayMenu,
}

impl SystemTraySubmenu {
  /// Creates a new submenu with the given title and menu items.
  pub fn new<S: Into<String>>(title: S, menu: SystemTrayMenu) -> Self {
    Self {
      title: title.into(),
      enabled: true,
      inner: menu,
    }
  }
}

impl SystemTrayMenu {
  /// Creates a new system tray menu.
  pub fn new() -> Self {
    Default::default()
  }

  /// Adds the custom menu item to the system tray menu.
  #[must_use]
  pub fn add_item(mut self, item: CustomMenuItem) -> Self {
    self.items.push(SystemTrayMenuEntry::CustomItem(item));
    self
  }

  /// Adds a native item to the system tray menu.
  #[must_use]
  pub fn add_native_item(mut self, item: SystemTrayMenuItem) -> Self {
    self.items.push(SystemTrayMenuEntry::NativeItem(item));
    self
  }

  /// Adds an entry with submenu.
  #[must_use]
  pub fn add_submenu(mut self, submenu: SystemTraySubmenu) -> Self {
    self.items.push(SystemTrayMenuEntry::Submenu(submenu));
    self
  }
}

/// An entry on the system tray menu.
#[derive(Debug, Clone)]
pub enum SystemTrayMenuEntry {
  /// A custom item.
  CustomItem(CustomMenuItem),
  /// A native item.
  NativeItem(SystemTrayMenuItem),
  /// An entry with submenu.
  Submenu(SystemTraySubmenu),
}

/// System tray menu item.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SystemTrayMenuItem {
  /// A separator.
  Separator,
}
