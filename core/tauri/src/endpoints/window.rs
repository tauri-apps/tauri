// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused_imports)]

use super::{InvokeContext, InvokeResponse};
#[cfg(window_create)]
use crate::runtime::{webview::WindowBuilder, Dispatch};
use crate::{
  runtime::{
    window::dpi::{Position, Size},
    UserAttentionType,
  },
  utils::config::WindowConfig,
  CursorIcon, Icon, Manager, Runtime,
};
use serde::Deserialize;
use tauri_macros::{command_enum, module_command_handler, CommandModule};

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

/// Window management API descriptor.
#[derive(Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum WindowManagerCmd {
  // Getters
  ScaleFactor,
  InnerPosition,
  OuterPosition,
  InnerSize,
  OuterSize,
  IsFullscreen,
  IsMaximized,
  IsDecorated,
  IsResizable,
  IsVisible,
  CurrentMonitor,
  PrimaryMonitor,
  AvailableMonitors,
  Theme,
  // Setters
  #[cfg(window_center)]
  Center,
  #[cfg(window_request_user_attention)]
  RequestUserAttention(Option<UserAttentionType>),
  #[cfg(window_set_resizable)]
  SetResizable(bool),
  #[cfg(window_set_title)]
  SetTitle(String),
  #[cfg(window_maximize)]
  Maximize,
  #[cfg(window_unmaximize)]
  Unmaximize,
  #[cfg(all(window_maximize, window_unmaximize))]
  ToggleMaximize,
  #[cfg(window_minimize)]
  Minimize,
  #[cfg(window_unminimize)]
  Unminimize,
  #[cfg(window_show)]
  Show,
  #[cfg(window_hide)]
  Hide,
  #[cfg(window_close)]
  Close,
  #[cfg(window_set_decorations)]
  SetDecorations(bool),
  #[cfg(window_set_always_on_top)]
  #[serde(rename_all = "camelCase")]
  SetAlwaysOnTop(bool),
  #[cfg(window_set_size)]
  SetSize(Size),
  #[cfg(window_set_min_size)]
  SetMinSize(Option<Size>),
  #[cfg(window_set_max_size)]
  SetMaxSize(Option<Size>),
  #[cfg(window_set_position)]
  SetPosition(Position),
  #[cfg(window_set_fullscreen)]
  SetFullscreen(bool),
  #[cfg(window_set_focus)]
  SetFocus,
  #[cfg(window_set_icon)]
  SetIcon {
    icon: IconDto,
  },
  #[cfg(window_set_skip_taskbar)]
  SetSkipTaskbar(bool),
  #[cfg(window_set_cursor_grab)]
  SetCursorGrab(bool),
  #[cfg(window_set_cursor_visible)]
  SetCursorVisible(bool),
  #[cfg(window_set_cursor_icon)]
  SetCursorIcon(CursorIcon),
  #[cfg(window_set_cursor_position)]
  SetCursorPosition(Position),
  #[cfg(window_start_dragging)]
  StartDragging,
  #[cfg(window_print)]
  Print,
  // internals
  #[cfg(all(window_maximize, window_unmaximize))]
  #[serde(rename = "__toggleMaximize")]
  InternalToggleMaximize,
  #[cfg(any(debug_assertions, feature = "devtools"))]
  #[serde(rename = "__toggleDevtools")]
  InternalToggleDevtools,
}

