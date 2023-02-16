use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle, Result as PluginResult},
  AppHandle, Manager, Runtime,
};

use std::result::Result as StdResult;

use crate::models::*;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "{{ android_package_id }}";

#[cfg(target_os = "ios")]
extern "C" {
  pub(crate) fn init_plugin_{{ plugin_name }}(webview: tauri::cocoa::base::id);
}

// initializes the Kotlin or Swift plugin classes
pub(crate) fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> PluginResult<()> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "ExamplePlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_{{ plugin_name }})?;
  app.manage({{ plugin_name_pascal_case }}Plugin(handle));
  Ok(())
}

// A helper class to access the mobile {{ plugin_name }} APIs.
struct {{ plugin_name_pascal_case }}Plugin<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> {{ plugin_name_pascal_case }}Plugin<R> {
  fn ping(&self, payload: PingRequest) -> tauri::Result<StdResult<PingResponse, String>> {
    self.0.run_mobile_plugin("ping", payload)
  }
}

impl<R: Runtime, T: Manager<R>> crate::{{ plugin_name_pascal_case }}Ext<R> for T {
  fn ping(&self, payload: PingRequest) -> tauri::Result<StdResult<PingResponse, String>> {
    self.state::<{{ plugin_name_pascal_case }}Plugin<R>>().ping(payload)
  }
}
