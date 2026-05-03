// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Weak,
  },
};

use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tauri_macros::default_runtime;
use tokio::sync::{mpsc, Mutex, RwLock};

use crate::{
  ipc::{InvokeResponseBody, IpcResponse},
  AppHandle, Manager, Runtime, Webview,
};

use super::{
  channel::send_channel_response,
  format_callback::format_raw_js,
  JavaScriptChannelId,
};

static PERSISTENT_CHANNEL_COUNTER: AtomicU64 = AtomicU64::new(1);

pub type PersistentChannelId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelMessageType {
  Data,
  Error,
  Close,
  Ping,
  Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
  pub channel_id: PersistentChannelId,
  pub message_type: ChannelMessageType,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub payload: Option<serde_json::Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub index: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum ChannelEvent {
  Message(serde_json::Value),
  Binary(Vec<u8>),
  Error(String),
  Close,
}

pub type MessageSender = mpsc::UnboundedSender<ChannelEvent>;
pub type MessageReceiver = mpsc::UnboundedReceiver<ChannelEvent>;

struct PersistentChannelInner<R: Runtime> {
  id: PersistentChannelId,
  app_handle: AppHandle<R>,
  webview_label: String,
  tx: MessageSender,
  rx: Arc<Mutex<MessageReceiver>>,
  callback_fn: u32,
  message_index: AtomicU64,
  is_closed: AtomicU64,
}

#[default_runtime(crate::Wry, wry)]
pub struct PersistentChannel<R: Runtime> {
  inner: Arc<PersistentChannelInner<R>>,
}

impl<R: Runtime> Clone for PersistentChannel<R> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
    }
  }
}

#[cfg(feature = "specta")]
const _: () = {
  #[derive(specta::Type)]
  #[specta(remote = super::PersistentChannel)]
  #[allow(dead_code, non_camel_case_types)]
  struct TAURI_PERSISTENT_CHANNEL<R: Runtime>(std::marker::PhantomData<R>);
};

#[derive(Default)]
pub struct PersistentChannelStore<R: Runtime> {
  channels: RwLock<HashMap<PersistentChannelId, Weak<PersistentChannelInner<R>>>>,
}

#[default_runtime(crate::Wry, wry)]
pub struct PersistentChannelManager<R: Runtime> {
  store: Arc<PersistentChannelStore<R>>,
  on_connect: Option<Box<dyn Fn(PersistentChannel<R>) + Send + Sync + 'static>>,
}

impl<R: Runtime> Clone for PersistentChannelManager<R> {
  fn clone(&self) -> Self {
    Self {
      store: self.store.clone(),
      on_connect: None,
    }
  }
}

impl<R: Runtime> PersistentChannel<R> {
  pub fn new(webview: &Webview<R>, callback_fn: u32) -> Self {
    let (tx, rx) = mpsc::unbounded_channel();
    let id = PERSISTENT_CHANNEL_COUNTER.fetch_add(1, Ordering::Relaxed);

    let inner = Arc::new(PersistentChannelInner {
      id,
      app_handle: webview.app_handle.clone(),
      webview_label: webview.label().to_string(),
      tx,
      rx: Arc::new(Mutex::new(rx)),
      callback_fn,
      message_index: AtomicU64::new(0),
      is_closed: AtomicU64::new(0),
    });

    Self { inner }
  }

  pub fn id(&self) -> PersistentChannelId {
    self.inner.id
  }

  pub fn is_closed(&self) -> bool {
    self.inner.is_closed.load(Ordering::Relaxed) != 0
  }

  pub fn send<T: Serialize>(&self, data: T) -> crate::Result<()> {
    self.send_inner(serde_json::to_value(data)?)
  }

  pub fn send_bytes(&self, bytes: Vec<u8>) -> crate::Result<()> {
    self.send_bytes_inner(bytes)
  }

  fn send_inner(&self, value: serde_json::Value) -> crate::Result<()> {
    if self.is_closed() {
      return Err(crate::Error::ChannelClosed);
    }

    let json_string = serde_json::to_string(&value)?;
    let body = InvokeResponseBody::Json(json_string);

    self.send_response(body)
  }

