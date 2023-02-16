{{#if license_header}}
{{ license_header }}
{{/if}}

use serde::{ser::Serializer,  Serialize};
use tauri::{
  command,
  plugin::{Builder, TauriPlugin},
  AppHandle, Manager, Runtime, State, Window,
};

use std::{collections::HashMap, sync::Mutex, result::Result as StdResult};

type Result<T> = StdResult<T, Error>;

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;
mod models;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
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

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the {{ plugin_name }} APIs.
pub trait {{ plugin_name_pascal_case }}Ext<R: Runtime> {
  fn ping(&self, payload: PingRequest) -> tauri::Result<StdResult<PingResponse, String>>;
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("{{ plugin_name }}")
    .invoke_handler(tauri::generate_handler![execute])
    .setup(|app, _api| {
      // initialize Android and iOS plugins
      #[cfg(any(target_os = "android", target_os = "ios"))]
      mobile::init(app, _api)?;
      // manage state so it is accessible by the commands
      app.manage(MyState::default());
      Ok(())
    })
    .build()
}
