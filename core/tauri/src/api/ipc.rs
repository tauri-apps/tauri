// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to Inter Procedure Call(IPC).
//!
//! This module includes utilities to send messages to the JS layer of the webview.

use std::{collections::HashMap, sync::Mutex};

use serde::{Deserialize, Serialize};
pub use serialize_to_javascript::Options as SerializeOptions;
use tauri_macros::default_runtime;

use crate::{
  command::{CommandArg, CommandItem},
  hooks::InvokeBody,
  InvokeError, Manager, Runtime, Window,
};

#[cfg(target_os = "linux")]
pub(crate) mod format_callback;

const CHANNEL_PREFIX: &str = "__CHANNEL__:";
pub(crate) const FETCH_CHANNEL_DATA_COMMAND: &str = "__tauriFetchChannelData__";

#[derive(Default)]
pub(crate) struct ChannelDataCache(pub(crate) Mutex<HashMap<u32, InvokeBody>>);

/// An IPC channel.
#[default_runtime(crate::Wry, wry)]
pub struct Channel<R: Runtime> {
  id: CallbackFn,
  window: Window<R>,
}

impl Serialize for Channel {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&format!("{CHANNEL_PREFIX}{}", self.id.0))
  }
}

impl<R: Runtime> Channel<R> {
  /// Sends the given data through the channel.
  pub fn send<T: IpcResponse>(&self, data: T) -> crate::Result<()> {
    #[cfg(target_os = "linux")]
    {
      let js = format_callback::format(self.id, data.body()?.into_json())?;
      self.window.eval(&js)
    }
    #[cfg(not(target_os = "linux"))]
    {
      let body = data.body()?;
      let data_id = rand::random();
      self
        .window
        .state::<ChannelDataCache>()
        .0
        .lock()
        .unwrap()
        .insert(data_id, body);
      self.window.eval(&format!(
        "__TAURI_INVOKE__('{FETCH_CHANNEL_DATA_COMMAND}', {{ id: {data_id} }}).then(window['_' + {}])",
        self.id.0
      ))
    }
  }
}

impl<'de, R: Runtime> CommandArg<'de, R> for Channel<R> {
  /// Grabs the [`Window`] from the [`CommandItem`] and returns the associated [`Channel`].
  fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
    let name = command.name;
    let arg = command.key;
    let window = command.message.window();
    let value: String =
      Deserialize::deserialize(command).map_err(|e| crate::Error::InvalidArgs(name, arg, e))?;
    if let Some(callback_id) = value
      .split_once(CHANNEL_PREFIX)
      .and_then(|(_prefix, id)| id.parse().ok())
    {
      return Ok(Channel {
        id: CallbackFn(callback_id),
        window,
      });
    }
    Err(InvokeError::from_anyhow(anyhow::anyhow!(
      "invalid channel value `{value}`, expected a string in the `{CHANNEL_PREFIX}ID` format"
    )))
  }
}

/// The IPC request.
#[derive(Debug)]
pub struct Request<'a> {
  body: &'a InvokeBody,
}

impl<'a> Request<'a> {
  /// The request body.
  pub fn body(&self) -> &InvokeBody {
    self.body
  }
}

impl<'a, R: Runtime> CommandArg<'a, R> for Request<'a> {
  /// Returns the invoke [`Request`].
  fn from_command(command: CommandItem<'a, R>) -> Result<Self, InvokeError> {
    Ok(Self {
      body: &command.message.payload,
    })
  }
}

/// Marks a type as a response to an IPC call.
pub trait IpcResponse {
  /// Resolve the IPC response body.
  fn body(self) -> crate::Result<InvokeBody>;
}

impl<T: Serialize> IpcResponse for T {
  fn body(self) -> crate::Result<InvokeBody> {
    serde_json::to_value(self)
      .map(Into::into)
      .map_err(Into::into)
  }
}

/// The IPC request.
pub struct Response {
  body: InvokeBody,
}

impl IpcResponse for Response {
  fn body(self) -> crate::Result<InvokeBody> {
    Ok(self.body)
  }
}

impl Response {
  /// Defines a response with the given body.
  pub fn new(body: impl Into<InvokeBody>) -> Self {
    Self { body: body.into() }
  }
}

/// The `Callback` type is the return value of the `transformCallback` JavaScript function.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CallbackFn(pub usize);
