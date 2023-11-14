// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
  },
};

use serde::{Deserialize, Serialize, Serializer};

use crate::{
  command,
  command::{CommandArg, CommandItem},
  plugin::{Builder as PluginBuilder, TauriPlugin},
  Manager, Runtime, State, Webview,
};

use super::{CallbackFn, InvokeBody, InvokeError, IpcResponse, Request, Response};

pub const IPC_PAYLOAD_PREFIX: &str = "__CHANNEL__:";
pub const CHANNEL_PLUGIN_NAME: &str = "__TAURI_CHANNEL__";
// TODO: ideally this const references CHANNEL_PLUGIN_NAME
pub const FETCH_CHANNEL_DATA_COMMAND: &str = "plugin:__TAURI_CHANNEL__|fetch";
pub(crate) const CHANNEL_ID_HEADER_NAME: &str = "Tauri-Channel-Id";

static CHANNEL_COUNTER: AtomicU32 = AtomicU32::new(0);
static CHANNEL_DATA_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Maps a channel id to a pending data that must be send to the JavaScript side via the IPC.
#[derive(Default, Clone)]
pub struct ChannelDataIpcQueue(pub(crate) Arc<Mutex<HashMap<u32, InvokeBody>>>);

/// An IPC channel.
#[derive(Clone)]
pub struct Channel {
  id: u32,
  on_message: Arc<dyn Fn(InvokeBody) -> crate::Result<()> + Send + Sync>,
}

impl Serialize for Channel {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&format!("{IPC_PAYLOAD_PREFIX}{}", self.id))
  }
}

impl Channel {
  /// Creates a new channel with the given message handler.
  pub fn new<F: Fn(InvokeBody) -> crate::Result<()> + Send + Sync + 'static>(
    on_message: F,
  ) -> Self {
    Self::new_with_id(CHANNEL_COUNTER.fetch_add(1, Ordering::Relaxed), on_message)
  }

  pub(crate) fn new_with_id<F: Fn(InvokeBody) -> crate::Result<()> + Send + Sync + 'static>(
    id: u32,
    on_message: F,
  ) -> Self {
    #[allow(clippy::let_and_return)]
    let channel = Self {
      id,
      on_message: Arc::new(on_message),
    };

    #[cfg(mobile)]
    crate::plugin::mobile::register_channel(channel.clone());

    channel
  }

  pub(crate) fn from_ipc<R: Runtime>(webview: Webview<R>, callback: CallbackFn) -> Self {
    Channel::new_with_id(callback.0, move |body| {
      let data_id = CHANNEL_DATA_COUNTER.fetch_add(1, Ordering::Relaxed);
      webview
        .state::<ChannelDataIpcQueue>()
        .0
        .lock()
        .unwrap()
        .insert(data_id, body);
      webview.eval(&format!(
        "window.__TAURI_INTERNALS__.invoke('{FETCH_CHANNEL_DATA_COMMAND}', null, {{ headers: {{ '{CHANNEL_ID_HEADER_NAME}': '{data_id}' }} }}).then(window['_' + {}]).catch(console.error)",
        callback.0
      ))
    })
  }

  pub(crate) fn load_from_ipc<R: Runtime>(
    webview: Webview<R>,
    value: impl AsRef<str>,
  ) -> Option<Self> {
    value
      .as_ref()
      .split_once(IPC_PAYLOAD_PREFIX)
      .and_then(|(_prefix, id)| id.parse().ok())
      .map(|callback_id| Self::from_ipc(webview, CallbackFn(callback_id)))
  }

  /// The channel identifier.
  pub fn id(&self) -> u32 {
    self.id
  }

  /// Sends the given data through the  channel.
  pub fn send<T: IpcResponse>(&self, data: T) -> crate::Result<()> {
    let body = data.body()?;
    (self.on_message)(body)
  }
}

impl<'de, R: Runtime> CommandArg<'de, R> for Channel {
  /// Grabs the [`Webview`] from the [`CommandItem`] and returns the associated [`Channel`].
  fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
    let name = command.name;
    let arg = command.key;
    let webview = command.message.webview();
    let value: String =
      Deserialize::deserialize(command).map_err(|e| crate::Error::InvalidArgs(name, arg, e))?;
    Channel::load_from_ipc(webview, &value).ok_or_else(|| {
      InvokeError::from_anyhow(anyhow::anyhow!(
        "invalid channel value `{value}`, expected a string in the `{IPC_PAYLOAD_PREFIX}ID` format"
      ))
    })
  }
}

#[command(root = "crate")]
fn fetch(
  request: Request<'_>,
  cache: State<'_, ChannelDataIpcQueue>,
) -> Result<Response, &'static str> {
  if let Some(id) = request
    .headers()
    .get(CHANNEL_ID_HEADER_NAME)
    .and_then(|v| v.to_str().ok())
    .and_then(|id| id.parse().ok())
  {
    if let Some(data) = cache.0.lock().unwrap().remove(&id) {
      Ok(Response::new(data))
    } else {
      Err("data not found")
    }
  } else {
    Err("missing channel id header")
  }
}

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
  PluginBuilder::new(CHANNEL_PLUGIN_NAME)
    .invoke_handler(crate::generate_handler![fetch])
    .build()
}