pub fn into_allowlist_error(variant: &str) -> crate::Error {
  match variant {
    "center" => crate::Error::ApiNotAllowlisted("window > center".to_string()),
    "requestUserAttention" => {
      crate::Error::ApiNotAllowlisted("window > requestUserAttention".to_string())
    }
    "setResizable" => crate::Error::ApiNotAllowlisted("window > setResizable".to_string()),
    "setTitle" => crate::Error::ApiNotAllowlisted("window > setTitle".to_string()),
    "maximize" => crate::Error::ApiNotAllowlisted("window > maximize".to_string()),
    "unmaximize" => crate::Error::ApiNotAllowlisted("window > unmaximize".to_string()),
    "toggleMaximize" => {
      crate::Error::ApiNotAllowlisted("window > maximize and window > unmaximize".to_string())
    }
    "minimize" => crate::Error::ApiNotAllowlisted("window > minimize".to_string()),
    "nnminimize" => crate::Error::ApiNotAllowlisted("window > unminimize".to_string()),
    "show" => crate::Error::ApiNotAllowlisted("window > show".to_string()),
    "hide" => crate::Error::ApiNotAllowlisted("window > hide".to_string()),
    "close" => crate::Error::ApiNotAllowlisted("window > close".to_string()),
    "setDecorations" => crate::Error::ApiNotAllowlisted("window > setDecorations".to_string()),
    "setAlwaysOnTop" => crate::Error::ApiNotAllowlisted("window > setAlwaysOnTop".to_string()),
    "setSize" => crate::Error::ApiNotAllowlisted("window > setSize".to_string()),
    "setMinSize" => crate::Error::ApiNotAllowlisted("window > setMinSize".to_string()),
    "setMaxSize" => crate::Error::ApiNotAllowlisted("window > setMaxSize".to_string()),
    "setPosition" => crate::Error::ApiNotAllowlisted("window > setPosition".to_string()),
    "setFullscreen" => crate::Error::ApiNotAllowlisted("window > setFullscreen".to_string()),
    "setIcon" => crate::Error::ApiNotAllowlisted("window > setIcon".to_string()),
    "setSkipTaskbar" => crate::Error::ApiNotAllowlisted("window > setSkipTaskbar".to_string()),
    "setCursorGrab" => crate::Error::ApiNotAllowlisted("window > setCursorGrab".to_string()),
    "setCursorVisible" => crate::Error::ApiNotAllowlisted("window > setCursorVisible".to_string()),
    "setCursorIcon" => crate::Error::ApiNotAllowlisted("window > setCursorIcon".to_string()),
    "setCursorPosition" => {
      crate::Error::ApiNotAllowlisted("window > setCursorPosition".to_string())
    }
    "startDragging" => crate::Error::ApiNotAllowlisted("window > startDragging".to_string()),
    "print" => crate::Error::ApiNotAllowlisted("window > print".to_string()),
    "internalToggleMaximize" => {
      crate::Error::ApiNotAllowlisted("window > maximize and window > unmaximize".to_string())
    }
    _ => crate::Error::ApiNotAllowlisted("window".to_string()),
  }
}

/// The API descriptor.
#[command_enum]
#[derive(Deserialize, CommandModule)]
#[cmd(async)]
#[serde(tag = "cmd", content = "data", rename_all = "camelCase")]
pub enum Cmd {
  #[cmd(window_create, "window > create")]
  CreateWebview { options: Box<WindowConfig> },
  Manage {
    label: Option<String>,
    cmd: WindowManagerCmd,
  },
}

impl Cmd {
  #[module_command_handler(window_create)]
  async fn create_webview<R: Runtime>(
    context: InvokeContext<R>,
    options: Box<WindowConfig>,
  ) -> super::Result<()> {
    let label = options.label.clone();
    let url = options.url.clone();
    let file_drop_enabled = options.file_drop_enabled;

    let mut builder = crate::window::Window::builder(&context.window, label, url);
    if !file_drop_enabled {
      builder = builder.disable_file_drop_handler();
    }

    builder.window_builder =
      <<R::Dispatcher as Dispatch<crate::EventLoopMessage>>::WindowBuilder>::with_config(*options);
    builder.build().map_err(crate::error::into_anyhow)?;

    Ok(())
  }

  async fn manage<R: Runtime>(
    context: InvokeContext<R>,
    label: Option<String>,
    cmd: WindowManagerCmd,
  ) -> super::Result<InvokeResponse> {
    Self::_manage(context, label, cmd)
      .await
      .map_err(crate::error::into_anyhow)
  }

