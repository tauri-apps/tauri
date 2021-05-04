// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  api::{config::Config, PackageInfo},
  hooks::{InvokeError, InvokeMessage, InvokeResolver},
  Invoke, Params, Window,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use std::sync::Arc;

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
mod process;
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
  Process(process::Cmd),
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
    window: Window<M>,
    resolver: InvokeResolver<M>,
    config: Arc<Config>,
    package_info: PackageInfo,
  ) {
    match self {
      Self::App(cmd) => resolver.respond_async(async move {
        cmd
          .run(package_info)
          .and_then(|r| r.json)
          .map_err(InvokeError::from)
      }),
      Self::Process(cmd) => resolver
        .respond_async(async move { cmd.run().and_then(|r| r.json).map_err(InvokeError::from) }),
      Self::Fs(cmd) => resolver.respond_async(async move {
        cmd
          .run(config)
          .and_then(|r| r.json)
          .map_err(InvokeError::from)
      }),
      Self::Window(cmd) => resolver.respond_async(async move {
        cmd
          .run(window)
          .await
          .and_then(|r| r.json)
          .map_err(InvokeError::from)
      }),
      Self::Shell(cmd) => resolver.respond_async(async move {
        cmd
          .run(window)
          .and_then(|r| r.json)
          .map_err(InvokeError::from)
      }),
      Self::Event(cmd) => resolver.respond_async(async move {
        cmd
          .run(window)
          .and_then(|r| r.json)
          .map_err(InvokeError::from)
      }),
      Self::Internal(cmd) => resolver.respond_async(async move {
        cmd
          .run(window)
          .and_then(|r| r.json)
          .map_err(InvokeError::from)
      }),
      Self::Dialog(cmd) => {
        let _ = window.run_on_main_thread(|| {
          resolver
            .respond_closure(move || cmd.run().and_then(|r| r.json).map_err(InvokeError::from))
        });
      }
      Self::Cli(cmd) => {
        if let Some(cli_config) = config.tauri.cli.clone() {
          resolver.respond_async(async move {
            cmd
              .run(&cli_config)
              .and_then(|r| r.json)
              .map_err(InvokeError::from)
          })
        }
      }
      Self::Notification(cmd) => resolver.respond_closure(move || {
        cmd
          .run(config)
          .and_then(|r| r.json)
          .map_err(InvokeError::from)
      }),
      Self::Http(cmd) => resolver.respond_async(async move {
        cmd
          .run()
          .await
          .and_then(|r| r.json)
          .map_err(InvokeError::from)
      }),
      Self::GlobalShortcut(cmd) => resolver.respond_async(async move {
        cmd
          .run(window)
          .and_then(|r| r.json)
          .map_err(InvokeError::from)
      }),
    }
  }
}

pub(crate) fn handle<P: Params>(
  module: String,
  invoke: Invoke<P>,
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
