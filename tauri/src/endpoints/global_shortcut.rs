#[cfg(global_shortcut_all)]
use crate::api::shortcuts::ShortcutManager;
use crate::{
  app::{InvokeResponse, WebviewDispatcher},
  async_runtime::Mutex,
};
use once_cell::sync::Lazy;
use serde::Deserialize;

use std::sync::Arc;

#[cfg(global_shortcut_all)]
type ShortcutManagerHandle = Arc<Mutex<ShortcutManager>>;

#[cfg(global_shortcut_all)]
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
  /// Register a list of global shortcuts.
  RegisterAll {
    shortcuts: Vec<String>,
    handler: String,
  },
  /// Unregister a global shortcut.
  Unregister { shortcut: String },
  /// Unregisters all registered shortcuts.
  UnregisterAll,
  /// Determines whether the given hotkey is registered or not.
  IsRegistered { shortcut: String },
}

#[cfg(global_shortcut_all)]
fn register_shortcut<A: crate::ApplicationDispatcherExt + 'static>(
  dispatcher: WebviewDispatcher<A>,
  manager: &mut ShortcutManager,
  shortcut: String,
  handler: String,
) -> crate::Result<()> {
  manager.register(shortcut.clone(), move || {
    let callback_string = crate::api::rpc::format_callback(
      handler.to_string(),
      serde_json::Value::String(shortcut.clone()),
    );
    let _ = dispatcher.eval(callback_string.as_str());
  })?;
  Ok(())
}

impl Cmd {
  pub async fn run<A: crate::ApplicationExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<A>,
  ) -> crate::Result<InvokeResponse> {
    #[cfg(not(global_shortcut_all))]
    return Err(crate::Error::ApiNotAllowlisted(
      "globalShortcut > all".to_string(),
    ));
    #[cfg(global_shortcut_all)]
    match self {
      Self::Register { shortcut, handler } => {
        let dispatcher = webview_manager.current_webview().await?.clone();
        let mut manager = manager_handle().lock().await;
        register_shortcut(dispatcher, &mut manager, shortcut, handler)?;
        Ok(().into())
      }
      Self::RegisterAll { shortcuts, handler } => {
        let dispatcher = webview_manager.current_webview().await?.clone();
        let mut manager = manager_handle().lock().await;
        for shortcut in shortcuts {
          register_shortcut(dispatcher.clone(), &mut manager, shortcut, handler.clone())?;
        }
        Ok(().into())
      }
      Self::Unregister { shortcut } => {
        let mut manager = manager_handle().lock().await;
        manager.unregister(shortcut)?;
        Ok(().into())
      }
      Self::UnregisterAll => {
        let mut manager = manager_handle().lock().await;
        manager.unregister_all()?;
        Ok(().into())
      }
      Self::IsRegistered { shortcut } => {
        let manager = manager_handle().lock().await;
        Ok(manager.is_registered(shortcut)?.into())
      }
    }
  }
}