  async fn _manage<R: Runtime>(
    context: InvokeContext<R>,
    label: Option<String>,
    cmd: WindowManagerCmd,
  ) -> crate::Result<InvokeResponse> {
    let window = match label {
      Some(l) if !l.is_empty() => context
        .window
        .get_window(&l)
        .ok_or(crate::Error::WebviewNotFound)?,
      _ => context.window,
    };
    match cmd {
      // Getters
      WindowManagerCmd::ScaleFactor => return Ok(window.scale_factor()?.into()),
      WindowManagerCmd::InnerPosition => return Ok(window.inner_position()?.into()),
      WindowManagerCmd::OuterPosition => return Ok(window.outer_position()?.into()),
      WindowManagerCmd::InnerSize => return Ok(window.inner_size()?.into()),
      WindowManagerCmd::OuterSize => return Ok(window.outer_size()?.into()),
      WindowManagerCmd::IsFullscreen => return Ok(window.is_fullscreen()?.into()),
      WindowManagerCmd::IsMaximized => return Ok(window.is_maximized()?.into()),
      WindowManagerCmd::IsDecorated => return Ok(window.is_decorated()?.into()),
      WindowManagerCmd::IsResizable => return Ok(window.is_resizable()?.into()),
      WindowManagerCmd::IsVisible => return Ok(window.is_visible()?.into()),
      WindowManagerCmd::CurrentMonitor => return Ok(window.current_monitor()?.into()),
      WindowManagerCmd::PrimaryMonitor => return Ok(window.primary_monitor()?.into()),
      WindowManagerCmd::AvailableMonitors => return Ok(window.available_monitors()?.into()),
      WindowManagerCmd::Theme => return Ok(window.theme()?.into()),
      // Setters
      #[cfg(window_center)]
      WindowManagerCmd::Center => window.center()?,
      #[cfg(window_request_user_attention)]
      WindowManagerCmd::RequestUserAttention(request_type) => {
        window.request_user_attention(request_type)?
      }
      #[cfg(window_set_resizable)]
      WindowManagerCmd::SetResizable(resizable) => window.set_resizable(resizable)?,
      #[cfg(window_set_title)]
      WindowManagerCmd::SetTitle(title) => window.set_title(&title)?,
      #[cfg(window_maximize)]
      WindowManagerCmd::Maximize => window.maximize()?,
      #[cfg(window_unmaximize)]
      WindowManagerCmd::Unmaximize => window.unmaximize()?,
      #[cfg(all(window_maximize, window_unmaximize))]
      WindowManagerCmd::ToggleMaximize => match window.is_maximized()? {
        true => window.unmaximize()?,
        false => window.maximize()?,
      },
      #[cfg(window_minimize)]
      WindowManagerCmd::Minimize => window.minimize()?,
      #[cfg(window_unminimize)]
      WindowManagerCmd::Unminimize => window.unminimize()?,
      #[cfg(window_show)]
      WindowManagerCmd::Show => window.show()?,
      #[cfg(window_hide)]
      WindowManagerCmd::Hide => window.hide()?,
      #[cfg(window_close)]
      WindowManagerCmd::Close => window.close()?,
      #[cfg(window_set_decorations)]
      WindowManagerCmd::SetDecorations(decorations) => window.set_decorations(decorations)?,
      #[cfg(window_set_always_on_top)]
      WindowManagerCmd::SetAlwaysOnTop(always_on_top) => window.set_always_on_top(always_on_top)?,
      #[cfg(window_set_size)]
      WindowManagerCmd::SetSize(size) => window.set_size(size)?,
      #[cfg(window_set_min_size)]
      WindowManagerCmd::SetMinSize(size) => window.set_min_size(size)?,
      #[cfg(window_set_max_size)]
      WindowManagerCmd::SetMaxSize(size) => window.set_max_size(size)?,
      #[cfg(window_set_position)]
      WindowManagerCmd::SetPosition(position) => window.set_position(position)?,
      #[cfg(window_set_fullscreen)]
      WindowManagerCmd::SetFullscreen(fullscreen) => window.set_fullscreen(fullscreen)?,
      #[cfg(window_set_focus)]
      WindowManagerCmd::SetFocus => window.set_focus()?,
      #[cfg(window_set_icon)]
      WindowManagerCmd::SetIcon { icon } => window.set_icon(icon.into())?,
      #[cfg(window_set_skip_taskbar)]
      WindowManagerCmd::SetSkipTaskbar(skip) => window.set_skip_taskbar(skip)?,
      #[cfg(window_set_cursor_grab)]
      WindowManagerCmd::SetCursorGrab(grab) => window.set_cursor_grab(grab)?,
      #[cfg(window_set_cursor_visible)]
      WindowManagerCmd::SetCursorVisible(visible) => window.set_cursor_visible(visible)?,
      #[cfg(window_set_cursor_icon)]
      WindowManagerCmd::SetCursorIcon(icon) => window.set_cursor_icon(icon)?,
      #[cfg(window_set_cursor_position)]
      WindowManagerCmd::SetCursorPosition(position) => window.set_cursor_position(position)?,
      #[cfg(window_start_dragging)]
      WindowManagerCmd::StartDragging => window.start_dragging()?,
      #[cfg(window_print)]
      WindowManagerCmd::Print => window.print()?,
      // internals
      #[cfg(all(window_maximize, window_unmaximize))]
      WindowManagerCmd::InternalToggleMaximize => {
        if window.is_resizable()? {
          match window.is_maximized()? {
            true => window.unmaximize()?,
            false => window.maximize()?,
          }
        }
      }
      #[cfg(any(debug_assertions, feature = "devtools"))]
      WindowManagerCmd::InternalToggleDevtools => {
        if window.is_devtools_open() {
          window.close_devtools();
        } else {
          window.open_devtools();
        }
      }
    }
    #[allow(unreachable_code)]
    Ok(().into())
  }
}
