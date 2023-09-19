// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fmt, hash::Hash};
use uuid::Uuid;

mod commands;
mod listener;
pub(crate) use listener::Listeners;

use crate::{
  plugin::{Builder, TauriPlugin},
  Runtime,
};

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

/// Represents an event handler.
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EventHandler(Uuid);

impl fmt::Display for EventHandler {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

/// An event that was triggered.
#[derive(Debug, Clone)]
pub struct Event {
  id: EventHandler,
  data: Option<String>,
}

impl Event {
  /// The [`EventHandler`] that was triggered.
  pub fn id(&self) -> EventHandler {
    self.id
  }

  /// The event payload.
  pub fn payload(&self) -> Option<&str> {
    self.data.as_deref()
  }
}

/// Initializes the event plugin.
pub(crate) fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("event")
    .invoke_handler(crate::generate_handler![
      commands::listen,
      commands::unlisten,
      commands::emit,
    ])
    .build()
}

pub fn unlisten_js(listeners_object_name: String, event_name: String, event_id: usize) -> String {
  format!(
    "
      (function () {{
        const listeners = (window['{listeners_object_name}'] || {{}})['{event_name}']
        if (listeners) {{
          const index = window['{listeners_object_name}']['{event_name}'].findIndex(e => e.id === {event_id})
          if (index > -1) {{
            window['{listeners_object_name}']['{event_name}'].splice(index, 1)
          }}
        }}
      }})()
    ",
  )
}

pub fn listen_js(
  listeners_object_name: String,
  event: String,
  event_id: usize,
  window_label: Option<String>,
  handler: String,
) -> String {
  format!(
    "
    (function () {{
      if (window['{listeners}'] === void 0) {{
        Object.defineProperty(window, '{listeners}', {{ value: Object.create(null) }});
      }}
      if (window['{listeners}'][{event}] === void 0) {{
        Object.defineProperty(window['{listeners}'], {event}, {{ value: [] }});
      }}
      const eventListeners = window['{listeners}'][{event}]
      const listener = {{
        id: {event_id},
        windowLabel: {window_label},
        handler: {handler}
      }};
      eventListeners.push(listener);
    }})()
  ",
    listeners = listeners_object_name,
    window_label = if let Some(l) = window_label {
      crate::runtime::window::assert_label_is_valid(&l);
      format!("'{l}'")
    } else {
      "null".to_owned()
    },
  )
}
