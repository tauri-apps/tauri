// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  hooks::{InvokeError, InvokeMessage, InvokeResolver},
  Config, Invoke, PackageInfo, Runtime, Window,
};
pub use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use std::sync::Arc;

mod app;
#[cfg(cli)]
mod cli;
#[cfg(clipboard_any)]
mod clipboard;
#[cfg(dialog_any)]
mod dialog;
mod event;
#[cfg(fs_any)]
mod file_system;
#[cfg(global_shortcut_any)]
mod global_shortcut;
#[cfg(http_any)]
mod http;
mod notification;
#[cfg(os_any)]
mod operating_system;
#[cfg(path_any)]
mod path;
#[cfg(process_any)]
mod process;
#[cfg(shell_any)]
mod shell;
mod window;

/// The context passed to the invoke handler.
pub struct InvokeContext<R: Runtime> {
  pub window: Window<R>,
  #[allow(dead_code)]
  pub config: Arc<Config>,
  pub package_info: PackageInfo,
}

#[cfg(test)]
impl<R: Runtime> Clone for InvokeContext<R> {
  fn clone(&self) -> Self {
    Self {
      window: self.window.clone(),
      config: self.config.clone(),
      package_info: self.package_info.clone(),
    }
  }
}

/// The response for a JS `invoke` call.
pub struct InvokeResponse {
  json: Result<JsonValue>,
}

impl<T: Serialize> From<T> for InvokeResponse {
  fn from(value: T) -> Self {
    Self {
      json: serde_json::to_value(value).map_err(Into::into),
    }
  }
}

#[derive(Deserialize)]
#[serde(tag = "module", content = "message")]
enum Module {
  App(app::Cmd),
  #[cfg(process_any)]
  Process(process::Cmd),
  #[cfg(fs_any)]
  Fs(file_system::Cmd),
  #[cfg(os_any)]
  Os(operating_system::Cmd),
  #[cfg(path_any)]
  Path(path::Cmd),
  Window(Box<window::Cmd>),
  #[cfg(shell_any)]
  Shell(shell::Cmd),
  Event(event::Cmd),
  #[cfg(dialog_any)]
  Dialog(dialog::Cmd),
  #[cfg(cli)]
  Cli(cli::Cmd),
  Notification(notification::Cmd),
  #[cfg(http_any)]
  Http(http::Cmd),
  #[cfg(global_shortcut_any)]
  GlobalShortcut(global_shortcut::Cmd),
  #[cfg(clipboard_any)]
  Clipboard(clipboard::Cmd),
}

impl Module {
  fn run<R: Runtime>(
    self,
    window: Window<R>,
    resolver: InvokeResolver<R>,
    config: Arc<Config>,
    package_info: PackageInfo,
  ) {
    let context = InvokeContext {
      window,
      config,
      package_info,
    };
    match self {
      Self::App(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      #[cfg(process_any)]
      Self::Process(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      #[cfg(fs_any)]
      Self::Fs(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      #[cfg(path_any)]
      Self::Path(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      #[cfg(os_any)]
      Self::Os(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      Self::Window(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .await
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      #[cfg(shell_any)]
      Self::Shell(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      Self::Event(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      #[cfg(dialog_any)]
      Self::Dialog(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      #[cfg(cli)]
      Self::Cli(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      Self::Notification(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      #[cfg(http_any)]
      Self::Http(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .await
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      #[cfg(global_shortcut_any)]
      Self::GlobalShortcut(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      #[cfg(clipboard_any)]
      Self::Clipboard(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
    }
  }
}

pub(crate) fn handle<R: Runtime>(
  module: String,
  invoke: Invoke<R>,
  config: Arc<Config>,
  package_info: &PackageInfo,
) {
  let Invoke { message, resolver } = invoke;
  let InvokeMessage {
    mut payload,
    window,
    ..
  } = message;

  if let JsonValue::Object(ref mut obj) = payload {
    obj.insert("module".to_string(), JsonValue::String(module.clone()));
  }

  match serde_json::from_value::<Module>(payload) {
    Ok(module) => module.run(window, resolver, config, package_info.clone()),
    Err(e) => {
      let message = e.to_string();
      if message.starts_with("unknown variant") {
        let mut s = message.split('`');
        s.next();
        if let Some(unknown_variant_name) = s.next() {
          if unknown_variant_name == module {
            return resolver.reject(format!(
              "The `{module}` module is not enabled. You must enable one of its APIs in the allowlist."
            ));
          } else if module == "Window" {
            return resolver.reject(window::into_allowlist_error(unknown_variant_name).to_string());
          }
        }
      }
      resolver.reject(message);
    }
  }
}
