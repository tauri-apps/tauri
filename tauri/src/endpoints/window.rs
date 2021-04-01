use super::InvokeResponse;
use crate::{
  app::{webview::WindowConfig, Icon, Managed},
  Manager, PendingWindow, Window,
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum IconDto {
  File(String),
  Raw(Vec<u8>),
}

impl Into<Icon> for IconDto {
  fn into(self) -> Icon {
    match self {
      Self::File(path) => Icon::File(path),
      Self::Raw(raw) => Icon::Raw(raw),
    }
  }
}

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  CreateWebview {
    options: crate::api::config::WindowConfig,
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
}

#[cfg(window_create)]
#[derive(Clone, serde::Serialize)]
struct WindowCreatedEvent {
  label: String,
}

impl Cmd {
  pub async fn run<M: Manager>(self, mut window: Window<M>) -> crate::Result<InvokeResponse> {
    if cfg!(not(window_all)) {
      Err(crate::Error::ApiNotAllowlisted("window > all".to_string()))
    } else {
      match self {
        Self::CreateWebview { options } => {
          #[cfg(not(window_create))]
          return Err(crate::Error::ApiNotAllowlisted(
            "window > create".to_string(),
          ));
          #[cfg(window_create)]
          {
            // todo: how to handle this?
            let label: M::Label = options
              .label
              .parse()
              .unwrap_or_else(|_| panic!("todo: label parsing"));

            // todo: how to handle this?
            let event: M::Event = "tauri://window-created"
              .parse()
              .unwrap_or_else(|_| panic!("todo: event parsing"));

            let url = options.url.clone();
            let pending = PendingWindow::new(WindowConfig(options), label.clone(), url);
            window.create_window(pending)?.emit_others(
              event,
              Some(WindowCreatedEvent {
                label: label.to_string(),
              }),
            )?;
          }
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
      }
      Ok(().into())
    }
  }
}
