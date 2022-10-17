// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Items specific to the [`Runtime`](crate::Runtime)'s webview.

use crate::{menu::Menu, window::DetachedWindow, Icon};

#[cfg(target_os = "macos")]
use tauri_utils::TitleBarStyle;
use tauri_utils::{
  config::{WindowConfig, WindowUrl},
  Theme,
};

#[cfg(windows)]
use windows::Win32::Foundation::HWND;

use std::{fmt, path::PathBuf};

/// The attributes used to create an webview.
#[derive(Debug, Clone)]
pub struct WebviewAttributes {
  pub url: WindowUrl,
  pub user_agent: Option<String>,
  pub initialization_scripts: Vec<String>,
  pub data_directory: Option<PathBuf>,
  pub file_drop_handler_enabled: bool,
  pub clipboard: bool,
  pub accept_first_mouse: bool,
}

impl WebviewAttributes {
  /// Initializes the default attributes for a webview.
  pub fn new(url: WindowUrl) -> Self {
    Self {
      url,
      user_agent: None,
      initialization_scripts: Vec::new(),
      data_directory: None,
      file_drop_handler_enabled: true,
      clipboard: false,
      accept_first_mouse: false,
    }
  }

  /// Sets the user agent
  #[must_use]
  pub fn user_agent(mut self, user_agent: &str) -> Self {
    self.user_agent = Some(user_agent.to_string());
    self
  }

  /// Sets the init script.
  #[must_use]
  pub fn initialization_script(mut self, script: &str) -> Self {
    self.initialization_scripts.push(script.to_string());
    self
  }

  /// Data directory for the webview.
  #[must_use]
  pub fn data_directory(mut self, data_directory: PathBuf) -> Self {
    self.data_directory.replace(data_directory);
    self
  }

  /// Disables the file drop handler. This is required to use drag and drop APIs on the front end on Windows.
  #[must_use]
  pub fn disable_file_drop_handler(mut self) -> Self {
    self.file_drop_handler_enabled = false;
    self
  }

  /// Enables clipboard access for the page rendered on **Linux** and **Windows**.
  ///
  /// **macOS** doesn't provide such method and is always enabled by default,
  /// but you still need to add menu item accelerators to use shortcuts.
  #[must_use]
  pub fn enable_clipboard_access(mut self) -> Self {
    self.clipboard = true;
    self
  }

  /// Sets whether clicking an inactive window also clicks through to the webview.
  #[must_use]
  pub fn accept_first_mouse(mut self, accept: bool) -> Self {
    self.accept_first_mouse = accept;
    self
  }
}

/// Do **NOT** implement this trait except for use in a custom [`Runtime`](crate::Runtime).
///
/// This trait is separate from [`WindowBuilder`] to prevent "accidental" implementation.
pub trait WindowBuilderBase: fmt::Debug + Clone + Sized {}

/// A builder for all attributes related to a single webview.
///
/// This trait is only meant to be implemented by a custom [`Runtime`](crate::Runtime)
/// and not by applications.
pub trait WindowBuilder: WindowBuilderBase {
  /// Initializes a new window attributes builder.
  fn new() -> Self;

  /// Initializes a new webview builder from a [`WindowConfig`]
  fn with_config(config: WindowConfig) -> Self;

  /// Sets the menu for the window.
  #[must_use]
  fn menu(self, menu: Menu) -> Self;

  /// Show window in the center of the screen.
  #[must_use]
  fn center(self) -> Self;

  /// The initial position of the window's.
  #[must_use]
  fn position(self, x: f64, y: f64) -> Self;

  /// Window size.
  #[must_use]
  fn inner_size(self, width: f64, height: f64) -> Self;

  /// Window min inner size.
  #[must_use]
  fn min_inner_size(self, min_width: f64, min_height: f64) -> Self;

  /// Window max inner size.
  #[must_use]
  fn max_inner_size(self, max_width: f64, max_height: f64) -> Self;

  /// Whether the window is resizable or not.
  #[must_use]
  fn resizable(self, resizable: bool) -> Self;

  /// The title of the window in the title bar.
  #[must_use]
  fn title<S: Into<String>>(self, title: S) -> Self;

  /// Whether to start the window in fullscreen or not.
  #[must_use]
  fn fullscreen(self, fullscreen: bool) -> Self;

  /// Whether the window will be initially focused or not.
  #[must_use]
  fn focused(self, focused: bool) -> Self;

  /// Whether the window should be maximized upon creation.
  #[must_use]
  fn maximized(self, maximized: bool) -> Self;

  /// Whether the window should be immediately visible upon creation.
  #[must_use]
  fn visible(self, visible: bool) -> Self;

  /// Whether the window should be transparent. If this is true, writing colors
  /// with alpha values different than `1.0` will produce a transparent window.
  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  #[cfg_attr(
    doc_cfg,
    doc(cfg(any(not(target_os = "macos"), feature = "macos-private-api")))
  )]
  #[must_use]
  fn transparent(self, transparent: bool) -> Self;

  /// Whether the window should have borders and bars.
  #[must_use]
  fn decorations(self, decorations: bool) -> Self;

  /// Whether the window should always be on top of other windows.
  #[must_use]
  fn always_on_top(self, always_on_top: bool) -> Self;

  /// Sets the window icon.
  fn icon(self, icon: Icon) -> crate::Result<Self>;

  /// Sets whether or not the window icon should be added to the taskbar.
  #[must_use]
  fn skip_taskbar(self, skip: bool) -> Self;

  /// Sets a parent to the window to be created.
  ///
  /// A child window has the WS_CHILD style and is confined to the client area of its parent window.
  ///
  /// For more information, see <https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#child-windows>
  #[cfg(windows)]
  #[must_use]
  fn parent_window(self, parent: HWND) -> Self;

  /// Sets a parent to the window to be created.
  ///
  /// A child window has the WS_CHILD style and is confined to the client area of its parent window.
  ///
  /// For more information, see <https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#child-windows>
  #[cfg(target_os = "macos")]
  #[must_use]
  fn parent_window(self, parent: *mut std::ffi::c_void) -> Self;

  /// Set an owner to the window to be created.
  ///
  /// From MSDN:
  /// - An owned window is always above its owner in the z-order.
  /// - The system automatically destroys an owned window when its owner is destroyed.
  /// - An owned window is hidden when its owner is minimized.
  ///
  /// For more information, see <https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#owned-windows>
  #[cfg(windows)]
  #[must_use]
  fn owner_window(self, owner: HWND) -> Self;

  /// Hide the titlebar. Titlebar buttons will still be visible.
  #[cfg(target_os = "macos")]
  #[must_use]
  fn title_bar_style(self, style: TitleBarStyle) -> Self;

  /// Hide the window title.
  #[cfg(target_os = "macos")]
  #[must_use]
  fn hidden_title(self, hidden: bool) -> Self;

  /// Forces a theme or uses the system settings if None was provided.
  fn theme(self, theme: Option<Theme>) -> Self;

  /// Whether the icon was set or not.
  fn has_icon(&self) -> bool;

  /// Gets the window menu.
  fn get_menu(&self) -> Option<&Menu>;
}

/// IPC handler.
pub type WebviewIpcHandler<T, R> = Box<dyn Fn(DetachedWindow<T, R>, String) + Send>;
