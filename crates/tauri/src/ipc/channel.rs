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

use super::{
  format_callback::format_raw_js, CallbackFn, InvokeError, InvokeResponseBody, IpcResponse,
  Request, Response,
};

pub const IPC_PAYLOAD_PREFIX: &str = "__CHANNEL__:";
// TODO: Change this to `channel` in v3
pub const CHANNEL_PLUGIN_NAME: &str = "__TAURI_CHANNEL__";
// TODO: Change this to `plugin:channel|fetch` in v3
pub const FETCH_CHANNEL_DATA_COMMAND: &str = "plugin:__TAURI_CHANNEL__|fetch";
const CHANNEL_ID_HEADER_NAME: &str = "Tauri-Channel-Id";

/// Maximum size a JSON we should send directly without going through the fetch process
// 8192 byte JSON payload runs roughly 2x faster through eval than through fetch on WebView2 v135
const MAX_JSON_DIRECT_EXECUTE_THRESHOLD: usize = 8192;
// 1024 byte payload runs  roughly 30% faster through eval than through fetch on macOS
const MAX_RAW_DIRECT_EXECUTE_THRESHOLD: usize = 1024;

static CHANNEL_COUNTER: AtomicU32 = AtomicU32::new(0);
static CHANNEL_DATA_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Maps a channel id to a pending data that must be send to the JavaScript side via the IPC.
#[derive(Default, Clone)]
pub struct ChannelDataIpcQueue(Arc<Mutex<HashMap<u32, InvokeResponseBody>>>);

/// An IPC channel.
pub struct Channel<TSend = InvokeResponseBody> {
  inner: Arc<ChannelInner>,
  phantom: std::marker::PhantomData<TSend>,
}

#[cfg(feature = "specta")]
const _: () = {
  #[derive(specta::Type)]
  #[specta(remote = super::Channel, rename = "TAURI_CHANNEL")]
  #[allow(dead_code)]
  struct Channel<TSend>(std::marker::PhantomData<TSend>);
};

impl<TSend> Clone for Channel<TSend> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      phantom: self.phantom,
    }
  }
}

type OnDropFn = Option<Box<dyn Fn() + Send + Sync + 'static>>;
type OnMessageFn = Box<dyn Fn(InvokeResponseBody) -> crate::Result<()> + Send + Sync>;

struct ChannelInner {
  id: u32,
  on_message: OnMessageFn,
  on_drop: OnDropFn,
}

impl Drop for ChannelInner {
  fn drop(&mut self) {
    if let Some(on_drop) = &self.on_drop {
      on_drop();
    }
  }
}

impl<TSend> Serialize for Channel<TSend> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&format!("{IPC_PAYLOAD_PREFIX}{}", self.inner.id))
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
    s.strip_prefix(IPC_PAYLOAD_PREFIX)
      .ok_or("invalid channel string")
      .and_then(|id| id.parse().map_err(|_| "invalid channel ID"))
      .map(|id| Self(CallbackFn(id)))
  }
}

