use super::cmd::NotificationOptions;
use crate::WebviewDispatcher;
use serde_json::Value as JsonValue;

use std::sync::mpsc::Sender;

pub async fn send<W: WebviewDispatcher>(
  webview: &mut W,
  options: NotificationOptions,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    async move {
      let mut notification = tauri_api::notification::Notification::new().title(options.title);
      if let Some(body) = options.body {
        notification = notification.body(body);
      }
      if let Some(icon) = options.icon {
        notification = notification.icon(icon);
      }
      notification.show()?;
      Ok(JsonValue::Null)
    },
    callback,
    error,
  )
  .await;
}

pub async fn is_permission_granted<W: WebviewDispatcher>(
  webview: &mut W,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    async move {
      let settings = crate::settings::read_settings()?;
      if let Some(allow_notification) = settings.allow_notification {
        Ok(JsonValue::String(allow_notification.to_string()))
      } else {
        Ok(JsonValue::Null)
      }
    },
    callback,
    error,
  )
  .await;
}

pub fn request_permission<W: WebviewDispatcher + 'static>(
  webview: &mut W,
  sync_task_sender: &Sender<crate::SyncTask>,
  callback: String,
  error: String,
) -> crate::Result<()> {
  crate::execute_promise_sync(
    webview,
    sync_task_sender,
    move || {
      let mut settings = crate::settings::read_settings()?;
      let granted = "granted".to_string();
      let denied = "denied".to_string();
      if let Some(allow_notification) = settings.allow_notification {
        return Ok(if allow_notification { granted } else { denied });
      }
      let answer = tauri_api::dialog::ask(
        "This app wants to show notifications. Do you allow?",
        "Permissions",
      );
      match answer {
        tauri_api::dialog::DialogSelection::Yes => {
          settings.allow_notification = Some(true);
          crate::settings::write_settings(settings)?;
          Ok(granted)
        }
        tauri_api::dialog::DialogSelection::No => {
          settings.allow_notification = Some(false);
          crate::settings::write_settings(settings)?;
          Ok(denied)
        }
        _ => Ok("default".to_string()),
      }
    },
    callback,
    error,
  );
  Ok(())
}
