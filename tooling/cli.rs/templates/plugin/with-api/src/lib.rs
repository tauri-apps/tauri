{{#if license_header}}
{{ license_header }}
{{/if}}

use serde::{ser::Serializer, Serialize};
use serde_json::Value as JsonValue;
use tauri::{command, plugin::Plugin, AppHandle, Invoke, Manager, Runtime, State, Window};

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

/// Tauri plugin.
pub struct YourPlugin<R: Runtime> {
  invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> Default for YourPlugin<R> {
  fn default() -> Self {
    Self {
      invoke_handler: Box::new(tauri::generate_handler![execute]),
    }
  }
}

impl<R: Runtime> Plugin<R> for YourPlugin<R> {
  fn name(&self) -> &'static str {
    "{{ plugin_name }}"
  }

  fn initialize(&mut self, app: &AppHandle<R>, _config: JsonValue) -> tauri::plugin::Result<()> {
    app.manage(MyState::default());
    Ok(())
  }

  fn extend_api(&mut self, message: Invoke<R>) {
    (self.invoke_handler)(message)
  }
}
