// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_runtime::{
  Icon, Result,
  dpi::{PhysicalSize, Size},
  window::{WindowBuilder, WindowBuilderBase, WindowSizeConstraints},
};
use tauri_utils::{
  Theme,
  config::{Color, PreventOverflowConfig, WindowConfig},
};
use winit::{
  dpi::{LogicalPosition, LogicalSize},
  monitor::Fullscreen,
  window::{WindowAttributes, WindowButtons},
};

use crate::window::{
  AppWindowAttrs, paired_size_constraint, tauri_theme_to_winit_theme, winit_theme_to_tauri_theme,
};

#[cfg(any(windows, target_os = "macos"))]
use winit::raw_window_handle::RawWindowHandle;

#[cfg(target_os = "macos")]
use std::ptr::NonNull;
#[cfg(target_os = "macos")]
use tauri_runtime::dpi::Position;
#[cfg(target_os = "macos")]
use tauri_utils::TitleBarStyle;

#[cfg(windows)]
use std::num::NonZeroIsize;
#[cfg(windows)]
use windows::Win32::Foundation::HWND;

#[derive(Clone, Default, Debug)]
pub struct WindowBuilderWrapper {
  pub(crate) attrs: AppWindowAttrs,
}

unsafe impl Send for WindowBuilderWrapper {}

impl WindowBuilderBase for WindowBuilderWrapper {}

impl WindowBuilder for WindowBuilderWrapper {
  fn new() -> Self {
    #[allow(unused_mut)]
    let mut builder = Self {
      attrs: AppWindowAttrs {
        inner: WindowAttributes::default()
          .with_title("Tauri App")
          .with_visible(true),
        ..Default::default()
      },
    }
    .focused(true);

    #[cfg(windows)]
    {
      builder = builder.window_classname("Tauri Window");
    }

    builder
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
      .content_protected(config.content_protected)
      .closable(config.closable)
      .maximizable(config.maximizable)
      .minimizable(config.minimizable)
      .skip_taskbar(config.skip_taskbar)
      .shadow(config.shadow)
      .visible_on_all_workspaces(config.visible_on_all_workspaces)
      .theme(config.theme);
    if config.always_on_bottom {
      builder = builder.always_on_bottom(true);
    } else if config.always_on_top {
      builder = builder.always_on_top(true);
    }
    #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
    {
      builder = builder.transparent(config.transparent);
    }
    if let (Some(min_width), Some(min_height)) = (config.min_width, config.min_height) {
      builder = builder.min_inner_size(min_width, min_height);
    }
    if let (Some(max_width), Some(max_height)) = (config.max_width, config.max_height) {
      builder = builder.max_inner_size(max_width, max_height);
    }
    if let Some(color) = config.background_color {
      builder = builder.background_color(color);
    }
    if let (Some(x), Some(y)) = (config.x, config.y) {
      builder = builder.position(x, y);
    }
    #[cfg(target_os = "macos")]
    {
      builder = builder
        .hidden_title(config.hidden_title)
        .title_bar_style(config.title_bar_style);
      if let Some(position) = &config.traffic_light_position {
        builder = builder.traffic_light_position(LogicalPosition::new(position.x, position.y));
      }
      if let Some(identifier) = &config.tabbing_identifier {
        builder = builder.tabbing_identifier(identifier);
      }
      let pl_attrs = (*platform_attrs(&mut builder.attrs.inner))
        .with_accepts_first_mouse(config.accept_first_mouse);

      builder.attrs.inner = builder
        .attrs
        .inner
        .with_platform_attributes(Box::new(pl_attrs));
    }
    if config.center {
      builder = builder.center();
    }
    if let Some(window_classname) = &config.window_classname {
      builder = builder.window_classname(window_classname);
    }
    if let Some(prevent_overflow) = &config.prevent_overflow {
      builder = match prevent_overflow {
        PreventOverflowConfig::Enable(true) => builder.prevent_overflow(),
        PreventOverflowConfig::Margin(margin) => {
          let margin = PhysicalSize::new(margin.width, margin.height);
          builder.prevent_overflow_with_margin(margin.into())
        }
        _ => builder,
      };
    }
    builder
  }

