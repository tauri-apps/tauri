// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;
use tauri_runtime::window::is_label_valid;

use crate::plugin::{Builder, TauriPlugin};
use crate::{command, ipc::CallbackFn, EventId, Manager, Result, Runtime, Window};

use super::is_event_name_valid;

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

pub struct WindowLabel(String);

impl AsRef<str> for WindowLabel {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl<'de> Deserialize<'de> for WindowLabel {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let event_id = String::deserialize(deserializer)?;
    if is_label_valid(&event_id) {
      Ok(WindowLabel(event_id))
    } else {
      Err(serde::de::Error::custom(
        "Window label must include only alphanumeric characters, `-`, `/`, `:` and `_`.",
      ))
    }
  }
}

#[command(root = "crate")]
pub fn listen<R: Runtime>(
  window: Window<R>,
  event: EventName,
  window_label: Option<WindowLabel>,
  handler: CallbackFn,
) -> Result<EventId> {
  window.listen_js(window_label.map(|l| l.0), event.0, handler)
}

#[command(root = "crate")]
pub fn unlisten<R: Runtime>(window: Window<R>, event: EventName, event_id: EventId) -> Result<()> {
  window.unlisten_js(event.as_ref(), event_id)
}

#[command(root = "crate")]
pub fn emit<R: Runtime>(
  window: Window<R>,
  event: EventName,
  window_label: Option<WindowLabel>,
  payload: Option<JsonValue>,
) -> Result<()> {
  if let Some(label) = window_label {
    window.emit_filter(&event.0, payload, |w| label.as_ref() == w.label())
  } else {
    window.emit(&event.0, payload)
  }
}

/// Initializes the event plugin.
pub(crate) fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("event")
    .invoke_handler(crate::generate_handler![listen, unlisten, emit,])
    .build()
}
