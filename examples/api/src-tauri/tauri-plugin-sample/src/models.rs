// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

#[derive(Serialize)]
pub struct PingRequest {
  pub value: Option<String>,
  #[serde(rename = "onEvent")]
  pub on_event: Channel,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PingResponse {
  pub value: Option<String>,
}
