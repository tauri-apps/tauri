// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeContext;
use crate::{sealed::ManagerBase, Manager, Runtime, Window};
use serde::Deserialize;
use tauri_macros::CommandModule;

/// The API descriptor.
#[derive(Deserialize, CommandModule)]
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
  fn listen<R: Runtime>(
    context: InvokeContext<R>,
    event: String,
    handler: String,
  ) -> crate::Result<u64> {
    let event_id = rand::random();
    context
      .window
      .eval(&listen_js(&context.window, event.clone(), event_id, handler))?;
    context.window.register_js_listener(event, event_id);
    Ok(event_id)
  }

  fn unlisten<R: Runtime>(context: InvokeContext<R>, event_id: u64) -> crate::Result<()> {
    context
      .window
      .eval(&unlisten_js(&context.window, event_id))?;
    context.window.unregister_js_listener(event_id);
    Ok(())
  }

  fn emit<R: Runtime>(
    context: InvokeContext<R>,
    event: String,
    window_label: Option<String>,
    payload: Option<String>,
  ) -> crate::Result<()> {
    // dispatch the event to Rust listeners
    context.window.trigger(&event, payload.clone());

    if let Some(target) = window_label {
      context.window.emit_to(&target, &event, payload)?;
    } else {
      context.window.emit_all(&event, payload)?;
    }
    Ok(())
  }
}

pub fn unlisten_js<R: Runtime>(window: &Window<R>, event_id: u64) -> String {
  format!(
    "
      for (var event in (window['{listeners}'] || {{}})) {{
        var listeners = (window['{listeners}'] || {{}})[event]
        if (listeners) {{
          window['{listeners}'][event] = window['{listeners}'][event].filter(function (e) {{ return e.id !== {event_id} }})
        }}
      }}
    ",
    listeners = window.manager().event_listeners_object_name(),
    event_id = event_id,
  )
}

pub fn listen_js<R: Runtime>(
  window: &Window<R>,
  event: String,
  event_id: u64,
  handler: String,
) -> String {
  format!(
    "if (window['{listeners}'] === void 0) {{
      window['{listeners}'] = Object.create(null)
    }}
    if (window['{listeners}']['{event}'] === void 0) {{
      window['{listeners}']['{event}'] = []
    }}
    window['{listeners}']['{event}'].push({{
      id: {event_id},
      handler: window['{handler}']
    }});
  ",
    listeners = window.manager().event_listeners_object_name(),
    event = event,
    event_id = event_id,
    handler = handler
  )
}
