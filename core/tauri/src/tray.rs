// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Tray icon types and utility functions

use std::path::{Path, PathBuf};

use crate::runtime::tray as tray_icon;
pub use crate::runtime::tray::TrayIconEvent;
use crate::{run_main_thread, AppHandle, Icon, Runtime};

// TODO(muda-migration): tray icon type `on_event` handler
// TODO(muda-migration): figure out js events

/// Attributes to use when creating a tray icon.
#[derive(Default)]
pub struct TrayIconAttributes {
  /// Tray icon tooltip
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub tooltip: Option<String>,

  /// Tray menu
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux**: once a menu is set, it cannot be removed.
  pub menu: Option<Box<dyn crate::menu::ContextMenu>>,

  /// Tray icon
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Sometimes the icon won't be visible unless a menu is set.
  ///     Setting an empty [`Menu`](crate::menu::Menu) is enough.
  pub icon: Option<Icon>,

  /// Tray icon temp dir path. **Linux only**.
  pub temp_dir_path: Option<PathBuf>,

  /// Use the icon as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc). **macOS only**.
  pub icon_is_template: bool,

  /// Whether to show the tray menu on left click or not, default is `true`. **macOS only**.
  pub menu_on_left_click: bool,

  /// Tray icon title.
  ///
  /// ## Platform-specific
  ///
  /// - **Linux:** The title will not be shown unless there is an icon
  /// as well.  The title is useful for numerical and other frequently
  /// updated information.  In general, it shouldn't be shown unless a
  /// user requests it as it can take up a significant amount of space
  /// on the user's panel.  This may not be shown in all visualizations.
  /// - **Windows:** Unsupported.
  pub title: Option<String>,
}

impl From<TrayIconAttributes> for tray_icon::TrayIconAttributes {
  fn from(value: TrayIconAttributes) -> Self {
    Self {
      tooltip: value.tooltip,
      menu: value.menu.map(|m| m.into_inner()),
      icon: value.icon.and_then(|i| {
        i.try_into()
          .ok()
          .and_then(|i: crate::runtime::Icon| i.try_into().ok())
      }),
      temp_dir_path: value.temp_dir_path,
      icon_is_template: value.icon_is_template,
      menu_on_left_click: value.menu_on_left_click,
      title: value.title,
    }
  }
}

/// [`TrayIcon`] builder struct and associated methods.
#[derive(Default)]
pub struct TrayIconBuilder(tray_icon::TrayIconBuilder);

impl TrayIconBuilder {
  /// Creates a new [`TrayIconBuilder`] with default [`TrayIconAttributes`].
  ///
  /// See [`TrayIcon::new`] for more info.
  pub fn new() -> Self {
    Self(tray_icon::TrayIconBuilder::new())
  }

  /// Sets the unique id to build the tray icon with.
  pub fn with_id(mut self, id: u32) -> Self {
    self.0 = self.0.with_id(id);
    self
  }

  /// Set the a menu for this tray icon.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux**: once a menu is set, it cannot be removed or replaced but you can change its content.
  pub fn with_menu(mut self, menu: &dyn crate::menu::ContextMenu) -> Self {
    self.0 = self.0.with_menu(menu.into_inner());
    self
  }

  /// Set an icon for this tray icon.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Sometimes the icon won't be visible unless a menu is set.
  /// Setting an empty [`Menu`](crate::menu::Menu) is enough.
  pub fn with_icon(mut self, icon: Icon) -> Self {
    let icon = icon
      .try_into()
      .ok()
      .and_then(|i: crate::runtime::Icon| i.try_into().ok());
    if let Some(icon) = icon {
      self.0 = self.0.with_icon(icon);
    }
    self
  }

