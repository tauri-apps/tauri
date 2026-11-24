// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [`WindowBuilder`] implementation: a plain attribute bag applied on the main thread.

use tauri_runtime::{
  Icon,
  dpi::{PixelUnit, Size},
  window::{WindowBuilder, WindowBuilderBase, WindowSizeConstraints},
};
use tauri_utils::{
  Theme,
  config::{Color, PreventOverflowConfig, WindowConfig},
};

#[derive(Debug, Clone)]
pub struct OwnedIcon {
  pub rgba: Vec<u8>,
  pub width: u32,
  pub height: u32,
}

#[derive(Debug, Clone)]
pub enum PreventOverflow {
  Off,
  On,
  Margin(Size),
}

/// Window attributes, in logical pixels.
#[derive(Debug, Clone)]
pub struct QtWindowBuilder {
  pub title: String,
  pub inner_size: Option<(f64, f64)>,
  pub position: Option<(f64, f64)>,
  pub center: bool,
  pub constraints: WindowSizeConstraints,
  pub prevent_overflow: PreventOverflow,
  pub resizable: bool,
  pub maximizable: bool,
  pub minimizable: bool,
  pub closable: bool,
  pub fullscreen: bool,
  pub focused: bool,
  pub focusable: bool,
  pub maximized: bool,
  pub visible: bool,
  pub transparent: bool,
  pub decorations: bool,
  pub always_on_bottom: bool,
  pub always_on_top: bool,
  pub visible_on_all_workspaces: bool,
  pub content_protected: bool,
  pub icon: Option<OwnedIcon>,
  pub skip_taskbar: bool,
  pub background_color: Option<Color>,
  pub shadow: bool,
  pub theme: Option<Theme>,
  /// `QWidget*` of the window to be transient for (see `transient_for_qt`).
  pub transient_parent: Option<usize>,
}

impl Default for QtWindowBuilder {
  fn default() -> Self {
    Self {
      title: "Tauri App".into(),
      inner_size: None,
      position: None,
      center: false,
      constraints: WindowSizeConstraints::default(),
      prevent_overflow: PreventOverflow::Off,
      resizable: true,
      maximizable: true,
      minimizable: true,
      closable: true,
      fullscreen: false,
      focused: true,
      focusable: true,
      maximized: false,
      visible: true,
      transparent: false,
      decorations: true,
      always_on_bottom: false,
      always_on_top: false,
      visible_on_all_workspaces: false,
      content_protected: false,
      icon: None,
      skip_taskbar: false,
      background_color: None,
      shadow: true,
      theme: None,
      transient_parent: None,
    }
  }
}

impl WindowBuilderBase for QtWindowBuilder {}

impl WindowBuilder for QtWindowBuilder {
  fn new() -> Self {
    Self::default()
  }

  fn with_config(config: &WindowConfig) -> Self {
    let mut w = Self::new()
      .inner_size(config.width, config.height)
      .title(config.title.clone())
      .focused(config.focus)
      .focusable(config.focusable)
      .visible(config.visible)
      .resizable(config.resizable)
      .fullscreen(config.fullscreen)
      .decorations(config.decorations)
      .maximized(config.maximized)
      .always_on_bottom(config.always_on_bottom)
      .always_on_top(config.always_on_top)
      .visible_on_all_workspaces(config.visible_on_all_workspaces)
      .content_protected(config.content_protected)
      .skip_taskbar(config.skip_taskbar)
      .theme(config.theme)
      .closable(config.closable)
      .maximizable(config.maximizable)
      .minimizable(config.minimizable)
      .shadow(config.shadow)
      .transparent(config.transparent);

    let unit = |v: Option<f64>| v.map(|v| PixelUnit::Logical(v.into()));
    w = w.inner_size_constraints(WindowSizeConstraints {
      min_width: unit(config.min_width),
      min_height: unit(config.min_height),
      max_width: unit(config.max_width),
      max_height: unit(config.max_height),
    });
    if let Some(color) = config.background_color {
      w = w.background_color(color);
    }
    if let (Some(x), Some(y)) = (config.x, config.y) {
      w = w.position(x, y);
    }
    if config.center {
      w = w.center();
    }
    match &config.prevent_overflow {
      Some(PreventOverflowConfig::Enable(true)) => w = w.prevent_overflow(),
      Some(PreventOverflowConfig::Margin(m)) => {
        // Physical pixels, as documented by `PreventOverflowMargin` (and as tauri-runtime-wry does).
        w = w.prevent_overflow_with_margin(Size::Physical((m.width, m.height).into()))
      }
      _ => {}
    }
    w
  }

