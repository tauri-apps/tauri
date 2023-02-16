{{#if license_header}}
{{ license_header }}
{{/if}}

use serde::{ser::Serializer, Serialize};
use tauri::{
  command,
  plugin::{Builder, TauriPlugin},
  AppHandle, Manager, Runtime, State, Window,
};

use std::{collections::HashMap, sync::Mutex};

type Result<T> = std::result::Result<T, Error>;

const PLUGIN_NAME: &str = "{{ plugin_name }}";
#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "{{ android_package_id }}";

#[cfg(target_os = "ios")]
extern "C" {
  fn init_plugin_{{ plugin_name }}(webview: tauri::cocoa::base::id);
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}

#[derive(Default)]
struct MyState(Mutex<HashMap<String, String>>);

#[command]
async fn execute<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  state: State<'_, MyState>,
) -> Result<String> {
  state.0.lock().unwrap().insert("key".into(), "value".into());
  Ok("success".to_string())
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new(PLUGIN_NAME)
    .invoke_handler(tauri::generate_handler![execute])
    .setup(|app| {
      // initialize mobile plugins
      #[cfg(target_os = "android")]
      app.initialize_android_plugin(PLUGIN_NAME, PLUGIN_IDENTIFIER, "ExamplePlugin")?;
      #[cfg(target_os = "ios")]
      app.initialize_ios_plugin(init_plugin_{{ plugin_name }})?;

      // manage state so it is accessible by the commands
      app.manage(MyState::default());
      Ok(())
    })
    .build()
}
