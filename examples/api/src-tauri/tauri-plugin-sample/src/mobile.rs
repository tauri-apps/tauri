use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle, Result as PluginResult},
  AppHandle, Manager, Runtime,
};

use crate::models::*;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.plugin.sample";

#[cfg(target_os = "ios")]
extern "C" {
  fn init_plugin_sample(webview: tauri::cocoa::base::id);
}

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
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

impl<R: Runtime, T: Manager<R>> crate::SampleExt<R> for T {
  fn ping(&self, payload: PingRequest) -> tauri::Result<Result<PingResponse, String>> {
    self.state::<SamplePlugin<R>>().ping(payload)
  }
}

// A helper class to access the mobile sample APIs.
struct SamplePlugin<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> SamplePlugin<R> {
  fn ping(&self, payload: PingRequest) -> tauri::Result<Result<PingResponse, String>> {
    self.0.run_mobile_plugin("ping", payload)
  }
}