  fn center(mut self) -> Self {
    self.center = true;
    self
  }
  fn position(mut self, x: f64, y: f64) -> Self {
    self.position = Some((x, y));
    self
  }
  fn inner_size(mut self, width: f64, height: f64) -> Self {
    self.inner_size = Some((width, height));
    self
  }
  fn min_inner_size(mut self, min_width: f64, min_height: f64) -> Self {
    self.constraints.min_width = Some(PixelUnit::Logical(min_width.into()));
    self.constraints.min_height = Some(PixelUnit::Logical(min_height.into()));
    self
  }
  fn max_inner_size(mut self, max_width: f64, max_height: f64) -> Self {
    self.constraints.max_width = Some(PixelUnit::Logical(max_width.into()));
    self.constraints.max_height = Some(PixelUnit::Logical(max_height.into()));
    self
  }
  fn inner_size_constraints(mut self, constraints: WindowSizeConstraints) -> Self {
    self.constraints = constraints;
    self
  }
  fn prevent_overflow(mut self) -> Self {
    self.prevent_overflow = PreventOverflow::On;
    self
  }
  fn prevent_overflow_with_margin(mut self, margin: Size) -> Self {
    self.prevent_overflow = PreventOverflow::Margin(margin);
    self
  }
  fn resizable(mut self, resizable: bool) -> Self {
    self.resizable = resizable;
    self
  }
  fn maximizable(mut self, maximizable: bool) -> Self {
    self.maximizable = maximizable;
    self
  }
  fn minimizable(mut self, minimizable: bool) -> Self {
    self.minimizable = minimizable;
    self
  }
  fn closable(mut self, closable: bool) -> Self {
    self.closable = closable;
    self
  }
  fn title<S: Into<String>>(mut self, title: S) -> Self {
    self.title = title.into();
    self
  }
  fn fullscreen(mut self, fullscreen: bool) -> Self {
    self.fullscreen = fullscreen;
    self
  }
  fn focused(mut self, focused: bool) -> Self {
    self.focused = focused;
    self
  }
  fn focusable(mut self, focusable: bool) -> Self {
    self.focusable = focusable;
    self
  }
  fn maximized(mut self, maximized: bool) -> Self {
    self.maximized = maximized;
    self
  }
  fn visible(mut self, visible: bool) -> Self {
    self.visible = visible;
    self
  }
  fn transparent(mut self, transparent: bool) -> Self {
    self.transparent = transparent;
    self
  }
  fn decorations(mut self, decorations: bool) -> Self {
    self.decorations = decorations;
    self
  }
  fn always_on_bottom(mut self, always_on_bottom: bool) -> Self {
    self.always_on_bottom = always_on_bottom;
    self
  }
  fn always_on_top(mut self, always_on_top: bool) -> Self {
    self.always_on_top = always_on_top;
    self
  }
  fn visible_on_all_workspaces(mut self, visible_on_all_workspaces: bool) -> Self {
    self.visible_on_all_workspaces = visible_on_all_workspaces;
    self
  }
  fn content_protected(mut self, protected: bool) -> Self {
    self.content_protected = protected;
    self
  }
  fn icon(mut self, icon: Icon) -> tauri_runtime::Result<Self> {
    self.icon = Some(OwnedIcon {
      rgba: icon.rgba.into_owned(),
      width: icon.width,
      height: icon.height,
    });
    Ok(self)
  }
  fn skip_taskbar(mut self, skip: bool) -> Self {
    self.skip_taskbar = skip;
    self
  }
  fn background_color(mut self, color: Color) -> Self {
    self.background_color = Some(color);
    self
  }
  fn shadow(mut self, enable: bool) -> Self {
    self.shadow = enable;
    self
  }
  /// Qt windows cannot be made transient for a GTK window; this is ignored.
  ///
  /// `parent` is an owned `GtkWindow*`, but this runtime never hands out GTK windows, so there
  /// is no reference to release here.
  fn transient_for(self, _parent: *mut std::ffi::c_void) -> Self {
    log::warn!("transient_for(GtkWindow) is not supported by the Qt runtime");
    self
  }

