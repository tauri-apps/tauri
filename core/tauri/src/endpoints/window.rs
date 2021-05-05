// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(window_create)]
use crate::Manager;
use crate::{
  api::config::WindowConfig,
  endpoints::InvokeResponse,
  runtime::{Position, Size},
  Params, Window,
};
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
  Resize(Size),
  SetMinSize(Option<Size>),
  SetMaxSize(Option<Size>),
  SetPosition(Position),
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
        Self::Resize(size) => window.resize(size)?,
        Self::SetMinSize(size) => window.set_min_size(size)?,
        Self::SetMaxSize(size) => window.set_max_size(size)?,
        Self::SetPosition(position) => window.set_position(position)?,
        Self::SetFullscreen { fullscreen } => window.set_fullscreen(fullscreen)?,
        Self::SetIcon { icon } => window.set_icon(icon.into())?,
        Self::StartDragging => window.start_dragging()?,
      }
      Ok(().into())
    }
  }
}
