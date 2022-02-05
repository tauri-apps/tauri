// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{InvokeContext, InvokeResponse};
#[cfg(window_create)]
use crate::runtime::{webview::WindowBuilder, Dispatch};
use crate::{
  runtime::{
    window::dpi::{Position, Size},
    Runtime, UserAttentionType,
  },
  utils::config::WindowConfig,
  Manager,
};
use serde::Deserialize;
use tauri_macros::{module_command_handler, CommandModule};

use crate::runtime::Icon;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum IconDto {
  File(PathBuf),
  Raw(Vec<u8>),
}

impl From<IconDto> for Icon {
  fn from(icon: IconDto) -> Self {
    match icon {
      IconDto::File(path) => Self::File(path),
      IconDto::Raw(raw) => Self::Raw(raw),
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
  // Setters
  Center,
  RequestUserAttention(Option<UserAttentionType>),
  SetResizable(bool),
  SetTitle(String),
  Maximize,
  Unmaximize,
  ToggleMaximize,
  Minimize,
  Unminimize,
  Show,
  Hide,
  Close,
  SetDecorations(bool),
  #[serde(rename_all = "camelCase")]
  SetAlwaysOnTop(bool),
  SetSize(Size),
  SetMinSize(Option<Size>),
  SetMaxSize(Option<Size>),
  SetPosition(Position),
  SetFullscreen(bool),
  SetFocus,
  SetIcon {
    icon: IconDto,
  },
  SetSkipTaskbar(bool),
  StartDragging,
  Print,
  // internals
  #[serde(rename = "__toggleMaximize")]
  InternalToggleMaximize,
}

impl WindowManagerCmd {
  fn into_allowlist_error(self) -> crate::Error {
    match self {
      Self::Center => crate::Error::ApiNotAllowlisted("window > center".to_string()),
      Self::RequestUserAttention(_) => {
        crate::Error::ApiNotAllowlisted("window > requestUserAttention".to_string())
      }
      Self::SetResizable(_) => crate::Error::ApiNotAllowlisted("window > setResizable".to_string()),
      Self::SetTitle(_) => crate::Error::ApiNotAllowlisted("window > setTitle".to_string()),
      Self::Maximize => crate::Error::ApiNotAllowlisted("window > maximize".to_string()),
      Self::Unmaximize => crate::Error::ApiNotAllowlisted("window > unmaximize".to_string()),
      Self::ToggleMaximize => {
        crate::Error::ApiNotAllowlisted("window > maximize and window > unmaximize".to_string())
      }
      Self::Minimize => crate::Error::ApiNotAllowlisted("window > minimize".to_string()),
      Self::Unminimize => crate::Error::ApiNotAllowlisted("window > unminimize".to_string()),
      Self::Show => crate::Error::ApiNotAllowlisted("window > show".to_string()),
      Self::Hide => crate::Error::ApiNotAllowlisted("window > hide".to_string()),
      Self::Close => crate::Error::ApiNotAllowlisted("window > close".to_string()),
      Self::SetDecorations(_) => {
        crate::Error::ApiNotAllowlisted("window > setDecorations".to_string())
      }
      Self::SetAlwaysOnTop(_) => {
        crate::Error::ApiNotAllowlisted("window > setAlwaysOnTop".to_string())
      }
      Self::SetSize(_) => crate::Error::ApiNotAllowlisted("window > setSize".to_string()),
      Self::SetMinSize(_) => crate::Error::ApiNotAllowlisted("window > setMinSize".to_string()),
      Self::SetMaxSize(_) => crate::Error::ApiNotAllowlisted("window > setMaxSize".to_string()),
      Self::SetPosition(_) => crate::Error::ApiNotAllowlisted("window > setPosition".to_string()),
      Self::SetFullscreen(_) => {
        crate::Error::ApiNotAllowlisted("window > setFullscreen".to_string())
      }
      Self::SetIcon { .. } => crate::Error::ApiNotAllowlisted("window > setIcon".to_string()),
      Self::SetSkipTaskbar(_) => {
        crate::Error::ApiNotAllowlisted("window > setSkipTaskbar".to_string())
      }
      Self::StartDragging => crate::Error::ApiNotAllowlisted("window > startDragging".to_string()),
      Self::Print => crate::Error::ApiNotAllowlisted("window > print".to_string()),
      Self::InternalToggleMaximize => {
        crate::Error::ApiNotAllowlisted("window > maximize and window > unmaximize".to_string())
      }
      _ => crate::Error::ApiNotAllowlisted("window > all".to_string()),
    }
  }
}

/// The API descriptor.
#[derive(Deserialize, CommandModule)]
#[cmd(async)]
#[serde(tag = "cmd", content = "data", rename_all = "camelCase")]
pub enum Cmd {
  CreateWebview {
    options: Box<WindowConfig>,
  },
  Manage {
    label: Option<String>,
    cmd: WindowManagerCmd,
  },
}

impl Cmd {
  #[module_command_handler(window_create, "window > create")]
  async fn create_webview<R: Runtime>(
    context: InvokeContext<R>,
    options: Box<WindowConfig>,
  ) -> crate::Result<()> {
    let mut window = context.window;
    let label = options.label.clone();
    let url = options.url.clone();

    window.create_window(label, url, |_, webview_attributes| {
      (
        <<R::Dispatcher as Dispatch>::WindowBuilder>::with_config(*options),
        webview_attributes,
      )
    })?;

    Ok(())
  }

  async fn manage<R: Runtime>(
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
      #[allow(unreachable_patterns)]
      _ => return Err(cmd.into_allowlist_error()),
    }
    #[allow(unreachable_code)]
    Ok(().into())
  }
}
