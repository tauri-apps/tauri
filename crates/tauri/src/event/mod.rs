// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod listener;
pub(crate) mod plugin;
use std::{convert::Infallible, str::FromStr};

pub(crate) use listener::Listeners;
use serde::{Deserialize, Serialize};

mod event_name;

pub(crate) use event_name::EventName;

use crate::{
  event::plugin::{FETCH_EVENT_PAYLOAD_COMMAND, PAYLOAD_ID_HEADER_NAME},
  ipc::CallbackFn,
  manager::BINARY_EVENT_PAYLOAD_PREFIX,
};

/// Unique id of an event.
pub type EventId = u32;

/// Event Target
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(tag = "kind")]
#[non_exhaustive]
pub enum EventTarget {
  /// Any and all event targets.
  Any,

  /// Any [`Window`](crate::Window), [`Webview`](crate::Webview) or [`WebviewWindow`](crate::WebviewWindow) that have this label.
  AnyLabel {
    /// Target label.
    label: String,
  },

  /// [`App`](crate::App) and [`AppHandle`](crate::AppHandle) targets.
  App,

  /// [`Window`](crate::Window) target.
  Window {
    /// window label.
    label: String,
  },

  /// [`Webview`](crate::Webview) target.
  Webview {
    /// webview label.
    label: String,
  },

  /// [`WebviewWindow`](crate::WebviewWindow) target.
  WebviewWindow {
    /// webview window label.
    label: String,
  },
}

impl EventTarget {
  /// [`Self::Any`] target.
  pub fn any() -> Self {
    Self::Any
  }

  /// [`Self::App`] target.
  pub fn app() -> Self {
    Self::App
  }

  /// [`Self::AnyLabel`] target.
  pub fn labeled(label: impl Into<String>) -> Self {
    Self::AnyLabel {
      label: label.into(),
    }
  }

  /// [`Self::Window`] target.
  pub fn window(label: impl Into<String>) -> Self {
    Self::Window {
      label: label.into(),
    }
  }

  /// [`Self::Webview`] target.
  pub fn webview(label: impl Into<String>) -> Self {
    Self::Webview {
      label: label.into(),
    }
  }

  /// [`Self::WebviewWindow`] target.
  pub fn webview_window(label: impl Into<String>) -> Self {
    Self::WebviewWindow {
      label: label.into(),
    }
  }
}

impl<T: AsRef<str>> From<T> for EventTarget {
  fn from(value: T) -> Self {
    Self::AnyLabel {
      label: value.as_ref().to_string(),
    }
  }
}

impl FromStr for EventTarget {
  type Err = Infallible;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self::AnyLabel {
      label: s.to_string(),
    })
  }
}

/// Serialized emit arguments.
#[derive(Clone)]
pub struct EmitArgs {
  /// event name.
  event: EventName,
  /// Serialized payload.
  payload: String,
  /// Raw payload bytes.
  raw: Option<Vec<u8>>,
}

impl EmitArgs {
  pub fn new<S: Serialize>(event: EventName<&str>, payload: &S) -> crate::Result<Self> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("window::emit::serialize").entered();
    Ok(EmitArgs {
      event: event.into_owned(),
      payload: serde_json::to_string(payload)?,
      raw: None,
    })
  }

  pub fn new_raw<S: Serialize>(
    event: EventName<&str>,
    payload: &S,
    raw_data: Vec<u8>,
  ) -> crate::Result<Self> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("window::emit::serialize").entered();
    Ok(EmitArgs {
      event: event.into_owned(),
      payload: serde_json::to_string(payload)?,
      raw: Some(raw_data),
    })
  }

  pub fn new_str(event: EventName<&str>, payload: String) -> Self {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("window::emit::json").entered();
    EmitArgs {
      event: event.into_owned(),
      payload,
      raw: None,
    }
  }
}

/// An event that was emitted.
#[derive(Debug, Clone)]
pub struct Event {
  id: EventId,
  data: String,
  raw: Option<Vec<u8>>,
}

impl Event {
  fn new(id: EventId, data: String, raw: Option<Vec<u8>>) -> Self {
    Self { id, data, raw }
  }

