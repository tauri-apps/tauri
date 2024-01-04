// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The tauri plugin to create and manipulate windows from JS.

use crate::{
  plugin::{Builder, TauriPlugin},
  Runtime,
};

#[cfg(desktop)]
mod desktop_commands {
  use serde::Deserialize;
  use tauri_runtime::ResizeDirection;
  use tauri_utils::ProgressBarState;

  use super::*;
  use crate::{
    command,
    utils::config::{WindowConfig, WindowEffectsConfig},
    AppHandle, CursorIcon, Icon, Manager, Monitor, PhysicalPosition, PhysicalSize, Position, Size,
    Theme, UserAttentionType, Window, WindowBuilder,
  };

  #[derive(Deserialize)]
  #[serde(untagged)]
  pub enum IconDto {
    #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
    File(std::path::PathBuf),
    #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
    Raw(Vec<u8>),
    Rgba {
      rgba: Vec<u8>,
      width: u32,
      height: u32,
    },
  }

  impl From<IconDto> for Icon {
    fn from(icon: IconDto) -> Self {
      match icon {
        #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
        IconDto::File(path) => Self::File(path),
        #[cfg(any(feature = "icon-png", feature = "icon-ico"))]
        IconDto::Raw(raw) => Self::Raw(raw),
        IconDto::Rgba {
          rgba,
          width,
          height,
        } => Self::Rgba {
          rgba,
          width,
          height,
        },
      }
    }
  }

  #[command(root = "crate")]
  pub async fn create<R: Runtime>(app: AppHandle<R>, options: WindowConfig) -> crate::Result<()> {
    WindowBuilder::from_config(&app, options).build()?;
    Ok(())
  }

  fn get_window<R: Runtime>(window: Window<R>, label: Option<String>) -> crate::Result<Window<R>> {
    match label {
      Some(l) if !l.is_empty() => window.get_window(&l).ok_or(crate::Error::WindowNotFound),
      _ => Ok(window),
    }
  }

  macro_rules! getter {
    ($cmd: ident, $ret: ty) => {
      #[command(root = "crate")]
      pub async fn $cmd<R: Runtime>(
        window: Window<R>,
        label: Option<String>,
      ) -> crate::Result<$ret> {
        get_window(window, label)?.$cmd().map_err(Into::into)
      }
    };
  }

  macro_rules! setter {
    ($cmd: ident) => {
      #[command(root = "crate")]
      pub async fn $cmd<R: Runtime>(window: Window<R>, label: Option<String>) -> crate::Result<()> {
        get_window(window, label)?.$cmd().map_err(Into::into)
      }
    };

    ($cmd: ident, $input: ty) => {
      #[command(root = "crate")]
      pub async fn $cmd<R: Runtime>(
        window: Window<R>,
        label: Option<String>,
        value: $input,
      ) -> crate::Result<()> {
        get_window(window, label)?.$cmd(value).map_err(Into::into)
      }
    };
  }

  getter!(scale_factor, f64);
  getter!(inner_position, PhysicalPosition<i32>);
  getter!(outer_position, PhysicalPosition<i32>);
  getter!(inner_size, PhysicalSize<u32>);
  getter!(outer_size, PhysicalSize<u32>);
  getter!(is_fullscreen, bool);
  getter!(is_minimized, bool);
  getter!(is_maximized, bool);
  getter!(is_focused, bool);
  getter!(is_decorated, bool);
  getter!(is_resizable, bool);
  getter!(is_maximizable, bool);
  getter!(is_minimizable, bool);
  getter!(is_closable, bool);
  getter!(is_visible, bool);
  getter!(title, String);
  getter!(current_monitor, Option<Monitor>);
  getter!(primary_monitor, Option<Monitor>);
  getter!(available_monitors, Vec<Monitor>);
  getter!(theme, Theme);

  setter!(center);
  setter!(request_user_attention, Option<UserAttentionType>);
  setter!(set_resizable, bool);
  setter!(set_maximizable, bool);
  setter!(set_minimizable, bool);
  setter!(set_closable, bool);
  setter!(set_title, &str);
  setter!(maximize);
  setter!(unmaximize);
  setter!(minimize);
  setter!(unminimize);
  setter!(show);
  setter!(hide);
  setter!(close);
  setter!(set_decorations, bool);
  setter!(set_shadow, bool);
  setter!(set_effects, Option<WindowEffectsConfig>);
  setter!(set_always_on_top, bool);
  setter!(set_always_on_bottom, bool);
  setter!(set_content_protected, bool);
  setter!(set_size, Size);
  setter!(set_min_size, Option<Size>);
  setter!(set_max_size, Option<Size>);
  setter!(set_position, Position);
  setter!(set_fullscreen, bool);
  setter!(set_focus);
  setter!(set_skip_taskbar, bool);
  setter!(set_cursor_grab, bool);
  setter!(set_cursor_visible, bool);
  setter!(set_cursor_icon, CursorIcon);
  setter!(set_cursor_position, Position);
  setter!(set_ignore_cursor_events, bool);
  setter!(start_dragging);
  setter!(start_resize_dragging, ResizeDirection);
  setter!(set_progress_bar, ProgressBarState);
  setter!(print);

