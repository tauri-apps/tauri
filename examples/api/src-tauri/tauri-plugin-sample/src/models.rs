// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

#[derive(Serialize)]
pub struct Event {
  pub kind: String,
  pub value: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingRequest {
  pub value: Option<String>,
  pub on_event: Channel<Event>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PingResponse {
  pub value: Option<String>,
}
