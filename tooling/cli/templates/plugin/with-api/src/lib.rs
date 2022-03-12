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
  Builder::new("{{ plugin_name }}")
    .invoke_handler(tauri::generate_handler![execute])
    .setup(|app| {
      app.manage(MyState::default());
      Ok(())
    })
    .build()
}
