{{#if license_header}}
{{ license_header }}
{{/if}}
use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<{{ plugin_name_pascal_case }}<R>> {
  Ok({{ plugin_name_pascal_case }}(app.clone()))
}

/// Access to the {{ plugin_name }} APIs.
pub struct {{ plugin_name_pascal_case }}<R: Runtime>(AppHandle<R>);

impl<R: Runtime> {{ plugin_name_pascal_case }}<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }
}
