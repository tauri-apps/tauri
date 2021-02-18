use crate::app::{ApplicationExt, Icon, InvokeResponse};
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
  SetTransparent {
    transparent: bool,
  },
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

#[cfg(create_window)]
#[derive(Clone, serde::Serialize)]
struct WindowCreatedEvent {
  label: String,
}

impl Cmd {
  pub async fn run<A: ApplicationExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<A>,
  ) -> crate::Result<InvokeResponse> {
    if cfg!(not(window)) {
      Err(crate::Error::ApiNotAllowlisted("setTitle".to_string()))
    } else {
      let current_webview = webview_manager.current_webview().await?;
      match self {
        Self::CreateWebview { options } => {
          #[cfg(not(create_window))]
          return Err(crate::Error::ApiNotAllowlisted("createWindow".to_string()));
          #[cfg(create_window)]
          {
            let label = options.label.to_string();
            webview_manager
              .create_webview(label.to_string(), options.url.clone(), |_| {
                Ok(crate::app::webview::WindowConfig(options).into())
              })
              .await?;
            webview_manager
              .emit_except(
                label.to_string(),
                "tauri://window-created",
                Some(WindowCreatedEvent { label }),
              )
              .await?;
          }
        }
        Self::SetResizable { resizable } => current_webview.set_resizable(resizable)?,
        Self::SetTitle { title } => current_webview.set_title(&title)?,
        Self::Maximize => current_webview.maximize()?,
        Self::Unmaximize => current_webview.unmaximize()?,
        Self::Minimize => current_webview.minimize()?,
        Self::Unminimize => current_webview.unminimize()?,
        Self::Show => current_webview.show()?,
        Self::Hide => current_webview.hide()?,
        Self::SetTransparent { transparent } => current_webview.set_transparent(transparent)?,
        Self::SetDecorations { decorations } => current_webview.set_decorations(decorations)?,
        Self::SetAlwaysOnTop { always_on_top } => {
          current_webview.set_always_on_top(always_on_top)?
        }
        Self::SetWidth { width } => current_webview.set_width(width)?,
        Self::SetHeight { height } => current_webview.set_height(height)?,
        Self::Resize { width, height } => current_webview.resize(width, height)?,
        Self::SetMinSize {
          min_width,
          min_height,
        } => current_webview.set_min_size(min_width, min_height)?,
        Self::SetMaxSize {
          max_width,
          max_height,
        } => current_webview.set_max_size(max_width, max_height)?,
        Self::SetX { x } => current_webview.set_x(x)?,
        Self::SetY { y } => current_webview.set_y(y)?,
        Self::SetPosition { x, y } => current_webview.set_position(x, y)?,
        Self::SetFullscreen { fullscreen } => current_webview.set_fullscreen(fullscreen)?,
        Self::SetIcon { icon } => current_webview.set_icon(icon.into())?,
      }
      Ok(().into())
    }
  }
}
