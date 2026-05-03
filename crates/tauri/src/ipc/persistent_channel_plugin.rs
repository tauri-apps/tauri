// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use serde::{Deserialize, Serialize};
use tauri_macros::default_runtime;
use tokio::sync::Mutex;

use crate::{
  command,
  ipc::{ChannelEvent, ChannelMessageType, JavaScriptChannelId, PersistentChannel},
  plugin::{Builder as PluginBuilder, TauriPlugin},
  AppHandle, Manager, Runtime, State, Webview,
};

use super::PersistentChannelManager;

pub const PERSISTENT_CHANNEL_PLUGIN_NAME: &str = "__TAURI_PERSISTENT_CHANNEL__";

static CHANNEL_MANAGER_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
  pub id: u64,
  pub connected: bool,
}

struct ChannelConnection<R: Runtime> {
  user_channel_id: String,
  channel: PersistentChannel<R>,
}

#[derive(Default)]
struct ConnectionRegistry<R: Runtime> {
  connections: Mutex<HashMap<String, ChannelConnection<R>>>,
}

impl<R: Runtime> ConnectionRegistry<R> {
  fn new() -> Self {
    Self::default()
  }

  async fn register(&self, user_channel_id: String, channel: PersistentChannel<R>) {
    let mut connections = self.connections.lock().await;
    connections.insert(
      user_channel_id.clone(),
      ChannelConnection {
        user_channel_id,
        channel,
      },
    );
  }

  async fn unregister(&self, user_channel_id: &str) {
    let mut connections = self.connections.lock().await;
    if let Some(conn) = connections.remove(user_channel_id) {
      let _ = conn.channel.close();
    }
  }

  async fn get(&self, user_channel_id: &str) -> Option<PersistentChannel<R>> {
    let connections = self.connections.lock().await;
    connections
      .get(user_channel_id)
      .map(|conn| conn.channel.clone())
  }

  async fn get_all(&self) -> Vec<PersistentChannel<R>> {
    let connections = self.connections.lock().await;
    connections
      .values()
      .map(|conn| conn.channel.clone())
      .collect()
  }
}

#[command(root = "crate")]
async fn connect<R: Runtime>(
  app: AppHandle<R>,
  webview: Webview<R>,
  user_channel_id: String,
  callback_id: u32,
) -> crate::Result<ChannelInfo> {
  let manager = app.state::<PersistentChannelManager<R>>();
  let registry = app.state::<ConnectionRegistry<R>>();

  let callback_channel_id = JavaScriptChannelId(crate::ipc::CallbackFn(callback_id));

  let channel = manager.create_channel(&webview, callback_channel_id).await;

  registry
    .register(user_channel_id.clone(), channel.clone())
    .await;

  Ok(ChannelInfo {
    id: channel.id(),
    connected: !channel.is_closed(),
  })
}

#[command(root = "crate")]
async fn send_message<R: Runtime>(
  app: AppHandle<R>,
  user_channel_id: String,
  message_type: String,
  payload: Option<serde_json::Value>,
  index: Option<u64>,
) -> crate::Result<()> {
  let manager = app.state::<PersistentChannelManager<R>>();
  let registry = app.state::<ConnectionRegistry<R>>();

  let channel = registry
    .get(&user_channel_id)
    .await
    .ok_or_else(|| crate::Error::ChannelNotFound(0))?;

  let channel_id = channel.id();

  let msg_type = match message_type.as_str() {
    "data" => ChannelMessageType::Data,
    "error" => ChannelMessageType::Error,
    "close" => {
      registry.unregister(&user_channel_id).await;
      return Ok(());
    }
    "ping" => ChannelMessageType::Ping,
    "pong" => ChannelMessageType::Pong,
    _ => ChannelMessageType::Data,
  };

  manager
    .handle_incoming_message(channel_id, msg_type, payload)
    .await?;

  if let Some(idx) = index {
    let _ = channel.send(serde_json::json!({
      "type": "ack",
      "index": idx
    }));
  }

  Ok(())
}

#[command(root = "crate")]
async fn send_binary<R: Runtime>(
  app: AppHandle<R>,
  user_channel_id: String,
  data: Vec<u8>,
) -> crate::Result<()> {
  let registry = app.state::<ConnectionRegistry<R>>();

  let channel = registry
    .get(&user_channel_id)
    .await
    .ok_or_else(|| crate::Error::ChannelNotFound(0))?;

  let message = ChannelEvent::Binary(data);
  channel
    .incoming_tx()
    .send(message)
    .map_err(|_| crate::Error::ChannelClosed)?;

  Ok(())
}

