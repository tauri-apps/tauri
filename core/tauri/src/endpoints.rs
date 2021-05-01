// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  api::{config::Config, PackageInfo},
  hooks::{InvokeMessage, InvokeResolver},
  Params,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
mod app;
mod cli;
mod dialog;
mod event;
#[allow(unused_imports)]
mod file_system;
mod global_shortcut;
mod http;
mod internal;
mod notification;
mod shell;
mod window;

/// The response for a JS `invoke` call.
pub struct InvokeResponse {
  json: crate::Result<JsonValue>,
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
  Fs(file_system::Cmd),
  Window(Box<window::Cmd>),
  Shell(shell::Cmd),
  Event(event::Cmd),
  Internal(internal::Cmd),
  Dialog(dialog::Cmd),
  Cli(cli::Cmd),
  Notification(notification::Cmd),
  Http(http::Cmd),
  GlobalShortcut(global_shortcut::Cmd),
}

impl Module {
  fn run<M: Params>(
    self,
    message: InvokeMessage<M>,
    resolver: InvokeResolver<M>,
    config: &Config,
    package_info: PackageInfo,
  ) {
    let window = message.window();
    match self {
      Self::App(cmd) => resolver.respond_async(async move {
        cmd
          .run(package_info)
          .and_then(|r| r.json)
          .map_err(|e| e.to_string())
          .into()
      }),
      Self::Fs(cmd) => resolver.respond_async(async move {
        cmd
          .run()
          .and_then(|r| r.json)
          .map_err(|e| e.to_string())
          .into()
      }),
      Self::Window(cmd) => resolver.respond_async(async move {
        cmd
          .run(window)
          .await
          .and_then(|r| r.json)
          .map_err(|e| e.to_string())
          .into()
      }),
      Self::Shell(cmd) => resolver.respond_async(async move {
        cmd
          .run(window)
          .and_then(|r| r.json)
          .map_err(|e| e.to_string())
          .into()
      }),
      Self::Event(cmd) => resolver.respond_async(async move {
        cmd
          .run(window)
          .and_then(|r| r.json)
          .map_err(|e| e.to_string())
          .into()
      }),
      Self::Internal(cmd) => resolver.respond_async(async move {
        cmd
          .run(window)
          .and_then(|r| r.json)
          .map_err(|e| e.to_string())
          .into()
      }),
      Self::Dialog(cmd) => resolver.respond_async(async move {
        cmd
          .run()
          .and_then(|r| r.json)
          .map_err(|e| e.to_string())
          .into()
      }),
      Self::Cli(cmd) => {
        if let Some(cli_config) = config.tauri.cli.clone() {
          resolver.respond_async(async move {
            cmd
              .run(&cli_config)
              .and_then(|r| r.json)
              .map_err(|e| e.to_string())
              .into()
          })
        }
      }
      Self::Notification(cmd) => {
        let identifier = config.tauri.bundle.identifier.clone();
        resolver.respond_async(async move {
          cmd
            .run(identifier)
            .and_then(|r| r.json)
            .map_err(|e| e.to_string())
            .into()
        })
      }
      Self::Http(cmd) => resolver.respond_async(async move {
        cmd
          .run()
          .await
          .and_then(|r| r.json)
          .map_err(|e| e.to_string())
          .into()
      }),
      Self::GlobalShortcut(cmd) => resolver.respond_async(async move {
        cmd
          .run(window)
          .and_then(|r| r.json)
          .map_err(|e| e.to_string())
          .into()
      }),
    }
  }
}

pub(crate) fn handle<M: Params>(
  module: String,
  message: InvokeMessage<M>,
  resolver: InvokeResolver<M>,
  config: &Config,
  package_info: &PackageInfo,
) {
  let mut payload = message.payload();
  if let JsonValue::Object(ref mut obj) = payload {
    obj.insert("module".to_string(), JsonValue::String(module));
  }
  match serde_json::from_value::<Module>(payload) {
    Ok(module) => module.run(message, resolver, config, package_info.clone()),
    Err(e) => resolver.reject(e.to_string()),
  }
}
