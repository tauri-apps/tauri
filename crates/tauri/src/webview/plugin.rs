// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The tauri plugin to create and manipulate windows from JS.

use crate::{
  AppHandle, Runtime, WebviewWindowBuilder, command,
  plugin::{Builder, TauriPlugin},
  sealed::ManagerBase,
  utils::config::WindowConfig,
};

#[derive(serde::Serialize)]
struct WebviewRef {
  window_label: String,
  label: String,
}

#[command(root = "crate")]
async fn get_all_webviews<R: Runtime>(app: AppHandle<R>) -> Vec<WebviewRef> {
  app
    .manager()
    .webviews()
    .values()
    .map(|webview| WebviewRef {
      window_label: webview.window_ref().label().into(),
      label: webview.label().into(),
    })
    .collect()
}

#[command(root = "crate")]
async fn create_webview_window<R: Runtime>(
  app: AppHandle<R>,
  options: WindowConfig,
) -> crate::Result<()> {
  WebviewWindowBuilder::from_config(&app, &options)?.build()?;
  Ok(())
}

#[cfg(desktop)]
mod desktop_commands {
  use super::*;
  use crate::{
    Webview, command,
    runtime::dpi::{Position, Size},
    utils::config::Color,
  };

  fn get_webview<R: Runtime>(
    webview: Webview<R>,
    label: Option<String>,
  ) -> crate::Result<Webview<R>> {
    match label {
      Some(l) if !l.is_empty() => webview
        .manager()
        .get_webview(&l)
        .ok_or(crate::Error::WebviewNotFound),
      _ => Ok(webview),
    }
  }

  macro_rules! getter {
    ($cmd: ident, $ret: ty) => {
      getter!($cmd, $cmd, $ret)
    };
    ($fn: ident, $cmd: ident, $ret: ty) => {
      #[command(root = "crate")]
      pub async fn $fn<R: Runtime>(
        webview: Webview<R>,
        label: Option<String>,
      ) -> crate::Result<$ret> {
        get_webview(webview, label)?.$cmd().map_err(Into::into)
      }
    };
  }

  macro_rules! setter {
    ($cmd: ident) => {
      setter!($cmd, $cmd);
    };
    ($fn: ident, $cmd: ident) => {
      #[command(root = "crate")]
      pub async fn $fn<R: Runtime>(
        webview: Webview<R>,
        label: Option<String>,
      ) -> crate::Result<()> {
        get_webview(webview, label)?.$cmd().map_err(Into::into)
      }
    };
    ($fn: ident, $cmd: ident, $input: ty) => {
      #[command(root = "crate")]
      pub async fn $fn<R: Runtime>(
        webview: Webview<R>,
        label: Option<String>,
        value: $input,
      ) -> crate::Result<()> {
        get_webview(webview, label)?.$cmd(value).map_err(Into::into)
      }
    };
  }

  // TODO
  getter!(
    webview_position,
    position,
    tauri_runtime::dpi::PhysicalPosition<i32>
  );
  getter!(webview_size, size, tauri_runtime::dpi::PhysicalSize<u32>);
  //getter!(is_focused, bool);

  setter!(print);
  setter!(webview_close, close);
  setter!(set_webview_size, set_size, Size);
  setter!(set_webview_position, set_position, Position);
  setter!(set_webview_focus, set_focus);
  setter!(set_webview_auto_resize, set_auto_resize, bool);
  setter!(webview_hide, hide);
  setter!(webview_show, show);
  setter!(set_webview_zoom, set_zoom, f64);
  setter!(
    set_webview_background_color,
    set_background_color,
    Option<Color>
  );
  setter!(clear_all_browsing_data, clear_all_browsing_data);

  #[cfg(not(feature = "unstable"))]
  #[command(root = "crate")]
  pub async fn create_webview() -> crate::Result<()> {
    Err(crate::Error::UnstableFeatureNotSupported)
  }

  #[cfg(feature = "unstable")]
  #[command(root = "crate")]
  pub async fn create_webview<R: Runtime>(
    app: crate::AppHandle<R>,
    window_label: String,
    options: WindowConfig,
  ) -> crate::Result<()> {
    use anyhow::Context;

    let window = app
      .manager()
      .get_window(&window_label)
      .ok_or(crate::Error::WindowNotFound)?;

    let x = options.x.context("missing parameter `options.x`")?;
    let y = options.y.context("missing parameter `options.y`")?;
    let width = options.width;
    let height = options.height;

    let builder = crate::webview::WebviewBuilder::from_config(&options);

    window.add_child(
      builder,
      tauri_runtime::dpi::LogicalPosition::new(x, y),
      tauri_runtime::dpi::LogicalSize::new(width, height),
    )?;

    Ok(())
  }

  #[command(root = "crate")]
  pub async fn reparent<R: Runtime>(
    webview: crate::Webview<R>,
    label: Option<String>,
    window: String,
  ) -> crate::Result<()> {
    let webview = get_webview(webview, label)?;
    if let Some(window) = webview.manager.get_window(&window) {
      webview.reparent(&window)?;
    }
    Ok(())
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  #[command(root = "crate")]
  pub async fn internal_toggle_devtools<R: Runtime>(
    webview: crate::Webview<R>,
    label: Option<String>,
  ) -> crate::Result<()> {
    let webview = get_webview(webview, label)?;
    if webview.devtools == Some(false) {
      return Ok(());
    }
    if webview.is_devtools_open() {
      webview.close_devtools();
    } else {
      webview.open_devtools();
    }
    Ok(())
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  #[allow(unused_mut)]
  let mut init_script = String::new();
  // window.print works on Linux/Windows; need to use the API on macOS
  #[cfg(any(target_os = "macos", target_os = "ios"))]
  {
    init_script.push_str(include_str!("./scripts/print.js"));
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  {
    use serialize_to_javascript::{DefaultTemplate, Template, default_template};

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

  let mut builder = Builder::new("webview");
  if !init_script.is_empty() {
    builder = builder.js_init_script(init_script);
  }

  builder
    .invoke_handler(crate::generate_handler![
      #![plugin(webview)]
      create_webview_window,
      get_all_webviews,
      #[cfg(desktop)] desktop_commands::create_webview,
      // getters
      #[cfg(desktop)] desktop_commands::webview_position,
      #[cfg(desktop)] desktop_commands::webview_size,
      // setters
      #[cfg(desktop)] desktop_commands::webview_close,
      #[cfg(desktop)] desktop_commands::set_webview_size,
      #[cfg(desktop)] desktop_commands::set_webview_position,
      #[cfg(desktop)] desktop_commands::set_webview_focus,
      #[cfg(desktop)] desktop_commands::set_webview_auto_resize,
      #[cfg(desktop)] desktop_commands::set_webview_background_color,
      #[cfg(desktop)] desktop_commands::set_webview_zoom,
      #[cfg(desktop)] desktop_commands::webview_hide,
      #[cfg(desktop)] desktop_commands::webview_show,
      #[cfg(desktop)] desktop_commands::print,
      #[cfg(desktop)] desktop_commands::clear_all_browsing_data,
      #[cfg(desktop)] desktop_commands::reparent,
      #[cfg(all(desktop, any(debug_assertions, feature = "devtools")))]
      desktop_commands::internal_toggle_devtools,
    ])
    .build()
}
