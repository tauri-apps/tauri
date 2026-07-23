// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Deserializer};

/// Checks if an event name is valid.
fn is_event_name_valid(event: &str) -> bool {
  event
    .chars()
    .all(|c| c.is_alphanumeric() || c == '-' || c == '/' || c == ':' || c == '_')
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct EventName<S = String>(S);

impl Copy for EventName<&str> {}

impl<S: AsRef<str>> EventName<S> {
  pub(crate) fn new(s: S) -> crate::Result<EventName<S>> {
    if !is_event_name_valid(s.as_ref()) {
      return Err(crate::Error::IllegalEventName(s.as_ref().to_string()));
    }
    Ok(EventName(s))
  }

  pub(crate) fn as_str_event(&self) -> EventName<&str> {
    EventName(self.0.as_ref())
  }

  pub(crate) fn as_str(&self) -> &str {
    self.0.as_ref()
  }
}

impl<S: std::fmt::Display> std::fmt::Display for EventName<S> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.0.fmt(f)
  }
}

impl EventName<&'static str> {
  // this convenience method is for using in const contexts to discharge the preconditions
  // &'static prevents using this function accidentally with dynamically built string slices
  pub(crate) const fn from_str(s: &'static str) -> EventName<&'static str> {
    EventName(s)
  }
}

impl EventName<&str> {
  pub fn into_owned(self) -> EventName {
    EventName(self.0.to_string())
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
