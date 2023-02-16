use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tauri::{
  plugin::{PluginApi, PluginHandle, Result as PluginResult},
  AppHandle, Manager, Runtime,
};

use std::result::Result as StdResult;

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

// ExamplePlugin::ping input arguments
#[derive(Debug, Serialize)]
pub struct PingRequest {
  pub value: Option<String>,
}

// ExamplePlugin::ping return value
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PingResponse {
  pub value: Option<String>,
}

// A helper class to access the mobile {{ plugin_name }} APIs.
struct {{ plugin_name_pascal_case }}Plugin<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> {{ plugin_name_pascal_case }}Plugin<R> {
  fn ping(&self, payload: PingRequest) -> tauri::Result<StdResult<PingResponse, String>> {
    self.0.run_mobile_plugin("ping", payload)
  }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the mobile {{ plugin_name }} APIs.
pub trait {{ plugin_name_pascal_case }}Ext<R: Runtime> {
  fn ping(&self, payload: PingRequest) -> tauri::Result<StdResult<PingResponse, String>>;
}

impl<R: Runtime, T: Manager<R>> {{ plugin_name_pascal_case }}Ext<R> for T {
  fn ping(&self, payload: PingRequest) -> tauri::Result<StdResult<PingResponse, String>> {
    self.state::<{{ plugin_name_pascal_case }}Plugin<R>>().ping(payload)
  }
}
