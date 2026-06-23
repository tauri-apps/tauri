// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_runtime::{
  Icon, Result,
  dpi::{Position, Size},
  window::{WindowBuilder, WindowBuilderBase, WindowSizeConstraints},
};
use tauri_utils::{
  Theme, TitleBarStyle,
  config::{Color, WindowConfig},
};
use winit::{
  dpi::{LogicalPosition, LogicalSize},
  monitor::Fullscreen,
  window::{WindowAttributes, WindowButtons},
};

#[cfg(windows)]
use windows::Win32::Foundation::HWND;

#[derive(Clone, Default, Debug)]
pub struct WindowBuilderWrapper {
  pub(crate) inner: WindowAttributes,
  pub(crate) center: bool,
}

unsafe impl Send for WindowBuilderWrapper {}

impl WindowBuilderBase for WindowBuilderWrapper {}

impl WindowBuilder for WindowBuilderWrapper {
  fn new() -> Self {
    Self {
      inner: WindowAttributes::default()
        .with_title("Tauri App")
        .with_visible(true),
      center: false,
    }
  }

  fn with_config(config: &WindowConfig) -> Self {
    let mut builder = Self::new()
      .title(config.title.to_string())
      .inner_size(config.width, config.height)
      .resizable(config.resizable)
      .fullscreen(config.fullscreen)
      .focused(config.focus)
      .focusable(config.focusable)
      .visible(config.visible)
      .decorations(config.decorations)
      .maximized(config.maximized)
      .always_on_top(config.always_on_top)
      .closable(config.closable)
      .maximizable(config.maximizable)
      .minimizable(config.minimizable)
      .theme(config.theme);
    if let (Some(x), Some(y)) = (config.x, config.y) {
      builder = builder.position(x, y);
    }
    if config.center {
      builder = builder.center();
    }
    builder
  }

  fn center(mut self) -> Self {
    self.center = true;
    self
  }

  fn position(mut self, x: f64, y: f64) -> Self {
    self.inner = self.inner.with_position(LogicalPosition::new(x, y));
    self
  }

  fn inner_size(mut self, width: f64, height: f64) -> Self {
    self.inner = self
      .inner
      .with_surface_size(LogicalSize::new(width, height));
    self
  }

  fn min_inner_size(mut self, min_width: f64, min_height: f64) -> Self {
    self.inner = self
      .inner
      .with_min_surface_size(LogicalSize::new(min_width, min_height));
    self
  }

  fn max_inner_size(mut self, max_width: f64, max_height: f64) -> Self {
    self.inner = self
      .inner
      .with_max_surface_size(LogicalSize::new(max_width, max_height));
    self
  }

  fn inner_size_constraints(self, _constraints: WindowSizeConstraints) -> Self {
    // TODO
    self
  }

  fn prevent_overflow(self) -> Self {
    // TODO
    self
  }

  fn prevent_overflow_with_margin(self, _margin: Size) -> Self {
    // TODO
    self
  }

  fn resizable(mut self, resizable: bool) -> Self {
    self.inner = self.inner.with_resizable(resizable);
    self
  }

  fn maximizable(mut self, maximizable: bool) -> Self {
    self
      .inner
      .enabled_buttons
      .set(WindowButtons::MAXIMIZE, maximizable);
    self
  }

  fn minimizable(mut self, minimizable: bool) -> Self {
    self
      .inner
      .enabled_buttons
      .set(WindowButtons::MINIMIZE, minimizable);
    self
  }

  fn closable(mut self, closable: bool) -> Self {
    self
      .inner
      .enabled_buttons
      .set(WindowButtons::CLOSE, closable);
    self
  }

  fn title<S: Into<String>>(mut self, title: S) -> Self {
    self.inner = self.inner.with_title(title);
    self
  }

  fn fullscreen(mut self, fullscreen: bool) -> Self {
    self.inner = self
      .inner
      .with_fullscreen(fullscreen.then_some(Fullscreen::Borderless(None)));
    self
  }

  fn focused(mut self, focused: bool) -> Self {
    self.inner = self.inner.with_active(focused);
    self
  }

  fn focusable(self, _focusable: bool) -> Self {
    // TODO
    self
  }

  fn maximized(mut self, maximized: bool) -> Self {
    self.inner = self.inner.with_maximized(maximized);
    self
  }

  fn visible(mut self, visible: bool) -> Self {
    self.inner = self.inner.with_visible(visible);
    self
  }

  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  fn transparent(mut self, transparent: bool) -> Self {
    self.inner = self.inner.with_transparent(transparent);
    self
  }

  fn decorations(mut self, decorations: bool) -> Self {
    self.inner = self.inner.with_decorations(decorations);
    self
  }

  fn always_on_bottom(mut self, always_on_bottom: bool) -> Self {
    self.inner = self.inner.with_window_level(if always_on_bottom {
      winit::window::WindowLevel::AlwaysOnBottom
    } else {
      winit::window::WindowLevel::Normal
    });
    self
  }

  fn always_on_top(mut self, always_on_top: bool) -> Self {
    self.inner = self.inner.with_window_level(if always_on_top {
      winit::window::WindowLevel::AlwaysOnTop
    } else {
      winit::window::WindowLevel::Normal
    });
    self
  }

  fn visible_on_all_workspaces(self, _visible_on_all_workspaces: bool) -> Self {
    // TODO
    self
  }

