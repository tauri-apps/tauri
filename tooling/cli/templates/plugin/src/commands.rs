{{#if license_header}}
{{ license_header }}
{{/if}}
use tauri::{AppHandle, command, Runtime, State, Window};

use crate::{MyState, Result};

#[command]
pub(crate) async fn execute<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  state: State<'_, MyState>,
) -> Result<String> {
  state.0.lock().unwrap().insert("key".into(), "value".into());
  Ok("success".to_string())
}
