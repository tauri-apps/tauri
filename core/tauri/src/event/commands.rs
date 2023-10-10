// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{command, ipc::CallbackFn, Manager, Result, Runtime, Window};
use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;
use tauri_runtime::window::is_label_valid;

use super::is_event_name_valid;

pub struct EventId(String);

impl<'de> Deserialize<'de> for EventId {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let event_id = String::deserialize(deserializer)?;
    if is_event_name_valid(&event_id) {
      Ok(EventId(event_id))
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
  event: EventId,
  window_label: Option<WindowLabel>,
  handler: CallbackFn,
) -> Result<usize> {
  window.listen_js(window_label.map(|l| l.0), event.0, handler)
}

#[command(root = "crate")]
pub fn unlisten<R: Runtime>(window: Window<R>, event: EventId, event_id: usize) -> Result<()> {
  window.unlisten_js(event.0, event_id)
}

#[command(root = "crate")]
pub fn emit<R: Runtime>(
  window: Window<R>,
  event: EventId,
  window_label: Option<WindowLabel>,
  payload: Option<JsonValue>,
) -> Result<()> {
  window.emit_filter(&event.0, payload, |l| {
    window_label
      .as_ref()
      .map(|label| label.0 == l.label())
      .unwrap_or(true)
  })
}
