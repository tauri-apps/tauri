// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(all(desktop, feature = "tray-icon"))]

//! Tray icon types and utility functions

use crate::app::{GlobalMenuEventListener, GlobalTrayIconEventListener};
use crate::menu::ContextMenu;
use crate::menu::MenuEvent;
use crate::{run_main_thread, AppHandle, Icon, Manager, Runtime};
use std::path::Path;
pub use tray_icon::TrayIconId;

// TODO(muda-migration): figure out js events

/// Describes a rectangle including position (x - y axis) and size.
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Rectangle {
  /// The x-coordinate of the upper-left corner of the rectangle.
  pub left: f64,
  /// The y-coordinate of the upper-left corner of the rectangle.
  pub top: f64,
  /// The x-coordinate of the lower-right corner of the rectangle.
  pub right: f64,
  /// The y-coordinate of the lower-right corner of the rectangle.
  pub bottom: f64,
}

/// Describes the click type that triggered this tray icon event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClickType {
  /// Left mouse click.
  Left,
  /// Right mouse click.
  Right,
  /// Double left mouse click.
  Double,
}

impl Default for ClickType {
  fn default() -> Self {
    Self::Left
  }
}

/// Describes a tray event emitted when a tray icon is clicked
///
/// ## Platform-specific:
///
/// - **Linux**: Unsupported. The event is not emmited even though the icon is shown,
/// the icon will still show a context menu on right click.
#[derive(Debug, Clone, Default)]
pub struct TrayIconEvent {
  /// Id of the tray icon which triggered this event.
  pub id: TrayIconId,
  /// Physical X Position of the click the triggered this event.
  pub x: f64,
  /// Physical Y Position of the click the triggered this event.
  pub y: f64,
  /// Position and size of the tray icon
  pub icon_rect: Rectangle,
  /// The click type that triggered this event.
  pub click_type: ClickType,
}

impl TrayIconEvent {
  /// Returns the id of the tray icon which triggered this event.
  pub fn id(&self) -> &TrayIconId {
    &self.id
  }
}

impl From<tray_icon::Rectangle> for Rectangle {
  fn from(value: tray_icon::Rectangle) -> Self {
    Self {
      bottom: value.bottom,
      left: value.left,
      top: value.top,
      right: value.right,
    }
  }
}

impl From<tray_icon::ClickType> for ClickType {
  fn from(value: tray_icon::ClickType) -> Self {
    match value {
      tray_icon::ClickType::Left => Self::Left,
      tray_icon::ClickType::Right => Self::Right,
      tray_icon::ClickType::Double => Self::Double,
    }
  }
}

impl From<tray_icon::TrayIconEvent> for TrayIconEvent {
  fn from(value: tray_icon::TrayIconEvent) -> Self {
    Self {
      id: value.id,
      x: value.x,
      y: value.y,
      icon_rect: value.icon_rect.into(),
      click_type: value.click_type.into(),
    }
  }
}

/// [`TrayIcon`] builder struct and associated methods.
#[derive(Default)]
pub struct TrayIconBuilder<R: Runtime> {
  on_menu_event: Option<GlobalMenuEventListener<AppHandle<R>>>,
  on_tray_event: Option<GlobalTrayIconEventListener<TrayIcon<R>>>,
  inner: tray_icon::TrayIconBuilder,
}

impl<R: Runtime> TrayIconBuilder<R> {
  /// Creates a new tray icon builder.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Sometimes the icon won't be visible unless a menu is set.
  /// Setting an empty [`Menu`](crate::menu::Menu) is enough.
  pub fn new() -> Self {
    Self {
      inner: tray_icon::TrayIconBuilder::new(),
      on_menu_event: None,
      on_tray_event: None,
    }
  }

  /// Creates a new tray icon builder with the specified id.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Sometimes the icon won't be visible unless a menu is set.
  /// Setting an empty [`Menu`](crate::menu::Menu) is enough.
  pub fn with_id<I: Into<TrayIconId>>(id: I) -> Self {
    let mut builder = Self::new();
    builder.inner = builder.inner.with_id(id);
    builder
  }

  /// Set the a menu for this tray icon.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux**: once a menu is set, it cannot be removed or replaced but you can change its content.
  pub fn menu<M: ContextMenu>(mut self, menu: &M) -> Self {
    self.inner = self.inner.with_menu(menu.inner_owned());
    self
  }

  /// Set an icon for this tray icon.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Sometimes the icon won't be visible unless a menu is set.
  /// Setting an empty [`Menu`](crate::menu::Menu) is enough.
  pub fn icon(mut self, icon: Icon) -> Self {
    let icon = icon.try_into().ok();
    if let Some(icon) = icon {
      self.inner = self.inner.with_icon(icon);
    }
    self
  }

