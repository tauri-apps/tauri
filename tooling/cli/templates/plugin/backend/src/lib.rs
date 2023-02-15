{{#if license_header}}
{{ license_header }}
{{/if}}
use tauri::{plugin::{Builder, TauriPlugin}, Runtime};

const PLUGIN_NAME: &str = "{{ plugin_name }}";
#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "{{ android_package_id }}";

#[cfg(target_os = "ios")]
extern "C" {
  fn init_plugin_{{ plugin_name }}(webview: tauri::cocoa::base::id);
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new(PLUGIN_NAME)
    .setup(|app| {
      #[cfg(target_os = "android")]
      app.initialize_android_plugin(PLUGIN_NAME, PLUGIN_IDENTIFIER, "ExamplePlugin")?;
      #[cfg(target_os = "ios")]
      app.initialize_ios_plugin(init_plugin_{{ plugin_name }})?;
      Ok(())
    })
    .build()
}