  fn center(mut self) -> Self {
    self.attrs.center = true;
    self
  }

  fn position(mut self, x: f64, y: f64) -> Self {
    self.attrs.inner = self.attrs.inner.with_position(LogicalPosition::new(x, y));
    self
  }

  fn inner_size(mut self, width: f64, height: f64) -> Self {
    self.attrs.inner = self
      .attrs
      .inner
      .with_surface_size(LogicalSize::new(width, height));
    self
  }

  fn min_inner_size(mut self, min_width: f64, min_height: f64) -> Self {
    self.attrs.inner = self
      .attrs
      .inner
      .with_min_surface_size(LogicalSize::new(min_width, min_height));
    self
  }

  fn max_inner_size(mut self, max_width: f64, max_height: f64) -> Self {
    self.attrs.inner = self
      .attrs
      .inner
      .with_max_surface_size(LogicalSize::new(max_width, max_height));
    self
  }

  fn inner_size_constraints(mut self, constraints: WindowSizeConstraints) -> Self {
    // TODO: upstream individual width/height size constraints to winit.
    self.attrs.inner.min_surface_size =
      paired_size_constraint(constraints.min_width, constraints.min_height);
    self.attrs.inner.max_surface_size =
      paired_size_constraint(constraints.max_width, constraints.max_height);
    self
  }

  fn prevent_overflow(mut self) -> Self {
    self.attrs.prevent_overflow = Some(PhysicalSize::new(0, 0).into());
    self
  }

  fn prevent_overflow_with_margin(mut self, margin: Size) -> Self {
    self.attrs.prevent_overflow = Some(margin);
    self
  }

  fn resizable(mut self, resizable: bool) -> Self {
    self.attrs.inner = self.attrs.inner.with_resizable(resizable);
    self
  }

  fn maximizable(mut self, maximizable: bool) -> Self {
    self
      .attrs
      .inner
      .enabled_buttons
      .set(WindowButtons::MAXIMIZE, maximizable);
    self
  }

  fn minimizable(mut self, minimizable: bool) -> Self {
    self
      .attrs
      .inner
      .enabled_buttons
      .set(WindowButtons::MINIMIZE, minimizable);
    self
  }

  fn closable(mut self, closable: bool) -> Self {
    self
      .attrs
      .inner
      .enabled_buttons
      .set(WindowButtons::CLOSE, closable);
    self
  }

  fn title<S: Into<String>>(mut self, title: S) -> Self {
    self.attrs.inner = self.attrs.inner.with_title(title);
    self
  }

  fn fullscreen(mut self, fullscreen: bool) -> Self {
    self.attrs.inner = self
      .attrs
      .inner
      .with_fullscreen(fullscreen.then_some(Fullscreen::Borderless(None)));
    self
  }

  fn focused(mut self, focused: bool) -> Self {
    self.attrs.inner = self.attrs.inner.with_active(focused);
    self
  }

  fn focusable(self, _focusable: bool) -> Self {
    // TODO
    self
  }

  fn maximized(mut self, maximized: bool) -> Self {
    self.attrs.inner = self.attrs.inner.with_maximized(maximized);
    self
  }

  fn visible(mut self, visible: bool) -> Self {
    self.attrs.inner = self.attrs.inner.with_visible(visible);
    self
  }

  #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
  fn transparent(mut self, transparent: bool) -> Self {
    self.attrs.inner = self.attrs.inner.with_transparent(transparent);
    self
  }

  fn decorations(mut self, decorations: bool) -> Self {
    self.attrs.inner = self.attrs.inner.with_decorations(decorations);
    self
  }

  fn always_on_bottom(mut self, always_on_bottom: bool) -> Self {
    self.attrs.inner = self.attrs.inner.with_window_level(if always_on_bottom {
      winit::window::WindowLevel::AlwaysOnBottom
    } else {
      winit::window::WindowLevel::Normal
    });
    self
  }

