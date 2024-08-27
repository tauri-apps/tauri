// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod listener;
pub(crate) mod plugin;
use std::{convert::Infallible, str::FromStr};

pub(crate) use listener::Listeners;
use serde::{Deserialize, Serialize};

/// Checks if an event name is valid.
pub fn is_event_name_valid(event: &str) -> bool {
  event
    .chars()
    .all(|c| c.is_alphanumeric() || c == '-' || c == '/' || c == ':' || c == '_')
}

pub fn assert_event_name_is_valid(event: &str) {
  assert!(
    is_event_name_valid(event),
    "Event name must include only alphanumeric characters, `-`, `/`, `:` and `_`."
  );
}

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
  /// Raw event name.
  pub event_name: String,
  /// Serialized event name.
  pub event: String,
  /// Serialized payload.
  pub payload: String,
}

impl EmitArgs {
  pub fn new<S: Serialize>(event: &str, payload: S) -> crate::Result<Self> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("window::emit::serialize").entered();
    Ok(EmitArgs {
      event_name: event.into(),
      event: serde_json::to_string(event)?,
      payload: serde_json::to_string(&payload)?,
    })
  }
}

/// An event that was emitted.
#[derive(Debug, Clone)]
pub struct Event {
  id: EventId,
  data: String,
}

impl Event {
  fn new(id: EventId, data: String) -> Self {
    Self { id, data }
  }

  /// The [`EventId`] of the handler that was triggered.
  pub fn id(&self) -> EventId {
    self.id
  }

  /// The event payload.
  pub fn payload(&self) -> &str {
    &self.data
  }
}

pub fn listen_js_script(
  listeners_object_name: &str,
  serialized_target: &str,
  event: &str,
  event_id: EventId,
  handler: &str,
) -> String {
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
        handler: {handler}
      }};
      Object.defineProperty(eventListeners, '{event_id}', {{ value: listener, configurable: true }});
    }})()
  ",
  )
}

pub fn emit_js_script(
  event_emit_function_name: &str,
  emit_args: &EmitArgs,
  serialized_ids: &str,
) -> crate::Result<String> {
  Ok(format!(
    "(function () {{ const fn = window['{}']; fn && fn({{event: {}, payload: {}}}, {ids}) }})()",
    event_emit_function_name,
    emit_args.event,
    emit_args.payload,
    ids = serialized_ids,
  ))
}

pub fn unlisten_js_script(
  listeners_object_name: &str,
  event_name: &str,
  event_id: EventId,
) -> String {
  format!(
    "(function () {{
        const listeners = (window['{listeners_object_name}'] || {{}})['{event_name}']
        if (listeners) {{
          delete window['{listeners_object_name}']['{event_name}'][{event_id}];
        }}
      }})()
    ",
  )
}

pub fn event_initialization_script(function: &str, listeners: &str) -> String {
  format!(
    "Object.defineProperty(window, '{function}', {{
      value: function (eventData, ids) {{
        const listeners = (window['{listeners}'] && window['{listeners}'][eventData.event]) || []
        for (const id of ids) {{
          const listener = listeners[id]
          if (listener && listener.handler) {{
            eventData.id = id
            listener.handler(eventData)
          }}
        }}
      }}
    }});
  "
  )
}