  #[command(root = "crate")]
  pub async fn set_icon<R: Runtime>(
    window: Window<R>,
    label: Option<String>,
    value: IconDto,
  ) -> crate::Result<()> {
    get_window(window, label)?
      .set_icon(value.into())
      .map_err(Into::into)
  }

  #[command(root = "crate")]
  pub async fn toggle_maximize<R: Runtime>(
    window: Window<R>,
    label: Option<String>,
  ) -> crate::Result<()> {
    let window = get_window(window, label)?;
    match window.is_maximized()? {
      true => window.unmaximize()?,
      false => window.maximize()?,
    };
    Ok(())
  }

  #[command(root = "crate")]
  pub async fn internal_toggle_maximize<R: Runtime>(
    window: Window<R>,
    label: Option<String>,
  ) -> crate::Result<()> {
    let window = get_window(window, label)?;
    if window.is_resizable()? {
      match window.is_maximized()? {
        true => window.unmaximize()?,
        false => window.maximize()?,
      };
    }
    Ok(())
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  #[command(root = "crate")]
  pub async fn internal_toggle_devtools<R: Runtime>(
    window: Window<R>,
    label: Option<String>,
  ) -> crate::Result<()> {
    let window = get_window(window, label)?;
    if window.is_devtools_open() {
      window.close_devtools();
    } else {
      window.open_devtools();
    }
    Ok(())
  }

  #[derive(Debug)]
  enum HitTestResult {
    CLIENT,
    LEFT,
    RIGHT,
    TOP,
    BOTTOM,
    TOPLEFT,
    TOPRIGHT,
    BOTTOMLEFT,
    BOTTOMRIGHT,
    NOWHERE,
  }

  impl HitTestResult {
    fn drag_resize_window<R: Runtime>(&self, window: &Window<R>) {
      let _ = window.start_resize_dragging(match self {
        HitTestResult::LEFT => ResizeDirection::West,
        HitTestResult::RIGHT => ResizeDirection::East,
        HitTestResult::TOP => ResizeDirection::North,
        HitTestResult::BOTTOM => ResizeDirection::South,
        HitTestResult::TOPLEFT => ResizeDirection::NorthWest,
        HitTestResult::TOPRIGHT => ResizeDirection::NorthEast,
        HitTestResult::BOTTOMLEFT => ResizeDirection::SouthWest,
        HitTestResult::BOTTOMRIGHT => ResizeDirection::SouthEast,
        _ => unreachable!(),
      });
    }

    fn change_cursor<R: Runtime>(&self, window: &Window<R>) {
      let _ = window.set_cursor_icon(match self {
        HitTestResult::LEFT => CursorIcon::WResize,
        HitTestResult::RIGHT => CursorIcon::EResize,
        HitTestResult::TOP => CursorIcon::NResize,
        HitTestResult::BOTTOM => CursorIcon::SResize,
        HitTestResult::TOPLEFT => CursorIcon::NwResize,
        HitTestResult::TOPRIGHT => CursorIcon::NeResize,
        HitTestResult::BOTTOMLEFT => CursorIcon::SwResize,
        HitTestResult::BOTTOMRIGHT => CursorIcon::SeResize,
        _ => CursorIcon::Default,
      });
    }
  }

  fn hit_test(window_size: PhysicalSize<u32>, x: i32, y: i32, scale: f64) -> HitTestResult {
    const BORDERLESS_RESIZE_INSET: f64 = 5.0;

    const CLIENT: isize = 0b0000;
    const LEFT: isize = 0b0001;
    const RIGHT: isize = 0b0010;
    const TOP: isize = 0b0100;
    const BOTTOM: isize = 0b1000;
    const TOPLEFT: isize = TOP | LEFT;
    const TOPRIGHT: isize = TOP | RIGHT;
    const BOTTOMLEFT: isize = BOTTOM | LEFT;
    const BOTTOMRIGHT: isize = BOTTOM | RIGHT;

    let top = 0;
    let left = 0;
    let bottom = top + window_size.height as i32;
    let right = left + window_size.width as i32;

    let inset = (BORDERLESS_RESIZE_INSET * scale) as i32;

    #[rustfmt::skip]
        let result =
            (LEFT * (if x < (left + inset) { 1 } else { 0 }))
          | (RIGHT * (if x >= (right - inset) { 1 } else { 0 }))
          | (TOP * (if y < (top + inset) { 1 } else { 0 }))
          | (BOTTOM * (if y >= (bottom - inset) { 1 } else { 0 }));

    match result {
      CLIENT => HitTestResult::CLIENT,
      LEFT => HitTestResult::LEFT,
      RIGHT => HitTestResult::RIGHT,
      TOP => HitTestResult::TOP,
      BOTTOM => HitTestResult::BOTTOM,
      TOPLEFT => HitTestResult::TOPLEFT,
      TOPRIGHT => HitTestResult::TOPRIGHT,
      BOTTOMLEFT => HitTestResult::BOTTOMLEFT,
      BOTTOMRIGHT => HitTestResult::BOTTOMRIGHT,
      _ => HitTestResult::NOWHERE,
    }
  }

  #[command(root = "crate")]
  pub async fn on_mousemove<R: Runtime>(window: Window<R>, x: i32, y: i32) -> crate::Result<()> {
    hit_test(window.inner_size()?, x, y, window.scale_factor()?).change_cursor(&window);
    Ok(())
  }

  #[command(root = "crate")]
  pub async fn on_mousedown<R: Runtime>(window: Window<R>, x: i32, y: i32) -> crate::Result<()> {
    let res = hit_test(window.inner_size()?, x, y, window.scale_factor()?);
    match res {
      HitTestResult::CLIENT | HitTestResult::NOWHERE => {}
      _ => res.drag_resize_window(&window),
    };
    Ok(())
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  use serialize_to_javascript::{default_template, DefaultTemplate, Template};

  let mut init_script = String::new();
  // window.print works on Linux/Windows; need to use the API on macOS
  #[cfg(any(target_os = "macos", target_os = "ios"))]
  {
    init_script.push_str(include_str!("./scripts/print.js"));
  }

  #[derive(Template)]
  #[default_template("./scripts/drag.js")]
  struct Drag<'a> {
    os_name: &'a str,
  }

  init_script.push_str(
    &Drag {
      os_name: std::env::consts::OS,
    }
    .render_default(&Default::default())
    .unwrap()
    .into_string(),
  );

  #[derive(Template)]
  #[default_template("./scripts/undecorated-resizing.js")]
  struct UndecoratedResizingJavascript<'a> {
    os_name: &'a str,
  }