  fn send_bytes_inner(&self, bytes: Vec<u8>) -> crate::Result<()> {
    if self.is_closed() {
      return Err(crate::Error::ChannelClosed);
    }

    let body = InvokeResponseBody::Raw(bytes);
    self.send_response(body)
  }

  fn get_webview(&self) -> crate::Result<Webview<R>> {
    self
      .inner
      .app_handle
      .get_webview(&self.inner.webview_label)
      .ok_or_else(|| crate::Error::WebviewNotFound)
  }

  fn send_response(&self, body: InvokeResponseBody) -> crate::Result<()> {
    let webview = self.get_webview()?;

    let index = self.inner.message_index.fetch_add(1, Ordering::Relaxed);
    let extra_js = format!("index: {index}, channelId: {}", self.inner.id);

    send_channel_response(&webview, self.inner.callback_fn, body, Some(&extra_js))
  }

  pub fn send_error(&self, error: String) -> crate::Result<()> {
    self.send(serde_json::json!({
      "error": error
    }))
  }

  pub fn close(&self) -> crate::Result<()> {
    if self
      .inner
      .is_closed
      .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
      .is_ok()
    {
      self.inner.tx.send(ChannelEvent::Close).ok();

      if let Ok(webview) = self.get_webview() {
        let index = self.inner.message_index.load(Ordering::Relaxed);
        let _ = webview.eval(format_raw_js(
          self.inner.callback_fn,
          format!(
            "{{ end: true, index: {index}, channelId: {} }}",
            self.inner.id
          ),
        ));
      }
    }
    Ok(())
  }

  pub fn receiver(&self) -> Arc<Mutex<MessageReceiver>> {
    self.inner.rx.clone()
  }

  pub fn incoming_tx(&self) -> MessageSender {
    self.inner.tx.clone()
  }

  pub async fn recv(&self) -> Option<ChannelEvent> {
    let mut rx = self.inner.rx.lock().await;
    rx.recv().await
  }

  pub fn into_stream(self) -> impl Stream<Item = ChannelEvent> {
    futures::stream::unfold(self, |channel| async move {
      let event = channel.recv().await;
      event.map(|e| (e, channel))
    })
  }
}

impl<R: Runtime> Drop for PersistentChannelInner<R> {
  fn drop(&mut self) {
    let _ = self.is_closed.store(1, Ordering::Relaxed);
  }
}

impl<R: Runtime> PersistentChannelStore<R> {
  pub fn new() -> Self {
    Self::default()
  }

  pub async fn register(&self, channel: &PersistentChannel<R>) {
    let mut channels = self.channels.write().await;
    channels.insert(channel.id(), Arc::downgrade(&channel.inner));
  }

  pub async fn unregister(&self, channel_id: PersistentChannelId) {
    let mut channels = self.channels.write().await;
    channels.remove(&channel_id);
  }

  pub async fn get(&self, channel_id: PersistentChannelId) -> Option<PersistentChannel<R>> {
    let channels = self.channels.read().await;
    channels
      .get(&channel_id)
      .and_then(|weak| weak.upgrade())
      .map(|inner| PersistentChannel { inner })
  }

  pub async fn get_all(&self) -> Vec<PersistentChannel<R>> {
    let channels = self.channels.read().await;
    channels
      .values()
      .filter_map(|weak| weak.upgrade())
      .map(|inner| PersistentChannel { inner })
      .collect()
  }

  pub async fn cleanup(&self) {
    let mut channels = self.channels.write().await;
    channels.retain(|_, weak| weak.strong_count() > 0);
  }
}

impl<R: Runtime> PersistentChannelManager<R> {
  pub fn new() -> Self {
    Self {
      store: Arc::new(PersistentChannelStore::new()),
      on_connect: None,
    }
  }

  pub fn with_on_connect<F>(mut self, f: F) -> Self
  where
    F: Fn(PersistentChannel<R>) + Send + Sync + 'static,
  {
    self.on_connect = Some(Box::new(f));
    self
  }

  pub async fn create_channel(
    &self,
    webview: &Webview<R>,
    callback_channel_id: JavaScriptChannelId,
  ) -> PersistentChannel<R> {
    let callback_fn = callback_channel_id.callback_id();
    let channel = PersistentChannel::new(webview, callback_fn);
    self.store.register(&channel).await;

    if let Some(on_connect) = &self.on_connect {
      on_connect(channel.clone());
    }

    channel
  }

