use tauri::{
  plugin::{Builder, PluginHandle, TauriPlugin},
  Manager, Runtime,
};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.plugin.sample";

#[cfg(target_os = "ios")]
extern "C" {
  fn init_plugin_sample(webview: tauri::cocoa::base::id);
}

pub struct SamplePlugin<R: Runtime>(PluginHandle<R>);

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("sample")
    .setup(|app, api| {
      #[cfg(any(target_os = "android", target_os = "ios"))]
      {
        #[cfg(target_os = "android")]
        let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "ExamplePlugin")?;
        #[cfg(target_os = "ios")]
        let handle = api.register_ios_plugin(init_plugin_sample)?;
        app.manage(SamplePlugin(handle));
      }

      Ok(())
    })
    .build()
}
