// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<Sample<R>> {
  Ok(Sample(app.clone()))
}

/// A helper class to access the sample APIs.
pub struct Sample<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Sample<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    payload.on_event.send(Event {
      kind: "ping".to_string(),
      value: payload.value.clone(),
    })?;
    Ok(PingResponse {
      value: payload.value,
    })
  }
}
