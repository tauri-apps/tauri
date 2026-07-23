// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
use serde_json::Value as JsonValue;
use serialize_to_javascript::{default_template, DefaultTemplate, Template};

use crate::plugin::{Builder, TauriPlugin};
use crate::{command, ipc::CallbackFn, EventId, Result, Runtime};
use crate::{AppHandle, Emitter, Manager, Webview};

use super::EventName;
use super::EventTarget;

#[command(root = "crate")]
async fn listen<R: Runtime>(
  webview: Webview<R>,
  event: EventName,
  target: EventTarget,
  handler: CallbackFn,
) -> Result<EventId> {
  webview.listen_js(event.as_str_event(), target, handler)
}

#[command(root = "crate")]
async fn unlisten<R: Runtime>(
  webview: Webview<R>,
  event: EventName,
  event_id: EventId,
) -> Result<()> {
  webview.unlisten_js(event.as_str_event(), event_id)
}

#[command(root = "crate")]
async fn emit<R: Runtime>(
  app: AppHandle<R>,
  event: EventName,
  payload: Option<JsonValue>,
) -> Result<()> {
  app.emit(event.as_str(), payload)
}

#[command(root = "crate")]
async fn emit_to<R: Runtime>(
  app: AppHandle<R>,
  target: EventTarget,
  event: EventName,
  payload: Option<JsonValue>,
) -> Result<()> {
  app.emit_to(target, event.as_str(), payload)
}

/// Initializes the event plugin.
pub(crate) fn init<R: Runtime, M: Manager<R>>(manager: &M) -> TauriPlugin<R> {
  let listeners = manager.manager().listeners();

  #[derive(Template)]
  #[default_template("./init.js")]
  struct InitJavascript {
    #[raw]
    unregister_listener_function: String,
  }

  let init_script = InitJavascript {
    unregister_listener_function: format!(
      "(event, eventId) => {}",
      crate::event::unlisten_js_script(listeners.listeners_object_name(), "event", "eventId")
    ),
  };

  Builder::new("event")
    .invoke_handler(crate::generate_handler![
      #![plugin(event)]
      listen, unlisten, emit, emit_to
    ])
    .js_init_script(
      init_script
        .render_default(&Default::default())
        .unwrap()
        .to_string(),
    )
    .build()
}