  fn always_on_top(mut self, always_on_top: bool) -> Self {
    self.attrs.inner = self.attrs.inner.with_window_level(if always_on_top {
      winit::window::WindowLevel::AlwaysOnTop
    } else {
      winit::window::WindowLevel::Normal
    });
    self
  }

  #[cfg_attr(windows, allow(unused))]
  fn visible_on_all_workspaces(mut self, visible_on_all_workspaces: bool) -> Self {
    #[cfg(any(
      target_os = "macos",
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "netbsd",
      target_os = "openbsd"
    ))]
    {
      self.attrs.visible_on_all_workspaces = visible_on_all_workspaces;
    }

    self
  }

  fn content_protected(mut self, protected: bool) -> Self {
    self.attrs.inner = self.attrs.inner.with_content_protected(protected);
    self
  }

  fn icon(mut self, icon: Icon) -> Result<Self> {
    let icon = super::window::tauri_icon_to_winit_icon(icon)?;
    self.attrs.inner = self.attrs.inner.with_window_icon(Some(icon));
    Ok(self)
  }

  #[allow(unused_mut)]
  fn skip_taskbar(mut self, skip: bool) -> Self {
    #[cfg(windows)]
    {
      let pl_attrs = platform_attrs(&mut self.attrs.inner).with_skip_taskbar(skip);
      self.attrs.inner = self
        .attrs
        .inner
        .with_platform_attributes(Box::new(pl_attrs));
    }

    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "netbsd",
      target_os = "openbsd"
    ))]
    {
      self.attrs.skip_taskbar = skip;
    }

    #[cfg(target_os = "macos")]
    let _skip = skip;

    self
  }

  fn background_color(mut self, color: Color) -> Self {
    self.attrs.background_color = Some(color);
    self
  }

  #[cfg_attr(
    not(any(windows, target_os = "macos")),
    allow(unused_mut, unused_variables)
  )]
  fn shadow(mut self, enable: bool) -> Self {
    #[cfg(windows)]
    {
      let pl_attrs = platform_attrs(&mut self.attrs.inner).with_undecorated_shadow(enable);
      self.attrs.inner = self
        .attrs
        .inner
        .with_platform_attributes(Box::new(pl_attrs));
    }

    #[cfg(target_os = "macos")]
    {
      let pl_attrs = platform_attrs(&mut self.attrs.inner).with_has_shadow(enable);
      self.attrs.inner = self
        .attrs
        .inner
        .with_platform_attributes(Box::new(pl_attrs));
    }

    self
  }

  #[cfg(windows)]
  fn owner(mut self, owner: HWND) -> Self {
    let pl_attrs = platform_attrs(&mut self.attrs.inner).with_owner_window(owner.0);
    self.attrs.inner = self
      .attrs
      .inner
      .with_platform_attributes(Box::new(pl_attrs));
    self
  }

  #[cfg(windows)]
  fn parent(mut self, parent: HWND) -> Self {
    if let Some(hwnd) = NonZeroIsize::new(parent.0 as isize) {
      let handle = RawWindowHandle::Win32(winit::raw_window_handle::Win32WindowHandle::new(hwnd));
      // SAFETY: Tauri passes a live parent HWND owned by the application.
      self.attrs.inner = unsafe { self.attrs.inner.with_parent_window(Some(handle)) };
    }
    self
  }

  #[cfg(windows)]
  fn drag_and_drop(mut self, enabled: bool) -> Self {
    let pl_attrs = platform_attrs(&mut self.attrs.inner).with_drag_and_drop(enabled);
    self.attrs.inner = self
      .attrs
      .inner
      .with_platform_attributes(Box::new(pl_attrs));
    self
  }

  #[cfg(target_os = "macos")]
  fn parent(mut self, parent: *mut std::ffi::c_void) -> Self {
    use objc2::rc::Retained;
    use objc2_app_kit::{NSView, NSWindow};

    let Some(nswindow) = NonNull::new(parent) else {
      return self;
    };
    let Some(nswindow) = (unsafe { Retained::<NSWindow>::from_raw(nswindow.as_ptr() as _) }) else {
      return self;
    };

    let Some(nsview) = nswindow.contentView() else {
      return self;
    };
    let nsview = Retained::<NSView>::into_raw(nsview);
    let Some(nsview) = NonNull::new(nsview as _) else {
      return self;
    };

    let handle = winit::raw_window_handle::AppKitWindowHandle::new(nsview);
    let handle = RawWindowHandle::AppKit(handle);
    self.attrs.inner = unsafe { self.attrs.inner.with_parent_window(Some(handle)) };

    self
  }

  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn transient_for(self, _parent: &impl gtk::glib::IsA<gtk::Window>) -> Self {
    self
  }

  #[cfg(target_os = "macos")]
  fn title_bar_style(mut self, style: TitleBarStyle) -> Self {
    let pl_attrs = *platform_attrs(&mut self.attrs.inner);
    let pl_attrs = match style {
      TitleBarStyle::Visible => pl_attrs
        .with_titlebar_transparent(false)
        .with_fullsize_content_view(false),
      TitleBarStyle::Transparent => pl_attrs
        .with_titlebar_transparent(true)
        .with_fullsize_content_view(false),
      TitleBarStyle::Overlay => pl_attrs
        .with_titlebar_transparent(true)
        .with_fullsize_content_view(true),
      _ => pl_attrs,
    };
    self.attrs.inner = self
      .attrs
      .inner
      .with_platform_attributes(Box::new(pl_attrs));
    self
  }

  #[cfg(target_os = "macos")]
  fn traffic_light_position<P: Into<Position>>(mut self, position: P) -> Self {
    self.attrs.traffic_light_position = Some(position.into());
    self
  }

  #[cfg(target_os = "macos")]
  fn hidden_title(mut self, hidden: bool) -> Self {
    let pl_attrs = platform_attrs(&mut self.attrs.inner).with_title_hidden(hidden);
    self.attrs.inner = self
      .attrs
      .inner
      .with_platform_attributes(Box::new(pl_attrs));
    self
  }

  #[cfg(target_os = "macos")]
  fn tabbing_identifier(mut self, identifier: &str) -> Self {
    let pl_attrs = platform_attrs(&mut self.attrs.inner).with_tabbing_identifier(identifier);
    self.attrs.inner = self
      .attrs
      .inner
      .with_platform_attributes(Box::new(pl_attrs));
    self
  }

  fn theme(mut self, theme: Option<Theme>) -> Self {
    self.attrs.inner = self
      .attrs
      .inner
      .with_theme(tauri_theme_to_winit_theme(theme));
    self
  }

  #[allow(unused_mut)]
  fn window_classname<S: Into<String>>(mut self, _window_classname: S) -> Self {
    #[cfg(windows)]
    {
      let pl_attrs =
        platform_attrs(&mut self.attrs.inner).with_class_name(_window_classname.into());
      self.attrs.inner = self
        .attrs
        .inner
        .with_platform_attributes(Box::new(pl_attrs));
    }

    self
  }

  // TODO
  fn no_redirection_bitmap(#[allow(unused_mut)] mut self, _enable: bool) -> Self {
    self
  }

  fn has_icon(&self) -> bool {
    self.attrs.inner.window_icon.is_some()
  }

  fn get_theme(&self) -> Option<Theme> {
    self
      .attrs
      .inner
      .preferred_theme
      .map(winit_theme_to_tauri_theme)
  }
}

#[cfg(windows)]
type PlatformAttributes = winit::platform::windows::WindowAttributesWindows;
#[cfg(target_os = "macos")]
type PlatformAttributes = winit::platform::macos::WindowAttributesMacOS;

#[cfg(any(windows, target_os = "macos"))]
fn platform_attrs(attrs: &mut WindowAttributes) -> Box<PlatformAttributes> {
  attrs
    .platform
    .take()
    .and_then(|attrs| attrs.cast::<PlatformAttributes>().ok())
    .unwrap_or_default()
}