impl JavaScriptChannelId {
  /// Gets a [`Channel`] for this channel ID on the given [`Webview`].
  pub fn channel_on<R: Runtime, TSend>(&self, webview: Webview<R>) -> Channel<TSend> {
    let callback_fn = self.0;
    let callback_id = callback_fn.0;

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    let webview_clone = webview.clone();

    Channel::new_with_id(
      callback_id,
      Box::new(move |body| {
        let current_index = counter.fetch_add(1, Ordering::Relaxed);

        if let Some(interceptor) = &webview.manager.channel_interceptor {
          if interceptor(&webview, callback_fn, current_index, &body) {
            return Ok(());
          }
        }

        match body {
          // Don't go through the fetch process if the payload is small
          InvokeResponseBody::Json(json_string)
            if json_string.len() < MAX_JSON_DIRECT_EXECUTE_THRESHOLD =>
          {
            webview.eval(format_raw_js(
              callback_id,
              format!("{{ message: {json_string}, index: {current_index} }}"),
            ))?;
          }
          InvokeResponseBody::Raw(bytes) if bytes.len() < MAX_RAW_DIRECT_EXECUTE_THRESHOLD => {
            let bytes_as_json_array = serde_json::to_string(&bytes)?;
            webview.eval(format_raw_js(callback_id, format!("{{ message: new Uint8Array({bytes_as_json_array}).buffer, index: {current_index} }}")))?;
          }
          // use the fetch API to speed up larger response payloads
          _ => {
            let data_id = CHANNEL_DATA_COUNTER.fetch_add(1, Ordering::Relaxed);

            webview
              .state::<ChannelDataIpcQueue>()
              .0
              .lock()
              .unwrap()
              .insert(data_id, body);

            webview.eval(format!(
              "window.__TAURI_INTERNALS__.invoke('{FETCH_CHANNEL_DATA_COMMAND}', null, {{ headers: {{ '{CHANNEL_ID_HEADER_NAME}': '{data_id}' }} }}).then((response) => window.__TAURI_INTERNALS__.runCallback({callback_id}, {{ message: response, index: {current_index} }})).catch(console.error)",
            ))?;
          }
        }

        Ok(())
      }),
      Some(Box::new(move || {
        let current_index = counter_clone.load(Ordering::Relaxed);
        let _ = webview_clone.eval(format_raw_js(
          callback_id,
          format!("{{ end: true, index: {current_index} }}"),
        ));
      })),
    )
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
    Self::new_with_id(
      CHANNEL_COUNTER.fetch_add(1, Ordering::Relaxed),
      Box::new(on_message),
      None,
    )
  }

  fn new_with_id(id: u32, on_message: OnMessageFn, on_drop: OnDropFn) -> Self {
    #[allow(clippy::let_and_return)]
    let channel = Self {
      inner: Arc::new(ChannelInner {
        id,
        on_message,
        on_drop,
      }),
      phantom: Default::default(),
    };

    #[cfg(mobile)]
    crate::plugin::mobile::register_channel(Channel {
      inner: channel.inner.clone(),
      phantom: Default::default(),
    });

    channel
  }

  // This is used from the IPC handler
  pub(crate) fn from_callback_fn<R: Runtime>(webview: Webview<R>, callback: CallbackFn) -> Self {
    let callback_id = callback.0;
    Channel::new_with_id(
      callback_id,
      Box::new(move |body| {
        match body {
          // Don't go through the fetch process if the payload is small
          InvokeResponseBody::Json(json_string)
            if json_string.len() < MAX_JSON_DIRECT_EXECUTE_THRESHOLD =>
          {
            webview.eval(format_raw_js(callback_id, json_string))?;
          }
          InvokeResponseBody::Raw(bytes) if bytes.len() < MAX_RAW_DIRECT_EXECUTE_THRESHOLD => {
            let bytes_as_json_array = serde_json::to_string(&bytes)?;
            webview.eval(format_raw_js(
              callback_id,
              format!("new Uint8Array({bytes_as_json_array}).buffer"),
            ))?;
          }
          // use the fetch API to speed up larger response payloads
          _ => {
            let data_id = CHANNEL_DATA_COUNTER.fetch_add(1, Ordering::Relaxed);

            webview
              .state::<ChannelDataIpcQueue>()
              .0
              .lock()
              .unwrap()
              .insert(data_id, body);

            webview.eval(format!(
              "window.__TAURI_INTERNALS__.invoke('{FETCH_CHANNEL_DATA_COMMAND}', null, {{ headers: {{ '{CHANNEL_ID_HEADER_NAME}': '{data_id}' }} }}).then((response) => window.__TAURI_INTERNALS__.runCallback({callback_id}, response)).catch(console.error)",
            ))?;
          }
        }

        Ok(())
      }),
      None,
    )
  }

  /// The channel identifier.
  pub fn id(&self) -> u32 {
    self.inner.id
  }

  /// Sends the given data through the channel.
  pub fn send(&self, data: TSend) -> crate::Result<()>
  where
    TSend: IpcResponse,
  {
    (self.inner.on_message)(data.body()?)
  }
}

impl<'de, R: Runtime, TSend> CommandArg<'de, R> for Channel<TSend> {
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
    .invoke_handler(crate::generate_handler![
      #![plugin(__TAURI_CHANNEL__)]
      fetch
    ])
    .build()
}
