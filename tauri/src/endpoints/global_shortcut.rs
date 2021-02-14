use crate::{api::shortcuts::ShortcutManager, async_runtime::Mutex};
use once_cell::sync::Lazy;
use serde::Deserialize;

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
  Register {
    shortcut: String,
    handler: String,
    callback: String,
    error: String,
  },
}

impl Cmd {
  #[allow(unused_variables)]
  pub async fn run<D: crate::ApplicationDispatcherExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<D>,
  ) -> crate::Result<()> {
    match self {
      #[allow(unused_variables)]
      Self::Register {
        shortcut,
        handler,
        callback,
        error,
      } => {
        #[cfg(global_shortcut)]
        {
          let dispatcher = webview_manager.current_webview()?.clone();
          crate::execute_promise(
            webview_manager,
            async move {
              let mut manager = manager_handle().lock().await;
              manager.register_shortcut(shortcut, move || {
                let callback_string =
                  crate::api::rpc::format_callback(handler.to_string(), serde_json::Value::Null);
                dispatcher.eval(callback_string.as_str());
              })?;
              Ok(())
            },
            callback,
            error,
          )
          .await;
        }
        #[cfg(not(global_shortcut))]
        super::allowlist_error(webview_manager, error, "globalShortcut");
      }
    }
    Ok(())
  }
}
