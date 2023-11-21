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
  use tauri_runtime::window::dpi::{
    LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size,
  };
  use tauri_utils::config::{WebviewUrl, WindowConfig};

  use super::*;
  use crate::{
    command, utils::config::WindowEffectsConfig, AppHandle, Manager, Webview, WebviewBuilder,
    WindowBuilder,
  };

  #[derive(Debug, PartialEq, Clone, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct WebviewConfig {
    #[serde(default)]
    url: WebviewUrl,
    user_agent: Option<String>,
    file_drop_enabled: Option<bool>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    #[serde(default)]
    transparent: bool,
    #[serde(default)]
    accept_first_mouse: bool,
    window_effects: Option<WindowEffectsConfig>,
    #[serde(default)]
    incognito: bool,
  }

  #[command(root = "crate")]
  pub async fn create_webview_window<R: Runtime>(
    app: AppHandle<R>,
    options: WindowConfig,
  ) -> crate::Result<()> {
    WindowBuilder::from_config(&app, options.clone())
      .with_webview(WebviewBuilder::from_config(options))?;
    Ok(())
  }

  #[command(root = "crate")]
  pub async fn create_webview<R: Runtime>(
    app: AppHandle<R>,
    label: String,
    window_label: String,
    options: WebviewConfig,
  ) -> crate::Result<()> {
    let window = app
      .get_window(&window_label)
      .ok_or(crate::Error::WindowNotFound)?;
    let mut builder = WebviewBuilder::new(label, options.url);

    builder.webview_attributes.user_agent = options.user_agent;
    builder.webview_attributes.file_drop_handler_enabled =
      options.file_drop_enabled.unwrap_or(true);
    builder.webview_attributes.transparent = options.transparent;
    builder.webview_attributes.accept_first_mouse = options.accept_first_mouse;
    builder.webview_attributes.window_effects = options.window_effects;
    builder.webview_attributes.incognito = options.incognito;

    window.add_child(
      builder,
      LogicalPosition::new(options.x, options.y),
      LogicalSize::new(options.width, options.height),
    )?;

    Ok(())
  }

  fn get_webview<R: Runtime>(
    webview: Webview<R>,
    label: Option<String>,
  ) -> crate::Result<Webview<R>> {
    match label {
      Some(l) if !l.is_empty() => webview.get_webview(&l).ok_or(crate::Error::WebviewNotFound),
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
  getter!(webview_position, position, PhysicalPosition<i32>);
  getter!(webview_size, size, PhysicalSize<u32>);
  //getter!(is_focused, bool);

  setter!(print);
  // setter!(close);
  setter!(set_webview_size, set_size, Size);
  setter!(set_webview_position, set_position, Position);
  setter!(set_webview_focus, set_focus);

  #[cfg(any(debug_assertions, feature = "devtools"))]
  #[command(root = "crate")]
  pub async fn internal_toggle_devtools<R: Runtime>(
    webview: crate::Webview<R>,
    label: Option<String>,
  ) -> crate::Result<()> {
    let webview = get_webview(webview, label)?;
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
    use serialize_to_javascript::{default_template, DefaultTemplate, Template};

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
    .invoke_handler(|invoke| {
      #[cfg(desktop)]
      {
        let handler: Box<dyn Fn(crate::ipc::Invoke<R>) -> bool> =
          Box::new(crate::generate_handler![
            desktop_commands::create_webview,
            desktop_commands::create_webview_window,
            // getters
            // TODO
            desktop_commands::webview_position,
            desktop_commands::webview_size,
            //desktop_commands::is_focused,
            // setters
            // desktop_commands::close,
            desktop_commands::set_webview_size,
            desktop_commands::set_webview_position,
            desktop_commands::set_webview_focus,
            desktop_commands::print,
            #[cfg(any(debug_assertions, feature = "devtools"))]
            desktop_commands::internal_toggle_devtools,
          ]);
        handler(invoke)
      }
      #[cfg(mobile)]
      {
        invoke
          .resolver
          .reject("Webview API not available on mobile");
        true
      }
    })
    .build()
}
