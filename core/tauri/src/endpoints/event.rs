// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  endpoints::InvokeResponse,
  event::{listen_js, unlisten_js},
  runtime::Runtime,
  sealed::ManagerBase,
  Manager, Window,
};
use serde::Deserialize;

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// Listen to an event.
  Listen { event: String, handler: String },
  /// Unlisten to an event.
  #[serde(rename_all = "camelCase")]
  Unlisten { event_id: u64 },
  /// Emit an event to the webview associated with the given window.
  /// If the window_label is omitted, the event will be triggered on all listeners.
  #[serde(rename_all = "camelCase")]
  Emit {
    event: String,
    window_label: Option<String>,
    payload: Option<String>,
  },
}

impl Cmd {
  pub fn run<R: Runtime>(self, window: Window<R>) -> crate::Result<InvokeResponse> {
    match self {
      Self::Listen { event, handler } => {
        let event_id = rand::random();
        window.eval(&listen_js(
          window.manager().event_listeners_object_name(),
          format!("'{}'", event),
          event_id,
          format!("'{}'", handler),
        ))?;
        window.register_js_listener(event, event_id);
        Ok(event_id.into())
      }
      Self::Unlisten { event_id } => {
        window.eval(&unlisten_js(
          window.manager().event_listeners_object_name(),
          event_id,
        ))?;
        window.unregister_js_listener(event_id);
        Ok(().into())
      }
      Self::Emit {
        event,
        window_label,
        payload,
      } => {
        // dispatch the event to Rust listeners
        window.trigger(&event, payload.clone());

        if let Some(target) = window_label {
          window.emit_to(&target, &event, payload)?;
        } else {
          window.emit_all(&event, payload)?;
        }
        Ok(().into())
      }
    }
  }
}
