use tauri::{
  plugin::{Builder, TauriPlugin},
  Runtime,
};

const PLUGIN_NAME: &str = "sample";
#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.plugin.test";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  #[allow(unused_mut)]
  let mut builder = Builder::new(PLUGIN_NAME);
  #[cfg(target_os = "android")]
  {
    use tauri::Manager;

    builder = builder.on_webview_ready(|window| {
      window
        .app_handle()
        .initialize_android_plugin(PLUGIN_NAME, PLUGIN_IDENTIFIER, "ExamplePlugin")
        .unwrap();
    });
  }
  builder.build()
}
