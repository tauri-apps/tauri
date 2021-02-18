use serde::Deserialize;
use serde_json::Value as JsonValue;
use tauri_api::{config::Config, notification::Notification};

/// The options for the notification API.
#[derive(Deserialize)]
pub struct NotificationOptions {
  /// The notification title.
  pub title: String,
  /// The notification body.
  pub body: Option<String>,
  /// The notification icon.
  pub icon: Option<String>,
}

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The show notification API.
  Notification { options: NotificationOptions },
  /// The request notification permission API.
  RequestNotificationPermission,
  /// The notification permission check API.
  IsNotificationPermissionGranted,
}

impl Cmd {
  pub async fn run(self, context: &crate::app::Context) -> crate::Result<JsonValue> {
    match self {
      Self::Notification { options } => {
        #[cfg(notification)]
        return send(options, &context.config)
          .await
          .and_then(super::to_value);
        #[cfg(not(notification))]
        Err(crate::Error::ApiNotAllowlisted("notification".to_string()))
      }
      Self::IsNotificationPermissionGranted => {
        #[cfg(notification)]
        return is_permission_granted().await.and_then(super::to_value);
        #[cfg(not(notification))]
        Err(crate::Error::ApiNotAllowlisted("notification".to_string()))
      }
      Self::RequestNotificationPermission => {
        #[cfg(notification)]
        return request_permission().map(JsonValue::String);
        #[cfg(not(notification))]
        Err(crate::Error::ApiNotAllowlisted("notification".to_string()))
      }
    }
  }
}

pub async fn send(options: NotificationOptions, config: &Config) -> crate::Result<JsonValue> {
  let identifier = config.tauri.bundle.identifier.clone();
  let mut notification = Notification::new(identifier).title(options.title);
  if let Some(body) = options.body {
    notification = notification.body(body);
  }
  if let Some(icon) = options.icon {
    notification = notification.icon(icon);
  }
  notification.show()?;
  Ok(JsonValue::Null)
}

pub async fn is_permission_granted() -> crate::Result<JsonValue> {
  let settings = crate::settings::read_settings()?;
  if let Some(allow_notification) = settings.allow_notification {
    Ok(JsonValue::String(allow_notification.to_string()))
  } else {
    Ok(JsonValue::Null)
  }
}

pub fn request_permission() -> crate::Result<String> {
  let mut settings = crate::settings::read_settings()?;
  let granted = "granted".to_string();
  let denied = "denied".to_string();
  if let Some(allow_notification) = settings.allow_notification {
    return Ok(if allow_notification { granted } else { denied });
  }
  let answer = tauri_api::dialog::ask(
    "Permissions",
    "This app wants to show notifications. Do you allow?",
  );
  match answer {
    tauri_api::dialog::AskResponse::Yes => {
      settings.allow_notification = Some(true);
      crate::settings::write_settings(settings)?;
      Ok(granted)
    }
    tauri_api::dialog::AskResponse::No => {
      settings.allow_notification = Some(false);
      crate::settings::write_settings(settings)?;
      Ok(denied)
    }
  }
}