  /// Set a tooltip for this tray icon.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Unsupported.
  pub fn with_tooltip<S: AsRef<str>>(mut self, s: S) -> Self {
    self.0 = self.0.with_tooltip(s);
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
  pub fn with_title<S: AsRef<str>>(mut self, title: S) -> Self {
    self.0 = self.0.with_title(title);
    self
  }

  /// Set tray icon temp dir path. **Linux only**.
  ///
  /// On Linux, we need to write the icon to the disk and usually it will
  /// be `$XDG_RUNTIME_DIR/tray-icon` or `$TEMP/tray-icon`.
  pub fn with_temp_dir_path<P: AsRef<Path>>(mut self, s: P) -> Self {
    self.0 = self.0.with_temp_dir_path(s);
    self
  }

  /// Use the icon as a [template](https://developer.apple.com/documentation/appkit/nsimage/1520017-template?language=objc). **macOS only**.
  pub fn with_icon_as_template(mut self, is_template: bool) -> Self {
    self.0 = self.0.with_icon_as_template(is_template);
    self
  }

  /// Whether to show the tray menu on left click or not, default is `true`. **macOS only**.
  pub fn with_menu_on_left_click(mut self, enable: bool) -> Self {
    self.0 = self.0.with_menu_on_left_click(enable);
    self
  }

  /// Access the unique id that will be assigned to the tray icon
  /// this builder will create.
  pub fn id(&self) -> u32 {
    self.0.id()
  }

  /// Builds and adds a new [`TrayIcon`] to the system tray.
  pub fn build<R: Runtime>(self, app_handle: &AppHandle<R>) -> crate::Result<TrayIcon<R>> {
    Ok(TrayIcon {
      inner: self.0.build()?,
      app_handle: app_handle.clone(),
    })
  }
}

/// Tray icon struct and associated methods.
///
/// This type is reference-counted and the icon is removed when the last instance is dropped.
pub struct TrayIcon<R: Runtime> {
  inner: tray_icon::TrayIcon,
  app_handle: AppHandle<R>,
}

impl<R: Runtime> Clone for TrayIcon<R> {
  fn clone(&self) -> Self {
    Self {
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
  /// Builds and adds a new tray icon to the system tray.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux:** Sometimes the icon won't be visible unless a menu is set.
  /// Setting an empty [`Menu`](crate::menu::Menu) is enough.
  pub fn new(app_handle: &AppHandle<R>, attrs: TrayIconAttributes) -> crate::Result<Self> {
    Ok(Self {
      inner: tray_icon::TrayIcon::new(attrs.into())?,
      app_handle: app_handle.clone(),
    })
  }

  /// Builds and adds a new tray icon to the system tray with the specified Id.
  ///
  /// See [`TrayIcon::new`] for more info.
  pub fn with_id(
    app_handle: &AppHandle<R>,
    attrs: TrayIconAttributes,
    id: u32,
  ) -> crate::Result<Self> {
    Ok(Self {
      inner: tray_icon::TrayIcon::with_id(attrs.into(), id)?,
      app_handle: app_handle.clone(),
    })
  }

  /// Returns the id associated with this tray icon.
  pub fn id(&self) -> crate::Result<u32> {
    run_main_thread!(self, |self_: Self| self_.inner.id())
  }

  /// Set new tray icon. If `None` is provided, it will remove the icon.
  pub fn set_icon(&self, icon: Option<Icon>) -> crate::Result<()> {
    let icon = icon.and_then(|i| {
      i.try_into()
        .ok()
        .and_then(|i: crate::runtime::Icon| i.try_into().ok())
    });
    run_main_thread!(self, |self_: Self| self_.inner.set_icon(icon))?.map_err(Into::into)
  }

  /// Set new tray menu.
  ///
  /// ## Platform-specific:
  ///
  /// - **Linux**: once a menu is set it cannot be removed so `None` has no effect
  pub fn set_menu(&self, menu: Option<Box<dyn crate::menu::ContextMenu>>) -> crate::Result<()> {
    run_main_thread!(self, |self_: Self| self_
      .inner
      .set_menu(menu.map(|m| m.into_inner())))
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