  init_script.push_str(
    &UndecoratedResizingJavascript {
      os_name: std::env::consts::OS,
    }
    .render_default(&Default::default())
    .unwrap()
    .into_string(),
  );

  #[cfg(any(debug_assertions, feature = "devtools"))]
  {
    #[derive(Template)]
    #[default_template("./scripts/toggle-devtools.js")]
    struct Devtools<'a> {
      os_name: &'a str,
    }

    init_script.push_str(
      &Devtools {
        os_name: std::env::consts::OS,
      }
      .render_default(&Default::default())
      .unwrap()
      .into_string(),
    );
  }

  Builder::new("window")
    .js_init_script(init_script)
    .invoke_handler(|invoke| {
      #[cfg(desktop)]
      {
        let handler: Box<dyn Fn(crate::ipc::Invoke<R>) -> bool> =
          Box::new(crate::generate_handler![
            desktop_commands::create,
            // getters
            desktop_commands::scale_factor,
            desktop_commands::inner_position,
            desktop_commands::outer_position,
            desktop_commands::inner_size,
            desktop_commands::outer_size,
            desktop_commands::is_fullscreen,
            desktop_commands::is_minimized,
            desktop_commands::is_maximized,
            desktop_commands::is_focused,
            desktop_commands::is_decorated,
            desktop_commands::is_resizable,
            desktop_commands::is_maximizable,
            desktop_commands::is_minimizable,
            desktop_commands::is_closable,
            desktop_commands::is_visible,
            desktop_commands::title,
            desktop_commands::current_monitor,
            desktop_commands::primary_monitor,
            desktop_commands::available_monitors,
            desktop_commands::theme,
            // setters
            desktop_commands::center,
            desktop_commands::request_user_attention,
            desktop_commands::set_resizable,
            desktop_commands::set_maximizable,
            desktop_commands::set_minimizable,
            desktop_commands::set_closable,
            desktop_commands::set_title,
            desktop_commands::maximize,
            desktop_commands::unmaximize,
            desktop_commands::minimize,
            desktop_commands::unminimize,
            desktop_commands::show,
            desktop_commands::hide,
            desktop_commands::close,
            desktop_commands::set_decorations,
            desktop_commands::set_shadow,
            desktop_commands::set_effects,
            desktop_commands::set_always_on_top,
            desktop_commands::set_always_on_bottom,
            desktop_commands::set_content_protected,
            desktop_commands::set_size,
            desktop_commands::set_min_size,
            desktop_commands::set_max_size,
            desktop_commands::set_position,
            desktop_commands::set_fullscreen,
            desktop_commands::set_focus,
            desktop_commands::set_skip_taskbar,
            desktop_commands::set_cursor_grab,
            desktop_commands::set_cursor_visible,
            desktop_commands::set_cursor_icon,
            desktop_commands::set_cursor_position,
            desktop_commands::set_ignore_cursor_events,
            desktop_commands::start_dragging,
            desktop_commands::start_resize_dragging,
            desktop_commands::set_progress_bar,
            desktop_commands::print,
            desktop_commands::set_icon,
            desktop_commands::toggle_maximize,
            desktop_commands::internal_toggle_maximize,
            #[cfg(any(debug_assertions, feature = "devtools"))]
            desktop_commands::internal_toggle_devtools,
            desktop_commands::on_mousemove,
            desktop_commands::on_mousedown,
          ]);
        handler(invoke)
      }
      #[cfg(mobile)]
      {
        invoke.resolver.reject("Window API not available on mobile");
        true
      }
    })
    .build()
}
