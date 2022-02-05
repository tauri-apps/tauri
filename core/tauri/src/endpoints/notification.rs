// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeContext;
use crate::Runtime;
use serde::Deserialize;
use tauri_macros::{module_command_handler, CommandModule};

#[cfg(notification_all)]
use crate::{api::notification::Notification, Env, Manager};

// `Granted` response from `request_permission`. Matches the Web API return value.
#[cfg(notification_all)]
const PERMISSION_GRANTED: &str = "granted";
// `Denied` response from `request_permission`. Matches the Web API return value.
const PERMISSION_DENIED: &str = "denied";

/// The options for the notification API.
#[derive(Debug, Clone, Deserialize)]
pub struct NotificationOptions {
  /// The notification title.
  pub title: String,
  /// The notification body.
  pub body: Option<String>,
  /// The notification icon.
  pub icon: Option<String>,
}

/// The API descriptor.
#[derive(Deserialize, CommandModule)]
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
  #[module_command_handler(notification_all, "notification > all")]
  fn notification<R: Runtime>(
    context: InvokeContext<R>,
    options: NotificationOptions,
  ) -> crate::Result<()> {
    let allowed = match is_permission_granted(&context) {
      Some(p) => p,
      None => request_permission(&context),
    };
    if !allowed {
      return Err(crate::Error::NotificationNotAllowed);
    }
    let mut notification =
      Notification::new(context.config.tauri.bundle.identifier.clone()).title(options.title);
    if let Some(body) = options.body {
      notification = notification.body(body);
    }
    if let Some(icon) = options.icon {
      notification = notification.icon(icon);
    }
    notification.show()?;
    Ok(())
  }

  #[cfg(notification_all)]
  fn request_notification_permission<R: Runtime>(
    context: InvokeContext<R>,
  ) -> crate::Result<&'static str> {
    if request_permission(&context) {
      Ok(PERMISSION_GRANTED)
    } else {
      Ok(PERMISSION_DENIED)
    }
  }

  #[cfg(not(notification_all))]
  fn request_notification_permission<R: Runtime>(
    _context: InvokeContext<R>,
  ) -> crate::Result<&'static str> {
    Ok(PERMISSION_DENIED)
  }

  #[cfg(notification_all)]
  fn is_notification_permission_granted<R: Runtime>(
    context: InvokeContext<R>,
  ) -> crate::Result<Option<bool>> {
    if let Some(allow_notification) = is_permission_granted(&context) {
      Ok(Some(allow_notification))
    } else {
      Ok(None)
    }
  }

  #[cfg(not(notification_all))]
  fn is_notification_permission_granted<R: Runtime>(
    _context: InvokeContext<R>,
  ) -> crate::Result<Option<bool>> {
    Ok(Some(false))
  }
}

#[cfg(notification_all)]
fn request_permission<R: Runtime>(context: &InvokeContext<R>) -> bool {
  let mut settings = crate::settings::read_settings(
    &context.config,
    &context.package_info,
    context.window.state::<Env>().inner(),
  );
  if let Some(allow_notification) = settings.allow_notification {
    return allow_notification;
  }
  let answer = crate::api::dialog::blocking::ask(
    Some(&context.window),
    "Permissions",
    "This app wants to show notifications. Do you allow?",
  );

  settings.allow_notification = Some(answer);
  let _ = crate::settings::write_settings(
    &context.config,
    &context.package_info,
    context.window.state::<Env>().inner(),
    settings,
  );

  answer
}

#[cfg(notification_all)]
fn is_permission_granted<R: Runtime>(context: &InvokeContext<R>) -> Option<bool> {
  crate::settings::read_settings(
    &context.config,
    &context.package_info,
    context.window.state::<Env>().inner(),
  )
  .allow_notification
}

#[cfg(test)]
mod tests {
  use super::NotificationOptions;

  use quickcheck::{Arbitrary, Gen};

  impl Arbitrary for NotificationOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        title: String::arbitrary(g),
        body: Option::arbitrary(g),
        icon: Option::arbitrary(g),
      }
    }
  }

  #[cfg(not(notification_all))]
  #[test]
  fn request_notification_permission() {
    assert_eq!(
      super::Cmd::request_notification_permission(crate::test::mock_invoke_context()).unwrap(),
      super::PERMISSION_DENIED
    )
  }

  #[cfg(not(notification_all))]
  #[test]
  fn is_notification_permission_granted() {
    assert_eq!(
      super::Cmd::is_notification_permission_granted(crate::test::mock_invoke_context()).unwrap(),
      Some(false)
    );
  }

  #[tauri_macros::module_command_test(notification_all, "notification > all")]
  #[quickcheck_macros::quickcheck]
  fn notification(_options: NotificationOptions) {}
}
