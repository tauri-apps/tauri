// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(window_create)]
use crate::Manager;
use crate::{api::config::WindowConfig, endpoints::InvokeResponse, Params, Window};
use serde::Deserialize;

use crate::Icon;
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

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  CreateWebview {
    options: WindowConfig,
  },
  SetResizable {
    resizable: bool,
  },
  SetTitle {
    title: String,
  },
  Maximize,
  Unmaximize,
  Minimize,
  Unminimize,
  Show,
  Hide,
  Close,
  SetDecorations {
    decorations: bool,
  },
  #[serde(rename_all = "camelCase")]
  SetAlwaysOnTop {
    always_on_top: bool,
  },
  SetWidth {
    width: f64,
  },
  SetHeight {
    height: f64,
  },
  Resize {
    width: f64,
    height: f64,
  },
  #[serde(rename_all = "camelCase")]
  SetMinSize {
    min_width: f64,
    min_height: f64,
  },
  #[serde(rename_all = "camelCase")]
  SetMaxSize {
    max_width: f64,
    max_height: f64,
  },
  SetX {
    x: f64,
  },
  SetY {
    y: f64,
  },
  SetPosition {
    x: f64,
    y: f64,
  },
  SetFullscreen {
    fullscreen: bool,
  },
  SetIcon {
    icon: IconDto,
  },
  StartDragging,
}

#[cfg(window_create)]
#[derive(Clone, serde::Serialize)]
struct WindowCreatedEvent {
  label: String,
}

impl Cmd {
  #[allow(dead_code)]
  pub async fn run<P: Params>(self, window: Window<P>) -> crate::Result<InvokeResponse> {
    if cfg!(not(window_all)) {
      Err(crate::Error::ApiNotAllowlisted("window > all".to_string()))
    } else {
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
          // Panic if the user's `Tag` type decided to return an error while parsing.
          let label: P::Label = options.label.parse().unwrap_or_else(|_| {
            panic!(
              "Window module received unknown window label: {}",
              options.label
            )
          });

          let url = options.url.clone();
          let pending = crate::runtime::window::PendingWindow::with_config(
            options,
            crate::runtime::webview::WebviewAttributes::new(url),
            label.clone(),
          );
          window.create_window(pending)?.emit_others(
            &crate::runtime::manager::tauri_event::<P::Event>("tauri://window-created"),
            Some(WindowCreatedEvent {
              label: label.to_string(),
            }),
          )?;
        }

        Self::SetResizable { resizable } => window.set_resizable(resizable)?,
        Self::SetTitle { title } => window.set_title(&title)?,
        Self::Maximize => window.maximize()?,
        Self::Unmaximize => window.unmaximize()?,
        Self::Minimize => window.minimize()?,
        Self::Unminimize => window.unminimize()?,
        Self::Show => window.show()?,
        Self::Hide => window.hide()?,
        Self::Close => window.close()?,
        Self::SetDecorations { decorations } => window.set_decorations(decorations)?,
        Self::SetAlwaysOnTop { always_on_top } => window.set_always_on_top(always_on_top)?,
        Self::SetWidth { width } => window.set_width(width)?,
        Self::SetHeight { height } => window.set_height(height)?,
        Self::Resize { width, height } => window.resize(width, height)?,
        Self::SetMinSize {
          min_width,
          min_height,
        } => window.set_min_size(min_width, min_height)?,
        Self::SetMaxSize {
          max_width,
          max_height,
        } => window.set_max_size(max_width, max_height)?,
        Self::SetX { x } => window.set_x(x)?,
        Self::SetY { y } => window.set_y(y)?,
        Self::SetPosition { x, y } => window.set_position(x, y)?,
        Self::SetFullscreen { fullscreen } => window.set_fullscreen(fullscreen)?,
        Self::SetIcon { icon } => window.set_icon(icon.into())?,
        Self::StartDragging => window.start_dragging()?,
      }
      Ok(().into())
    }
  }
}