  fn content_protected(mut self, protected: bool) -> Self {
    self.inner = self.inner.with_content_protected(protected);
    self
  }

  fn icon(mut self, icon: Icon) -> Result<Self> {
    let icon = super::window::tauri_icon_to_winit_icon(icon)?;
    self.inner = self.inner.with_window_icon(Some(icon));
    Ok(self)
  }

  fn skip_taskbar(self, _skip: bool) -> Self {
    #[cfg(windows)]
    {
      let pl_attrs = platfomr_atts(&mut self.inner).with_skip_taskbar(_skip);
      self.inner = self.inner.with_platform_attributes(Box::new(pl_attrs));
      return self;
    }

    self
  }

  fn background_color(self, _color: Color) -> Self {
    // TODO
    self
  }

  fn shadow(mut self, enable: bool) -> Self {
    #[cfg(windows)]
    {
      let pl_attrs = platfomr_atts(&mut self.inner).with_undecorated_shadow(enable);
      self.inner = self.inner.with_platform_attributes(Box::new(pl_attrs));
    }

    #[cfg(target_os = "macos")]
    {
      let pl_attrs = platfomr_atts(&mut self.inner).with_has_shadow(enable);
      self.inner = self.inner.with_platform_attributes(Box::new(pl_attrs));
    }

    self
  }

  #[cfg(windows)]
  fn owner(self, _owner: HWND) -> Self {
    // TODO
    self
  }

  #[cfg(windows)]
  fn parent(self, _parent: HWND) -> Self {
    // TODO
    self
  }

  #[cfg(windows)]
  fn drag_and_drop(mut self, enabled: bool) -> Self {
    let pl_attrs = platfomr_atts(&mut self.inner).with_drag_and_drop(enabled);
    self.inner = self.inner.with_platform_attributes(Box::new(pl_attrs));
    self
  }

  #[cfg(target_os = "macos")]
  fn parent(self, _parent: *mut std::ffi::c_void) -> Self {
    // TODO
    self
  }

  #[cfg(target_os = "macos")]
  fn title_bar_style(mut self, style: TitleBarStyle) -> Self {
    let pl_attrs = *platfomr_atts(&mut self.inner);
    let pl_attrs = match style {
      TitleBarStyle::Visible => pl_attrs
        .with_titlebar_transparent(false)
        .with_fullsize_content_view(true),
      TitleBarStyle::Transparent => pl_attrs
        .with_titlebar_transparent(true)
        .with_fullsize_content_view(false),
      TitleBarStyle::Overlay => pl_attrs
        .with_titlebar_transparent(true)
        .with_fullsize_content_view(true),
      _ => pl_attrs,
    };
    self.inner = self.inner.with_platform_attributes(Box::new(pl_attrs));
    self
  }

  #[cfg(target_os = "macos")]
  fn traffic_light_position<P: Into<Position>>(self, _position: P) -> Self {
    // TODO
    self
  }

  #[cfg(target_os = "macos")]
  fn hidden_title(mut self, hidden: bool) -> Self {
    let pl_attrs = platfomr_atts(&mut self.inner).with_title_hidden(hidden);
    self.inner = self.inner.with_platform_attributes(Box::new(pl_attrs));
    self
  }

  #[cfg(target_os = "macos")]
  fn tabbing_identifier(mut self, identifier: &str) -> Self {
    let pl_attrs = platfomr_atts(&mut self.inner).with_tabbing_identifier(identifier);
    self.inner = self.inner.with_platform_attributes(Box::new(pl_attrs));
    self
  }

  fn theme(mut self, theme: Option<Theme>) -> Self {
    self.inner = self.inner.with_theme(theme.map(|theme| match theme {
      Theme::Light => winit::window::Theme::Light,
      Theme::Dark => winit::window::Theme::Dark,
      _ => winit::window::Theme::Light,
    }));
    self
  }

  fn window_classname<S: Into<String>>(self, _window_classname: S) -> Self {
    #[cfg(windows)]
    {
      let pl_attrs = platfomr_atts(&mut self.inner).with_class_name(_window_classname.into());
      self.inner = self.inner.with_platform_attributes(Box::new(pl_attrs));
      return self;
    }

    self
  }

  fn has_icon(&self) -> bool {
    self.inner.window_icon.is_some()
  }

  fn get_theme(&self) -> Option<Theme> {
    self.inner.preferred_theme.map(|theme| match theme {
      winit::window::Theme::Light => Theme::Light,
      winit::window::Theme::Dark => Theme::Dark,
    })
  }
}

#[cfg(windows)]
type PlatformAttributes = winit::platform::windows::WindowAttributesWindows;
#[cfg(target_os = "macos")]
type PlatformAttributes = winit::platform::macos::WindowAttributesMacOS;
#[cfg(any(
  target_os = "linux",
  target_os = "freebsd",
  target_os = "dragonfly",
  target_os = "openbsd",
  target_os = "netbsd"
))]
type PlatformAttributes = winit::platform::unix::WindowAttributesUnix;

fn platfomr_atts(attrs: &mut WindowAttributes) -> Box<PlatformAttributes> {
  attrs
    .platform
    .take()
    .and_then(|attrs| attrs.cast::<PlatformAttributes>().ok())
    .unwrap_or_default()
}
