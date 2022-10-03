// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused_imports)]

use super::InvokeContext;
use crate::Runtime;
use serde::Deserialize;
use tauri_macros::{command_enum, module_command_handler, CommandModule};

#[cfg(notification_all)]
use crate::{api::notification::Notification, Env, Manager};

// `Granted` response from `request_permission`. Matches the Web API return value.
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
#[command_enum]
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The show notification API.
  #[cmd(notification_all, "notification > all")]
  Notification { options: NotificationOptions },
  /// The request notification permission API.
  RequestNotificationPermission,
  /// The notification permission check API.
  IsNotificationPermissionGranted,
}

impl Cmd {
  #[module_command_handler(notification_all)]
  fn notification<R: Runtime>(
    context: InvokeContext<R>,
    options: NotificationOptions,
  ) -> super::Result<()> {
    let mut notification =
      Notification::new(context.config.tauri.bundle.identifier.clone()).title(options.title);
    if let Some(body) = options.body {
      notification = notification.body(body);
    }
    if let Some(icon) = options.icon {
      notification = notification.icon(icon);
    }
    #[cfg(feature = "windows7-compat")]
    {
      notification.notify(&context.window.app_handle)?;
    }
    #[cfg(not(feature = "windows7-compat"))]
    notification.show()?;
    Ok(())
  }

  fn request_notification_permission<R: Runtime>(
    _context: InvokeContext<R>,
  ) -> super::Result<&'static str> {
    Ok(if cfg!(notification_all) {
      PERMISSION_GRANTED
    } else {
      PERMISSION_DENIED
    })
  }

  fn is_notification_permission_granted<R: Runtime>(
    _context: InvokeContext<R>,
  ) -> super::Result<bool> {
    Ok(cfg!(notification_all))
  }
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
      if cfg!(notification_all) {
        super::PERMISSION_GRANTED
      } else {
        super::PERMISSION_DENIED
      }
    )
  }

  #[cfg(not(notification_all))]
  #[test]
  fn is_notification_permission_granted() {
    let expected = cfg!(notification_all);
    assert_eq!(
      super::Cmd::is_notification_permission_granted(crate::test::mock_invoke_context()).unwrap(),
      expected,
    );
  }

  #[tauri_macros::module_command_test(notification_all, "notification > all")]
  #[quickcheck_macros::quickcheck]
  fn notification(_options: NotificationOptions) {}
}