  /// The [`EventId`] of the handler that was triggered.
  pub fn id(&self) -> EventId {
    self.id
  }

  /// The event payload.
  ///
  /// Note that you must use [`Self::payload_raw`] for events emitted with a binary payload.
  pub fn payload(&self) -> &str {
    &self.data
  }

  /// Consumes the event and return its payload.
  ///
  /// Note that you must use [`Self::into_payload_raw`] for events emitted with a binary payload.
  pub fn into_payload(self) -> String {
    self.data
  }

  /// The event payload bytes.
  pub fn payload_raw(&self) -> &[u8] {
    self.raw.as_deref().unwrap_or(self.data.as_bytes())
  }

  /// Consumes the event and return its payload bytes.
  pub fn into_payload_raw(self) -> Vec<u8> {
    self.raw.unwrap_or_else(|| self.data.into_bytes())
  }
}

pub(crate) fn listen_js_script(
  listeners_object_name: &str,
  serialized_target: &str,
  event: EventName<&str>,
  event_id: EventId,
  handler: CallbackFn,
) -> String {
  let handler_id = handler.0;
  format!(
    "(function () {{
      if (window['{listeners_object_name}'] === void 0) {{
        Object.defineProperty(window, '{listeners_object_name}', {{ value: Object.create(null) }});
      }}
      if (window['{listeners_object_name}']['{event}'] === void 0) {{
        Object.defineProperty(window['{listeners_object_name}'], '{event}', {{ value: Object.create(null) }});
      }}
      const eventListeners = window['{listeners_object_name}']['{event}']
      const listener = {{
        target: {serialized_target},
        handlerId: {handler_id}
      }};
      Object.defineProperty(eventListeners, '{event_id}', {{ value: listener, configurable: true }});
    }})()
  ",
  )
}

pub(crate) fn emit_js_script(
  event_emit_function_name: &str,
  emit_args: &EmitArgs,
  serialized_ids: &str,
) -> crate::Result<String> {
  Ok(format!(
    "(function () {{ const fn = window['{}']; fn && fn({{event: '{}', payload: {}}}, {ids}) }})()",
    event_emit_function_name,
    emit_args.event,
    emit_args.payload,
    ids = serialized_ids,
  ))
}

pub(crate) fn unlisten_js_script(
  listeners_object_name: &str,
  event_arg: &str,
  event_id_arg: &str,
) -> String {
  format!(
    "(function () {{
        const listeners = (window['{listeners_object_name}'] || {{}})[{event_arg}]
        if (listeners) {{
          window.__TAURI_INTERNALS__.unregisterCallback(listeners[{event_id_arg}].handlerId)
        }}
      }})()
    ",
  )
}

pub(crate) fn event_initialization_script(function_name: &str, listeners: &str) -> String {
  format!(
    "Object.defineProperty(window, '{function_name}', {{
      value: async function (eventData, ids) {{
        const listeners = (window['{listeners}'] && window['{listeners}'][eventData.event]) || []

        if (typeof eventData.payload === 'string' && eventData.payload.startsWith('{BINARY_EVENT_PAYLOAD_PREFIX}')) {{
          const payloadId = eventData.payload.slice('{BINARY_EVENT_PAYLOAD_PREFIX}'.length)
          const data = await window.__TAURI_INTERNALS__.invoke('{FETCH_EVENT_PAYLOAD_COMMAND}', null, {{ headers: {{ '{PAYLOAD_ID_HEADER_NAME}': payloadId }} }})
          eventData.payload = data
        }}

        for (const id of ids) {{
          const listener = listeners[id]
          if (listener) {{
            eventData.id = id
            window.__TAURI_INTERNALS__.runCallback(listener.handlerId, eventData)
          }}
        }}
      }}
    }});
  ",
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_illegal_event_name() {
    let s = EventName::new("some\r illegal event name")
      .unwrap_err()
      .to_string();
    assert_eq!("only alphanumeric, '-', '/', ':', '_' permitted for event names: \"some\\r illegal event name\"", s);
  }
}
