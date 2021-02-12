use crate::ApplicationDispatcherExt;

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
  Notification {
    options: NotificationOptions,
    callback: String,
    error: String,
  },
  /// The request notification permission API.
  RequestNotificationPermission { callback: String, error: String },
  /// The notification permission check API.
  IsNotificationPermissionGranted { callback: String, error: String },
}

impl Cmd {
  pub async fn run<D: ApplicationDispatcherExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<D>,
    context: &crate::app::Context,
  ) -> crate::Result<()> {
    match self {
      Self::Notification {
        options,
        callback,
        error,
      } => {
        #[cfg(notification)]
        send(webview_manager, options, callback, error, &context.config).await;
        #[cfg(not(notification))]
        allowlist_error(webview_manager, error, "notification");
      }
      Self::IsNotificationPermissionGranted { callback, error } => {
        #[cfg(notification)]
        is_permission_granted(webview_manager, callback, error).await;
        #[cfg(not(notification))]
        allowlist_error(webview_manager, error, "notification");
      }
      Self::RequestNotificationPermission { callback, error } => {
        #[cfg(notification)]
        request_permission(webview_manager, callback, error)?;
        #[cfg(not(notification))]
        allowlist_error(webview_manager, error, "notification");
      }
    }
    Ok(())
  }
}

pub async fn send<D: ApplicationDispatcherExt>(
  webview_manager: &crate::WebviewManager<D>,
  options: NotificationOptions,
  callback: String,
  error: String,
  config: &Config,
) {
  let identifier = config.tauri.bundle.identifier.clone();

  crate::execute_promise(
    webview_manager,
    async move {
      let mut notification = Notification::new(identifier).title(options.title);
      if let Some(body) = options.body {
        notification = notification.body(body);
      }
      if let Some(icon) = options.icon {
        notification = notification.icon(icon);
      }
      notification.show()?;
      crate::Result::Ok(JsonValue::Null)
    },
    callback,
    error,
  )
  .await;
}

pub async fn is_permission_granted<D: ApplicationDispatcherExt>(
  webview_manager: &crate::WebviewManager<D>,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview_manager,
    async move {
      let settings = crate::settings::read_settings()?;
      if let Some(allow_notification) = settings.allow_notification {
        crate::Result::Ok(JsonValue::String(allow_notification.to_string()))
      } else {
        crate::Result::Ok(JsonValue::Null)
      }
    },
    callback,
    error,
  )
  .await;
}

pub fn request_permission<D: ApplicationDispatcherExt + 'static>(
  webview_manager: &crate::WebviewManager<D>,
  callback: String,
  error: String,
) -> crate::Result<()> {
  crate::execute_promise_sync(
    webview_manager,
    move || {
      let mut settings = crate::settings::read_settings()?;
      let granted = "granted".to_string();
      let denied = "denied".to_string();
      if let Some(allow_notification) = settings.allow_notification {
        return crate::Result::Ok(if allow_notification { granted } else { denied });
      }
      let answer = tauri_api::dialog::ask(
        "This app wants to show notifications. Do you allow?",
        "Permissions",
      );
      match answer {
        tauri_api::dialog::DialogSelection::Yes => {
          settings.allow_notification = Some(true);
          crate::settings::write_settings(settings)?;
          crate::Result::Ok(granted)
        }
        tauri_api::dialog::DialogSelection::No => {
          settings.allow_notification = Some(false);
          crate::settings::write_settings(settings)?;
          crate::Result::Ok(denied)
        }
        _ => crate::Result::Ok("default".to_string()),
      }
    },
    callback,
    error,
  );
  Ok(())
}
