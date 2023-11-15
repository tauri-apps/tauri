// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;
use tauri_runtime::window::is_label_valid;

use crate::plugin::{Builder, TauriPlugin};
use crate::Webview;
use crate::{command, ipc::CallbackFn, EventId, Manager, Result, Runtime};

use super::{is_event_name_valid, EventSource};

pub struct EventName(String);

impl AsRef<str> for EventName {
  fn as_ref(&self) -> &str {
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
pub fn listen<R: Runtime>(
  webview: Webview<R>,
  event: EventName,
  webview_label: Option<WebviewLabel>,
  handler: CallbackFn,
) -> Result<EventId> {
  webview.listen_js(
    webview_label
      .map(|l| EventSource::Webview { label: l.0 })
      .unwrap_or(EventSource::Global),
    event.0,
    handler,
  )
}

#[command(root = "crate")]
pub fn unlisten<R: Runtime>(
  webview: Webview<R>,
  event: EventName,
  event_id: EventId,
) -> Result<()> {
  webview.unlisten_js(event.as_ref(), event_id)
}

#[command(root = "crate")]
pub fn emit<R: Runtime>(
  webview: Webview<R>,
  event: EventName,
  webview_label: Option<WebviewLabel>,
  payload: Option<JsonValue>,
) -> Result<()> {
  if let Some(label) = webview_label {
    webview.emit_to(&label.0, &event.0, payload)
  } else {
    webview.emit(&event.0, payload)
  }
}

/// Initializes the event plugin.
pub(crate) fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("event")
    .invoke_handler(crate::generate_handler![listen, unlisten, emit,])
    .build()
}