  /// Set a tooltip for this tray icon.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn tooltip<S: AsRef<str>>(mut self, s: S) -> Self {
    self.inner = self.inner.with_tooltip(s);
    self
  }

  /// Set the tray icon title.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** The title will not be shown unless there is an icon
  /// as well.  The title is useful for numerical and other frequently
  /// updated information.  In general, it shouldn't be shown unless a
  /// user requests it as it can take up a significant amount of space
  /// on the user's panel.  This may not be shown in all visualizations.
  /// - **Windows:** Unsupported.
  pub fn title<S: AsRef<str>>(mut self, title: S) -> Self {
    self.inner = self.inner.with_title(title);
    self
  }

  /// Set tray icon temp dir path. **Linux only**.
  ///
  /// On Linux, we need to write the icon to the disk and usually it will
  /// be `$XDG_RUNTIME_DIR/tray-icon` or `$TEMP/tray-icon`.
  pub fn temp_dir_path<P: AsRef<Path>>(mut self, s: P) -> Self {
    self.inner = self.inner.with_temp_dir_path(s);
    self
  }

  /// Use the icon as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc). **macOS only**.
  pub fn icon_as_template(mut self, is_template: bool) -> Self {
    self.inner = self.inner.with_icon_as_template(is_template);
    self
  }

  /// Whether to show the tray menu on left click or not, default is `true`. **macOS only**.
  pub fn menu_on_left_click(mut self, enable: bool) -> Self {
    self.inner = self.inner.with_menu_on_left_click(enable);
    self
  }

  /// Set a handler for menu events.
  ///
  /// Note that this handler is called for any menu event,
  /// whether it is coming from this window, another window or from the tray icon menu.
  pub fn on_menu_event<F: Fn(&AppHandle<R>, MenuEvent) + Sync + Send + 'static>(
    mut self,
    f: F,
  ) -> Self {
    self.on_menu_event.replace(Box::new(f));
    self
  }

  /// Set a handler for this tray icon events.
  pub fn on_tray_event<F: Fn(&TrayIcon<R>, TrayIconEvent) + Sync + Send + 'static>(
    mut self,
    f: F,
  ) -> Self {
    self.on_tray_event.replace(Box::new(f));
    self
  }

  /// Access the unique id that will be assigned to the tray icon
  /// this builder will create.
  pub fn id(&self) -> &TrayIconId {
    self.inner.id()
  }

  /// Builds and adds a new [`TrayIcon`] to the system tray.
  pub fn build<M: Manager<R>>(self, manager: &M) -> crate::Result<TrayIcon<R>> {
    let id = self.id().clone();
    let inner = self.inner.build()?;
    let icon = TrayIcon {
      id,
      inner,
      app_handle: manager.app_handle().clone(),
    };

    icon.register(&icon.app_handle, self.on_menu_event, self.on_tray_event);

    Ok(icon)
  }
}

/// Tray icon struct and associated methods.
///
/// This type is reference-counted and the icon is removed when the last instance is dropped.
///
/// See [TrayIconBuilder] to construct this type.
pub struct TrayIcon<R: Runtime> {
  id: TrayIconId,
  inner: tray_icon::TrayIcon,
  app_handle: AppHandle<R>,
}

impl<R: Runtime> Clone for TrayIcon<R> {
  fn clone(&self) -> Self {
    Self {
      id: self.id.clone(),
      inner: self.inner.clone(),
      app_handle: self.app_handle.clone(),
    }
  }
}

/// # Safety
///
/// We make sure it always runs on the main thread.
unsafe impl<R: Runtime> Sync for TrayIcon<R> {}
unsafe impl<R: Runtime> Send for TrayIcon<R> {}

impl<R: Runtime> TrayIcon<R> {
  fn register(
    &self,
    app_handle: &AppHandle<R>,
    on_menu_event: Option<GlobalMenuEventListener<AppHandle<R>>>,
    on_tray_event: Option<GlobalTrayIconEventListener<TrayIcon<R>>>,
  ) {
    if let Some(handler) = on_menu_event {
      app_handle
        .manager
        .inner
        .menu_event_listeners
        .lock()
        .unwrap()
        .push(handler);
    }

    if let Some(handler) = on_tray_event {
      app_handle
        .manager
        .inner
        .tray_event_listeners
        .lock()
        .unwrap()
        .insert(self.id.clone(), handler);
    }

    app_handle
      .manager
      .inner
      .tray_icons
      .lock()
      .unwrap()
      .push(self.clone());
  }

