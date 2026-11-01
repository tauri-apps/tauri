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

use crate::ipc::CallbackFn;

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
}

impl EmitArgs {
  pub fn new<S: Serialize>(event: EventName<&str>, payload: &S) -> crate::Result<Self> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("window::emit::serialize").entered();
    Ok(EmitArgs {
      event: event.into_owned(),
      payload: serde_json::to_string(payload)?,
    })
  }

  pub fn new_str(event: EventName<&str>, payload: String) -> crate::Result<Self> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("window::emit::json").entered();
    Ok(EmitArgs {
      event: event.into_owned(),
      payload,
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

/// Script that tells the page to fetch the events queued for it.
///
/// It does not contain any event data, so it is the same every time.
///
/// The page fetches one batch at a time, so events are dispatched in the order they were emitted.
pub(crate) fn notify_js_script(event_emit_function_name: &str) -> String {
  format!("(function () {{ const fn = window['{event_emit_function_name}']; fn && fn() }})()")
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
          const listener = listeners[{event_id_arg}]
          if (listener) {{
            window.__TAURI_INTERNALS__.unregisterCallback(listener.handlerId)
          }}
        }}
      }})()
    ",
  )
}

pub(crate) fn event_initialization_script(function_name: &str, listeners: &str) -> String {
  // starts with `;` because this is appended to another script and a `(` would otherwise be parsed as a call
  format!(
    ";(function () {{
      let fetching = false
      let fetchAgain = false

      function dispatch(item) {{
        const eventData = {{ event: item.event, payload: item.payload }}
        const listeners = (window['{listeners}'] && window['{listeners}'][item.event]) || []
        for (const id of item.ids) {{
          const listener = listeners[id]
          if (listener) {{
            eventData.id = id
            window.__TAURI_INTERNALS__.runCallback(listener.handlerId, eventData)
          }}
        }}
      }}

      function fetchEvents() {{
        if (fetching) {{
          fetchAgain = true
          return
        }}
        fetching = true
        window.__TAURI_INTERNALS__
          .invoke('{fetch_command}', null, {{ headers: {{ '{queue_header}': '1' }} }})
          .then((items) => {{
            for (const item of items) dispatch(item)
          }})
          .catch(console.error)
          .finally(() => {{
            fetching = false
            if (fetchAgain) {{
              fetchAgain = false
              fetchEvents()
            }}
          }})
      }}

      Object.defineProperty(window, '{function_name}', {{ value: fetchEvents }})
    }})();
  ",
    fetch_command = crate::ipc::channel::FETCH_CHANNEL_DATA_COMMAND,
    queue_header = crate::ipc::channel::EVENT_QUEUE_HEADER_NAME,
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
    assert_eq!(
      "only alphanumeric, '-', '/', ':', '_' permitted for event names: \"some\\r illegal event name\"",
      s
    );
  }
}
