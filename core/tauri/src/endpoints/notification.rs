// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;
use serde::Deserialize;

#[cfg(notification_all)]
use crate::api::notification::Notification;

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
  #[allow(unused_variables)]
  pub fn run(self, identifier: String) -> crate::Result<InvokeResponse> {
    match self {
      #[cfg(notification_all)]
      Self::Notification { options } => send(options, identifier).map(Into::into),
      #[cfg(not(notification_all))]
      Self::Notification { .. } => Err(crate::Error::ApiNotAllowlisted("notification".to_string())),
      Self::IsNotificationPermissionGranted => {
        #[cfg(notification_all)]
        return is_permission_granted().map(Into::into);
        #[cfg(not(notification_all))]
        Err(crate::Error::ApiNotAllowlisted("notification".to_string()))
      }
      Self::RequestNotificationPermission => {
        #[cfg(notification_all)]
        return request_permission().map(Into::into);
        #[cfg(not(notification_all))]
        Err(crate::Error::ApiNotAllowlisted("notification".to_string()))
      }
    }
  }
}

#[cfg(notification_all)]
pub fn send(options: NotificationOptions, identifier: String) -> crate::Result<InvokeResponse> {
  let mut notification = Notification::new(identifier).title(options.title);
  if let Some(body) = options.body {
    notification = notification.body(body);
  }
  if let Some(icon) = options.icon {
    notification = notification.icon(icon);
  }
  notification.show()?;
  Ok(().into())
}

#[cfg(notification_all)]
pub fn is_permission_granted() -> crate::Result<InvokeResponse> {
  let settings = crate::settings::read_settings()?;
  if let Some(allow_notification) = settings.allow_notification {
    Ok(allow_notification.into())
  } else {
    Ok(().into())
  }
}

#[cfg(notification_all)]
pub fn request_permission() -> crate::Result<String> {
  let mut settings = crate::settings::read_settings()?;
  let granted = "granted".to_string();
  let denied = "denied".to_string();
  if let Some(allow_notification) = settings.allow_notification {
    return Ok(if allow_notification { granted } else { denied });
  }
  let answer = crate::api::dialog::ask(
    "Permissions",
    "This app wants to show notifications. Do you allow?",
  );
  match answer {
    crate::api::dialog::AskResponse::Yes => {
      settings.allow_notification = Some(true);
      crate::settings::write_settings(settings)?;
      Ok(granted)
    }
    crate::api::dialog::AskResponse::No => {
      settings.allow_notification = Some(false);
      crate::settings::write_settings(settings)?;
      Ok(denied)
    }
  }
}