#[command(root = "crate")]
async fn broadcast<R: Runtime>(
  app: AppHandle<R>,
  message: serde_json::Value,
) -> crate::Result<()> {
  let manager = app.state::<PersistentChannelManager<R>>();
  manager.broadcast(message).await
}

#[command(root = "crate")]
async fn list_channels<R: Runtime>(app: AppHandle<R>) -> crate::Result<Vec<ChannelInfo>> {
  let registry = app.state::<ConnectionRegistry<R>>();
  let channels = registry.get_all().await;

  Ok(
    channels
      .into_iter()
      .map(|c| ChannelInfo {
        id: c.id(),
        connected: !c.is_closed(),
      })
      .collect(),
  )
}

#[default_runtime(crate::Wry, wry)]
pub struct PersistentChannelPluginBuilder<R: Runtime> {
  on_connect: Option<Box<dyn Fn(AppHandle<R>, PersistentChannel<R>) + Send + Sync + 'static>>,
}

impl<R: Runtime> Default for PersistentChannelPluginBuilder<R> {
  fn default() -> Self {
    Self::new()
  }
}

impl<R: Runtime> PersistentChannelPluginBuilder<R> {
  pub fn new() -> Self {
    Self { on_connect: None }
  }

  pub fn on_connect<F>(mut self, f: F) -> Self
  where
    F: Fn(AppHandle<R>, PersistentChannel<R>) + Send + Sync + 'static,
  {
    self.on_connect = Some(Box::new(f));
    self
  }

  pub fn build(self) -> TauriPlugin<R> {
    let on_connect = self.on_connect;

    PluginBuilder::new(PERSISTENT_CHANNEL_PLUGIN_NAME)
      .setup(move |app, _api| {
        if !CHANNEL_MANAGER_INITIALIZED.swap(true, Ordering::SeqCst) {
          let manager = if let Some(f) = on_connect {
            let app_clone = app.clone();
            PersistentChannelManager::new().with_on_connect(move |channel| {
              f(app_clone.clone(), channel);
            })
          } else {
            PersistentChannelManager::new()
          };

          app.manage(manager);
          app.manage(ConnectionRegistry::<R>::new());
        }
        Ok(())
      })
      .js_init_script(include_str!("../../scripts/persistent-channel.js"))
      .invoke_handler(crate::generate_handler![
        #![plugin(__TAURI_PERSISTENT_CHANNEL__)]
        connect,
        send_message,
        send_binary,
        broadcast,
        list_channels
      ])
      .build()
  }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  PersistentChannelPluginBuilder::new().build()
}

pub fn builder<R: Runtime>() -> PersistentChannelPluginBuilder<R> {
  PersistentChannelPluginBuilder::new()
}

pub mod async_channel {
  use super::*;
  use futures::Stream;

  #[default_runtime(crate::Wry, wry)]
  pub struct AsyncChannel<R: Runtime> {
    inner: PersistentChannel<R>,
  }

  impl<R: Runtime> Clone for AsyncChannel<R> {
    fn clone(&self) -> Self {
      Self {
        inner: self.inner.clone(),
      }
    }
  }

  impl<R: Runtime> AsyncChannel<R> {
    pub fn new(inner: PersistentChannel<R>) -> Self {
      Self { inner }
    }

    pub fn id(&self) -> u64 {
      self.inner.id()
    }

    pub fn is_closed(&self) -> bool {
      self.inner.is_closed()
    }

    pub async fn send<T: Serialize>(&self, data: T) -> crate::Result<()> {
      self.inner.send(data)
    }

    pub async fn send_bytes(&self, bytes: Vec<u8>) -> crate::Result<()> {
      self.inner.send_bytes(bytes)
    }

    pub async fn recv(&self) -> Option<ChannelEvent> {
      self.inner.recv().await
    }

    pub fn into_stream(self) -> impl Stream<Item = ChannelEvent> {
      self.inner.into_stream()
    }

    pub fn close(&self) -> crate::Result<()> {
      self.inner.close()
    }

    pub fn into_inner(self) -> PersistentChannel<R> {
      self.inner
    }
  }

  impl<R: Runtime> From<PersistentChannel<R>> for AsyncChannel<R> {
    fn from(channel: PersistentChannel<R>) -> Self {
      Self::new(channel)
    }
  }
}