  /// The application handle associated with this type.
  pub fn app_handle(&self) -> &AppHandle<R> {
    &self.app_handle
  }

  /// Register a handler for menu events.
  ///
  /// Note that this handler is called for any menu event,
  /// whether it is coming from this window, another window or from the tray icon menu.
  pub fn on_menu_event<F: Fn(&AppHandle<R>, MenuEvent) + Sync + Send + 'static>(&self, f: F) {
    self
      .app_handle
      .manager
      .inner
      .menu_event_listeners
      .lock()
      .unwrap()
      .push(Box::new(f));
  }

  /// Register a handler for this tray icon events.
  pub fn on_tray_event<F: Fn(&TrayIcon<R>, TrayIconEvent) + Sync + Send + 'static>(&self, f: F) {
    self
      .app_handle
      .manager
      .inner
      .tray_event_listeners
      .lock()
      .unwrap()
      .insert(self.id.clone(), Box::new(f));
  }

  /// Returns the id associated with this tray icon.
  pub fn id(&self) -> &TrayIconId {
    &self.id
  }

  /// Set new tray icon. If `None` is provided, it will remove the icon.
  pub fn set_icon(&self, icon: Option<Icon>) -> crate::Result<()> {
    let icon = icon.and_then(|i| i.try_into().ok());
    run_main_thread!(self, |self_: Self| self_.inner.set_icon(icon))?.map_err(Into::into)
  }

  /// Set new tray menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux**: once a menu is set it cannot be removed so `None` has no effect
  pub fn set_menu<M: ContextMenu + 'static>(&self, menu: Option<M>) -> crate::Result<()> {
    run_main_thread!(self, |self_: Self| self_
      .inner
      .set_menu(menu.map(|m| m.inner_owned())))
  }

  /// Sets the tooltip for this tray icon.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported
  pub fn set_tooltip<S: AsRef<str>>(&self, tooltip: Option<S>) -> crate::Result<()> {
    let s = tooltip.map(|s| s.as_ref().to_string());
    run_main_thread!(self, |self_: Self| self_.inner.set_tooltip(s))?.map_err(Into::into)
  }

  /// Sets the tooltip for this tray icon.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** The title will not be shown unless there is an icon
  /// as well.  The title is useful for numerical and other frequently
  /// updated information.  In general, it shouldn't be shown unless a
  /// user requests it as it can take up a significant amount of space
  /// on the user's panel.  This may not be shown in all visualizations.
  /// - **Windows:** Unsupported
  pub fn set_title<S: AsRef<str>>(&self, title: Option<S>) -> crate::Result<()> {
    let s = title.map(|s| s.as_ref().to_string());
    run_main_thread!(self, |self_: Self| self_.inner.set_title(s))
  }

  /// Show or hide this tray icon
  pub fn set_visible(&self, visible: bool) -> crate::Result<()> {
    run_main_thread!(self, |self_: Self| self_.inner.set_visible(visible))?.map_err(Into::into)
  }

  /// Sets the tray icon temp dir path. **Linux only**.
  ///
  /// On Linux, we need to write the icon to the disk and usually it will
  /// be `$XDG_RUNTIME_DIR/tray-icon` or `$TEMP/tray-icon`.
  pub fn set_temp_dir_path<P: AsRef<Path>>(&self, path: Option<P>) -> crate::Result<()> {
    #[allow(unused)]
    let p = path.map(|p| p.as_ref().to_path_buf());
    #[cfg(target_os = "linux")]
    run_main_thread!(self, |self_: Self| self_.inner.set_temp_dir_path(p))?;
    Ok(())
  }

  /// Set the current icon as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc). **macOS only**.
  pub fn set_icon_as_template(&self, #[allow(unused)] is_template: bool) -> crate::Result<()> {
    #[cfg(target_os = "macos")]
    run_main_thread!(self, |self_: Self| self_
      .inner
      .set_icon_as_template(is_template))?;
    Ok(())
  }

  /// Disable or enable showing the tray menu on left click. **macOS only**.
  pub fn set_show_menu_on_left_click(&self, #[allow(unused)] enable: bool) -> crate::Result<()> {
    #[cfg(target_os = "macos")]
    run_main_thread!(self, |self_: Self| self_
      .inner
      .set_show_menu_on_left_click(enable))?;
    Ok(())
  }
}

impl TryFrom<Icon> for tray_icon::Icon {
  type Error = crate::Error;

  fn try_from(value: Icon) -> Result<Self, Self::Error> {
    let value: crate::runtime::Icon = value.try_into()?;
    tray_icon::Icon::from_rgba(value.rgba, value.width, value.height).map_err(Into::into)
  }
}
