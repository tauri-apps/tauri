use crate::{api::shortcuts::ShortcutManager, async_runtime::Mutex};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::Value as JsonValue;

use std::sync::Arc;

type ShortcutManagerHandle = Arc<Mutex<ShortcutManager>>;

pub fn manager_handle() -> &'static ShortcutManagerHandle {
  static MANAGER: Lazy<ShortcutManagerHandle> = Lazy::new(Default::default);
  &MANAGER
}

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// Register a global shortcut.
  Register { shortcut: String, handler: String },
  /// Unregister a global shortcut.
  Unregister { shortcut: String },
}

impl Cmd {
  pub async fn run<D: crate::ApplicationDispatcherExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<D>,
  ) -> crate::Result<JsonValue> {
    #[cfg(not(global_shortcut))]
    return Err(crate::Error::ApiNotAllowlisted("globalShortcut".to_string()));
    #[cfg(global_shortcut)]
    match self {
      Self::Register { shortcut, handler } => {
        let dispatcher = webview_manager.current_webview()?.clone();
        let mut manager = manager_handle().lock().await;
        manager.register_shortcut(shortcut, move || {
          let callback_string =
            crate::api::rpc::format_callback(handler.to_string(), serde_json::Value::Null);
          dispatcher.eval(callback_string.as_str());
        })?;
        Ok(JsonValue::Null)
      }
      Self::Unregister { shortcut } => {
        let mut manager = manager_handle().lock().await;
        manager.unregister_shortcut(shortcut)?;
        Ok(JsonValue::Null)
      }
    }
  }
}
