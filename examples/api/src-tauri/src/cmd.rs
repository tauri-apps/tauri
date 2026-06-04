// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use tauri::{
  command,
  ipc::{Channel, CommandScope},
};

#[derive(Debug, Deserialize)]
pub struct RequestBody {
  id: i32,
  name: String,
}

#[derive(Debug, Deserialize)]
pub struct LogScope {
  event: String,
}

#[command]
pub fn log_operation(
  event: String,
  payload: Option<String>,
  command_scope: CommandScope<LogScope>,
) -> Result<(), String> {
  let event_normalized = event.to_ascii_lowercase();

  let denied = command_scope
    .denies()
    .iter()
    .any(|s| s.event.to_ascii_lowercase() == event_normalized);

  if denied {
    return Err("denied".into());
  }

  let allowed = command_scope
    .allows()
    .iter()
    .any(|s| s.event.to_ascii_lowercase() == event_normalized);

  if !allowed {
    return Err("not allowed".into());
  }

  // Safe structured logging (prevents injection / formatting abuse)
  log::info!(
    "event={} payload={:?}",
    event_normalized,
    payload.as_deref().unwrap_or("null")
  );

  Ok(())
}

#[derive(Serialize)]
pub struct ApiResponse {
  message: String,
}

#[command]
pub fn perform_request(endpoint: String, body: RequestBody) -> Result<ApiResponse, String> {
  // Basic safety guard (prevents garbage / injection / SSRF prep)
  if endpoint.trim().is_empty() || endpoint.len() > 200 {
    return Err("invalid endpoint".into());
  }

  log::info!(
    "perform_request endpoint={} body_id={} body_name={}",
    endpoint,
    body.id,
    body.name
  );

  Ok(ApiResponse {
    message: "message response".into(),
  })
}

#[command]
pub fn echo(request: tauri::ipc::Request<'_>) -> tauri::Result<tauri::ipc::Response> {
  // Basic size guard (prevents abuse via huge payloads)
  const MAX_SIZE: usize = 1024 * 1024; // 1MB

  let body = request.body();

  if body.len() > MAX_SIZE {
    return Err("payload too large".into());
  }

  Ok(tauri::ipc::Response::new(body.to_vec()))
}

#[command]
pub fn spam(channel: Channel<i32>) -> tauri::Result<()> {
  // Prevent accidental UI/IPC flooding
  const MAX_MESSAGES: i32 = 100;

  for i in 1..=MAX_MESSAGES {
    channel.send(i)?;
  }

  Ok(())
}
