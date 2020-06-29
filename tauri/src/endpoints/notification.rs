use super::cmd::NotificationOptions;
use serde_json::Value as JsonValue;
use web_view::WebView;

pub fn send<T: 'static>(
  webview: &mut WebView<'_, T>,
  options: NotificationOptions,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      let mut notification = tauri_api::notification::Notification::new();
      notification = notification.body(options.body);
      if let Some(title) = options.title {
        notification = notification.title(title);
      }
      if let Some(icon) = options.icon {
        notification = notification.icon(icon);
      }
      notification.show()?;
      Ok("".to_string())
    },
    callback,
    error,
  );
}

pub fn is_permission_granted<T: 'static>(
  webview: &mut WebView<'_, T>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview,
    move || {
      let settings = crate::settings::read_settings()?;
      if let Some(allow_notification) = settings.allow_notification {
        Ok(JsonValue::String(allow_notification.to_string()))
      } else {
        Ok(JsonValue::Null)
      }
    },
    callback,
    error,
  );
}

pub fn request_permission<T: 'static>(
  webview: &mut WebView<'_, T>,
  callback: String,
  error: String,
) -> crate::Result<()> {
  crate::execute_promise_sync(
    webview,
    move || {
      let mut settings = crate::settings::read_settings()?;
      let granted = r#""granted""#.to_string();
      let denied = r#""denied""#.to_string();
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
        _ => Ok(r#""default""#.to_string()),
      }
    },
    callback,
    error,
  )?;
  Ok(())
}
