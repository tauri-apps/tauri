// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(window_create)]
use crate::runtime::{webview::WindowBuilder, Dispatch};
use crate::{
  api::config::WindowConfig,
  endpoints::InvokeResponse,
  runtime::{
    window::dpi::{Position, Size},
    Runtime, UserAttentionType,
  },
  Manager, Window,
};
use serde::Deserialize;

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
}

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", content = "data", rename_all = "camelCase")]
pub enum Cmd {
  CreateWebview {
    options: WindowConfig,
  },
  Manage {
    label: Option<String>,
    cmd: WindowManagerCmd,
  },
}

#[cfg(window_create)]
#[derive(Clone, serde::Serialize)]
struct WindowCreatedEvent {
  label: String,
}

impl Cmd {
  #[allow(dead_code)]
  pub async fn run<R: Runtime>(self, window: Window<R>) -> crate::Result<InvokeResponse> {
    match self {
      #[cfg(not(window_create))]
      Self::CreateWebview { .. } => {
        return Err(crate::Error::ApiNotAllowlisted(
          "window > create".to_string(),
        ));
      }
      #[cfg(window_create)]
      Self::CreateWebview { options } => {
        let mut window = window;
        let label = options.label.clone();
        let url = options.url.clone();

        window
          .create_window(label.clone(), url, |_, webview_attributes| {
            (
              <<R::Dispatcher as Dispatch>::WindowBuilder>::with_config(options),
              webview_attributes,
            )
          })?
          .emit_others("tauri://window-created", Some(WindowCreatedEvent { label }))?;
      }
      Self::Manage { label, cmd } => {
        let window = if let Some(l) = label {
          window.get_window(&l).ok_or(crate::Error::WebviewNotFound)?
        } else {
          window
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
          WindowManagerCmd::Center => window.center()?,
          WindowManagerCmd::RequestUserAttention(request_type) => {
            window.request_user_attention(request_type)?
          }
          WindowManagerCmd::SetResizable(resizable) => window.set_resizable(resizable)?,
          WindowManagerCmd::SetTitle(title) => window.set_title(&title)?,
          WindowManagerCmd::Maximize => window.maximize()?,
          WindowManagerCmd::Unmaximize => window.unmaximize()?,
          WindowManagerCmd::ToggleMaximize => match window.is_maximized()? {
            true => window.unmaximize()?,
            false => window.maximize()?,
          },
          WindowManagerCmd::Minimize => window.minimize()?,
          WindowManagerCmd::Unminimize => window.unminimize()?,
          WindowManagerCmd::Show => window.show()?,
          WindowManagerCmd::Hide => window.hide()?,
          WindowManagerCmd::Close => window.close()?,
          WindowManagerCmd::SetDecorations(decorations) => window.set_decorations(decorations)?,
          WindowManagerCmd::SetAlwaysOnTop(always_on_top) => {
            window.set_always_on_top(always_on_top)?
          }
          WindowManagerCmd::SetSize(size) => window.set_size(size)?,
          WindowManagerCmd::SetMinSize(size) => window.set_min_size(size)?,
          WindowManagerCmd::SetMaxSize(size) => window.set_max_size(size)?,
          WindowManagerCmd::SetPosition(position) => window.set_position(position)?,
          WindowManagerCmd::SetFullscreen(fullscreen) => window.set_fullscreen(fullscreen)?,
          WindowManagerCmd::SetFocus => window.set_focus()?,
          WindowManagerCmd::SetIcon { icon } => window.set_icon(icon.into())?,
          WindowManagerCmd::SetSkipTaskbar(skip) => window.set_skip_taskbar(skip)?,
          WindowManagerCmd::StartDragging => window.start_dragging()?,
          WindowManagerCmd::Print => window.print()?,
        }
      }
    }
    Ok(().into())
  }
}
