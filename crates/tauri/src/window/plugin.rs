// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The tauri plugin to create and manipulate windows from JS.

use crate::{
  Runtime, Window,
  plugin::{Builder, TauriPlugin},
  sealed::ManagerBase,
};

fn get_window<R: Runtime>(window: Window<R>, label: Option<String>) -> crate::Result<Window<R>> {
  match label {
    Some(l) if !l.is_empty() => window
      .manager()
      .get_window(&l)
      .ok_or(crate::Error::WindowNotFound),
    _ => Ok(window),
  }
}
macro_rules! getter {
  ($cmd: ident, $ret: ty) => {
    #[command(root = "crate")]
    pub async fn $cmd<R: Runtime>(window: Window<R>, label: Option<String>) -> crate::Result<$ret> {
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

mod commands {
  use tauri_runtime::window::WindowSizeConstraints;

  use super::*;
  use crate::{
    AppHandle, Monitor, PhysicalPosition, PhysicalSize, Position, Size, Theme, Window, command,
    sealed::ManagerBase, utils::config::WindowConfig, window::Color, window::WindowBuilder,
  };

  #[command(root = "crate")]
  pub async fn get_all_windows<R: Runtime>(app: AppHandle<R>) -> Vec<String> {
    app.manager().windows().keys().cloned().collect()
  }

  #[command(root = "crate")]
  pub async fn create<R: Runtime>(window: Window<R>, options: WindowConfig) -> crate::Result<()> {
    WindowBuilder::from_config(&window, &options)?.build()?;
    Ok(())
  }

  getter!(scale_factor, f64);
  getter!(inner_position, PhysicalPosition<i32>);
  getter!(outer_position, PhysicalPosition<i32>);
  getter!(inner_size, PhysicalSize<u32>);
  getter!(outer_size, PhysicalSize<u32>);
  getter!(is_focused, bool);
  getter!(is_resizable, bool);
  getter!(is_visible, bool);
  getter!(is_enabled, bool);
  getter!(title, String);
  getter!(theme, Theme);
  #[cfg(target_os = "android")]
  getter!(activity_name, String);
  #[cfg(target_os = "ios")]
  getter!(scene_identifier, String);

  setter!(set_resizable, bool);
  setter!(set_title, &str);
  setter!(show);
  setter!(hide);
  setter!(close);
  setter!(destroy);
  setter!(set_content_protected, bool);
  setter!(set_size, Size);
  setter!(set_min_size, Option<Size>);
  setter!(set_max_size, Option<Size>);
  setter!(set_position, Position);
  setter!(set_focus);
  setter!(set_focusable, bool);
  setter!(set_background_color, Option<Color>);
  setter!(set_size_constraints, WindowSizeConstraints);
  setter!(set_theme, Option<Theme>);
  setter!(set_enabled, bool);

  getter!(current_monitor, Option<Monitor>);
  getter!(primary_monitor, Option<Monitor>);
  getter!(available_monitors, Vec<Monitor>);

  #[command(root = "crate")]
  pub async fn monitor_from_point<R: Runtime>(
    window: Window<R>,
    label: Option<String>,
    x: f64,
    y: f64,
  ) -> crate::Result<Option<Monitor>> {
    let window = get_window(window, label)?;
    window.monitor_from_point(x, y)
  }
}

#[cfg(desktop)]
mod desktop_commands {
  use tauri_runtime::ResizeDirection;
  use tauri_utils::TitleBarStyle;

  use super::*;
  use crate::{
    CursorIcon, Manager, PhysicalPosition, Position, UserAttentionType, Webview, command,
    utils::config::WindowEffectsConfig, window::ProgressBarState,
  };

  getter!(is_fullscreen, bool);
  getter!(is_minimized, bool);
  getter!(is_maximized, bool);
  getter!(is_decorated, bool);
  getter!(is_maximizable, bool);
  getter!(is_minimizable, bool);
  getter!(is_closable, bool);
  getter!(cursor_position, PhysicalPosition<f64>);
  getter!(is_always_on_top, bool);

  setter!(center);
  setter!(request_user_attention, Option<UserAttentionType>);
  setter!(set_maximizable, bool);
  setter!(set_minimizable, bool);
  setter!(set_closable, bool);
  setter!(maximize);
  setter!(unmaximize);
  setter!(minimize);
  setter!(unminimize);
  setter!(set_decorations, bool);
  setter!(set_shadow, bool);
  setter!(set_effects, Option<WindowEffectsConfig>);
  setter!(set_always_on_top, bool);
  setter!(set_always_on_bottom, bool);
  setter!(set_fullscreen, bool);
  setter!(set_simple_fullscreen, bool);
  setter!(set_skip_taskbar, bool);
  setter!(set_cursor_grab, bool);
  setter!(set_cursor_visible, bool);
  setter!(set_cursor_icon, CursorIcon);
  setter!(set_cursor_position, Position);
  setter!(set_ignore_cursor_events, bool);
  setter!(start_dragging);
  setter!(start_resize_dragging, ResizeDirection);
  setter!(set_progress_bar, ProgressBarState);
  setter!(set_badge_count, Option<i64>);
  #[cfg(target_os = "macos")]
  setter!(set_badge_label, Option<String>);
  setter!(set_visible_on_all_workspaces, bool);
  setter!(set_title_bar_style, TitleBarStyle);

  #[command(root = "crate")]
  #[cfg(target_os = "windows")]
  pub async fn set_overlay_icon<R: Runtime>(
    webview: Webview<R>,
    window: Window<R>,
    label: Option<String>,
    value: Option<crate::image::JsImage>,
  ) -> crate::Result<()> {
    let window = get_window(window, label)?;
    let resources_table = webview.resources_table();

    let value = match value {
      Some(value) => Some(value.into_img(&resources_table)?.as_ref().clone()),
      None => None,
    };

    window.set_overlay_icon(value)
  }

  #[command(root = "crate")]
  pub async fn set_icon<R: Runtime>(
    webview: Webview<R>,
    window: Window<R>,
    label: Option<String>,
    value: crate::image::JsImage,
  ) -> crate::Result<()> {
    let window = get_window(window, label)?;
    let resources_table = webview.resources_table();
    window.set_icon(value.into_img(&resources_table)?.as_ref().clone())
  }

  #[command(root = "crate")]
  pub async fn toggle_maximize<R: Runtime>(
    window: Window<R>,
    label: Option<String>,
  ) -> crate::Result<()> {
    let window = get_window(window, label)?;
    if window.is_maximized()? {
      window.unmaximize()?
    } else {
      window.maximize()?
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
      if window.is_maximized()? {
        window.unmaximize()?
      } else if window.is_maximizable()? {
        window.maximize()?
      };
    }
    Ok(())
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  use serialize_to_javascript::{DefaultTemplate, Template, default_template};

  let mut init_script = String::new();

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

  Builder::new("window")
    .js_init_script(init_script)
    .invoke_handler(crate::generate_handler![
      #![plugin(window)]
      commands::create,
      // getters
      commands::get_all_windows,
      commands::scale_factor,
      commands::inner_position,
      commands::outer_position,
      commands::inner_size,
      commands::outer_size,
      commands::is_focused,
      commands::is_resizable,
      commands::is_visible,
      commands::is_enabled,
      commands::title,
      commands::theme,
      #[cfg(target_os = "android")]
      commands::activity_name,
      #[cfg(target_os = "ios")]
      commands::scene_identifier,

      commands::set_resizable,
      commands::set_title,
      commands::show,
      commands::hide,
      commands::close,
      commands::destroy,
      commands::set_content_protected,
      commands::set_size,
      commands::set_min_size,
      commands::set_max_size,
      commands::set_position,
      commands::set_size_constraints,
      commands::set_focus,
      commands::set_focusable,
      commands::set_enabled,
      commands::set_background_color,
      commands::set_theme,
      commands::current_monitor,
      commands::primary_monitor,
      commands::monitor_from_point,
      commands::available_monitors,

      #[cfg(desktop)] desktop_commands::is_fullscreen,
      #[cfg(desktop)] desktop_commands::is_minimized,
      #[cfg(desktop)] desktop_commands::is_maximized,
      #[cfg(desktop)] desktop_commands::is_decorated,
      #[cfg(desktop)] desktop_commands::is_maximizable,
      #[cfg(desktop)] desktop_commands::is_minimizable,
      #[cfg(desktop)] desktop_commands::is_closable,

      #[cfg(desktop)] desktop_commands::cursor_position,
      #[cfg(desktop)] desktop_commands::is_always_on_top,
      // setters
      #[cfg(desktop)] desktop_commands::center,
      #[cfg(desktop)] desktop_commands::request_user_attention,
      #[cfg(desktop)] desktop_commands::set_maximizable,
      #[cfg(desktop)] desktop_commands::set_minimizable,
      #[cfg(desktop)] desktop_commands::set_closable,
      #[cfg(desktop)] desktop_commands::maximize,
      #[cfg(desktop)] desktop_commands::unmaximize,
      #[cfg(desktop)] desktop_commands::minimize,
      #[cfg(desktop)] desktop_commands::unminimize,
      #[cfg(desktop)] desktop_commands::set_decorations,
      #[cfg(desktop)] desktop_commands::set_shadow,
      #[cfg(desktop)] desktop_commands::set_effects,
      #[cfg(desktop)] desktop_commands::set_always_on_top,
      #[cfg(desktop)] desktop_commands::set_always_on_bottom,
      #[cfg(desktop)] desktop_commands::set_fullscreen,
      #[cfg(desktop)] desktop_commands::set_simple_fullscreen,
      #[cfg(desktop)] desktop_commands::set_skip_taskbar,
      #[cfg(desktop)] desktop_commands::set_cursor_grab,
      #[cfg(desktop)] desktop_commands::set_cursor_visible,
      #[cfg(desktop)] desktop_commands::set_cursor_icon,
      #[cfg(desktop)] desktop_commands::set_cursor_position,
      #[cfg(desktop)] desktop_commands::set_ignore_cursor_events,
      #[cfg(desktop)] desktop_commands::start_dragging,
      #[cfg(desktop)] desktop_commands::start_resize_dragging,
      #[cfg(desktop)] desktop_commands::set_badge_count,
      #[cfg(target_os = "macos")]
      #[cfg(desktop)] desktop_commands::set_badge_label,
      #[cfg(desktop)] desktop_commands::set_progress_bar,
      #[cfg(target_os = "windows")]
      #[cfg(desktop)] desktop_commands::set_overlay_icon,
      #[cfg(desktop)] desktop_commands::set_icon,
      #[cfg(desktop)] desktop_commands::set_visible_on_all_workspaces,
      #[cfg(desktop)] desktop_commands::set_title_bar_style,
      #[cfg(desktop)] desktop_commands::toggle_maximize,
      #[cfg(desktop)] desktop_commands::internal_toggle_maximize,
    ])
    .build()
}
