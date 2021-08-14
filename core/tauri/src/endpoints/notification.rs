// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;
use serde::Deserialize;

#[cfg(notification_all)]
use crate::api::notification::Notification;
use crate::{Config, PackageInfo};

use std::sync::Arc;

// `Granted` response from `request_permission`. Matches the Web API return value.
#[cfg(notification_all)]
const PERMISSION_GRANTED: &str = "granted";
// `Denied` response from `request_permission`. Matches the Web API return value.
const PERMISSION_DENIED: &str = "denied";

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
  pub fn run(
    self,
    config: Arc<Config>,
    package_info: &PackageInfo,
  ) -> crate::Result<InvokeResponse> {
    match self {
      #[cfg(notification_all)]
      Self::Notification { options } => send(options, &config).map(Into::into),
      #[cfg(not(notification_all))]
      Self::Notification { .. } => Err(crate::Error::ApiNotAllowlisted("notification".to_string())),
      Self::IsNotificationPermissionGranted => {
        #[cfg(notification_all)]
        return is_permission_granted(&config, package_info).map(Into::into);
        #[cfg(not(notification_all))]
        Ok(false.into())
      }
      Self::RequestNotificationPermission => {
        #[cfg(notification_all)]
        return request_permission(&config, package_info).map(Into::into);
        #[cfg(not(notification_all))]
        Ok(PERMISSION_DENIED.into())
      }
    }
  }
}

#[cfg(notification_all)]
pub fn send(options: NotificationOptions, config: &Config) -> crate::Result<InvokeResponse> {
  let mut notification =
    Notification::new(config.tauri.bundle.identifier.clone()).title(options.title);
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
pub fn is_permission_granted(
  config: &Config,
  package_info: &PackageInfo,
) -> crate::Result<InvokeResponse> {
  let settings = crate::settings::read_settings(config, package_info);
  if let Some(allow_notification) = settings.allow_notification {
    Ok(allow_notification.into())
  } else {
    Ok(().into())
  }
}

#[cfg(notification_all)]
pub fn request_permission(config: &Config, package_info: &PackageInfo) -> crate::Result<String> {
  let mut settings = crate::settings::read_settings(config, package_info);
  if let Some(allow_notification) = settings.allow_notification {
    return Ok(if allow_notification {
      PERMISSION_GRANTED.to_string()
    } else {
      PERMISSION_DENIED.to_string()
    });
  }
  let (tx, rx) = std::sync::mpsc::channel();
  crate::api::dialog::ask(
    "Permissions",
    "This app wants to show notifications. Do you allow?",
    move |answer| {
      tx.send(answer).unwrap();
    },
  );

  let answer = rx.recv().unwrap();

  settings.allow_notification = Some(answer);
  crate::settings::write_settings(config, package_info, settings)?;

  if answer {
    Ok(PERMISSION_GRANTED.to_string())
  } else {
    Ok(PERMISSION_DENIED.to_string())
  }
}
