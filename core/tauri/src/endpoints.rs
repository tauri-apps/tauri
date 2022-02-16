// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  hooks::{InvokeError, InvokeMessage, InvokeResolver},
  runtime::Runtime,
  Config, Invoke, PackageInfo, Window,
};
pub use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use std::sync::Arc;

mod app;
mod cli;
mod clipboard;
mod dialog;
mod event;
#[allow(unused_imports)]
mod file_system;
mod global_shortcut;
mod http;
mod notification;
mod operating_system;
mod path;
mod process;
mod shell;
mod window;

/// The context passed to the invoke handler.
pub struct InvokeContext<R: Runtime> {
  pub window: Window<R>,
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
  Process(process::Cmd),
  Fs(file_system::Cmd),
  Os(operating_system::Cmd),
  Path(path::Cmd),
  Window(Box<window::Cmd>),
  Shell(shell::Cmd),
  Event(event::Cmd),
  Dialog(dialog::Cmd),
  Cli(cli::Cmd),
  Notification(notification::Cmd),
  Http(http::Cmd),
  GlobalShortcut(global_shortcut::Cmd),
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
      Self::Process(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      Self::Fs(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      Self::Path(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
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
      Self::Dialog(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
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
      Self::Http(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .await
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
      Self::GlobalShortcut(cmd) => resolver.respond_async(async move {
        cmd
          .run(context)
          .and_then(|r| r.json)
          .map_err(InvokeError::from_anyhow)
      }),
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
    obj.insert("module".to_string(), JsonValue::String(module));
  }

  match serde_json::from_value::<Module>(payload) {
    Ok(module) => module.run(window, resolver, config, package_info.clone()),
    Err(e) => resolver.reject(e.to_string()),
  }
}
