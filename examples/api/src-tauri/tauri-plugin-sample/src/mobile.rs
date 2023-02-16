use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tauri::{
  plugin::{PluginApi, PluginHandle, Result as PluginResult},
  AppHandle, Manager, Runtime,
};

use std::result::Result as StdResult;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.plugin.sample";

#[cfg(target_os = "ios")]
extern "C" {
  pub(crate) fn init_plugin_sample(webview: tauri::cocoa::base::id);
}

// initializes the Kotlin or Swift plugin classes
pub(crate) fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> PluginResult<()> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "ExamplePlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_sample)?;
  app.manage(SamplePlugin(handle));
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

// A helper class to access the mobile sample APIs.
struct SamplePlugin<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> SamplePlugin<R> {
  fn ping(&self, payload: PingRequest) -> tauri::Result<StdResult<PingResponse, String>> {
    self.0.run_mobile_plugin("ping", payload)
  }
}

// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the mobile sample APIs.
pub trait SampleExt<R: Runtime> {
  fn ping(&self, payload: PingRequest) -> tauri::Result<StdResult<PingResponse, String>>;
}

impl<R: Runtime, T: Manager<R>> SampleExt<R> for T {
  fn ping(&self, payload: PingRequest) -> tauri::Result<StdResult<PingResponse, String>> {
    self.state::<SamplePlugin<R>>().ping(payload)
  }
}
