// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  str::FromStr,
  sync::{
    atomic::{AtomicU32, AtomicUsize, Ordering},
    Arc, Mutex,
  },
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
  command,
  ipc::{CommandArg, CommandItem},
  plugin::{Builder as PluginBuilder, TauriPlugin},
  Manager, Runtime, State, Webview,
};

use super::{CallbackFn, InvokeError, InvokeResponseBody, IpcResponse, Request, Response};

pub const IPC_PAYLOAD_PREFIX: &str = "__CHANNEL__:";
pub const CHANNEL_PLUGIN_NAME: &str = "__TAURI_CHANNEL__";
pub const FETCH_CHANNEL_DATA_COMMAND: &str = "plugin:__TAURI_CHANNEL__|fetch";
pub(crate) const CHANNEL_ID_HEADER_NAME: &str = "Tauri-Channel-Id";

static CHANNEL_COUNTER: AtomicU32 = AtomicU32::new(0);
static CHANNEL_DATA_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Maps a channel id to a pending data that must be send to the JavaScript side via the IPC.
#[derive(Default, Clone)]
pub struct ChannelDataIpcQueue(pub(crate) Arc<Mutex<HashMap<u32, InvokeResponseBody>>>);

/// An IPC channel.
#[derive(Clone)]
pub struct Channel<TSend = InvokeResponseBody> {
  id: u32,
  on_message: Arc<dyn Fn(InvokeResponseBody) -> crate::Result<()> + Send + Sync>,
  phantom: std::marker::PhantomData<TSend>,
}

#[cfg(feature = "specta")]
const _: () = {
  #[derive(specta::Type)]
  #[specta(remote = super::Channel, rename = "TAURI_CHANNEL")]
  struct Channel<TSend>(std::marker::PhantomData<TSend>);
};

impl<TSend> Serialize for Channel<TSend> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&format!("{IPC_PAYLOAD_PREFIX}{}", self.id))
  }
}

/// The ID of a channel that was defined on the JavaScript layer.
///
/// Useful when expecting [`Channel`] as part of a JSON object instead of a top-level command argument.
///
/// # Examples
///
/// ```rust
/// use tauri::{ipc::JavaScriptChannelId, Runtime, Webview};
///
/// #[derive(serde::Deserialize)]
/// #[serde(rename_all = "camelCase")]
/// struct Button {
///   label: String,
///   on_click: JavaScriptChannelId,
/// }
///
/// #[tauri::command]
/// fn add_button<R: Runtime>(webview: Webview<R>, button: Button) {
///   let channel = button.on_click.channel_on(webview);
///   channel.send("clicked").unwrap();
/// }
/// ```
pub struct JavaScriptChannelId(CallbackFn);

impl FromStr for JavaScriptChannelId {
  type Err = &'static str;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    s.split_once(IPC_PAYLOAD_PREFIX)
      .ok_or("invalid channel string")
      .and_then(|(_prefix, id)| id.parse().map_err(|_| "invalid channel ID"))
      .map(|id| Self(CallbackFn(id)))
  }
}

impl JavaScriptChannelId {
  /// Gets a [`Channel`] for this channel ID on the given [`Webview`].
  pub fn channel_on<R: Runtime, TSend>(&self, webview: Webview<R>) -> Channel<TSend> {
    let callback_id = self.0;
    let counter = AtomicUsize::new(0);

    Channel::new_with_id(callback_id.0, move |body| {
      let i = counter.fetch_add(1, Ordering::Relaxed);

      if let Some(interceptor) = &webview.manager.channel_interceptor {
        if interceptor(&webview, callback_id, i, &body) {
          return Ok(());
        }
      }

      let data_id = CHANNEL_DATA_COUNTER.fetch_add(1, Ordering::Relaxed);

      webview
        .state::<ChannelDataIpcQueue>()
        .0
        .lock()
        .unwrap()
        .insert(data_id, body);

      webview.eval(&format!(
        "window.__TAURI_INTERNALS__.invoke('{FETCH_CHANNEL_DATA_COMMAND}', null, {{ headers: {{ '{CHANNEL_ID_HEADER_NAME}': '{data_id}' }} }}).then((response) => window['_' + {}]({{ message: response, id: {i} }})).catch(console.error)",
        callback_id.0
      ))?;

      Ok(())
    })
  }
}

impl<'de> Deserialize<'de> for JavaScriptChannelId {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let value: String = Deserialize::deserialize(deserializer)?;
    Self::from_str(&value).map_err(|_| {
      serde::de::Error::custom(format!(
        "invalid channel value `{value}`, expected a string in the `{IPC_PAYLOAD_PREFIX}ID` format"
      ))
    })
  }
}

impl<TSend> Channel<TSend> {
  /// Creates a new channel with the given message handler.
  pub fn new<F: Fn(InvokeResponseBody) -> crate::Result<()> + Send + Sync + 'static>(
    on_message: F,
  ) -> Self {
    Self::new_with_id(CHANNEL_COUNTER.fetch_add(1, Ordering::Relaxed), on_message)
  }

  fn new_with_id<F: Fn(InvokeResponseBody) -> crate::Result<()> + Send + Sync + 'static>(
    id: u32,
    on_message: F,
  ) -> Self {
    #[allow(clippy::let_and_return)]
    let channel = Self {
      id,
      on_message: Arc::new(on_message),
      phantom: Default::default(),
    };

    #[cfg(mobile)]
    crate::plugin::mobile::register_channel(Channel {
      id,
      on_message: channel.on_message.clone(),
      phantom: Default::default(),
    });

    channel
  }

  pub(crate) fn from_callback_fn<R: Runtime>(webview: Webview<R>, callback: CallbackFn) -> Self {
    Channel::new_with_id(callback.0, move |body| {
      let data_id = CHANNEL_DATA_COUNTER.fetch_add(1, Ordering::Relaxed);

      webview
        .state::<ChannelDataIpcQueue>()
        .0
        .lock()
        .unwrap()
        .insert(data_id, body);

      webview.eval(&format!(
        "window.__TAURI_INTERNALS__.invoke('{FETCH_CHANNEL_DATA_COMMAND}', null, {{ headers: {{ '{CHANNEL_ID_HEADER_NAME}': '{data_id}' }} }}).then((response) => window['_' + {}](response)).catch(console.error)",
        callback.0
      ))?;

      Ok(())
    })
  }

  /// The channel identifier.
  pub fn id(&self) -> u32 {
    self.id
  }

  /// Sends the given data through the channel.
  pub fn send(&self, data: TSend) -> crate::Result<()>
  where
    TSend: IpcResponse,
  {
    (self.on_message)(data.body()?)
  }
}

impl<'de, R: Runtime, TSend: Clone> CommandArg<'de, R> for Channel<TSend> {
  /// Grabs the [`Webview`] from the [`CommandItem`] and returns the associated [`Channel`].
  fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
    let name = command.name;
    let arg = command.key;
    let webview = command.message.webview();
    let value: String =
      Deserialize::deserialize(command).map_err(|e| crate::Error::InvalidArgs(name, arg, e))?;
    JavaScriptChannelId::from_str(&value)
      .map(|id| id.channel_on(webview))
      .map_err(|_| {
        InvokeError::from(format!(
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