  pub async fn get_channel(&self, channel_id: PersistentChannelId) -> Option<PersistentChannel<R>> {
    self.store.get(channel_id).await
  }

  pub async fn handle_incoming_message(
    &self,
    channel_id: PersistentChannelId,
    message_type: ChannelMessageType,
    payload: Option<serde_json::Value>,
  ) -> crate::Result<()> {
    let channel = self
      .store
      .get(channel_id)
      .await
      .ok_or_else(|| crate::Error::ChannelNotFound(channel_id))?;

    match message_type {
      ChannelMessageType::Data => {
        if let Some(payload) = payload {
          channel
            .incoming_tx()
            .send(ChannelEvent::Message(payload))
            .map_err(|_| crate::Error::ChannelClosed)?;
        }
      }
      ChannelMessageType::Error => {
        if let Some(payload) = payload {
          let error_msg = payload
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| payload.to_string());
          channel
            .incoming_tx()
            .send(ChannelEvent::Error(error_msg))
            .map_err(|_| crate::Error::ChannelClosed)?;
        }
      }
      ChannelMessageType::Close => {
        channel.close()?;
        self.store.unregister(channel_id).await;
      }
      ChannelMessageType::Ping => {
        channel.send(serde_json::json!({ "type": "pong" }))?;
      }
      ChannelMessageType::Pong => {}
    }

    Ok(())
  }

  pub async fn broadcast<T: Serialize + Clone>(&self, data: T) -> crate::Result<()> {
    let channels = self.store.get_all().await;
    for channel in channels {
      let _ = channel.send(data.clone());
    }
    Ok(())
  }

  pub fn store(&self) -> Arc<PersistentChannelStore<R>> {
    self.store.clone()
  }
}

impl<R: Runtime> Default for PersistentChannelManager<R> {
  fn default() -> Self {
    Self::new()
  }
}

#[default_runtime(crate::Wry, wry)]
pub struct StreamSender<T: IpcResponse, R: Runtime> {
  channel: PersistentChannel<R>,
  _phantom: std::marker::PhantomData<T>,
}

impl<R: Runtime, T: IpcResponse> StreamSender<T, R> {
  pub fn new(channel: PersistentChannel<R>) -> Self {
    Self {
      channel,
      _phantom: std::marker::PhantomData,
    }
  }

  pub fn send(&self, item: T) -> crate::Result<()> {
    let body = item.body()?;
    match body {
      InvokeResponseBody::Json(json) => {
        self.channel.send_inner(serde_json::from_str(&json)?)
      }
      InvokeResponseBody::Raw(bytes) => self.channel.send_bytes_inner(bytes),
    }
  }

  pub fn close(&self) -> crate::Result<()> {
    self.channel.close()
  }

  pub fn channel(&self) -> &PersistentChannel<R> {
    &self.channel
  }
}

impl<R: Runtime> IpcResponse for ChannelMessage {
  fn body(self) -> crate::Result<InvokeResponseBody> {
    serde_json::to_string(&self)
      .map(InvokeResponseBody::Json)
      .map_err(Into::into)
  }
}

pub mod async_channel {
  use super::*;

  #[default_runtime(crate::Wry, wry)]
  pub struct AsyncChannel<R: Runtime> {
    channel: PersistentChannel<R>,
  }

  impl<R: Runtime> AsyncChannel<R> {
    pub fn new(channel: PersistentChannel<R>) -> Self {
      Self { channel }
    }

    pub async fn send<T: Serialize>(&self, data: T) -> crate::Result<()> {
      self.channel.send(data)
    }

    pub async fn send_bytes(&self, bytes: Vec<u8>) -> crate::Result<()> {
      self.channel.send_bytes(bytes)
    }

    pub async fn recv(&self) -> Option<ChannelEvent> {
      self.channel.recv().await
    }

    pub fn close(&self) -> crate::Result<()> {
      self.channel.close()
    }

    pub fn id(&self) -> PersistentChannelId {
      self.channel.id()
    }

    pub fn into_inner(self) -> PersistentChannel<R> {
      self.channel
    }
  }
}
