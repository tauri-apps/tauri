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
  // dispatch the event to Rust listeners
  window.trigger(
    &event.0,
    payload.as_ref().and_then(|p| {
      serde_json::to_string(&p)
        .map_err(|e| {
          #[cfg(debug_assertions)]
          eprintln!("{e}");
          e
        })
        .ok()
    }),
  );

  // emit event to JS
  if let Some(target) = window_label {
    window.emit_to(&target.0, &event.0, payload)
  } else {
    window.emit_all(&event.0, payload)
  }
}

/// Initializes the event plugin.
pub(crate) fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("event")
    .invoke_handler(crate::generate_handler![listen, unlisten, emit,])
    .build()
}
