// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::ops::Deref;

use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;
use tauri_runtime::window::is_label_valid;

use crate::plugin::{Builder, TauriPlugin};
use crate::{command, ipc::CallbackFn, EventId, Result, Runtime};
use crate::{AppHandle, Emitter, Webview};

use super::{is_event_name_valid, EventTarget};

pub struct EventName(String);

impl Deref for EventName {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'de> Deserialize<'de> for EventName {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let event_id = String::deserialize(deserializer)?;
    if is_event_name_valid(&event_id) {
      Ok(EventName(event_id))
    } else {
      Err(serde::de::Error::custom(
        "Event name must include only alphanumeric characters, `-`, `/`, `:` and `_`.",
      ))
    }
  }
}

pub struct WebviewLabel(String);

impl AsRef<str> for WebviewLabel {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl<'de> Deserialize<'de> for WebviewLabel {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let event_id = String::deserialize(deserializer)?;
    if is_label_valid(&event_id) {
      Ok(WebviewLabel(event_id))
    } else {
      Err(serde::de::Error::custom(
        "Webview label must include only alphanumeric characters, `-`, `/`, `:` and `_`.",
      ))
    }
  }
}

#[command(root = "crate")]
pub async fn listen<R: Runtime>(
  webview: Webview<R>,
  event: EventName,
  target: EventTarget,
  handler: CallbackFn,
) -> Result<EventId> {
  webview.listen_js(&event, target, handler)
}

#[command(root = "crate")]
pub async fn unlisten<R: Runtime>(
  webview: Webview<R>,
  event: EventName,
  event_id: EventId,
) -> Result<()> {
  webview.unlisten_js(&event, event_id)
}

#[command(root = "crate")]
pub async fn emit<R: Runtime>(
  app: AppHandle<R>,
  event: EventName,
  payload: Option<JsonValue>,
) -> Result<()> {
  app.emit(&event, payload)
}

#[command(root = "crate")]
pub async fn emit_to<R: Runtime>(
  app: AppHandle<R>,
  target: EventTarget,
  event: EventName,
  payload: Option<JsonValue>,
) -> Result<()> {
  app.emit_to(target, &event, payload)
}

/// Initializes the event plugin.
pub(crate) fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("event")
    .invoke_handler(crate::generate_handler![listen, unlisten, emit, emit_to])
    .build()
}
