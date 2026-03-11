// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{
  AppHandle, Runtime,
  plugin::{PluginApi, PluginHandle},
};

use crate::models::*;

type TauriRuntime = tauri_runtime_wry::Wry<tauri::EventLoopMessage>;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.plugin.sample";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_sample);

#[cfg(target_os = "ios")]
use tauri_runtime_wry::PluginApiWryExt;

// initializes the Kotlin or Swift plugin classes
pub fn init<C: DeserializeOwned>(
  _app: &AppHandle<TauriRuntime>,
  api: PluginApi<TauriRuntime, C>,
) -> crate::Result<Sample<TauriRuntime>> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "ExamplePlugin")?;
  let handle = api.register_ios_plugin(init_plugin_sample)?;
  Ok(Sample(handle))
}

/// A helper class to access the sample APIs.
pub struct Sample<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Sample<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    self
      .0
      .run_mobile_plugin("ping", payload)
      .map_err(Into::into)
  }
}
