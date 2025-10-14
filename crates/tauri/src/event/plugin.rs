use serde::Deserialize;
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
use serialize_to_javascript::{default_template, DefaultTemplate, Template};

use crate::ipc::{InvokeBody, Request, Response};
use crate::manager::EventPayloadStore;
use crate::plugin::{Builder, TauriPlugin};
use crate::{command, ipc::CallbackFn, EventId, Result, Runtime};
use crate::{AppHandle, Emitter, Manager, State, Webview};

use super::EventName;
use super::EventTarget;

const EVENT_NAME_HEADER_NAME: &str = "Tauri-Event-Name";
const EVENT_TARGET_HEADER_NAME: &str = "Tauri-Event-Target";
pub const FETCH_EVENT_PAYLOAD_COMMAND: &str = "plugin:event|fetch_payload";
pub const PAYLOAD_ID_HEADER_NAME: &str = "Tauri-Payload-Id";

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
async fn emit<R: Runtime>(app: AppHandle<R>, request: Request<'_>) -> Result<()> {
  if let Some(event_header) = request
    .headers()
    .get(EVENT_NAME_HEADER_NAME)
    .and_then(|v| v.to_str().ok())
  {
    let event = EventName::new(event_header)?;
    match request.body() {
      InvokeBody::Json(payload) => app.emit(event.as_str(), payload),
      InvokeBody::Raw(payload) => app.emit_raw(event.as_str(), payload.clone()),
    }
  } else if let InvokeBody::Json(payload) = request.body() {
    let args: EmitArgs = serde_json::from_value(payload.clone())?;
    app.emit(args.event.as_str(), args.payload)
  } else {
    Err(anyhow::anyhow!("unexpected emit request format").into())
  }
}

#[derive(Deserialize)]
struct EmitArgs {
  event: EventName,
  payload: Option<serde_json::Value>,
}

#[command(root = "crate")]
async fn emit_to<R: Runtime>(app: AppHandle<R>, request: Request<'_>) -> Result<()> {
  if let Some(event_header) = request
    .headers()
    .get(EVENT_NAME_HEADER_NAME)
    .and_then(|v| v.to_str().ok())
  {
    let event = EventName::new(event_header)?;
    let target = if let Some(target_header) = request
      .headers()
      .get(EVENT_TARGET_HEADER_NAME)
      .and_then(|v| v.to_str().ok())
    {
      serde_json::from_str::<EventTarget>(target_header)?
    } else {
      return Err(anyhow::anyhow!("missing event target header").into());
    };

    match request.body() {
      InvokeBody::Json(payload) => app.emit_to(target, event.as_str(), payload),
      InvokeBody::Raw(payload) => app.emit_raw_to(target, event.as_str(), payload.clone()),
    }
  } else if let InvokeBody::Json(payload) = request.body() {
    let args: EmitToArgs = serde_json::from_value(payload.clone())?;
    app.emit_to(args.target, args.event.as_str(), args.payload)
  } else {
    Err(anyhow::anyhow!("unexpected emit request format").into())
  }
}

#[derive(Deserialize)]
struct EmitToArgs {
  event: EventName,
  payload: Option<serde_json::Value>,
  target: EventTarget,
}

#[command(root = "crate")]
fn fetch_payload(
  request: Request<'_>,
  store: State<'_, EventPayloadStore>,
) -> std::result::Result<Response, &'static str> {
  if let Some(id) = request
    .headers()
    .get(PAYLOAD_ID_HEADER_NAME)
    .and_then(|v| v.to_str().ok())
    .and_then(|id| id.parse().ok())
  {
    let r = if let Some(data) = store.0.lock().unwrap().get(&id) {
      // fetch_add returns the previous value so we must +1
      let read_count = data
        .read_count
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        + 1;

      Ok((
        Response::new(data.payload.clone()),
        read_count == data.target_count,
      ))
    } else {
      Err("data not found")
    };

    match r {
      Ok((response, done)) => {
        if done {
          store.0.lock().unwrap().remove(&id);
        }
        Ok(response)
      }
      Err(e) => Err(e),
    }
  } else {
    Err("missing payload id header")
  }
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
      listen, unlisten, emit, emit_to, fetch_payload
    ])
    .js_init_script(
      init_script
        .render_default(&Default::default())
        .unwrap()
        .to_string(),
    )
    .build()
}
