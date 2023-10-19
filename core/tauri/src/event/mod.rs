// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod listener;
pub(crate) mod plugin;
pub(crate) use listener::Listeners;

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

/// An event that was triggered.
#[derive(Debug, Clone)]
pub struct Event {
  id: EventId,
  data: Option<String>,
}

impl Event {
  /// The [`EventId`] of the handler that was triggered.
  pub fn id(&self) -> EventId {
    self.id
  }

  /// The event payload.
  pub fn payload(&self) -> Option<&str> {
    self.data.as_deref()
  }
}

pub fn unlisten_js(listeners_object_name: &str, event_name: &str, event_id: EventId) -> String {
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
  listeners_object_name: &str,
  event: &str,
  event_id: EventId,
  window_label: Option<&str>,
  handler: &str,
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
      crate::runtime::window::assert_label_is_valid(l);
      format!("'{l}'")
    } else {
      "null".to_owned()
    },
  )
}