  /// Make the window transient for a Qt window (a `QWidget*` from `qt_window`).
  fn transient_for_qt(mut self, parent: *mut std::ffi::c_void) -> Self {
    self.transient_parent = (!parent.is_null()).then_some(parent as usize);
    self
  }
  fn theme(mut self, theme: Option<Theme>) -> Self {
    self.theme = theme;
    self
  }
  fn has_icon(&self) -> bool {
    self.icon.is_some()
  }
  fn get_theme(&self) -> Option<Theme> {
    self.theme
  }
  fn window_classname<S: Into<String>>(self, _window_classname: S) -> Self {
    self
  }
  fn no_redirection_bitmap(self, _enable: bool) -> Self {
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use tauri_runtime::dpi::{LogicalSize, PhysicalSize};

  #[test]
  fn defaults_match_tauri() {
    let b = QtWindowBuilder::new();
    assert_eq!(b.title, "Tauri App");
    assert!(b.resizable && b.maximizable && b.minimizable && b.closable);
    assert!(b.visible && b.decorations && b.focused && b.focusable && b.shadow);
    assert!(!b.fullscreen && !b.maximized && !b.transparent && !b.center);
    assert!(!b.always_on_top && !b.always_on_bottom && !b.skip_taskbar);
    assert!(b.inner_size.is_none() && b.position.is_none() && b.icon.is_none());
    assert!(matches!(b.prevent_overflow, PreventOverflow::Off));
    assert!(!b.has_icon());
    assert_eq!(b.get_theme(), None);
  }

  #[test]
  fn setters_fill_the_attribute_bag() {
    let b = QtWindowBuilder::new()
      .title("hello")
      .center()
      .position(10.0, 20.0)
      .inner_size(300.0, 200.0)
      .min_inner_size(100.0, 50.0)
      .max_inner_size(900.0, 700.0)
      .resizable(false)
      .maximizable(false)
      .minimizable(false)
      .closable(false)
      .fullscreen(true)
      .focused(false)
      .focusable(false)
      .maximized(true)
      .visible(false)
      .transparent(true)
      .decorations(false)
      .always_on_bottom(true)
      .always_on_top(true)
      .visible_on_all_workspaces(true)
      .content_protected(true)
      .skip_taskbar(true)
      .background_color(Color(1, 2, 3, 4))
      .shadow(false)
      .theme(Some(Theme::Dark))
      .window_classname("ignored")
      .no_redirection_bitmap(true)
      .icon(Icon {
        rgba: vec![255; 16].into(),
        width: 2,
        height: 2,
      })
      .unwrap();
    assert_eq!(b.title, "hello");
    assert!(b.center);
    assert_eq!(b.position, Some((10.0, 20.0)));
    assert_eq!(b.inner_size, Some((300.0, 200.0)));
    assert_eq!(
      b.constraints.min_width,
      Some(PixelUnit::Logical(100.0.into()))
    );
    assert_eq!(
      b.constraints.min_height,
      Some(PixelUnit::Logical(50.0.into()))
    );
    assert_eq!(
      b.constraints.max_width,
      Some(PixelUnit::Logical(900.0.into()))
    );
    assert_eq!(
      b.constraints.max_height,
      Some(PixelUnit::Logical(700.0.into()))
    );
    assert!(!b.resizable && !b.maximizable && !b.minimizable && !b.closable);
    assert!(b.fullscreen && !b.focused && !b.focusable && b.maximized && !b.visible);
    assert!(b.transparent && !b.decorations && b.always_on_bottom && b.always_on_top);
    assert!(b.visible_on_all_workspaces && b.content_protected && b.skip_taskbar);
    assert_eq!(b.background_color, Some(Color(1, 2, 3, 4)));
    assert!(!b.shadow);
    assert_eq!(b.get_theme(), Some(Theme::Dark));
    assert!(b.has_icon());
    let icon = b.icon.as_ref().unwrap();
    assert_eq!((icon.width, icon.height, icon.rgba.len()), (2, 2, 16));
  }

  #[test]
  fn prevent_overflow_variants() {
    let b = QtWindowBuilder::new().prevent_overflow();
    assert!(matches!(b.prevent_overflow, PreventOverflow::On));
    let b = QtWindowBuilder::new().prevent_overflow_with_margin(LogicalSize::new(5.0, 6.0).into());
    assert!(
      matches!(b.prevent_overflow, PreventOverflow::Margin(Size::Logical(s)) if s == LogicalSize::new(5.0, 6.0))
    );
  }

  #[test]
  fn transient_parents() {
    // A GTK parent is ignored; a null Qt parent clears it.
    let b = QtWindowBuilder::new().transient_for(0x1000 as *mut _);
    assert_eq!(b.transient_parent, None);
    let b = QtWindowBuilder::new().transient_for_qt(0x1000 as *mut _);
    assert_eq!(b.transient_parent, Some(0x1000));
    let b = b.transient_for_qt(std::ptr::null_mut());
    assert_eq!(b.transient_parent, None);
  }

  fn config() -> WindowConfig {
    serde_json::from_value(serde_json::json!({
      "title": "from config",
      "width": 640.0,
      "height": 480.0,
      "minWidth": 200.0,
      "minHeight": 100.0,
      "maxWidth": 1000.0,
      "x": 5.0,
      "y": 6.0,
      "center": true,
      "resizable": false,
      "maximizable": false,
      "minimizable": false,
      "closable": false,
      "fullscreen": true,
      "focus": false,
      "focusable": false,
      "maximized": true,
      "visible": false,
      "transparent": true,
      "decorations": false,
      "alwaysOnBottom": true,
      "alwaysOnTop": true,
      "visibleOnAllWorkspaces": true,
      "contentProtected": true,
      "skipTaskbar": true,
      "theme": "Dark",
      "shadow": false,
      "backgroundColor": "#11223344",
      "preventOverflow": true
    }))
    .unwrap()
  }

  #[test]
  fn with_config_reads_every_field() {
    let b = QtWindowBuilder::with_config(&config());
    assert_eq!(b.title, "from config");
    assert_eq!(b.inner_size, Some((640.0, 480.0)));
    assert_eq!(
      b.constraints.min_width,
      Some(PixelUnit::Logical(200.0.into()))
    );
    assert_eq!(
      b.constraints.min_height,
      Some(PixelUnit::Logical(100.0.into()))
    );
    assert_eq!(
      b.constraints.max_width,
      Some(PixelUnit::Logical(1000.0.into()))
    );
    assert_eq!(b.constraints.max_height, None);
    assert_eq!(b.position, Some((5.0, 6.0)));
    assert!(b.center);
    assert!(!b.resizable && !b.maximizable && !b.minimizable && !b.closable);
    assert!(b.fullscreen && !b.focused && !b.focusable && b.maximized && !b.visible);
    assert!(b.transparent && !b.decorations && b.always_on_bottom && b.always_on_top);
    assert!(b.visible_on_all_workspaces && b.content_protected && b.skip_taskbar);
    assert_eq!(b.theme, Some(Theme::Dark));
    assert!(!b.shadow);
    assert_eq!(b.background_color, Some(Color(0x11, 0x22, 0x33, 0x44)));
    assert!(matches!(b.prevent_overflow, PreventOverflow::On));
  }

  #[test]
  fn with_config_defaults() {
    let b = QtWindowBuilder::with_config(&WindowConfig::default());
    assert_eq!(b.inner_size, Some((800.0, 600.0)));
    assert!(b.position.is_none() && !b.center);
    assert!(matches!(b.prevent_overflow, PreventOverflow::Off));
    assert!(b.background_color.is_none());
  }

  #[test]
  fn with_config_prevent_overflow_margin_is_physical() {
    let c = WindowConfig {
      prevent_overflow: Some(PreventOverflowConfig::Margin(
        tauri_utils::config::PreventOverflowMargin {
          width: 10,
          height: 20,
        },
      )),
      ..Default::default()
    };
    let b = QtWindowBuilder::with_config(&c);
    assert!(
      matches!(b.prevent_overflow, PreventOverflow::Margin(Size::Physical(s)) if s == PhysicalSize::new(10, 20)),
      "{:?}",
      b.prevent_overflow
    );
  }
}
